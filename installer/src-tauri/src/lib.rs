//! OctoWoW HD Installer: installs, updates and removes HD Switch and the optional packs.
//!
//! Source of truth: `manifest.json` in the latest GitHub release. It lists every component's
//! files, their destination in the game folder and their SHA-256. Files are downloaded as
//! `.hdi-part`, verified, then moved into place; nothing is touched while the game runs.

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

const REPO: &str = "fmustafayaman/OctoWoW-HD-Switch";
const UA: &str = concat!("OctoWoW-HD-Installer/", env!("CARGO_PKG_VERSION"));
const STATE_FILE: &str = "mods/octowow-hd-installer.json";
const PART: &str = ".hdi-part";
const BACKUP: &str = ".hdi-backup";

// ---------------------------------------------------------------- veri tipleri

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ManifestFile {
    pub asset: String,
    pub dest: String,
    pub sha256: String,
    pub size: u64,
    #[serde(default)]
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub nsfw: bool,
    #[serde(default)]
    pub credits: String,
    #[serde(default)]
    pub dll_line: Option<String>,
    pub files: Vec<ManifestFile>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Manifest {
    pub release: String,
    #[serde(default)]
    pub notes: String,
    pub components: Vec<Component>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct InstalledState {
    /// component id -> installed version
    components: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Config {
    pub wow_dir: Option<String>,
    #[serde(default)]
    pub nsfw_ok: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct ComponentStatus {
    pub id: String,
    /// "installed" | "update" | "different" | "missing"
    pub status: String,
    pub installed_version: Option<String>,
}

#[derive(Serialize, Clone)]
struct Progress<'a> {
    component: &'a str,
    file: &'a str,
    done: u64,
    total: u64,
}

type Res<T> = Result<T, String>;

fn err<E: std::fmt::Display>(ctx: &str) -> impl FnOnce(E) -> String + '_ {
    move |e| format!("{ctx}: {e}")
}

// ---------------------------------------------------------------- ayarlar

fn config_path() -> Res<PathBuf> {
    let dir = dirs::config_dir().ok_or("settings folder not found")?.join("OctoWoW-HD-Installer");
    std::fs::create_dir_all(&dir).map_err(err("settings folder"))?;
    Ok(dir.join("config.json"))
}

#[tauri::command]
fn get_config() -> Res<Config> {
    let p = config_path()?;
    if !p.exists() {
        return Ok(Config::default());
    }
    serde_json::from_slice(&std::fs::read(&p).map_err(err("could not read settings"))?).map_err(err("settings file is damaged"))
}

#[tauri::command]
fn save_config(config: Config) -> Res<()> {
    let data = serde_json::to_vec_pretty(&config).map_err(err("settings"))?;
    std::fs::write(config_path()?, data).map_err(err("could not save settings"))
}

// ---------------------------------------------------------------- game folder

fn is_wow_dir(p: &Path) -> bool {
    p.join("WoW.exe").is_file() && p.join("Data").is_dir()
}

/// Checks the chosen folder; finds WoW.exe one level down if needed.
#[tauri::command]
fn check_wow_dir(path: String) -> Res<String> {
    let p = PathBuf::from(&path);
    if is_wow_dir(&p) {
        return Ok(p.to_string_lossy().into());
    }
    if let Ok(rd) = std::fs::read_dir(&p) {
        for e in rd.flatten() {
            let sub = e.path();
            if is_wow_dir(&sub) {
                return Ok(sub.to_string_lossy().into());
            }
        }
    }
    Err("No WoW.exe and Data folder here. Choose the folder OctoWoW is installed in.".into())
}

/// Looks for an OctoWoW folder in the usual places.
#[tauri::command]
fn find_wow_dirs() -> Vec<String> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(h) = dirs::home_dir() {
        roots.extend([h.join("Games"), h.join("Applications"), h.join("Desktop"), h.join("Documents"), h.clone()]);
    }
    if cfg!(windows) {
        for d in ["C", "D", "E", "F"] {
            roots.extend([PathBuf::from(format!("{d}:\\")), PathBuf::from(format!("{d}:\\Games")),
                PathBuf::from(format!("{d}:\\Program Files")), PathBuf::from(format!("{d}:\\Program Files (x86)"))]);
        }
    } else {
        roots.push(PathBuf::from("/Applications"));
        if let Ok(rd) = std::fs::read_dir("/Volumes") {
            roots.extend(rd.flatten().map(|e| e.path()));
        }
    }
    let mut found = Vec::new();
    for r in roots {
        // two levels: <root>/<x>/ and <root>/<x>/<y>/
        let Ok(rd) = std::fs::read_dir(&r) else { continue };
        for e in rd.flatten() {
            let a = e.path();
            let name = a.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
            if !(name.contains("octo") || name.contains("wow") || name.contains("warcraft")) {
                continue;
            }
            if is_wow_dir(&a) {
                found.push(a.to_string_lossy().into());
                continue;
            }
            if let Ok(rd2) = std::fs::read_dir(&a) {
                for e2 in rd2.flatten() {
                    if is_wow_dir(&e2.path()) {
                        found.push(e2.path().to_string_lossy().into());
                    }
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// Files that load over our packs (Project Reforged's patch-L loads after Patch-F).
#[tauri::command]
fn conflicts(wow_dir: String) -> Vec<String> {
    let data = PathBuf::from(&wow_dir).join("Data");
    let Ok(rd) = std::fs::read_dir(&data) else { return Vec::new() };
    rd.flatten()
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .filter(|n| n.eq_ignore_ascii_case("patch-L.mpq"))
        .map(|n| format!("Data/{n}"))
        .collect()
}

// ---------------------------------------------------------------- is the game running

#[tauri::command]
fn game_running() -> bool {
    if std::env::var("HDI_TEST_NO_GAME").is_ok() {
        return false;
    }
    use sysinfo::{ProcessRefreshKind, RefreshKind, System};
    let sys = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));
    sys.processes().values().any(|p| {
        let name = p.name().to_string_lossy().to_lowercase();
        if name == "wow.exe" || name.ends_with("\\wow.exe") || name.ends_with("/wow.exe") {
            return true;
        }
        p.cmd().iter().any(|a| a.to_string_lossy().to_lowercase().ends_with("wow.exe"))
    })
}

// ---------------------------------------------------------------- manifest

fn client() -> Res<reqwest::Client> {
    reqwest::Client::builder().user_agent(UA).build().map_err(err("network"))
}

#[tauri::command]
async fn fetch_manifest() -> Res<Manifest> {
    let c = client()?;
    // development/tests: a manifest on a local server (files next to it)
    if let Ok(base) = std::env::var("HDI_MANIFEST_BASE") {
        let mut m: Manifest = c.get(format!("{base}/manifest.json")).send().await.map_err(err("manifest"))?
            .error_for_status().map_err(err("manifest"))?.json().await.map_err(err("manifest is damaged"))?;
        for comp in &mut m.components {
            for f in &mut comp.files {
                f.url = format!("{base}/{}", f.asset);
            }
        }
        return Ok(m);
    }
    let rel: serde_json::Value = c
        .get(format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .header("Accept", "application/vnd.github+json")
        .send().await.map_err(err("could not reach GitHub"))?
        .error_for_status().map_err(err("GitHub"))?
        .json().await.map_err(err("GitHub reply"))?;
    let assets = rel["assets"].as_array().ok_or("the release has no files")?;
    let url_of = |name: &str| -> Option<String> {
        assets.iter().find(|a| a["name"] == name).and_then(|a| a["browser_download_url"].as_str()).map(String::from)
    };
    let murl = url_of("manifest.json").ok_or("This release has no manifest.json, so it cannot be installed with the installer.")?;
    let mut m: Manifest = c.get(murl).send().await.map_err(err("manifest"))?
        .error_for_status().map_err(err("manifest"))?
        .json().await.map_err(err("manifest is damaged"))?;
    for comp in &mut m.components {
        for f in &mut comp.files {
            f.url = url_of(&f.asset).ok_or_else(|| format!("the release has no '{}'", f.asset))?;
        }
    }
    Ok(m)
}

// ---------------------------------------------------------------- durum

fn safe_dest(wow: &Path, dest: &str) -> Res<PathBuf> {
    // never trust manifest paths: inside the game folder only, no '..'
    if dest.contains("..") || dest.starts_with('/') || dest.starts_with('\\') || dest.contains(':') {
        return Err(format!("invalid destination path: {dest}"));
    }
    Ok(dest.split(['/', '\\']).fold(wow.to_path_buf(), |p, s| p.join(s)))
}

fn read_state(wow: &Path) -> InstalledState {
    std::fs::read(wow.join(STATE_FILE)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn write_state(wow: &Path, s: &InstalledState) -> Res<()> {
    let p = wow.join(STATE_FILE);
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d).map_err(err("mods folder"))?;
    }
    std::fs::write(p, serde_json::to_vec_pretty(s).map_err(err("state"))?).map_err(err("could not save the install state"))
}

fn file_sha256(p: &Path) -> Option<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(p).ok()?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Some(hex::encode(h.finalize()))
}

#[tauri::command]
async fn component_status(wow_dir: String, manifest: Manifest) -> Res<Vec<ComponentStatus>> {
    tauri::async_runtime::spawn_blocking(move || {
        let wow = PathBuf::from(&wow_dir);
        let st = read_state(&wow);
        let mut out = Vec::new();
        for c in &manifest.components {
            let mut all_match = true;
            let mut any_exists = false;
            for f in &c.files {
                let p = safe_dest(&wow, &f.dest)?;
                if p.exists() {
                    any_exists = true;
                    // size first, then the hash (avoids reading big files needlessly)
                    let same_size = std::fs::metadata(&p).map(|m| m.len() == f.size).unwrap_or(false);
                    if !same_size || file_sha256(&p).as_deref() != Some(f.sha256.as_str()) {
                        all_match = false;
                    }
                } else {
                    all_match = false;
                }
            }
            let installed_version = st.components.get(&c.id).cloned();
            let status = if all_match {
                "installed"
            } else if installed_version.is_some() {
                "update"
            } else if any_exists {
                "different"
            } else {
                "missing"
            };
            out.push(ComponentStatus { id: c.id.clone(), status: status.into(), installed_version });
        }
        Ok(out)
    })
    .await
    .map_err(err("state"))?
}

// ---------------------------------------------------------------- dlls.txt

fn dll_line_present(text: &str, line: &str) -> bool {
    let norm = |s: &str| s.trim().replace('\\', "/").to_lowercase();
    text.lines().any(|l| norm(l) == norm(line))
}

fn ensure_dll_line(wow: &Path, line: &str) -> Res<()> {
    let p = wow.join("dlls.txt");
    let text = std::fs::read_to_string(&p).unwrap_or_default();
    if dll_line_present(&text, line) {
        return Ok(());
    }
    if p.exists() && !wow.join(format!("dlls.txt{BACKUP}")).exists() {
        std::fs::copy(&p, wow.join(format!("dlls.txt{BACKUP}"))).map_err(err("dlls.txt backup"))?;
    }
    let mut t = text;
    if !t.is_empty() && !t.ends_with('\n') {
        t.push_str("\r\n");
    }
    t.push_str(line);
    t.push_str("\r\n");
    std::fs::write(&p, t).map_err(err("could not write dlls.txt"))
}

fn remove_dll_line(wow: &Path, line: &str) -> Res<()> {
    let p = wow.join("dlls.txt");
    let Ok(text) = std::fs::read_to_string(&p) else { return Ok(()) };
    if !dll_line_present(&text, line) {
        return Ok(());
    }
    let norm = |s: &str| s.trim().replace('\\', "/").to_lowercase();
    let kept: Vec<&str> = text.lines().filter(|l| norm(l) != norm(line)).collect();
    std::fs::write(&p, kept.join("\r\n") + "\r\n").map_err(err("could not write dlls.txt"))
}

// ---------------------------------------------------------------- install / remove

async fn download<R: tauri::Runtime>(app: &AppHandle<R>, comp: &str, f: &ManifestFile, to: &Path) -> Res<()> {
    let c = client()?;
    let resp = c.get(&f.url).send().await.map_err(err("download"))?.error_for_status().map_err(err("download"))?;
    let total = resp.content_length().unwrap_or(f.size);
    let mut out = tokio::fs::File::create(to).await.map_err(err("temporary file"))?;
    let mut h = Sha256::new();
    let mut done = 0u64;
    let mut last = 0u64;
    let mut stream = resp.bytes_stream();
    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(err("download interrupted"))?;
        h.update(&chunk);
        out.write_all(&chunk).await.map_err(err("could not write to disk"))?;
        done += chunk.len() as u64;
        if done - last > 256 * 1024 || done == total {
            last = done;
            let _ = app.emit("progress", Progress { component: comp, file: &f.asset, done, total });
        }
    }
    out.flush().await.map_err(err("could not write to disk"))?;
    drop(out);
    let got = hex::encode(h.finalize());
    if got != f.sha256 {
        let _ = std::fs::remove_file(to);
        return Err(format!("'{}' failed verification (SHA-256 mismatch). The download may be damaged; try again.", f.asset));
    }
    Ok(())
}

#[tauri::command]
async fn install<R: tauri::Runtime>(app: AppHandle<R>, wow_dir: String, component: Component) -> Res<()> {
    let wow = PathBuf::from(&wow_dir);
    if !is_wow_dir(&wow) {
        return Err("The game folder is not valid.".into());
    }
    // download and verify everything first; then, if the game is closed, put it all in place
    let mut staged = Vec::new();
    for f in &component.files {
        let dest = safe_dest(&wow, &f.dest)?;
        if let Some(d) = dest.parent() {
            std::fs::create_dir_all(d).map_err(err("could not create the folder"))?;
        }
        let part = PathBuf::from(format!("{}{PART}", dest.to_string_lossy()));
        download(&app, &component.id, f, &part).await?;
        staged.push((part, dest));
    }
    if game_running() {
        for (part, _) in &staged {
            let _ = std::fs::remove_file(part);
        }
        return Err("The game is running. Files cannot be changed while it runs; close the game and try again.".into());
    }
    let st0 = read_state(&wow);
    let ours = st0.components.contains_key(&component.id);
    for (part, dest) in &staged {
        if dest.exists() && !ours {
            // back up a file the installer did not put there, once, before replacing it
            let bak = PathBuf::from(format!("{}{BACKUP}", dest.to_string_lossy()));
            if !bak.exists() {
                std::fs::rename(dest, &bak).map_err(err("could not back up"))?;
            }
        }
        std::fs::rename(part, dest).map_err(err("could not put the file in place"))?;
    }
    if let Some(line) = &component.dll_line {
        ensure_dll_line(&wow, line)?;
    }
    let mut st = read_state(&wow);
    st.components.insert(component.id.clone(), component.version.clone());
    write_state(&wow, &st)
}

#[tauri::command]
async fn uninstall(wow_dir: String, component: Component) -> Res<()> {
    let wow = PathBuf::from(&wow_dir);
    if game_running() {
        return Err("The game is running. Close it and try again.".into());
    }
    for f in &component.files {
        let dest = safe_dest(&wow, &f.dest)?;
        if dest.exists() {
            std::fs::remove_file(&dest).map_err(err("could not delete"))?;
        }
        let bak = PathBuf::from(format!("{}{BACKUP}", dest.to_string_lossy()));
        if bak.exists() {
            std::fs::rename(&bak, &dest).map_err(err("could not restore the backup"))?;
        }
    }
    if let Some(line) = &component.dll_line {
        remove_dll_line(&wow, line)?;
    }
    let mut st = read_state(&wow);
    st.components.remove(&component.id);
    write_state(&wow, &st)
}

// ---------------------------------------------------------------- uygulama

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_config, save_config, check_wow_dir, find_wow_dirs, game_running,
            fetch_manifest, component_status, install, uninstall, conflicts
        ])
        .run(tauri::generate_context!())
        .expect("could not start the application");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End to end: local manifest -> install -> status -> damaged download -> remove (backup restored).
    /// Runs when HDI_MANIFEST_BASE and HDI_TEST_WOW are set (scripts/e2e.sh).
    #[test]
    fn end_to_end_install_uninstall() {
        let (Ok(_), Ok(wow)) = (std::env::var("HDI_MANIFEST_BASE"), std::env::var("HDI_TEST_WOW")) else { return };
        let app = tauri::test::mock_app();
        let h = app.handle().clone();
        tauri::async_runtime::block_on(async move {
            let m = fetch_manifest().await.expect("manifest");
            for c in &m.components {
                if c.id == "broken" { continue; }
                install(h.clone(), wow.clone(), c.clone()).await.unwrap_or_else(|e| panic!("{}: {e}", c.id));
            }
            let st = component_status(wow.clone(), m.clone()).await.unwrap();
            for s in &st {
                let want = if s.id == "broken" { "missing" } else { "installed" };
                assert_eq!(s.status, want, "{}", s.id);
            }
            let dlls = std::fs::read_to_string(Path::new(&wow).join("dlls.txt")).unwrap();
            assert_eq!(dlls.matches("mods/HDToggle.dll").count(), 1, "dlls.txt line exactly once");
            // was the existing (not ours) file backed up
            assert!(Path::new(&wow).join("Data/Patch-F.mpq.hdi-backup").exists());
            // bad SHA: error, nothing placed, no temporary file left
            let bad = m.components.iter().find(|c| c.id == "broken").unwrap().clone();
            let e = install(h.clone(), wow.clone(), bad.clone()).await.unwrap_err();
            assert!(e.contains("SHA-256"), "{e}");
            for f in &bad.files {
                let d = Path::new(&wow).join(&f.dest);
                assert!(!d.exists() && !PathBuf::from(format!("{}{PART}", d.to_string_lossy())).exists());
            }
            // install again: idempotent
            let hd = m.components.iter().find(|c| c.id == "hdswitch").unwrap().clone();
            install(h.clone(), wow.clone(), hd.clone()).await.unwrap();
            assert_eq!(std::fs::read_to_string(Path::new(&wow).join("dlls.txt")).unwrap().matches("HDToggle").count(), 1);
            // remove
            for c in m.components.iter().filter(|c| c.id != "broken") {
                uninstall(wow.clone(), c.clone()).await.unwrap();
            }
            assert_eq!(std::fs::read(Path::new(&wow).join("Data/Patch-F.mpq")).unwrap(), b"USERS OLD FILE");
            assert!(!Path::new(&wow).join("mods/HDToggle.dll").exists());
            assert!(!std::fs::read_to_string(Path::new(&wow).join("dlls.txt")).unwrap().contains("HDToggle"));
            assert!(std::fs::read_to_string(Path::new(&wow).join("dlls.txt")).unwrap().contains("VanillaHelpers"));
        });
    }

    #[test]
    fn dll_line_detection_is_lenient() {
        assert!(dll_line_present("mods/VanillaHelpers.dll\r\nmods\\HDToggle.dll\r\n", "mods/HDToggle.dll"));
        assert!(dll_line_present("  MODS/HDTOGGLE.DLL  ", "mods/HDToggle.dll"));
        assert!(!dll_line_present("mods/HDToggle.dll.bak", "mods/HDToggle.dll"));
    }

    #[test]
    fn dest_paths_cannot_escape() {
        let w = Path::new("/w");
        assert!(safe_dest(w, "../x").is_err());
        assert!(safe_dest(w, "/etc/x").is_err());
        assert!(safe_dest(w, "C:\\x").is_err());
        assert_eq!(safe_dest(w, "Data/Patch-F.mpq").unwrap(), Path::new("/w/Data/Patch-F.mpq"));
        assert_eq!(safe_dest(w, "Interface\\AddOns\\HDSwitch\\a.lua").unwrap(), Path::new("/w/Interface/AddOns/HDSwitch/a.lua"));
    }

    #[test]
    fn dll_line_add_and_remove_roundtrip() {
        let d = std::env::temp_dir().join(format!("hdi-test-{}", std::process::id()));
        std::fs::create_dir_all(d.join("Data")).unwrap();
        std::fs::write(d.join("WoW.exe"), b"x").unwrap();
        std::fs::write(d.join("dlls.txt"), "mods/VanillaHelpers.dll\r\n").unwrap();
        ensure_dll_line(&d, "mods/HDToggle.dll").unwrap();
        ensure_dll_line(&d, "mods/HDToggle.dll").unwrap();
        let t = std::fs::read_to_string(d.join("dlls.txt")).unwrap();
        assert_eq!(t.matches("HDToggle").count(), 1);
        assert!(d.join("dlls.txt.hdi-backup").exists());
        remove_dll_line(&d, "mods/HDToggle.dll").unwrap();
        let t = std::fs::read_to_string(d.join("dlls.txt")).unwrap();
        assert!(!t.contains("HDToggle") && t.contains("VanillaHelpers"));
        std::fs::remove_dir_all(&d).unwrap();
    }
}
