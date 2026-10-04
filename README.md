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

## Tested with

**HD packs**

| File | Pack | Version |
|---|---|---|
| `patch-A.mpq` | Project Reforged – Player Characters & NPCs | 5.5.0 |
| `patch-B.mpq` | Project Reforged – Buildings | 5.0.0 |
| `patch-C.mpq` | Project Reforged – Creatures | 5.5.1 |
| `patch-D.mpq` | Project Reforged – Doodads | 5.4.2 |
| `patch-E.mpq` | Project Reforged – Environment | 5.4.1 |
| `patch-G.mpq` | Project Reforged – Gear & Weapons | 5.4.1 |
| `patch-I.mpq` | Project Reforged – Interface | 5.3.0 |
| `patch-M.mpq` | Project Reforged – Maps & Loading Screens | 5.4.2 |
| `patch-S.mpq` | Project Reforged – Sounds & Music | 5.3.4 |
| `patch-T.mpq` | Project Reforged – HD Character Textures & Gear (Standard) | 5.5.0 |
| `patch-O.mpq` | Twow Raid Visuals | 1.4.18 |
| `patch-Z.mpq` | DBC merge patch (combines the packs' tables) | handled automatically, cannot be turned off |

Project Reforged: <https://projectreforged.github.io/> · Twow Raid Visuals:
<https://github.com/MarcelineVQ/twow-raid-visuals>

Other letter packs (`patch-<letter>.mpq`) are listed and can be toggled too, but only
the packs above have been tested.

**Other mods loaded at the same time:** VanillaFixes, VanillaHelpers, nampower,
transmogfix, UnitXP_SP3, VanillaMultiMonitorFix.

**Platform:** macOS on Apple Silicon with WoWSilicon (Wine). Windows has not been tested yet.

## Installation

### With the installer (recommended)

Download **OctoWoW HD Installer** from [Releases](../../releases):
`OctoWoW-HD-Installer_*_Windows-x64-setup.exe` for Windows, `OctoWoW-HD-Installer_*_macOS-arm64.dmg`
for macOS (Apple Silicon).

It finds your OctoWoW folder (or lets you pick it), then installs, updates and removes
HD Switch and the [optional packs](#optional-packs) with one click. Every file is checked
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
| Key Bindings → **HD Switch** | Bind keys to the toggle and the panel |
| Character select / create: **HD: On** button | All packs off / on, the character on screen is rebuilt |

A toggle takes effect right away. The world reloads within a second or two; you may
see the landscape stream back in. If you click during a loading screen, the change is
applied as soon as you are in the world.

Your choice is saved in `mods\HDToggle.ini`.

## Optional packs

These are **modified versions of other people's work**. All credit for the models and
textures goes to their authors; the changes listed below are the only parts made for
HD Switch. Please support the original projects.

| Pack | Based on | What was changed |
|---|---|---|
| **Female models** (`Data\Patch-F.mpq`) | *A Little Extra for Females*, **Less Thicc Version** by **Deezhugs**, built on *A Little Extra* by **Watchers3D** (High Elf animations and weapon scaling by **Starrfury**) | Night Elf and Human heads re-mapped to OctoWoW's HD face textures (the originals were mapped for another texture set and looked broken). High Elves get OctoWoW's original eye glow back. Troll models unchanged. |
| **Invisible tree fix** (`Data\Patch-H.mpq`) | Seven tree models from **Project Reforged** *Patch-D Doodads* by **Stormhand81** and contributors | The models had no vertex weights, so the game drew nothing but still blocked your way (Durotar, Westfall, Redridge, Silverpine, Duskwood). Each vertex gets full weight on the model's single bone; nothing else changed. |
| **Nude skins, 18+** (`Data\Patch-Y.mpq`) | *A Little Extra Retextured* by **Necropheus**, upscaled for HD Reforged | The torso and pelvis parts of Forest Troll skin color 4 were missing (the HD underwear showed through with a different chest color). They were rebuilt from the same pack's skin texture. |

The female models replace those of Project Reforged's *Patch-L (A Little Extra)*. If you
have `patch-L.mpq` installed, remove it: as a later letter it would load over Patch-F.

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
- **Something looks wrong after a toggle:** `mods\HDToggle.log` describes every toggle
  (what was rebuilt, what was skipped and why). Please attach it, together with the
  newest file in your game's `Errors` folder if the game crashed, when you report a
  problem.

## Known limitations

- A texture that is still loading at the exact moment you toggle may keep its old look
  until the next toggle.
- Some HD pack files reference textures or sounds that the packs do not include. Those
  look and sound the same as with the pack installed normally; HD Switch does not
  change them.
- Windows has not been tested yet.

## Uninstall

With the installer: click **Remove** next to each pack.

By hand:

Remove the `mods/HDToggle.dll` line from `dlls.txt`. Then delete `mods\HDToggle.dll`,
`mods\HDToggle.ini`, `mods\HDToggle.log` and `Interface\AddOns\HDSwitch`. Your HD packs
are not changed by HD Switch; they stay installed exactly as they were.

## Changelog

**1.2.0**
- Works with OctoWoW's fallback login addresses (a bare IP in `realmlist.wtf`).
- 4× character skins: the skin canvas is sized for the highest-resolution skin pack.
- New: OctoWoW HD Installer for Windows and macOS.
- New optional packs: female models, invisible tree fix, nude skins (18+). See
  [Optional packs](#optional-packs) for credits.

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
people's work under their own terms; see [Optional packs](#optional-packs).

HD Switch is a fan-made tool. It is not affiliated with OctoWoW, Project Reforged,
Twow Raid Visuals or Blizzard Entertainment. World of Warcraft is a trademark of
Blizzard Entertainment.

Ai agents are used in project.
