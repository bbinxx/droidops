# DroidOps

A desktop Android device manager for ADB, Fastboot, files, applications, and screen
mirroring. A Rust-native **egui/eframe preview** now runs from the root workspace.
The Tauri/React implementation remains available until native feature acceptance.

## Run the native app

Install Rust and your platform's native build tools, then:

```bash
cargo run --locked
```

For a release executable:

```bash
cargo build --release --locked
```

Linux executable: `target/release/droidops-desktop`. Native development does not
require Node, Tauri, or WebKitGTK. ADB and Fastboot must be on PATH for device
features, and scrcpy is needed for mirroring. Linux users also need normal USB
permissions and a working X11/Wayland/OpenGL session.

See [native setup and validation](docs/development/NATIVE.md) for prerequisites,
platform caveats, and manual acceptance checks.

## Arch Linux installation

On Arch Linux x86_64, install the build tools once, then build as your normal
user from the repository root:

```bash
sudo pacman -S --needed base-devel
cd packaging/arch
makepkg -si
```

Then launch **DroidOps** from the application menu or run `droidops`. Do not run
makepkg or DroidOps as root. To install an already-built package:

```bash
sudo pacman -U ./droidops-0.1.2-1-x86_64.pkg.tar.zst
```

Check installation with `pacman -Q droidops`; remove it with
`sudo pacman -R droidops`.
See [the Arch guide](docs/development/ARCH.md) for package contents, dependencies,
source pinning, and inspection commands.

## Native preview

- Explicit device selection, ADB/Fastboot discovery, and device dashboard.
- Package inventory with available app names, APK installation, uninstall, and app-data actions.
- [Independent Android app windows](docs/features/ANDROID_APP_WINDOWS.md) using scrcpy virtual displays, with per-app launch settings and Stop controls.
- File browsing, native upload/download dialogs, folder creation, rename, delete,
  and APK/file drag-and-drop.
- DCIM/Pictures gallery, album filtering, bounded photo preview, and downloads.
- Confirmed one-shot ADB commands, recovery ZIP sideload, and power/Fastboot reboot.
- Independent scrcpy mirroring, background cancellation, timeouts, and typed errors.
- A light desktop layout with thin dividers, a responsive tool grid, consistent
  vector icons, plus saved dark mode and display scaling.

This is a development preview. APK app icons, persistent thumbnail
caching, full media coverage, installers outside Arch Linux, and Windows/macOS hardware
acceptance remain incomplete. Implemented workflows have not all been exercised on
physical devices. See the [feature parity matrix](docs/migration/FEATURE_PARITY.md).

## Architecture

```text
crates/droidops-core/             Device models, policy, parsing, typed errors
crates/droidops-device-services/  ADB/Fastboot/scrcpy and supervised processes
apps/droidops-desktop/            egui screens, UI state, dialogs, runtime wiring
src/                             Retained React application
src-tauri/                       Retained Tauri package and separate Cargo.lock
```

The root workspace excludes `src-tauri` so legacy build paths and lockfiles stay
independent. The workspace Cargo manifest is the native version authority. Transition manifests
are synchronized to `0.2.0`; no release history or tags are reset. See
[the changelog](CHANGELOG.md) and [versioning policy](docs/VERSIONING.md).

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo doc --workspace --no-deps --locked
python3 scripts/check_agent_infrastructure.py --self-test
python3 scripts/check_agent_infrastructure.py
```

Tests use mocks and disposable local fixtures. They never require destructive
physical-device actions. Native CI provides a Linux/Windows/macOS build/test matrix;
local Linux evidence does not imply successful validation on other platforms.

## Legacy application

The original application remains available with its existing dependencies:

```bash
npm ci
npm run tauri dev
```

Its npm scripts currently contain POSIX and developer-specific PATH assumptions.
Linux Tauri builds additionally need WebKitGTK/GTK development libraries. The
legacy release workflows are retained and need their separately audited repair
before release use; native publishing is not enabled.

## Project conventions

Never use emojis in UI text, documentation, comments, logs, commit messages,
pull requests, release notes, or agent responses. Interface icons are drawn as
vectors rather than Unicode pictograms. See [AGENTS.md](AGENTS.md) for the
canonical development and safety rules.

## Development documentation

- [Canonical contributor/agent instructions](AGENTS.md)
- [Project skills and discovery](docs/development/AI_AGENTS.md)
- [Native development](docs/development/NATIVE.md)
- [Arch packaging and installation](docs/development/ARCH.md)
- [Migration audit](docs/migration/TAURI_TO_EGUI.md)
- [Migration plan](docs/migration/MIGRATION_PLAN.md)
- [Feature parity](docs/migration/FEATURE_PARITY.md)
- [Historical release notes](RELEASES.md)

Use focused branches from `dev` and Conventional Commits. Production remains on
`master`. Do not publish, merge, rewrite tags, or perform destructive device actions
without explicit authorization.

The prior README stated MIT licensing, but no license text or confirmed copyright
holder is present. License formalization remains a maintainer task; this migration
does not invent ownership or relicense the project.
