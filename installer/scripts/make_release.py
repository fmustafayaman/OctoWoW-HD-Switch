#!/usr/bin/env python3
"""Builds the files to upload to a GitHub release, and manifest.json, from a release folder.

usage: make_release.py <release folder> <tag> [notes.md]
Expected in the folder: OctoWoW-HD-Switch-v<x>/ (mods/, Interface/), Patch-F.mpq, Patch-H.mpq, Patch-Y.mpq
Output: <folder>/upload/ (manifest.json plus every file to upload, flat names) and the HD Switch zip.
Credits come from the CREDIT_FEMALE, CREDIT_TREES and CREDIT_NUDE environment variables.
"""
import hashlib, json, os, shutil, sys, zipfile

SRC, TAG = sys.argv[1], sys.argv[2]
NOTES = open(sys.argv[3]).read() if len(sys.argv) > 3 else ""
VER = TAG.lstrip("v")
OUT = os.path.join(SRC, "upload")
shutil.rmtree(OUT, ignore_errors=True)
os.makedirs(OUT)


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


hs = os.path.join(SRC, f"OctoWoW-HD-Switch-v{VER}")
addon = os.path.join(hs, "Interface", "AddOns", "HDSwitch")
components = [
    {
        "id": "hdswitch", "name": "HD Switch", "version": VER,
        "description": "Turn your HD packs on and off from inside the game, without restarting.",
        "dll_line": "mods/HDToggle.dll",
        "files": [add("HDToggle.dll", os.path.join(hs, "mods", "HDToggle.dll"), "mods/HDToggle.dll")]
        + [add(f"HDSwitch-{n}", os.path.join(addon, n), f"Interface/AddOns/HDSwitch/{n}")
           for n in ("HDSwitch.toc", "HDSwitch.lua", "Bindings.xml")],
    },
    {
        "id": "female", "name": "Female models", "version": "1.0.0",
        "description": "Slimmer female models for Night Elf, Human, Troll and High Elf, fitted to the HD textures. "
                       "High Elves keep the original eye glow.",
        "credits": os.environ.get("CREDIT_FEMALE", ""),
        "files": [add("Patch-F.mpq", os.path.join(SRC, "Patch-F.mpq"), "Data/Patch-F.mpq")],
    },
    {
        "id": "trees", "name": "Invisible tree fix", "version": "1.0.0",
        "description": "Fixes trees in Durotar, Westfall, Redridge, Silverpine and Duskwood that are invisible "
                       "but still block your way. Needs the HD pack D.",
        "credits": os.environ.get("CREDIT_TREES", ""),
        "files": [add("Patch-H.mpq", os.path.join(SRC, "Patch-H.mpq"), "Data/Patch-H.mpq")],
    },
    {
        "id": "nude", "name": "Nude skins (18+)", "version": "1.0.0", "nsfw": True,
        "description": "4× nude skins for Night Elf, Human, Troll and High Elf women.",
        "credits": os.environ.get("CREDIT_NUDE", ""),
        "files": [add("Patch-Y.mpq", os.path.join(SRC, "Patch-Y.mpq"), "Data/Patch-Y.mpq")],
    },
]
json.dump({"release": TAG, "notes": NOTES, "components": components},
          open(os.path.join(OUT, "manifest.json"), "w"), indent=1, ensure_ascii=False)

# HD Switch zip for manual installs (same layout as earlier releases)
z = os.path.join(OUT, f"OctoWoW-HD-Switch-{TAG}.zip")
with zipfile.ZipFile(z, "w", zipfile.ZIP_DEFLATED) as zf:
    for root, _, files in os.walk(hs):
        for f in files:
            if f.startswith("._") or f == ".DS_Store":
                continue
            p = os.path.join(root, f)
            zf.write(p, os.path.relpath(p, SRC))

for c in components:
    print(f"{c['id']:9} {sum(f['size'] for f in c['files']) / 1048576:7.1f} MB  {len(c['files'])} dosya")
print("output:", OUT)
