//! OctoWoW HD Installer: installs, updates and removes HD Switch and the optional packs.
//!
//! Source of truth: `manifest-v2.json` in the latest GitHub release (`manifest.json` there is for
//! installer 1.0, which cannot download files hosted elsewhere). It lists every component's
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
const MANIFEST: &str = "manifest-v2.json";
const STATE_FILE: &str = "mods/octowow-hd-installer.json";
const HASH_CACHE: &str = "mods/octowow-hd-installer.hashes.json";
/// Component ids of installer 1.0 (one component per pack) and the files they put in place.
const LEGACY: &[(&str, &[&str])] = &[
    ("hdswitch", &["mods/HDToggle.dll", "Interface/AddOns/HDSwitch/HDSwitch.toc",
                   "Interface/AddOns/HDSwitch/HDSwitch.lua", "Interface/AddOns/HDSwitch/Bindings.xml"]),
    ("female", &["Data/Patch-F.mpq"]),
    ("trees", &["Data/Patch-H.mpq"]),
    ("nude", &["Data/Patch-Y.mpq"]),
];
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
    /// who hosts the file, for messages ("Project Reforged"); empty for our own release
    #[serde(default)]
    pub source: String,
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
    /// files the installer put in place (dest path); only these are deleted on remove
    #[serde(default)]
    placed: std::collections::BTreeSet<String>,
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

/// Letter packs in Data that are not part of the supported setup (OctoWoW's own numbered
/// patches are not counted).
#[tauri::command]
fn unsupported_packs(wow_dir: String, manifest: Manifest) -> Vec<String> {
    let known: Vec<String> = manifest.components.iter().flat_map(|c| c.files.iter())
        .map(|f| f.dest.replace('\\', "/").to_lowercase()).collect();
    let data = PathBuf::from(&wow_dir).join("Data");
    let Ok(rd) = std::fs::read_dir(&data) else { return Vec::new() };
    let mut out: Vec<String> = rd.flatten()
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .filter(|n| {
            let l = n.to_lowercase();
            l.len() == 11 && l.starts_with("patch-") && l.ends_with(".mpq") && l.as_bytes()[6].is_ascii_alphabetic()
        })
        .filter(|n| !known.contains(&format!("data/{}", n.to_lowercase())))
        .collect();
    out.sort();
    out
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
        let mut m: Manifest = c.get(format!("{base}/{MANIFEST}")).send().await.map_err(err("manifest"))?
            .error_for_status().map_err(err("manifest"))?.json().await.map_err(err("manifest is damaged"))?;
        for comp in &mut m.components {
            for f in &mut comp.files {
                if f.url.is_empty() {
                    f.url = format!("{base}/{}", f.asset);
                }
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
    let murl = url_of(MANIFEST).ok_or("This release cannot be installed with the installer (it has no manifest).")?;
    let mut m: Manifest = c.get(murl).send().await.map_err(err("manifest"))?
        .error_for_status().map_err(err("manifest"))?
        .json().await.map_err(err("manifest is damaged"))?;
    for comp in &mut m.components {
        for f in &mut comp.files {
            // files hosted by their authors (e.g. Project Reforged) carry their own URL
            if f.url.is_empty() {
                f.url = url_of(&f.asset).ok_or_else(|| format!("the release has no '{}'", f.asset))?;
            }
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
    let mut st: InstalledState =
        std::fs::read(wow.join(STATE_FILE)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
    // installer 1.0 did not record placed files: its components' files were all placed by it
    for (id, files) in LEGACY {
        if st.components.contains_key(*id) {
            st.placed.extend(files.iter().map(|f| f.to_string()));
        }
    }
    st
}

/// Paths in the game folder are compared without case (Windows and macOS folders ignore it).
fn is_placed(st: &InstalledState, dest: &str) -> bool {
    st.placed.iter().any(|d| d.eq_ignore_ascii_case(dest))
}

fn write_state(wow: &Path, s: &InstalledState) -> Res<()> {
    let p = wow.join(STATE_FILE);
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d).map_err(err("mods folder"))?;
    }
    std::fs::write(p, serde_json::to_vec_pretty(s).map_err(err("state"))?).map_err(err("could not save the install state"))
}

fn free_space(p: &Path) -> Option<u64> {
    use sysinfo::Disks;
    let disks = Disks::new_with_refreshed_list();
    let canon = std::fs::canonicalize(p).ok()?;
    disks.list().iter()
        .filter(|d| canon.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().as_os_str().len())
        .map(|d| d.available_space())
}

/// SHA-256 of files in the game folder, remembered by size and modification time, so that a
/// status check does not read the 14 GB of packs again on every start.
#[derive(Serialize, Deserialize, Default)]
struct HashCache {
    files: BTreeMap<String, CachedHash>,
    #[serde(skip)]
    dirty: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct CachedHash {
    size: u64,
    mtime_s: u64,
    mtime_ns: u32,
    sha256: String,
}

impl HashCache {
    fn load(wow: &Path) -> Self {
        std::fs::read(wow.join(HASH_CACHE)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
    }

    fn save(&self, wow: &Path) {
        if self.dirty {
            if let Ok(b) = serde_json::to_vec(self) {
                let _ = std::fs::write(wow.join(HASH_CACHE), b);
            }
        }
    }

    fn stamp(p: &Path) -> Option<(u64, u64, u32)> {
        let m = std::fs::metadata(p).ok()?;
        let t = m.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?;
        Some((m.len(), t.as_secs(), t.subsec_nanos()))
    }

    /// The file's hash, if its size is `size` (other sizes cannot match, so they are not read).
    fn sha256(&mut self, key: &str, p: &Path, size: u64) -> Option<String> {
        let (len, s, ns) = Self::stamp(p)?;
        if len != size {
            return None;
        }
        if let Some(c) = self.files.get(&key.to_lowercase()) {
            if (c.size, c.mtime_s, c.mtime_ns) == (len, s, ns) {
                return Some(c.sha256.clone());
            }
        }
        let h = file_sha256(p)?;
        // the file must not have changed while it was read
        if Self::stamp(p)? == (len, s, ns) {
            self.files.insert(key.to_lowercase(), CachedHash { size: len, mtime_s: s, mtime_ns: ns, sha256: h.clone() });
            self.dirty = true;
        }
        Some(h)
    }

    /// Records a file whose hash is already known (just downloaded and verified).
    fn put(&mut self, key: &str, p: &Path, sha256: &str) {
        if let Some((len, s, ns)) = Self::stamp(p) {
            self.files.insert(key.to_lowercase(), CachedHash { size: len, mtime_s: s, mtime_ns: ns, sha256: sha256.into() });
            self.dirty = true;
        }
    }
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
        let mut cache = HashCache::load(&wow);
        let mut out = Vec::new();
        for c in &manifest.components {
            let mut all_match = true;
            let mut any_exists = false;
            for f in &c.files {
                let p = safe_dest(&wow, &f.dest)?;
                if p.exists() {
                    any_exists = true;
                    if cache.sha256(&f.dest, &p, f.size).as_deref() != Some(f.sha256.as_str()) {
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
        cache.save(&wow);
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

fn untested(f: &ManifestFile) -> String {
    let who = if f.source.is_empty() { "The server".to_string() } else { f.source.clone() };
    format!("{who} now has a different version of {} than the one HD Switch was tested with. \
             It will be installable once the tested setup is updated.", f.asset)
}

async fn download<R: tauri::Runtime>(app: &AppHandle<R>, comp: &str, f: &ManifestFile, to: &Path) -> Res<()> {
    let c = client()?;
    // a partial download from an earlier attempt is continued, not restarted
    let have = std::fs::metadata(to).map(|m| m.len()).unwrap_or(0);
    let have = if have > 0 && have < f.size { have } else { 0 };
    let mut req = c.get(&f.url);
    if have > 0 {
        req = req.header("Range", format!("bytes={have}-"));
    }
    let resp = req.send().await.map_err(err("download"))?.error_for_status().map_err(err("download"))?;
    let resumed = have > 0 && resp.status() == reqwest::StatusCode::PARTIAL_CONTENT;
    let total = if resumed {
        resp.headers().get("content-range").and_then(|v| v.to_str().ok())
            .and_then(|v| v.rsplit('/').next()).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0)
    } else {
        resp.content_length().unwrap_or(0)
    };
    if total != f.size {
        return Err(untested(f));
    }
    let mut h = Sha256::new();
    let mut out = if resumed {
        use std::io::Read;
        let mut r = std::fs::File::open(to).map_err(err("temporary file"))?;
        let mut buf = vec![0u8; 1 << 20];
        loop {
            let n = r.read(&mut buf).map_err(err("temporary file"))?;
            if n == 0 { break; }
            h.update(&buf[..n]);
        }
        tokio::fs::OpenOptions::new().append(true).open(to).await.map_err(err("temporary file"))?
    } else {
        tokio::fs::File::create(to).await.map_err(err("temporary file"))?
    };
    let mut done = if resumed { have } else { 0 };
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
        if !f.source.is_empty() {
            return Err(untested(f));
        }
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
    // files that are already exactly right are not downloaded again
    let mut cache = HashCache::load(&wow);
    let mut todo = Vec::new();
    for f in &component.files {
        let dest = safe_dest(&wow, &f.dest)?;
        let ok = cache.sha256(&f.dest, &dest, f.size).as_deref() == Some(f.sha256.as_str());
        if ok {
            let _ = app.emit("progress", Progress { component: &component.id, file: &f.asset, done: f.size, total: f.size });
        } else {
            todo.push(f);
        }
    }
    cache.save(&wow);
    let need: u64 = todo.iter().map(|f| f.size).sum();
    if let Some(free) = free_space(&wow) {
        if free < need + 256 * 1024 * 1024 {
            return Err(format!("Not enough disk space: {:.1} GB needed, {:.1} GB free.",
                need as f64 / 1e9, free as f64 / 1e9));
        }
    }
    // download and verify everything first; then, if the game is closed, put it all in place
    let mut staged = Vec::new();
    let staged_files = todo.clone();
    for f in todo {
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
    for (f, (part, dest)) in staged_files.iter().zip(&staged) {
        if dest.exists() && !is_placed(&st0, &f.dest) {
            // back up a file the installer did not put there, once, before replacing it
            let bak = PathBuf::from(format!("{}{BACKUP}", dest.to_string_lossy()));
            if !bak.exists() {
                std::fs::rename(dest, &bak).map_err(err("could not back up"))?;
            }
        }
        std::fs::rename(part, dest).map_err(err("could not put the file in place"))?;
        cache.put(&f.dest, dest, &f.sha256);
    }
    cache.save(&wow);
    let mut st = read_state(&wow);
    for f in &component.files {
        let dest = safe_dest(&wow, &f.dest)?;
        if staged.iter().any(|(_, d)| d == &dest) {
            st.placed.insert(f.dest.clone());
        }
    }
    write_state(&wow, &st)?;
    if let Some(line) = &component.dll_line {
        ensure_dll_line(&wow, line)?;
    }
    let mut st = read_state(&wow);
    // installer 1.0 ids whose files are all part of this component are superseded by it
    for (id, files) in LEGACY {
        if *id != component.id && files.iter().all(|lf| component.files.iter().any(|f| f.dest.eq_ignore_ascii_case(lf))) {
            st.components.remove(*id);
        }
    }
    st.components.insert(component.id.clone(), component.version.clone());
    write_state(&wow, &st)
}

#[tauri::command]
async fn uninstall(wow_dir: String, component: Component) -> Res<()> {
    let wow = PathBuf::from(&wow_dir);
    if game_running() {
        return Err("The game is running. Close it and try again.".into());
    }
    let st0 = read_state(&wow);
    for f in &component.files {
        let dest = safe_dest(&wow, &f.dest)?;
        // a file that was already there before the installer (e.g. Project Reforged packs the
        // player installed) is left alone
        if !is_placed(&st0, &f.dest) {
            continue;
        }
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
    for f in &component.files {
        st.placed.retain(|d| !d.eq_ignore_ascii_case(&f.dest));
    }
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
            fetch_manifest, component_status, install, uninstall, unsupported_packs
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
            let later = ["broken", "external", "newer", "preinstalled"];
            for c in &m.components {
                if later.contains(&c.id.as_str()) { continue; }
                install(h.clone(), wow.clone(), c.clone()).await.unwrap_or_else(|e| panic!("{}: {e}", c.id));
            }
            let st = component_status(wow.clone(), m.clone()).await.unwrap();
            for s in &st {
                if later.contains(&s.id.as_str()) && s.id != "broken" { continue; }
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
            // external file (own URL): installed, size-checked
            let ext = m.components.iter().find(|c| c.id == "external").unwrap().clone();
            // a partial earlier download is continued, not restarted
            let d = Path::new(&wow).join(&ext.files[0].dest);
            let part = PathBuf::from(format!("{}{PART}", d.to_string_lossy()));
            let full = std::fs::read(std::env::var("HDI_TEST_EXT_FILE").unwrap()).unwrap();
            std::fs::write(&part, &full[..full.len() / 3]).unwrap();
            install(h.clone(), wow.clone(), ext.clone()).await.unwrap();
            assert_eq!(std::fs::read(&d).unwrap(), full, "resumed download is byte-identical");
            // a different upstream version is reported, nothing is placed
            let newer = m.components.iter().find(|c| c.id == "newer").unwrap().clone();
            let e = install(h.clone(), wow.clone(), newer.clone()).await.unwrap_err();
            assert!(e.contains("different version"), "{e}");
            assert!(!Path::new(&wow).join(&newer.files[0].dest).exists());
            // a file that was already exactly right is not replaced and not removed later
            let pre = m.components.iter().find(|c| c.id == "preinstalled").unwrap().clone();
            let pd = Path::new(&wow).join(&pre.files[0].dest);
            let before = std::fs::metadata(&pd).unwrap().modified().unwrap();
            install(h.clone(), wow.clone(), pre.clone()).await.unwrap();
            assert_eq!(std::fs::metadata(&pd).unwrap().modified().unwrap(), before, "identical file not rewritten");
            // packs outside the supported setup are reported
            let un = unsupported_packs(wow.clone(), m.clone());
            assert_eq!(un, vec!["patch-L.mpq".to_string()], "{un:?}");
            // remove
            for c in m.components.iter().filter(|c| c.id != "broken" && c.id != "newer") {
                uninstall(wow.clone(), c.clone()).await.unwrap();
            }
            assert!(pd.exists(), "a pack the player had before is left alone");
            assert!(!d.exists(), "a pack the installer placed is removed");
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

    fn sha_hex(b: &[u8]) -> String {
        hex::encode(Sha256::digest(b))
    }

    #[test]
    fn hash_cache_reuses_and_invalidates() {
        let d = std::env::temp_dir().join(format!("hdi-cache-{}", std::process::id()));
        std::fs::create_dir_all(d.join("mods")).unwrap();
        let f = d.join("a.mpq");
        std::fs::write(&f, b"first").unwrap();
        let mut c = HashCache::default();
        assert_eq!(c.sha256("A.mpq", &f, 5).unwrap(), sha_hex(b"first"));
        assert_eq!(c.sha256("a.mpq", &f, 4), None, "a different size is not read");
        c.save(&d);
        // a cache entry is trusted while size and mtime match: poison it to prove it is used
        let mut c = HashCache::load(&d);
        c.files.get_mut("a.mpq").unwrap().sha256 = "cached".into();
        assert_eq!(c.sha256("a.mpq", &f, 5).unwrap(), "cached");
        // same size, new content and mtime: hashed again
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&f, b"other").unwrap();
        assert_eq!(c.sha256("a.mpq", &f, 5).unwrap(), sha_hex(b"other"));
        std::fs::remove_dir_all(&d).unwrap();
    }

    /// Installer 1.0 kept one component per pack and no list of placed files. Installing the
    /// 1.2.1 bundle over it must not treat its own Patch-F as a foreign file, and a later
    /// remove must delete it, while a pack the player had installed stays.
    #[test]
    fn upgrade_from_installer_1_0() {
        let d = std::env::temp_dir().join(format!("hdi-legacy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("Data")).unwrap();
        std::fs::create_dir_all(d.join("mods")).unwrap();
        std::fs::write(d.join("WoW.exe"), b"x").unwrap();
        std::fs::write(d.join("Data/Patch-F.mpq"), b"female").unwrap();
        std::fs::write(d.join("Data/patch-A.mpq"), b"reforged").unwrap();
        std::fs::write(d.join(STATE_FILE), r#"{"components":{"female":"1.0.0","nude":"1.0.0"}}"#).unwrap();
        let file = |n: &str, b: &[u8]| ManifestFile {
            asset: n.into(), dest: format!("Data/{n}"), sha256: sha_hex(b), size: b.len() as u64,
            url: String::new(), source: String::new(),
        };
        let hd = Component {
            id: "packs".into(), name: "OctoWoW HD packs".into(), version: "1.2.1".into(),
            description: String::new(), nsfw: false, credits: String::new(), dll_line: None,
            files: vec![file("Patch-F.mpq", b"female"), file("patch-A.mpq", b"reforged")],
        };
        // SAFETY: tests in this module that read the variable do not depend on it being unset
        unsafe { std::env::set_var("HDI_TEST_NO_GAME", "1") };
        let app = tauri::test::mock_app();
        let wow = d.to_string_lossy().to_string();
        tauri::async_runtime::block_on(install(app.handle().clone(), wow.clone(), hd.clone())).unwrap();
        assert!(!d.join("Data/Patch-F.mpq.hdi-backup").exists(), "own file was backed up");
        let st = read_state(&d);
        assert_eq!(st.components.keys().collect::<Vec<_>>(), ["nude", "packs"]);
        assert!(is_placed(&st, "Data/Patch-F.mpq") && is_placed(&st, "Data/Patch-Y.mpq"));
        assert!(!is_placed(&st, "Data/patch-A.mpq"));
        tauri::async_runtime::block_on(uninstall(wow, hd)).unwrap();
        assert!(!d.join("Data/Patch-F.mpq").exists());
        assert!(d.join("Data/patch-A.mpq").exists(), "the player's own pack was removed");
        std::fs::remove_dir_all(&d).unwrap();
    }

    /// Downloads the smallest Project Reforged pack of the real manifest from Project Reforged's
    /// server and checks it. Network; run by hand: cargo test real_reforged -- --ignored
    #[test]
    #[ignore]
    fn real_reforged_download() {
        let app = tauri::test::mock_app();
        let h = app.handle().clone();
        tauri::async_runtime::block_on(async move {
            let m = fetch_manifest().await.expect("manifest");
            let f = m.components.iter().flat_map(|c| &c.files).filter(|f| !f.source.is_empty())
                .min_by_key(|f| f.size).expect("a Project Reforged file").clone();
            let to = std::env::temp_dir().join(format!("hdi-real-{}", std::process::id()));
            download(&h, "packs", &f, &to).await.expect("download");
            assert_eq!(std::fs::metadata(&to).unwrap().len(), f.size);
            std::fs::remove_file(&to).unwrap();
        });
    }
}
