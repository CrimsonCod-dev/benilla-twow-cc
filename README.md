<div align="center">
  <h1>benilla</h1>
  <p><b>A complete World of Warcraft 1.12.1 client written from scratch in Rust and <a href="https://bevy.org">Bevy</a></b></p>
  <p><i>Enhanced for TurtleWoW protocol compatibility and multi-platform play</i></p>
  <p>
    <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue?style=for-the-badge" alt="License"></a>
    <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/language-Rust-orange?style=for-the-badge&logo=rust" alt="Rust"></a>
    <a href="https://bevy.org"><img src="https://img.shields.io/badge/engine-Bevy_0.18-fd366e?style=for-the-badge" alt="Bevy"></a>
  </p>
</div>

---

**benilla** is a complete, modern, clean-room implementation of the World of Warcraft 1.12.1 client built in Rust using the [Bevy](https://bevy.org) game engine. It connects to 1.12.1 servers over the original network protocol and streams assets directly from your local 1.12.1 game installation.

Every asset reader, network layer, rendering pipeline, and interface engine is implemented from scratch with **zero original client code**, **no bundled game assets**, and **no third-party WoW binary dependencies**.

This fork (`benilla-twow`) extends **benilla** with TurtleWoW client patch compatibility, cross-platform state persistence, Android touch controls, and enhanced font engine support.

---

## Key Features

- **Asset Stack & Formats:** Readers for MPQ patch chains, BLP textures, DBC databases, ADT terrain, WDT/WDL map structures, M2 models, and WMO world objects — exposed natively to Bevy as an asset source.
- **World & Rendering:** Horizon terrain streaming, portal-culled WMO interiors with dynamic lighting, doodads, ground clutter, swimmable water, sky/weather, and authentic day/night lighting, fog, and gamma control.
- **Models & Characters:** GPU-skinned M2 models with full animation controllers, particles, ribbons, spell effects, full character customization, armor composite texturing, weapon enchant glows, shapeshift forms, stealth, and mounts.
- **Movement & Physics:** Server-authoritative movement, mounts, roots, slow fall, water walking, flight paths, boats/zeppelins, and follow camera with terrain collision.
- **Networking:** SRP6 authentication, world-session RC4 encryption, ECS-synced object mirroring, chat, combat, spells, groups/raids, quests, trading, mail, auction house, and battlegrounds.
- **Interface & Addons:** Custom FrameXML engine and embedded Lua 5.1 dialect. Runs stock 1.12 interface UI as well as popular 1.12 addons (**Questie**, **pfUI**, **Bagnon**, **Bartender2**, **MikScrollingBattleText**, etc.).
- **Audio:** Spatialized music, ambience, and sound effects via the [Kira](https://github.com/tesselode/kira) audio engine with zone reverb and interior/underwater transitions.

---

## TurtleWoW Compatibility

While **benilla** speaks standard 1.12.1 protocol out of the box, this fork bridges the functional gaps introduced by TurtleWoW's custom client patch:

- **Custom Playable Races:** Full support for **Goblin** (Horde) and **High Elf / Blood Elf** (Alliance) across character creation, UI race-select icons, equipment fitting, and helm/armor model variations.
- **Lua & FrameXML Enhancements:**
  - Implemented missing `getfenv` and `setfenv` Lua globals required by TurtleWoW's `Globals.lua` and UI utility functions (`wipe`, `trim`, `explode`, `sizeof`, `print`).
  - Extended camera Lua API with `CameraZoomIn`, `CameraZoomOut`, and `FlipCameraYaw`.
  - Native loading support for `json.lua` and TurtleWoW's custom FrameXML load order via `benilla.toc`.
  - Fallback slider bounds for TurtleWoW's revised Graphics Options interface (`OptionsFrame.lua`).
- **Resilient Audio Handling:** Missing sound kit IDs and stream URLs (such as Everlook's internet radio tower paths) warn once and gracefully output silence instead of spamming errors.
- **Unenforced Warden Packets:** Ignores incoming `SMSG_WARDEN_DATA` packets without dropping the connection for servers that send Warden data without requiring client execution.

---

## Platform Support & Touch Controls

**benilla** targets Desktop platforms and Android from a single unified codebase.

| Platform | Support Status | Config & Local State Path |
| :--- | :--- | :--- |
| **Windows** | Fully supported & actively developed | `%USERPROFILE%\Documents\benilla-twow\benilla-config` |
| **Android** | Supported (Target: NativeActivity, API 30+) | `/sdcard/Android/media/com.benilla.twow/benilla-config` |
| **Linux** | Supported | `<project-root>/benilla-config` or `<exe-dir>/benilla-config` |
| **macOS** | Supported | `<project-root>/benilla-config` or `<exe-dir>/benilla-config` |

### Touch Controls (Android / Mobile)

When running on Android (or with `WOW_TOUCH=1` forced):
- **Dynamic Virtual Joystick:** Located in the bottom-left screen area; automatically re-centers wherever your left thumb touches, controlling WASD movement.
- **Free-Swipe Camera Look:** Touching anywhere outside the movement joystick maps directly to camera orbit/look.
- **Finger Classification:** Each touch input is classified on touchdown (Joystick, UI, or Camera) and maintains its role until release to prevent dropped input events.
- **Soft Keyboard Integration:** Tapping chat edit boxes or text input fields automatically invokes the native Android soft keyboard.

---

## Configuration & Local State (`benilla-config`)

**benilla** stores all user state, settings, logs, and screenshots inside a dedicated `benilla-config/` directory. It **never writes to or modifies your original WoW installation**.

### Directory Structure

```text
benilla-config/
├── config.toml           # CVar overrides and client settings (Config.wtf equivalent)
├── account               # Saved account name
├── password              # Saved password (when "Remember Password" is checked)
├── AddOns/               # User 1.12 addons
├── Fonts/                # Custom TrueType font overrides & fallbacks
├── addon-state/          # Per-character addon enable/disable states
├── bindings/             # Key bindings (account and per-character)
├── macros/               # Account and character macros
├── saved/                # SavedVariables for addons (account & character scope)
├── camera/               # Saved third-person camera positions
├── chat/                 # Chat frame layouts, channels, font sizes, and tints
├── layout/               # Frame positions and UI layout cache
├── cache/                # Name cache TSV files per realm
├── Logs/                 # WoWChatLog.txt and WoWCombatLog.txt
└── Screenshots/          # Screen captures taken in-game
```

### Path Resolution Order

Local state resolves in the following order:
1. `$BENILLA_HOME` environment variable (if explicitly set).
2. Temporary directory `/tmp/benilla-run-<pid>/benilla-config` (unattended / test mode).
3. **Windows:** `%USERPROFILE%\Documents\benilla-twow\benilla-config` via `SHGetKnownFolderPath(FOLDERID_Documents)`.
4. **Android:** `/sdcard/Android/media/com.benilla.twow/benilla-config`.
5. `<project-root>/benilla-config` (development builds) or `<exe-dir>/benilla-config` (release binaries).

### Custom Font Engine & Overrides

Place custom TrueType (`.ttf`), OpenType (`.otf`), or TTC (`.ttc`) files in `benilla-config/Fonts/` (or `fonts/`):

1. **Standard Client Font Overrides:**
   Files matching any of the 5 primary font basenames override the game's default UI typefaces:
   - `ARIALN.ttf`
   - `FRITZQT__.ttf`
   - `MORPHEUS.ttf`
   - `SKURRI.ttf`
   - `WarSansTT-Bliz-500.ttf`

2. **International Character Fallback (CJK / Cyrillic):**
   Any additional font files dropped into `benilla-config/Fonts/` that do not match the standard basenames are automatically registered as **coverage fallback fonts**. When rendering text containing CJK or Cyrillic characters not present in the standard Latin faces, the font engine seamlessly falls back to these additional faces.

---

## Building and Running

### Prerequisites

1. **Client Data:** An English World of Warcraft 1.12.1 client (build 5875).
2. **Server:** A 1.12.1 server core (e.g. [vmangos](https://github.com/vmangos/core), cMaNGOS, or TurtleWoW).
3. **Build Toolchain:**
   - **Rust:** Stable toolchain (`rustc`, `cargo`).
   - **Windows:** MSVC Build Tools.
   - **Linux:** `pkg-config`, `libudev-dev`, `libalsa-dev` (or `alsa-lib-devel`).
   - **macOS:** Xcode Command Line Tools.

### Running from Source

Set the `WOW_DATA` environment variable to point to your client's `Data` directory:

#### Linux / macOS (Bash)
```bash
WOW_DATA="/path/to/WoW/Data" cargo run --release -p benilla
```

#### Windows (PowerShell)
```powershell
$env:WOW_DATA="C:\path\to\WoW\Data"; cargo run --release -p benilla
```

Alternatively, create a directory link or junction named `WoW` at the repository root pointing to your WoW directory:
```powershell
# Windows PowerShell (Junction)
New-Item -ItemType Junction -Path WoW -Target "C:\path\to\WoW"
```

### Useful Environment Variables

| Variable | Description |
| :--- | :--- |
| `WOW_DATA` | Path to the 1.12.1 `Data` directory (containing `.MPQ` files). |
| `WOW_HOST` | Target server address (default: `localhost:3724`). Example: `play.example.com:3724`. |
| `WOW_USER` | Account username for auto-login. |
| `WOW_PASS` | Account password for auto-login. |
| `WOW_TOUCH` | Set to `1` to force mobile virtual joystick and touch camera controls. |
| `BENILLA_HOME` | Custom path override for the `benilla-config` state directory. |

---

## Codebase Architecture

The project is structured as a Cargo workspace with distinct, modular crates:

- `crates/benilla`: Entry-point binary and app launcher.
- `crates/benilla-app`: Core Bevy plugin suite, ECS systems, UI bridge, player controllers, and audio.
- `crates/benilla-assets`: MPQ patch chain loader and asset streaming pipeline.
- `crates/benilla-formats`: Format parsers for BLP, DBC, ADT, WDT, WDL, M2, and WMO.
- `crates/benilla-protocol`: World of Warcraft 1.12.1 network packet definitions and serialization.
- `crates/benilla-srp`: SRP6 authentication cryptography.
- `crates/benilla-ui`: FrameXML parser, layout engine, and Lua 5.1 environment wrapper.

Detailed subsystem design and architectural guidelines can be found in:
- [`docs/MAP.md`](docs/MAP.md) — Comprehensive crate and module map.
- [`docs/METHOD.md`](docs/METHOD.md) — System design rules and technical constraints.
- [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) — Development guidelines and PR standards.

---

## Disclaimer & Licensing

**benilla** is an independent, open-source fan project. It is **not** affiliated with, endorsed by, or associated with Blizzard Entertainment, Inc.

- **No Proprietary Assets:** This repository contains no Blizzard copyrighted material — no artwork, models, audio, maps, or FrameXML files. All game assets are read at runtime from the user's legally owned 1.12.1 client installation.
- **License:** Open source code in this repository is dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).
- Vendored components under `third_party/` ([Kira audio engine](third_party/kira) and [Lua 5.1 dialect](third_party/lua-src)) retain their respective upstream open-source licenses.
