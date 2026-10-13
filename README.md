<div align="center">
  <h1>benilla</h1>
  <p><strong>A complete World of Warcraft 1.12.1 client, written from scratch in Rust and <a href="https://bevy.org">Bevy</a>.</strong></p>
  <p>
    <a href="https://github.com/CrimsonCod-dev/benilla-twow-cc/actions/workflows/ci_build.yml"><img src="https://img.shields.io/github/actions/workflow/status/CrimsonCod-dev/benilla-twow-cc/ci_build.yml?branch=main&label=CI" alt="CI"></a>
    <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue" alt="License: MIT / Apache-2.0"></a>
    <a href="https://discord.gg/wJSJx467G4"><img src="https://img.shields.io/discord/1529280129518538922?style=flat&logo=discord&logoColor=white&label=Discord&color=5865F2" alt="Discord"></a>
    <a href="https://www.youtube.com/playlist?list=PLdCnpZNKxyb8"><img src="https://img.shields.io/badge/devlog-YouTube-FF0000?style=flat&logo=youtube&logoColor=white" alt="YouTube devlog"></a>
  </p>
  <p>
    <code>benilla-twow-cc</code> is a fork of
    <a href="https://github.com/jhinzuo2/benilla-twow">jhinzuo2/benilla-twow</a><br>
    Upstream project: <a href="https://github.com/samwhosung/benilla">samwhosung/benilla</a>
  </p>
</div>

---

## Overview

> [!IMPORTANT]
> This is a fork, not the upstream benilla repository. Feature, compatibility, and platform statements describe the current state of this repository and inherit upstream documentation where applicable.

**benilla** plays the whole game: character creation, questing and professions, dungeons and raids, battlegrounds and honor, groups, guilds, trade, mail, and the auction house. It uses the stock 1.12 interface and runs third-party 1.12 addons.

The client connects to a 1.12.1 server over the original protocol and reads game data directly from your own legally obtained 1.12.1 installation. Every file format parser, the network protocol layer, and the interface engine are written from scratch with **zero original client code**, **no third-party WoW crates**, and **no bundled Blizzard game assets**.

This repository (`benilla-twow-cc`) tracks the TurtleWoW/Android fork lineage and incorporates TurtleWoW custom client patch compatibility, multi-platform state persistence, Android touch controls, and international font rendering support.

---

## Contents

- [Fork Lineage & Status](#fork-lineage--status)
- [Key Features](#key-features)
- [Platform Support & Status](#platform-support--status)
- [Quick Start](#quick-start)
- [Configuration & Local Files (`benilla-config`)](#configuration--local-files-benilla-config)
- [Android Setup & Touch Controls](#android-setup--touch-controls)
- [TurtleWoW Compatibility](#turtlewow-compatibility)
- [Development & Verification](#development--verification)
- [Contributing](#contributing)
- [License and Legal](#license-and-legal)

---

## Fork Lineage & Status

- **Original Project:** [samwhosung/benilla](https://github.com/samwhosung/benilla) — The clean-room World of Warcraft 1.12.1 client written in Rust and Bevy.
- **Parent Fork:** [jhinzuo2/benilla-twow](https://github.com/jhinzuo2/benilla-twow) — Adds TurtleWoW compatibility work, custom races, and the Android entry point.
- **This Repository:** [CrimsonCod-dev/benilla-twow-cc](https://github.com/CrimsonCod-dev/benilla-twow-cc) — Maintenance fork incorporating build fixes, platform config handling, and expanded documentation.
- **Issue Tracking:** Local GitHub Issues are disabled in this fork. TurtleWoW-specific gaps inherited from the parent project remain tracked in the parent fork repository.
- **Artifacts:** Desktop binaries and Android APK packages are built directly by CI workflows (`.github/workflows/ci_build.yml`).

---

## Key Features

| Subsystem | Implementation Details |
| :--- | :--- |
| **Formats** | MPQ patch chains, BLP textures, DBC databases, ADT terrain, WDT/WDL map structures, M2 models, and WMO world objects integrated as Bevy asset sources. |
| **World** | Horizon-distance terrain streaming, portal-culled buildings and interiors, dynamic interior lighting, doodads, ground clutter, swimmable water, sky/weather, day/night lighting, fog, and gamma. |
| **Models** | GPU-skinned M2 models, animation controller, particles, ribbons, spell visuals, character customization, armor composite texturing, weapon enchant glows, shapeshift forms, stealth, and mounts. |
| **Movement** | Bidirectional networked movement, server-granted movement modes (roots, slow fall, water walking), collision-aware follow camera, boats, zeppelins, and flight paths. |
| **Networking** | SRP6 authentication, world-session RC4 crypto, and ECS object mirroring covering movement, chat, combat, spells, groups, raids, quests, trade, mail, auction house, and battlegrounds. |
| **Interface** | FrameXML and Lua 5.1 engine running stock 1.12 UI as well as third-party 1.12 addons (**Questie**, **pfUI**, **Bagnon**, **Bartender2**, **MikScrollingBattleText**, etc.). |
| **Audio** | Music, ambience, and sound effects using the [Kira](https://github.com/tesselode/kira) engine with zone reverb, interior/underwater transitions, and crossfade rules. |

---

## Platform Support & Status

**benilla** targets Desktop platforms and Android from a single source tree.

| Platform | Build Status | Runtime Status | Config & Local State Location |
| :--- | :--- | :--- | :--- |
| **Windows** | CI verified (`windows-latest`) | Actively developed & verified | `%USERPROFILE%\Documents\benilla-twow\benilla-config` |
| **Android** | CI verified (`aarch64-linux-android`) | Touch controls implemented; APK signed in CI | `/sdcard/Android/media/com.benilla.twow/benilla-config` |
| **Linux** | CI verified (`ubuntu-latest`) | Primary Bevy development target | `<project-root>/benilla-config` or `<exe-dir>/benilla-config` |
| **macOS** | CI verified (`macos-latest`) | Inherited build coverage | `<project-root>/benilla-config` or `<exe-dir>/benilla-config` |

---

## Quick Start

### 1. Clone the Repository

```sh
git clone https://github.com/CrimsonCod-dev/benilla-twow-cc.git
cd benilla-twow-cc
```

### 2. Prerequisites

- **Game Data:** An English World of Warcraft 1.12.1 client (build 5875). **benilla** only reads this data.
- **Server:** A 1.12.1 server core with Warden disabled (e.g. [vmangos](https://github.com/vmangos/core), cMaNGOS, or TurtleWoW).
- **Rust Toolchain:** Stable Rust **1.98.1** (pinned in [`rust-toolchain.toml`](rust-toolchain.toml)).
- **Build Dependencies:**
  - **Linux (Ubuntu/Debian):** `pkg-config`, `libasound2-dev`, `libudev-dev`.
  - **macOS:** Xcode Command Line Tools.
  - **Windows:** MSVC C++ Build Tools.

### 3. Provide Game Data Location

Set the `WOW_DATA` environment variable:

```sh
export WOW_DATA=/path/to/WoW/Data
```

Or create a `WoW` symlink at the root of the repository:

```sh
ln -s /path/to/WoW WoW
```

On Windows (PowerShell junction, no admin rights required):

```powershell
New-Item -ItemType Junction -Path WoW -Target "C:\path\to\WoW"
```

### 4. Build and Run

```sh
cargo run --release -p benilla
```

On Windows (PowerShell):

```powershell
$env:WOW_DATA="C:\path\to\WoW\Data"; cargo run --release -p benilla
```

For optimized player builds without dev tools, use the `ship` profile:

```sh
cargo build --profile ship --locked --no-default-features -p benilla
```

The compiled binary will be located in `target/ship/`.

### 5. Logging In

The client connects to `localhost:3724` by default. Point `WOW_HOST` at a remote server (e.g. `WOW_HOST=play.example.com` or `play.example.com:3724`) or enter the address using the Realmlist button on the login screen. Auto-login can be configured via `WOW_USER` and `WOW_PASS`.

---

## Configuration & Local Files (`benilla-config`)

**benilla** keeps all user settings, logs, UI layouts, and screenshots inside a dedicated `benilla-config` directory and **never writes to your original WoW installation**.

### Environment Variables

| Variable | Description |
| :--- | :--- |
| `WOW_DATA` | Path to the 1.12.1 `Data` directory (containing `.MPQ` files). |
| `WOW_HOST` | Target server host address and optional port (default: `localhost:3724`). |
| `WOW_USER` / `WOW_PASS` | Optional credentials for auto-login. |
| `WOW_TOUCH` | Force touch controls on desktop (`1` enables, `0` disables). |
| `BENILLA_HOME` | Custom directory path override for `benilla-config`. |

### Config Path Resolution Order

1. `$BENILLA_HOME` (when explicitly set).
2. Temporary directory `/tmp/benilla-run-<pid>/benilla-config` (unattended / test runs).
3. **Windows:** `%USERPROFILE%\Documents\benilla-twow\benilla-config` via Windows shell API (`SHGetKnownFolderPath`).
4. **Android:** `/sdcard/Android/media/com.benilla.twow/benilla-config`.
5. `<project-root>/benilla-config` (development builds) or `<exe-dir>/benilla-config` (release binaries).

### Addons and Font Overrides

- **Addons:** Place 1.12 addons in `<benilla-config>/AddOns/`.
- **Fonts:** Custom TrueType/OpenType font files live in `<benilla-config>/Fonts/` (or `fonts/`):
  - Standard font overrides match the 5 primary face basenames:
    - `ARIALN.ttf`
    - `FRITZQT__.ttf`
    - `MORPHEUS.ttf`
    - `SKURRI.ttf`
    - `WarSansTT-Bliz-500.ttf`
  - **International Fallback (CJK / Cyrillic):** Any additional `.ttf`, `.otf`, or `.ttc` files placed in `Fonts/` that do not replace a standard face are automatically registered as coverage fallback fonts for missing CJK and Cyrillic glyphs.

---

## Android Setup & Touch Controls

The Android target uses package ID `com.benilla.twow` and targets `aarch64-linux-android` (Android API 26+ / Android 8.0+).

### Data & State Directory Structure

1. Copy the 1.12.1 `Data` directory to:
   ```text
   /sdcard/Android/media/com.benilla.twow/WOWDATA
   ```
2. Android configuration and local state are maintained at:
   ```text
   /sdcard/Android/media/com.benilla.twow/benilla-config
   ```

### Touch Control Scheme

- **Floating Virtual Joystick:** Located on the bottom-left screen area; dynamically re-centers under your thumb on touchdown to control movement (WASD equivalent).
- **Free-Swipe Camera Look:** Swiping anywhere outside the movement joystick controls camera orbit and direction.
- **Touch Role Locking:** Touches are classified on touchdown (Joystick, UI, or Camera) and retain their role until lifted to prevent accidental input drops.
- **Soft Keyboard Integration:** Tapping text entry fields triggers the native Android soft keyboard.

### Building the APK Locally

Building the APK requires JDK 17, Android SDK, Android NDK `r26d`, the `aarch64-linux-android` Rust target, and `cargo-apk`:

```sh
cargo apk build --release -p benilla-android
```

---

## TurtleWoW Compatibility

**benilla** speaks stock 1.12.1 protocol out of the box, allowing connection to TurtleWoW realms. This fork incorporates compatibility layers for TurtleWoW's client patch features:

- **Playable Races:** Goblin (Horde) and High Elf / Blood Elf (Alliance) in character creation, race-select UI art, and model/helm variants.
- **Audio Error Resilience:** Missing sound kit IDs absent from vanilla DBCs warn once and play as silence.
- **Radio Stream Support:** Recognized Everlook internet-radio tower paths as stream URLs (silences zone music gracefully).
- **Lua Camera APIs:** Added `CameraZoomIn`, `CameraZoomOut`, and `FlipCameraYaw`.
- **FrameXML & Codecs:** Loads `json.lua` and TurtleWoW's custom FrameXML load order via `benilla.toc`.
- **Graphics Options:** Fallback slider bounds provided for TurtleWoW's custom `OptionsFrame.lua`.
- **Lua Environment:** Implemented `getfenv` and `setfenv` globals required by `Globals.lua` and UI helpers (`wipe`, `trim`, `explode`, `sizeof`, `print`).
- **Warden Handling:** Silently accepts `SMSG_WARDEN_DATA` without crashing or dropping connection (Note: Warden execution itself is not implemented).

*Known parent fork open issues:* Everlook radio streaming ([#3](https://github.com/jhinzuo2/benilla-twow/issues/3)), TransmogUI ([#2](https://github.com/jhinzuo2/benilla-twow/issues/2)), Inspect Talents tab ([#7](https://github.com/jhinzuo2/benilla-twow/issues/7)), and white rendering on certain Tauren gear combinations ([#6](https://github.com/jhinzuo2/benilla-twow/issues/6)).

---

## Development & Verification

Before modifying code, review the project guides:

- [`AGENTS.md`](AGENTS.md) — Guidelines and instructions for AI agents working in this repository.
- [`docs/METHOD.md`](docs/METHOD.md) — Binding architectural rules and engineering methodology.
- [`docs/MAP.md`](docs/MAP.md) — Generated crate and subsystem map.
- [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) — Development setup, verification gates, and PR submission rules.

### Standard Verification Commands

```sh
# Code formatting check
cargo fmt --all -- --check

# Workspace lints
cargo clippy --workspace --all-targets -- -D warnings

# Unit & integration tests
cargo test --workspace

# Scoped check script
scripts/check.sh

# Full local gate check
scripts/gates.sh
```

---

## Contributing

Pull requests for changes and fixes in this fork are welcome. Upstream tracking remains with the parent projects:

- [samwhosung/benilla issues](https://github.com/samwhosung/benilla/issues)
- [jhinzuo2/benilla-twow issues](https://github.com/jhinzuo2/benilla-twow/issues)

Questions, ideas, and discussions are also hosted on the upstream [Discord](https://discord.gg/wJSJx467G4).

---

## License and Legal

Early inspiration and file format guidance came from the [wowemulation-dev](https://github.com/wowemulation-dev) community, particularly [warcraft-rs](https://github.com/wowemulation-dev/warcraft-rs).

**benilla** is an independent fan project, not affiliated with or endorsed by Blizzard Entertainment, Inc.

- **No Blizzard Content:** This repository contains no Blizzard copyrighted material (no art, models, sounds, maps, MPQ files, or stock FrameXML). Users supply their own legally obtained 1.12.1 client installation.
- **License:** Open source code in this repository is dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).
- Vendored components under `third_party/` ([Kira audio engine](third_party/kira) and [Lua 5.1 dialect](third_party/lua-src)) retain their respective upstream open-source licenses.
