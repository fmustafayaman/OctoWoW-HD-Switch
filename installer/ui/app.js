"use strict";
// Outside Tauri (in a plain browser) it runs against a mock backend, for design and flow checks.
const T = window.__TAURI__;
const MOCK = !T;

const api = MOCK ? mockApi() : {
  invoke: (cmd, args) => T.core.invoke(cmd, args),
  listen: (ev, fn) => T.event.listen(ev, fn),
  pickFolder: () => T.dialog.open({ directory: true, multiple: false, title: "Choose your OctoWoW folder" }),
  checkSelfUpdate: () => T.updater.check(),
  relaunch: () => T.process.relaunch(),
  appVersion: () => T.app.getVersion(),
};

const $ = (id) => document.getElementById(id);
const state = { config: { wow_dir: null, nsfw_ok: false }, manifest: null, status: {}, busy: null };

const STATUS_TEXT = {
  installed: "Installed",
  update: "Update available",
  different: "Different version found",
  missing: "Not installed",
};

// ------------------------------------------------------------ start
async function init() {
  $("app-ver").textContent = "Installer " + (await api.appVersion());
  state.config = await api.invoke("get_config");
  $("nsfw").checked = !!state.config.nsfw_ok;

  if (!state.config.wow_dir) {
    const found = await api.invoke("find_wow_dirs");
    if (found.length === 1) await setFolder(found[0], true);
  } else {
    await setFolder(state.config.wow_dir, true);
  }
  renderFolder();

  try {
    state.manifest = await api.invoke("fetch_manifest");
    $("release-line").textContent = "Latest release: " + state.manifest.release;
    $("notes-body").textContent = state.manifest.notes || "No notes for this release.";
  } catch (e) {
    $("release-line").textContent = String(e);
  }
  await refreshStatus();
  watchGame();
  checkSelfUpdate();
}

// ------------------------------------------------------------ folder
async function setFolder(path, quiet) {
  try {
    const ok = await api.invoke("check_wow_dir", { path });
    state.config.wow_dir = ok;
    await api.invoke("save_config", { config: state.config });
    $("folder-hint").textContent = "WoW.exe found. Packs are installed into this folder.";
    $("folder-hint").classList.remove("bad");
  } catch (e) {
    if (!quiet) {
      $("folder-hint").textContent = String(e);
      $("folder-hint").classList.add("bad");
    }
  }
  renderFolder();
}

function renderFolder() {
  $("wow-dir").textContent = state.config.wow_dir || "Not chosen";
  $("wow-dir").title = state.config.wow_dir || "";
}

$("pick").addEventListener("click", async () => {
  const p = await api.pickFolder();
  if (p) { await setFolder(p, false); await refreshStatus(); }
});

// ------------------------------------------------------------ status and list
async function refreshStatus() {
  state.status = {};
  if (state.config.wow_dir && state.manifest) {
    const c = await api.invoke("unsupported_packs", { wowDir: state.config.wow_dir, manifest: state.manifest });
    $("conflict").hidden = !c.length;
    $("conflict").textContent = c.length
      ? `Not part of the supported setup: ${c.join(", ")}. HD Switch is tested and supported only with the OctoWoW HD packs; with other packs it may still work, but there is no guarantee.`
      : "";
  }
  if (state.manifest && state.config.wow_dir) {
    try {
      const list = await api.invoke("component_status", { wowDir: state.config.wow_dir, manifest: state.manifest });
      for (const s of list) state.status[s.id] = s;
    } catch (e) {
      $("folder-hint").textContent = String(e);
      $("folder-hint").classList.add("bad");
    }
  }
  render();
}

function render() {
  const list = $("list");
  list.textContent = "";
  if (!state.manifest) {
    list.innerHTML = '<p class="empty">Packs appear here once the latest release is loaded.</p>';
    return;
  }
  const comps = state.manifest.components.filter((c) => !c.nsfw || state.config.nsfw_ok);
  for (const c of comps) list.appendChild(rowFor(c));
}

function rowFor(c) {
  const node = $("row-tpl").content.firstElementChild.cloneNode(true);
  node.dataset.id = c.id;
  if (!c.nsfw) node.classList.add("core");
  if (c.nsfw) node.classList.add("nsfw");
  node.querySelector(".row-name").textContent = c.name;
  node.querySelector(".row-desc").textContent = c.description;
  node.querySelector(".row-credit").textContent = c.credits || "";

  const st = state.status[c.id];
  const badge = node.querySelector(".badge");
  const actions = node.querySelector(".actions");
  const noFolder = !state.config.wow_dir;
  const busy = !!state.busy;

  if (!st) {
    badge.textContent = noFolder ? "Choose a game folder first" : "";
  } else {
    const bundle = c.files.length > 4;
    const size = c.files.reduce((a, f) => a + f.size, 0);
    badge.textContent = (bundle && st.status === "different" ? "Partly installed" : STATUS_TEXT[st.status])
      + (st.installed_version && st.status !== "installed" ? " (have " + st.installed_version + ")" : "")
      + (st.status === "installed" ? " · " + c.version : "")
      + (st.status === "missing" && size > 1e8 ? " · " + mb(size) : "");
    badge.classList.add(st.status);
  }

  const btn = (label, cls, fn) => {
    const b = document.createElement("button");
    b.type = "button"; b.className = cls; b.textContent = label;
    b.disabled = busy || noFolder || state.gameOpen;
    b.addEventListener("click", fn);
    actions.appendChild(b);
  };
  const s = st ? st.status : "missing";
  if (state.busy === c.id) { badge.textContent = "Working…"; return node; }
  if (s === "missing") btn("Install", "gold", () => run("install", c));
  const label = s === "update" ? "Update" : c.files.length > 4 ? "Complete setup" : "Replace";
  if (s === "update" || s === "different") btn(label, "gold", () => run("install", c));
  if (s !== "missing") btn("Remove", "danger small", () => run("uninstall", c));
  return node;
}

// ------------------------------------------------------------ install / remove
async function run(kind, c) {
  state.busy = c.id;
  render();
  const row = document.querySelector(`.row[data-id="${CSS.escape(c.id)}"]`);
  const bar = row.querySelector(".bar");
  const err = row.querySelector(".row-error");
  err.hidden = true;
  if (kind === "install") {
    bar.hidden = false;
    setBar(row, 0, c.files.reduce((a, f) => a + f.size, 0), "Downloading");
  }
  try {
    await api.invoke(kind, { wowDir: state.config.wow_dir, component: c });
    state.busy = null;
    await refreshStatus();
  } catch (e) {
    state.busy = null;
    await refreshStatus();
    const r = document.querySelector(`.row[data-id="${CSS.escape(c.id)}"]`);
    const er = r.querySelector(".row-error");
    er.textContent = String(e);
    er.hidden = false;
  }
}

const progressTotals = {};
api.listen("progress", (ev) => {
  const p = ev.payload;
  const row = document.querySelector(`.row[data-id="${CSS.escape(p.component)}"]`);
  if (!row) return;
  const comp = state.manifest.components.find((c) => c.id === p.component);
  progressTotals[p.component] = progressTotals[p.component] || {};
  progressTotals[p.component][p.file] = p.done;
  const done = Object.values(progressTotals[p.component]).reduce((a, b) => a + b, 0);
  const total = comp.files.reduce((a, f) => a + f.size, 0);
  setBar(row, done, total, done >= total ? "Verifying" : "Downloading");
});

function setBar(row, done, total, label) {
  const bar = row.querySelector(".bar");
  bar.hidden = false;
  const pct = total ? Math.min(100, (done / total) * 100) : 0;
  row.querySelector(".bar-fill").style.width = pct.toFixed(1) + "%";
  row.querySelector(".bar-text").textContent = `${label} ${mb(done)} of ${mb(total)}`;
}
const mb = (b) => b >= 1e9 ? (b / 1e9).toFixed(2) + " GB" : (b / 1048576 >= 10 ? (b / 1048576).toFixed(0) : (b / 1048576).toFixed(1)) + " MB";

// ------------------------------------------------------------ is the game running
async function watchGame() {
  const open = await api.invoke("game_running");
  if (open !== !!state.gameOpen) {
    state.gameOpen = open;
    $("game-open").hidden = !open;
    render();
  }
  setTimeout(watchGame, 3000);
}

// ------------------------------------------------------------ 18+
$("nsfw").addEventListener("change", async (e) => {
  state.config.nsfw_ok = e.target.checked;
  await api.invoke("save_config", { config: state.config });
  render();
});

// ------------------------------------------------------------ release notes
$("notes-btn").addEventListener("click", () => toggleNotes(true));
$("notes-close").addEventListener("click", () => toggleNotes(false));
document.addEventListener("keydown", (e) => { if (e.key === "Escape") toggleNotes(false); });
function toggleNotes(show) {
  $("notes").hidden = !show;
  $("notes-btn").setAttribute("aria-expanded", String(show));
  if (show) $("notes-close").focus();
}

// ------------------------------------------------------------ installer self-update
async function checkSelfUpdate() {
  try {
    const upd = await api.checkSelfUpdate();
    if (!upd) return;
    $("self-update-text").textContent = `Installer ${upd.version} is available.`;
    $("self-update").hidden = false;
    $("self-update-btn").onclick = async () => {
      $("self-update-btn").disabled = true;
      $("self-update-text").textContent = `Downloading installer ${upd.version}…`;
      await upd.downloadAndInstall();
      await api.relaunch();
    };
  } catch (_) { /* offline or an unsigned dev build: ignore */ }
}

// ------------------------------------------------------------ mock mode
function mockApi() {
  const files = (n, mbs) => [{ asset: n, dest: "Data/" + n, sha256: "x", size: mbs * 1048576 }];
  // the real v1.2.1 manifest, hashes left out
  const manifest = {"release": "v1.2.1", "notes": "**OctoWoW HD 1.2.1: one supported setup**\n\nThe installer now offers the exact pack set HD Switch is tested with, the same files byte for byte for everyone:\n\n- **HD Switch** can still be installed on its own and works with any letter packs, but only the set below is tested and supported.\n- **OctoWoW HD packs:** Project Reforged A, B, C, D, E, G, I, M, P, S and T, downloaded straight from Project Reforged's server and checked against the versions HD Switch was tested with; plus Twow Raid Visuals (O), the merged game tables (Z), the slimmer female models (F) and the invisible tree fix (H).\n- **Nude skins (18+)** stay optional.\n\nThe installer lists any other letter packs in your Data folder as not part of the supported setup. Packs you already have in the right version are not downloaded again, interrupted downloads resume, and a remove only deletes files the installer put there.\n", "components": [{"id": "hdswitch", "name": "HD Switch", "version": "1.2.0", "description": "Turn your HD packs on and off from inside the game, without restarting. It works with any letter packs, but only the OctoWoW HD packs below are tested and supported.", "dll_line": "mods/HDToggle.dll", "files": [{"asset": "HDToggle.dll", "dest": "mods/HDToggle.dll", "sha256": "x", "size": 72704}, {"asset": "HDSwitch-HDSwitch.toc", "dest": "Interface/AddOns/HDSwitch/HDSwitch.toc", "sha256": "x", "size": 300}, {"asset": "HDSwitch-HDSwitch.lua", "dest": "Interface/AddOns/HDSwitch/HDSwitch.lua", "sha256": "x", "size": 13115}, {"asset": "HDSwitch-Bindings.xml", "dest": "Interface/AddOns/HDSwitch/Bindings.xml", "sha256": "x", "size": 174}]}, {"id": "packs", "name": "OctoWoW HD packs", "version": "1.2.1", "description": "The supported setup, the same files byte for byte for everyone: Project Reforged A, B, C, D, E, G, I, M, P, S and T (downloaded from Project Reforged), Twow Raid Visuals, slimmer female models, the invisible tree fix and the merged game tables.", "credits": "Project Reforged by Stormhand81 and contributors. Twow Raid Visuals by MarcelineVQ. Female models: A Little Extra for Females, Less Thicc Version by Deezhugs, on Watchers3D's A Little Extra (High Elf animations by Starrfury). Full credits in the README.", "files": [{"asset": "patch-O.mpq", "dest": "Data/patch-O.mpq", "sha256": "x", "size": 9315256}, {"asset": "patch-Z.mpq", "dest": "Data/patch-Z.mpq", "sha256": "x", "size": 287750}, {"asset": "Patch-F.mpq", "dest": "Data/Patch-F.mpq", "sha256": "x", "size": 53234521}, {"asset": "Patch-H.mpq", "dest": "Data/Patch-H.mpq", "sha256": "x", "size": 433256}, {"asset": "patch-P.mpq", "dest": "Data/patch-P.mpq", "sha256": "x", "size": 7272085, "url": "https://pub-0f05631d243e4046993fc02ca7be9542.r2.dev/patches/patch-P.mpq", "source": "Project Reforged"}, {"asset": "patch-I.mpq", "dest": "Data/patch-I.mpq", "sha256": "x", "size": 162091469, "url": "https://pub-0f05631d243e4046993fc02ca7be9542.r2.dev/patches/patch-I.mpq", "source": "Project Reforged"}, {"asset": "patch-S.mpq", "dest": "Data/patch-S.mpq", "sha256": "x", "size": 262721653, "url": "https://pub-0f05631d243e4046993fc02ca7be9542.r2.dev/patches/patch-S.mpq", "source": "Project Reforged"}, {"asset": "patch-M.mpq", "dest": "Data/patch-M.mpq", "sha256": "x", "size": 438564005, "url": "https://pub-0f05631d243e4046993fc02ca7be9542.r2.dev/patches/patch-M.mpq", "source": "Project Reforged"}, {"asset": "patch-T.mpq", "dest": "Data/patch-T.mpq", "sha256": "x", "size": 541957195, "url": "https://pub-0f05631d243e4046993fc02ca7be9542.r2.dev/patches/extras/patch-T.mpq", "source": "Project Reforged"}, {"asset": "patch-E.mpq", "dest": "Data/patch-E.mpq", "sha256": "x", "size": 719685533, "url": "https://pub-0f05631d243e4046993fc02ca7be9542.r2.dev/patches/patch-E.mpq", "source": "Project Reforged"}, {"asset": "patch-G.mpq", "dest": "Data/patch-G.mpq", "sha256": "x", "size": 774538222, "url": "https://pub-0f05631d243e4046993fc02ca7be9542.r2.dev/patches/patch-G.mpq", "source": "Project Reforged"}, {"asset": "patch-B.mpq", "dest": "Data/patch-B.mpq", "sha256": "x", "size": 1608961816, "url": "https://pub-0f05631d243e4046993fc02ca7be9542.r2.dev/patches/patch-B.mpq", "source": "Project Reforged"}, {"asset": "patch-D.mpq", "dest": "Data/patch-D.mpq", "sha256": "x", "size": 1648472586, "url": "https://pub-0f05631d243e4046993fc02ca7be9542.r2.dev/patches/patch-D.mpq", "source": "Project Reforged"}, {"asset": "patch-A.mpq", "dest": "Data/patch-A.mpq", "sha256": "x", "size": 1748152913, "url": "https://pub-0f05631d243e4046993fc02ca7be9542.r2.dev/patches/patch-A.mpq", "source": "Project Reforged"}, {"asset": "patch-C.mpq", "dest": "Data/patch-C.mpq", "sha256": "x", "size": 2083036611, "url": "https://pub-0f05631d243e4046993fc02ca7be9542.r2.dev/patches/patch-C.mpq", "source": "Project Reforged"}]}, {"id": "nude", "name": "Nude skins (18+)", "version": "1.0.0", "nsfw": true, "description": "4× nude skins for Night Elf, Human, Troll and High Elf women.", "credits": "A Little Extra Retextured by Necropheus. Missing Forest Troll parts added by HD Switch.", "files": [{"asset": "Patch-Y.mpq", "dest": "Data/Patch-Y.mpq", "sha256": "x", "size": 50219767}]}]};
  const st = { hdswitch: { id: "hdswitch", status: "update", installed_version: "1.1.2" }, packs: { id: "packs", status: "different" }, nude: { id: "nude", status: "missing" } };
  let cfg = { wow_dir: "/Volumes/Yaman SSD/OctoWoW/Wow", nsfw_ok: location.hash.includes("nsfw") };
  const listeners = {};
  return {
    invoke: async (cmd, a) => {
      switch (cmd) {
        case "get_config": return cfg;
        case "save_config": cfg = a.config; return;
        case "find_wow_dirs": return [];
        case "check_wow_dir": return a.path;
        case "fetch_manifest": return manifest;
        case "component_status": return Object.values(st);
        case "game_running": return location.hash.includes("game");
        case "unsupported_packs": return location.hash.includes("conflict") ? ["patch-L.mpq"] : [];
        case "install": {
          for (const f of a.component.files.slice(0, 3)) for (let d = 0; d <= f.size; d += f.size / (location.hash.includes("demo") ? 400 : 20)) {
            await new Promise((r) => setTimeout(r, 30));
            (listeners.progress || []).forEach((fn) => fn({ payload: { component: a.component.id, file: f.asset, done: d, total: f.size } }));
          }
          st[a.component.id] = { id: a.component.id, status: "installed", installed_version: a.component.version };
          return;
        }
        case "uninstall": st[a.component.id] = { id: a.component.id, status: "missing" }; return;
      }
    },
    listen: (ev, fn) => { (listeners[ev] = listeners[ev] || []).push(fn); },
    pickFolder: async () => null,
    checkSelfUpdate: async () => (location.hash.includes("upd") ? { version: "1.0.1", downloadAndInstall: async () => {} } : null),
    relaunch: async () => {},
    appVersion: async () => "1.0.0",
  };
}

init().then(() => { if (MOCK && location.hash.includes("demo")) setTimeout(() => document.querySelector('.row[data-id="packs"] .gold').click(), 50); });
