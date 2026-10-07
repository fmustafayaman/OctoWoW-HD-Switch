#!/usr/bin/env python3
"""Builds the files to upload to a GitHub release, and manifest.json, from a release folder.

usage: make_release.py <release folder> <tag> [notes.md]

Expected in the folder:
  OctoWoW-HD-Switch-v<x>/      the HD Switch release (mods/, Interface/)
  Patch-F.mpq, Patch-H.mpq     female models, OctoWoW HD fixes
  patch-O.mpq, patch-Z.mpq     Twow Raid Visuals with OctoWoW's tables, DBC merge
  Patch-Y.mpq                  nude skins (18+)
  Patch-X.mpq                  optional: full (not "less") A Little Extra female models, loads over Patch-F
  Patch-W.mpq                  optional: bikini armor textures (A Little Extra's own Patch-Y, renamed)
  project-reforged.json        {letter: {url, sha256, size}} of the Project Reforged packs
                               in the supported setup, checked byte for byte against their server

Output: <folder>/upload/ (manifest.json plus every file to upload, flat names) and the HD Switch zip.
Project Reforged packs are not uploaded: the installer downloads them from Project Reforged.
"""
import glob, hashlib, json, os, shutil, sys, zipfile

SRC, TAG = sys.argv[1], sys.argv[2]
NOTES = open(sys.argv[3]).read() if len(sys.argv) > 3 else ""
OUT = os.path.join(SRC, "upload")
shutil.rmtree(OUT, ignore_errors=True)
os.makedirs(OUT)

(hs,) = glob.glob(os.path.join(SRC, "OctoWoW-HD-Switch-v*"))
HS_VER = os.path.basename(hs).rsplit("-v", 1)[1]
PACKS_VER = TAG.lstrip("v")


def sha(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def add(asset, src, dest):
    dst = os.path.join(OUT, asset)
    shutil.copyfile(src, dst)
    return {"asset": asset, "dest": dest, "sha256": sha(dst), "size": os.path.getsize(dst)}


def reforged(letter, pin):
    assert pin["url"].startswith("https://") and len(pin["sha256"]) == 64 and pin["size"] > 0, letter
    return {"asset": f"patch-{letter}.mpq", "dest": f"Data/patch-{letter}.mpq", "sha256": pin["sha256"],
            "size": pin["size"], "url": pin["url"], "source": "Project Reforged"}


pins = json.load(open(os.path.join(SRC, "project-reforged.json")))
assert sorted(pins) == sorted("ABCDEGIMPST"), sorted(pins)
addon = os.path.join(hs, "Interface", "AddOns", "HDSwitch")

components = [
    {
        "id": "hdswitch", "name": "HD Switch", "version": HS_VER,
        "description": "Turn your HD packs on and off from inside the game, without restarting. "
                       "It works with any letter packs, but only the OctoWoW HD packs below are tested "
                       "and supported.",
        "dll_line": "mods/HDToggle.dll",
        "files": [add("HDToggle.dll", os.path.join(hs, "mods", "HDToggle.dll"), "mods/HDToggle.dll")]
        + [add(f"HDSwitch-{n}", os.path.join(addon, n), f"Interface/AddOns/HDSwitch/{n}")
           for n in ("HDSwitch.toc", "HDSwitch.lua", "Bindings.xml")],
    },
    {
        "id": "packs", "name": "OctoWoW HD packs", "version": PACKS_VER,
        "description": "The supported setup, the same files byte for byte for everyone: Project Reforged "
                       "A, B, C, D, E, G, I, M, P, S and T (downloaded from Project Reforged), Twow Raid "
                       "Visuals, slimmer female models, fixes for defects in the packs (invisible trees, white "
                       "surfaces, an invisible creature) and the merged game tables.",
        "credits": "Project Reforged by Stormhand81 and contributors. Twow Raid Visuals by MarcelineVQ. "
                   "Female models: A Little Extra for Females, Less Thicc Version by Deezhugs, on "
                   "Watchers3D's A Little Extra (High Elf animations by Starrfury). Full credits in the README.",
        "files": [
            add("patch-O.mpq", os.path.join(SRC, "patch-O.mpq"), "Data/patch-O.mpq"),
            add("patch-Z.mpq", os.path.join(SRC, "patch-Z.mpq"), "Data/patch-Z.mpq"),
            add("Patch-F.mpq", os.path.join(SRC, "Patch-F.mpq"), "Data/Patch-F.mpq"),
            add("Patch-H.mpq", os.path.join(SRC, "Patch-H.mpq"), "Data/Patch-H.mpq"),
        ] + [reforged(l, pins[l]) for l in sorted(pins, key=lambda l: pins[l]["size"])],
    },
    {
        "id": "nude", "name": "Nude skins (18+)", "version": "1.0.0", "nsfw": True,
        "description": "4× nude skins for Night Elf, Human, Troll and High Elf women.",
        "credits": "A Little Extra Retextured by Necropheus. Missing Forest Troll parts added by HD Switch.",
        "files": [add("Patch-Y.mpq", os.path.join(SRC, "Patch-Y.mpq"), "Data/Patch-Y.mpq")],
    },
    {
        "id": "female-full", "name": "Little Extra", "version": "1.1.0",
        "description": "The original A Little Extra bodies for Night Elf, Human, Troll and High Elf women instead of "
                       "the slimmer Little Extra Less ones in the OctoWoW HD packs, with the faces fitted to the HD textures. "
                       "Remove it to go back to Little Extra Less.",
        "credits": "A Little Extra by Watchers3D (High Elf animations by Starrfury).",
        "files": [add("Patch-X.mpq", os.path.join(SRC, "Patch-X.mpq"), "Data/Patch-X.mpq")],
    },
    {
        "id": "bikini", "name": "Bikini armor", "version": "1.0.0",
        "description": "Revealing versions of many chest and leg armor textures, 4× resolution. Female textures, "
                       "plus a few unisex ones (Bloodfang sleeves and pants, some leather chests and pants) "
                       "that also show on men.",
        "credits": "A Little Extra by Watchers3D.",
        "files": [add("Patch-W.mpq", os.path.join(SRC, "Patch-W.mpq"), "Data/Patch-W.mpq")],
    },
]

dests = [f["dest"].lower() for c in components for f in c["files"]]
assert len(dests) == len(set(dests)), "a file is in two components"

json.dump({"release": TAG, "notes": NOTES, "components": components},
          open(os.path.join(OUT, "manifest-v2.json"), "w"), indent=1, ensure_ascii=False)

# manifest.json for installer 1.0: it looks every file up among this release's assets, so it gets
# only files hosted here, one component per pack as before, and is asked to update itself
own = {f["dest"].lower(): f for c in components for f in c["files"] if not f.get("url")}
legacy = [
    dict(components[0], description=components[0]["description"].split(" It works")[0]),
    {"id": "female", "name": "Little Extra Less", "version": "1.1.0",
     "description": "Slimmer female models for Night Elf, Human, Troll and High Elf, fitted to the HD textures.",
     "credits": "A Little Extra for Females, Less Thicc Version by Deezhugs, on Watchers3D's A Little Extra.",
     "files": [own["data/patch-f.mpq"]]},
    {"id": "trees", "name": "OctoWoW HD fixes", "version": PACKS_VER,
     "description": "Fixes for defects in the Project Reforged packs: invisible trees, white surfaces, "
                    "an invisible creature. Needs Project Reforged's packs A, C, D and G.",
     "credits": "Models from Project Reforged by Stormhand81 and contributors.",
     "files": [own["data/patch-h.mpq"]]},
    components[2],
]
json.dump({"release": TAG, "components": legacy,
           "notes": "**Update the installer** (the banner at the top) to get the full OctoWoW HD pack set.\n\n" + NOTES},
          open(os.path.join(OUT, "manifest.json"), "w"), indent=1, ensure_ascii=False)

# HD Switch zip for manual installs (same layout as earlier releases)
z = os.path.join(OUT, f"OctoWoW-HD-Switch-v{HS_VER}.zip")
with zipfile.ZipFile(z, "w", zipfile.ZIP_DEFLATED) as zf:
    for root, _, files in os.walk(hs):
        for f in files:
            if f.startswith("._") or f == ".DS_Store":
                continue
            p = os.path.join(root, f)
            zf.write(p, os.path.relpath(p, SRC))

for c in components:
    ext = sum(f["size"] for f in c["files"] if f.get("url"))
    own = sum(f["size"] for f in c["files"] if not f.get("url"))
    print(f"{c['id']:9} {len(c['files']):2} files  ours {own / 1048576:7.1f} MB  Project Reforged {ext / 1e9:5.2f} GB")
print("output:", OUT)
