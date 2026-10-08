# OctoWoW HD Switch

Turn your HD graphics packs on and off **from inside the game**, without restarting
and without logging out. Toggle them all with one click, or pick them one by one.

Textures, characters (faces, skin, hair, gear), creatures, buildings, doodads,
the world, spell effects and sounds all switch live.

> **OctoWoW only.** HD Switch works only while you are connected to
> [OctoWoW](https://octowow.st). On any other server it locks itself: every pack stays as
> installed and nothing can be toggled.

---

## Features

- **One-click toggle:** the minimap button turns every HD pack off. Click again and
  exactly the packs you had on come back.
- **Per-pack control:** right-click the minimap button to open a panel with a switch
  for each installed pack.
- **Live:** nothing is restarted. The world reloads in a second or two, and characters,
  NPCs and objects around you are rebuilt with the new files.
- **Character select and character create:** an **HD: On / Off** button at the top of
  both screens. On character create your race, sex, skin, face and hair are kept.
- **Remembers your choice:** it applies from the next game start, login screen included.
- **Fixes white textures:** some HD pack files are missing their smallest mip level,
  which made the cast bar and other textures turn white after a resolution change
  (or any time the game re-uploads textures). HD Switch completes them in memory;
  the pack files are not changed.
- **2× and 4× character skins:** works with VanillaHelpers' high-resolution skins. When
  the game starts, HD Switch measures each pack's skin resolution and sizes the skin
  canvas for the largest one, so a 4× skin pack works next to 2× packs. Skin parts in
  another resolution are scaled to fit their place on the body.
- **Works with OctoWoW's fallback login addresses:** a bare IP in `realmlist.wtf` is
  accepted as long as the selected realm is one of OctoWoW's.
- **Pack check:** `/hd check` (or **Check my packs** in the panel) compares every letter pack
  in your Data folder with the supported OctoWoW HD packs, byte for byte, and lists what is
  missing, in another version or not part of the set. Run it first when something looks wrong.
- **Key bindings and slash commands.**

## Requirements

| | |
|---|---|
| Server | **OctoWoW** |
| Client | The OctoWoW client: World of Warcraft **1.12.1, build 5875** |
| DLL loader | [VanillaFixes](https://github.com/hannesmann/vanillafixes) (loads the DLLs listed in `dlls.txt`) |
| HD packs | Installed as `Data\patch-<letter>.mpq` (see the tested packs below) |
| Optional | [VanillaHelpers](https://github.com/isfir/VanillaHelpers) for 2× character skins |

The DLL checks the game build before it does anything. On a client that is not 1.12.1
(5875), or if another mod has already changed the same game code, it stays inactive and
the addon tells you why.

## Supported setup

HD Switch is tested with one exact set of packs, and that set is the only one it is
supported with: the **OctoWoW HD packs**. Everyone who installs them gets the same files,
byte for byte. The installer checks every file's SHA-256.

| File | Pack | Version | Comes from |
|---|---|---|---|
| `patch-A.mpq` | Project Reforged – Player Characters & NPCs | 5.5.0 | Project Reforged |
| `patch-B.mpq` | Project Reforged – Buildings | 5.0.0 | Project Reforged |
| `patch-C.mpq` | Project Reforged – Creatures | 5.5.1 | Project Reforged |
| `patch-D.mpq` | Project Reforged – Doodads | 5.4.2 | Project Reforged |
| `patch-E.mpq` | Project Reforged – Environment | 5.4.1 | Project Reforged |
| `patch-G.mpq` | Project Reforged – Gear & Weapons | 5.4.1 | Project Reforged |
| `patch-I.mpq` | Project Reforged – Interface | 5.3.0 | Project Reforged |
| `patch-M.mpq` | Project Reforged – Maps & Loading Screens | 5.4.2 | Project Reforged |
| `patch-P.mpq` | Project Reforged – Particle Effects for Spells | 5.5.0 | Project Reforged |
| `patch-S.mpq` | Project Reforged – Sounds & Music | 5.3.4 | Project Reforged |
| `patch-T.mpq` | Project Reforged – HD Character Textures & Gear (Standard) | 5.5.0 | Project Reforged |
| `patch-O.mpq` | Twow Raid Visuals with OctoWoW's own spell tables | 1.4.18 | this repository's releases |
| `patch-Z.mpq` | Merged game tables (DBC) of the packs above | – | this repository's releases |
| `Patch-F.mpq` | Little Extra Less: female models, slimmer (see [the packs we modified](#packs-we-modified)) | 1.3.0 | this repository's releases |
| `Patch-H.mpq` | OctoWoW HD fixes (see [OctoWoW HD fixes](#octowow-hd-fixes)) | 1.2.2 | this repository's releases |
| `Patch-X.mpq` | Little Extra: full female models, **optional**, loads over Little Extra Less (`Patch-F`) | 1.3.0 | this repository's releases |
| `Patch-W.mpq` | Bikini armor, **optional** | 1.0.0 | this repository's releases |
| `Patch-Y.mpq` | Nude skins, 18+, **optional** | 1.0.0 | this repository's releases |

**Ultra HD (optional).** Project Reforged's 4× character textures. They come as a set of three
that replaces the standard `patch-T.mpq`; the installer swaps T in and back out by itself:

| File | Pack | Version | Comes from |
|---|---|---|---|
| `patch-T.mpq` | Project Reforged – HD Character & Gear Textures (**Ultra-base**), in place of the standard T | 5.5.0 | Project Reforged |
| `patch-U.mpq` | Project Reforged – Ultra HD Character Textures & Gear | 5.4.0 | Project Reforged |
| `Patch-V.mpq` | Fixes for Ultra HD (see [the packs we modified](#packs-we-modified)) | 1.0.0 | this repository's releases |

Ultra HD needs noticeably more video memory than the standard set.

**Every file of the current release, with its size, SHA-256 and a download link:**
[OctoWoW HD files](https://pub-a854abaa40ad406a86b1fc6390dd259c.r2.dev/index.html).

The Project Reforged packs are downloaded **from Project Reforged's own server**, not
re-hosted here; only the files that do not exist upstream in this exact form are part of
the releases. If Project Reforged publishes a new version of one of them, the installer
stops and says so instead of installing a version HD Switch was not tested with.

**Other packs.** HD Switch can be installed on its own and it lists and toggles any
`patch-<letter>.mpq`, but with other packs or other versions there is no guarantee that
everything looks and works right, and problems that only happen with them cannot be
fixed here. The installer's **Setup check** and HD Switch's `/hd check` show whether your
packs are exactly the supported ones, standard or Ultra HD, and list anything missing, in
another version or extra.

**Other mods loaded at the same time:** VanillaFixes, VanillaHelpers, nampower,
transmogfix, UnitXP_SP3, VanillaMultiMonitorFix.

**Platform:** macOS on Apple Silicon with WoWSilicon (Wine). Windows has not been tested yet.

## Installation

### With the installer (recommended)

Download **OctoWoW HD Installer** from [Releases](../../releases):
`OctoWoW-HD-Installer_*_Windows-x64-setup.exe` for Windows, `OctoWoW-HD-Installer_*_macOS-arm64.dmg`
for macOS (Apple Silicon).

It finds your OctoWoW folder (or lets you pick it), then installs, updates and removes
HD Switch, the [OctoWoW HD packs](#supported-setup) and the optional packs (Little Extra,
bikini armor, 18+ skins, Ultra HD) with one click. Its **Setup check** at the top tells you
whether your Data folder is exactly the supported setup (Standard HD with the full T, or Ultra HD
with U and its Ultra-base T) and names every file that is missing, in another version or extra.
HD Switch can also be installed on its own. Packs you already have in the right version
are not downloaded again, an interrupted download continues where it stopped, and
**Remove** only deletes files the installer put there. Every file is checked
against its SHA-256 before it is put in place, nothing is changed while the game is
running, and any file it replaces is backed up and restored when you remove the pack.
The installer updates itself, too.

The installer is not code-signed yet:
- **Windows:** SmartScreen says "Windows protected your PC". Click **More info**, then
  **Run anyway**.
- **macOS:** right-click the app, choose **Open**, then **Open** again.

### By hand

1. Download the latest `OctoWoW-HD-Switch-v*.zip` from
   [Releases](../../releases) and extract it.
2. Copy `mods\HDToggle.dll` into your game's `mods` folder (the folder VanillaFixes
   loads DLLs from).
3. Open `dlls.txt` in your game folder and add this line at the end:
   ```
   mods/HDToggle.dll
   ```
4. Copy the `Interface\AddOns\HDSwitch` folder into your game's `Interface\AddOns`.
5. **Restart the game once.** After that you never need to restart for a toggle.

When it is running you will see the HD Switch button on the minimap, and an
**HD: On** button at the top of the character select screen.

## Usage

| Action | Effect |
|---|---|
| Minimap button, left-click | All packs off / restore your selection |
| Minimap button, right-click | Panel: one switch per pack, plus **All on** / **All off** |
| Drag the minimap button | Moves it around the minimap |
| `/hd` | Opens the panel |
| `/hd on` · `/hd off` · `/hd toggle` | Turn on · turn off · toggle |
| `/hd check` · panel: **Check my packs** | Compares your packs with the supported OctoWoW HD packs (the first check reads every pack once, about a minute; later checks are instant) |
| Key Bindings → **HD Switch** | Bind keys to the toggle and the panel |
| Character select / create: **HD: On** button | All packs off / on, the character on screen is rebuilt |

A toggle takes effect right away. The world reloads within a second or two; you may
see the landscape stream back in. If you click during a loading screen, the change is
applied as soon as you are in the world.

Your choice is saved in `mods\HDToggle.ini`.

## Packs we modified

`Patch-F`, `Patch-H`, `Patch-V`, `Patch-X`, `Patch-W` and `Patch-Y` are **modified versions of other people's work**. All credit for the models and
textures goes to their authors; the changes listed below are the only parts made for
HD Switch. Please support the original projects.

| Pack | Based on | What was changed |
|---|---|---|
| **Little Extra Less** (`Data\Patch-F.mpq`) | *A Little Extra for Females*, **Less Thicc Version** by **Deezhugs**, built on *A Little Extra* by **Watchers3D** (High Elf animations and weapon scaling by **Starrfury**) | Night Elf and Human heads re-mapped to OctoWoW's HD face textures (the originals were mapped for another texture set and looked broken). The hairstyles OctoWoW added (Human styles 19-23, Night Elf 7-11) taken from OctoWoW's own models, so they no longer all look like the same short hair. High Elf eyes like the High Elf men's in Project Reforged: their glowing cyan eye from the men's face texture on the eyeball, and the men's soft light card in front of each eye, scaled to the women's face; the light fades as the eyelids close. Human: the animation events the original model had lost (37 animations: mining, fishing, sitting, emote sounds) restored from OctoWoW's HD Human. Troll models unchanged. |
| **Little Extra** (`Data\Patch-X.mpq`) | *A Little Extra* **0.9.9.5** by **Watchers3D** (High Elf animations and weapon scaling by **Starrfury**) | The same changes as Little Extra Less: Night Elf and Human heads re-mapped to OctoWoW's HD face textures, OctoWoW's added hairstyles, the High Elf eyes like the men's, the Human animation events restored, Troll models unchanged. Renamed from `Patch-F` so both can be installed; as a later letter it loads over Little Extra Less. |
| **Bikini armor** (`Data\Patch-W.mpq`) | The bikini pack of *A Little Extra* **0.9.9.5** by **Watchers3D** (published there as `Patch-Y`) | Renamed to `Patch-W` (it would clash with the nude skins) and its `VanillaHelpers\ResizeCharacterSkin.txt` (4×) removed: HD Switch takes the skin scale from the skin packs and fits these 4× textures itself, while the marker would force 4× skin layout over the 2× skins. Textures unchanged. The unisex (`_U`) textures in it show on men too, as in the original. |
| **Fixes for Ultra HD** (`Data\Patch-V.mpq`) | Textures from **Project Reforged** *patch-T 5.5.0 (Standard)* | 25 goblin textures copied unchanged. In `patch-U` they lost their transparency and covered the face in black: male goblins with hair styles 2, 3 and 9 or features 5-9, and a hard edge at the female goblins' hairline. Loads over `patch-U`; with HD Switch it turns off together with U. |
| **OctoWoW HD fixes** (`Data\Patch-H.mpq`) | Models from **Project Reforged** *Patch-A, C, D and G*, and original game models | Corrected copies of the HD pack files listed in [OctoWoW HD fixes](#octowow-hd-fixes) below; nothing else changed. |
| **Nude skins, 18+** (`Data\Patch-Y.mpq`) | *A Little Extra Retextured* by **Necropheus**, upscaled for HD Reforged | The torso and pelvis parts of Forest Troll skin color 4 were missing (the HD underwear showed through with a different chest color). They were rebuilt from the same pack's skin texture. |

`patch-O.mpq` is **Twow Raid Visuals 1.4.18** by **MarcelineVQ**
(<https://github.com/MarcelineVQ/twow-raid-visuals>) with OctoWoW's own spell, spell visual,
spell icon, sound and creature model tables, so OctoWoW's spells keep working. `patch-Z.mpq`
merges the game tables of all the packs above, so no pack's table hides another's.

The female models replace those of Project Reforged's *Patch-L (A Little Extra)*, which
is not part of the supported setup: as a later letter it would load over Patch-F.

### OctoWoW HD fixes

Every model, building and terrain file of the supported setup was checked with the game's own
loading rules (taken from `WoW.exe`): which model versions it accepts, how it reads a texture
name, and what it does when a texture is missing. Everything below showed up in the game and is
fixed in `Patch-H.mpq`. Each fix changes only the broken part; where the missing piece exists
nowhere, the original game's model is used.

| What you saw | Files | Cause | Fix |
|---|---|---|---|
| Invisible trees that still block your way (Durotar, Westfall, Redridge, Silverpine, Duskwood) | 7 tree models (D) | No vertex weights | Full weight on the model's single bone |
| White tree canopies, stumps and a rock (Redridge, Duskwood, Elwynn, Durotar) | 8 models (D) | Damaged texture names (a stray zero byte, a missing terminator) | The correct name of the texture that is already in the pack |
| White glow on the Horde PvP cloth helms | 36 models (G) | The glow texture is not in the pack | The texture the original helm uses there |
| White patch on the bat taxi | `BatTaxi.m2` (C) | Typo: `BatSkin000002` | `BatSkin02`, as in the original model |
| White eye glow on the High Elf NPC models | 2 models (A) | Typo: `EyeGlowBlue1` | The correct name |
| White surfaces on Gorth, the Worgen caster and a rug in the Night Elf hunters' lodge | 3 models (C, D) | No extension, `.bpl`, `… copy.blp` | The correct names |
| Invisible undead horse creatures | `UndeadHorse.m2` (C) | A WotLK model (version 264) without its `.skin` files; the game rejects it | The original model; the HD texture fits it |
| White trunk on a Moonglade tree | `moongladetree07.m2` (D) | The trunk texture is not in the pack | The original model; the HD textures fit it |
| White parts on the Northshire Abbey gates | `abbeygate01`, `abbeygate02` (D) | Three textures are not in the pack | The original gates |
| White smear on the Seal of Righteousness impact; white squares in the sparkles of the Swamp of Sorrows huts, in forge smoke and in the war raptor's exhaust | P, D, C | The texture is in no pack | A fully transparent texture: those parts are not drawn. Every use was checked to be a blended one, so nothing turns black |

What the check still lists is harmless: models that nothing in the game uses, texture slots no
surface uses, and one brazier inside the Night Elf hunters' lodge that is not drawn (a WotLK
model) but changes nothing else.

**With HD Switch:** when you turn a pack off, the fixes for that pack's files are off too, so
those files fall back exactly like the rest of the pack.

**Project Reforged** (<https://projectreforged.github.io/>) is made by **Stormhand81**
with **Watchers3D**, **Appletrey** (Classiic HD), **Isfir** (VanillaHelpers), **Vish**
(Faithful Upscale), **Bibsan / Space_slam** and **Kraulspine**; Patch-A builds on
**Falarson**'s work, updated by **Oran1**. *A Little Extra* thread:
<https://forum.turtlecraft.gg/viewtopic.php?t=12120>.

**To the authors:** if you would like your work removed from these packs, or credited
differently, please open an issue and it will be done right away.

## Troubleshooting

- **No minimap button / "HDToggle.dll is not loaded":** check that `mods/HDToggle.dll` is
  in `dlls.txt` and that you restarted the game after installing.
- **"HD Switch works on OctoWoW only":** you are not connected to OctoWoW (or picked a
  realm that is not OctoWoW's). Every pack stays on.
- **Something looks wrong or sounds missing:** run `/hd check` (or the installer's Setup check)
  first. A pack in another version, a missing one or an extra one causes most reports; the
  installer puts the supported files in place.
- **Something looks wrong after a toggle:** `mods\HDToggle.log` describes every toggle
  (what was rebuilt, what was skipped and why). For a detailed log, add the line `debug=2`
  under `[HDToggle]` in `mods\HDToggle.ini`, restart the game and toggle once: every texture,
  skin part, model and character rebuilt is written down. Please attach the log (and `dlls.txt`),
  together with the newest file in your game's `Errors` folder if the game crashed, when you
  report a problem.
- **"WARNING: another HDToggle.dll is loaded too" in the log, or `/hd check` says the DLL cannot
  check packs after an update:** an old copy from a manual install is still loaded. Delete it and
  its line in `dlls.txt`; only `mods/HDToggle.dll` should be listed. The installer (1.2.1 and
  later) removes such lines when it installs HD Switch.

## Known limitations

- A texture that is still loading at the exact moment you toggle may keep its old look
  until the next toggle.
- Some HD pack files reference textures or sounds that the packs do not include. Those
  look and sound the same as with the pack installed normally; HD Switch does not
  change them.
- Windows has not been tested yet.

## Uninstall

With the installer: click **Remove** next to each part. Packs you had before the
installer (for example Project Reforged packs you downloaded yourself) are left in place.

By hand:

Remove the `mods/HDToggle.dll` line from `dlls.txt`. Then delete `mods\HDToggle.dll`,
`mods\HDToggle.ini`, `mods\HDToggle.log` and `Interface\AddOns\HDSwitch`. Your HD packs
are not changed by HD Switch; they stay installed exactly as they were.

## Changelog

**1.2.7**
- **Faces broken after turning the packs off or on** (fixed): the character's own skin stayed in
  the previous pack's version (for example the 4× HD skin on OctoWoW's own model after turning
  everything off) until the next login. The skin parts the game keeps in memory without an alpha
  channel were not refreshed; now every part is, decoded the way the game asked for it.
- High Elf women's eyes like the men's: the glowing cyan eye of Project Reforged's High Elf men,
  with their soft light in front of the eyes, fading as the eyelids close (Little Extra and Little
  Extra Less).
- Installer 1.2.1: removes a second `HDToggle.dll` line from `dlls.txt` (left by an old manual
  install), which loaded two copies of HD Switch. HD Switch also warns in its log when it finds
  another copy loaded.
- Detailed troubleshooting log: `debug=1` or `debug=2` under `[HDToggle]` in `mods\HDToggle.ini`.
- Our packs are now downloaded from our own server (Cloudflare R2) instead of GitHub; the
  installer handles it by itself.

**1.2.6**
- **Ultra HD** in the installer, optional: Project Reforged's 4× character textures (`patch-U`)
  with its Ultra-base T in place of the standard T, plus **Fixes for Ultra HD** (`Patch-V`). The
  installer swaps the two T versions by itself; removing Ultra HD puts the standard T back.
- Fixes for Ultra HD: in `patch-U` 25 goblin textures lost their transparency, which turned
  male goblin faces black with hair styles 2, 3 and 9 or features 5-9 and gave the female
  goblins' hairline a hard edge. `Patch-V` brings back the versions of the standard set.
- Little Extra and Little Extra Less: the Human female model had lost the animation events of
  37 animations, so mining, fishing, sitting and many emotes made no sound. They are restored
  from OctoWoW's HD Human model.
- **Pack check:** `/hd check` or **Check my packs** in the HD Switch panel compares every letter
  pack in your Data folder with the supported packs (SHA-256, computed in the background and
  remembered while a file does not change) and names what is missing, different or extra.
- Installer 1.2.0: a **Setup check** at the top (Standard HD or Ultra HD, and every file that is
  missing, in another version or not part of the set).
- HD Switch names `patch-U` and `Patch-V` in its panel; turning U off turns its fixes off too.

**1.2.5**
- Little Extra and Little Extra Less: the hairstyles OctoWoW added (Human styles 19-23,
  Night Elf 7-11) were missing from these models and all showed as the same short hair.
  They are now taken from OctoWoW's own models. With them the Human model needed more
  triangles than the game can address in one list (it keeps each part's start in 16 bits),
  so the parts are ordered to stay under that limit.
- High Elf eye glow rebuilt in both packs: the eyes themselves glow in the High Elf blue
  (no more opaque blue eyeball next to the light), with a soft glow on the skin around them
  instead of a floating light card, so it looks right from the side too. The glow fades as
  the eyelids close, timed like the Night Elf glow, and no longer dims by itself while the eyes are open.

**1.2.4**
- Two new optional packs in the installer: **Little Extra** (`Patch-X`, the original
  *A Little Extra* bodies instead of the slimmer Little Extra Less, faces fitted to the HD textures) and
  **Bikini armor** (`Patch-W`). Both can be added or removed on their own; removing Little Extra
  brings Little Extra Less back. The nude skins (18+) stay optional as before.
- HD Switch names the Little Extra, Little Extra Less, bikini and nude skin packs in its panel, and
  pointing at a pack whose name is not enough explains what it does.

**1.2.3**
- Fixed a crash at the character select screen with `patch-T` on some PCs (ERROR #132,
  access violation at `0x0044B06B`). With 2× or 4× skins the game's texture pool, which
  only has slots for textures up to 256 pixels, was given the larger skin canvas and wrote
  outside its table. HD Switch now keeps the pool's size limit at the game's own value, so
  the skin canvas is allocated directly, as the game does whenever the pool is empty.
- HD Switch's version now matches the release, so the game no longer says a newer version
  is available when you already have the latest one.

**1.2.2**
- The invisible tree fix is now **OctoWoW HD fixes** (`Patch-H`): every defect a check of all
  models, buildings and terrain in the supported setup found, fixed (see
  [OctoWoW HD fixes](#octowow-hd-fixes)).
- HD Switch 1.2.1: turning a pack off also turns off the fixes for that pack's files.

**1.2.1**
- One supported setup: the installer offers the exact OctoWoW HD pack set HD Switch is
  tested with. Project Reforged packs are downloaded from Project Reforged and checked
  byte for byte; `patch-O`, `patch-Z`, `Patch-F` and `Patch-H` come from this release.
- HD Switch can still be installed on its own (no guarantee with other packs).
- Installer 1.1.0: resumes interrupted downloads, skips packs you already have, checks
  free disk space, lists packs that are not part of the supported setup, and only
  removes files it put there. HD Switch itself is unchanged (1.2.0).

**1.2.0**
- Works with OctoWoW's fallback login addresses (a bare IP in `realmlist.wtf`).
- 4× character skins: the skin canvas is sized for the highest-resolution skin pack.
- New: OctoWoW HD Installer for Windows and macOS.
- New optional packs: female models, invisible tree fix, nude skins (18+). See
  [Packs we modified](#packs-we-modified) for credits.

**1.1.2**
- Fixed a crash in character skin compositing right after turning packs off: the
  game's texture cache kept the old HD size of re-decoded face and skin textures.
  The outdated info is now dropped and recomputed by the game.

**1.1.1**
- Fixed a crash when toggling packs in the game (game objects such as campfires and
  chests were rebuilt with a wrong call into the game)
- Fixed white textures (cast bar border and others) after a resolution change or a
  toggle, caused by HD pack files with an incomplete mip chain
- Characters and game objects are now rebuilt before the world reloads, and every
  object is checked again right before it is touched

**1.1.0**
- First release

To update: replace `mods\HDToggle.dll` and the `Interface\AddOns\HDSwitch` folder,
then restart the game.

## License

Copyright © 2026 Mustafa Yaman. All rights reserved.

You may download and use HD Switch on OctoWoW. You may not redistribute it, modify it,
or use it with other servers without permission.

This license covers HD Switch and the installer only. The optional packs contain other
people's work under their own terms; see [Packs we modified](#packs-we-modified).

HD Switch is a fan-made tool. It is not affiliated with OctoWoW, Project Reforged,
Twow Raid Visuals or Blizzard Entertainment. World of Warcraft is a trademark of
Blizzard Entertainment.

Ai agents are used in project.
