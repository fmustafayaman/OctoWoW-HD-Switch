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
  if (state.config.wow_dir) {
    const c = await api.invoke("conflicts", { wowDir: state.config.wow_dir });
    $("conflict").hidden = !c.length;
    $("conflict").textContent = c.length ? `${c.join(", ")} (Project Reforged's A Little Extra) loads over the female models. Remove it from your Data folder to see them.` : "";
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
  if (c.id === "hdswitch") node.classList.add("core");
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
    badge.textContent = STATUS_TEXT[st.status] + (st.installed_version && st.status !== "installed" ? " (have " + st.installed_version + ")" : "") + (st.status === "installed" ? " · " + c.version : "");
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
  if (s === "update" || s === "different") btn(s === "different" ? "Replace" : "Update", "gold", () => run("install", c));
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
const mb = (b) => (b / 1048576 >= 10 ? (b / 1048576).toFixed(0) : (b / 1048576).toFixed(1)) + " MB";

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
  const manifest = {
    release: "v1.2.0",
    notes: "HD Switch 1.2.0\n- Works with OctoWoW's fallback login IPs.\n- 4× character skins.\n\nFemale models\n- Night Elf and Human faces fitted to the HD textures.\n- High Elf eye glow like the original game.",
    components: [
      { id: "hdswitch", name: "HD Switch", version: "1.2.0", description: "Turn your HD packs on and off from inside the game, without restarting.", files: [{ asset: "HDToggle.dll", dest: "mods/HDToggle.dll", sha256: "x", size: 66560 }] },
      { id: "female", name: "Female models", version: "1.0.0", description: "Night Elf, Human, Troll and High Elf female models, fitted to the HD textures. High Elves keep their eye glow.", credits: "Models: A Little Extra by Watchers3d. Adapted for OctoWoW HD.", files: files("Patch-F.mpq", 51) },
      { id: "trees", name: "Invisible tree fix", version: "1.0.0", description: "Fixes trees in Durotar, Westfall, Redridge, Silverpine and Duskwood that are invisible and block your path.", credits: "Fix for Project Reforged pack D.", files: files("Patch-H.mpq", 0.4) },
      { id: "nude", name: "Nude skins (18+)", version: "1.0.0", nsfw: true, description: "4× nude skins for Night Elf, Human, Troll and High Elf women.", credits: "Skins: Patch-Y.", files: files("Patch-Y.mpq", 48) },
    ],
  };
  const st = { hdswitch: { id: "hdswitch", status: "update", installed_version: "1.1.2" }, female: { id: "female", status: "installed", installed_version: "1.0.0" }, trees: { id: "trees", status: "missing" }, nude: { id: "nude", status: "missing" } };
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
        case "conflicts": return location.hash.includes("conflict") ? ["Data/patch-L.mpq"] : [];
        case "install": {
          const f = a.component.files[0];
          for (let d = 0; d <= f.size; d += f.size / (location.hash.includes("demo") ? 400 : 20)) {
            await new Promise((r) => setTimeout(r, 60));
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

init().then(() => { if (MOCK && location.hash.includes("demo")) setTimeout(() => document.querySelector('.row[data-id="female"] .gold, .row[data-id="trees"] .gold').click(), 50); });
