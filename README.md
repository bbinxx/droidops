# DroidOps

A desktop Android device manager for ADB, Fastboot, files, applications, and screen
mirroring. A Rust-native **egui/eframe preview** now runs from the root workspace.
Rust/egui is the only current application source. The previous web UI is available
in Git history at `08414ac`; see the [cleanup record](docs/migration/LEGACY_REMOVAL.md).

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

## Native preview

- Explicit device selection, ADB/Fastboot discovery, and device dashboard.
- Package inventory, APK installation, uninstall, and app-data actions.
- File browsing, native upload/download dialogs, folder creation, rename, delete,
  and APK/file drag-and-drop.
- DCIM/Pictures gallery, album filtering, bounded photo preview, and downloads.
- Confirmed one-shot ADB commands, recovery ZIP sideload, and power/Fastboot reboot.
- Independent scrcpy mirroring, background cancellation, timeouts, and typed errors.
- A light desktop layout with thin dividers, a responsive tool grid, consistent
  vector icons, plus saved dark mode and display scaling.

This is a development preview. Friendly app labels/icons, persistent thumbnail
caching, full media coverage, native installers, and Windows/macOS hardware
acceptance remain incomplete. Implemented workflows have not all been exercised on
physical devices. See the [feature parity matrix](docs/migration/FEATURE_PARITY.md).

## Architecture

```text
crates/droidops-core/             Device models, policy, parsing, typed errors
crates/droidops-device-services/  ADB/Fastboot/scrcpy and supervised processes
apps/droidops-desktop/            egui screens, UI state, dialogs, runtime wiring
```

The root Cargo workspace and Cargo.lock own the native application dependencies.
Workspace package metadata is the version authority; `0.1.2` remains development
metadata inherited from the migration baseline. Existing release history and tags
are preserved.

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

## Migration status

The obsolete React/Tauri source, Node configuration, unused assets, legacy build
scripts, and automatic merge/publishing workflows have been removed. Historical
release notes remain available. Native installers and release publishing are not
configured yet; the remaining acceptance gaps are tracked in the parity matrix.

## Project conventions

Never use emojis in UI text, documentation, comments, logs, commit messages,
pull requests, release notes, or agent responses. Interface icons are drawn as
vectors rather than Unicode pictograms. See [AGENTS.md](AGENTS.md) for the
canonical development and safety rules.

Always create a dedicated feature branch from `dev` before editing, including
documentation and cleanup. Never implement or commit directly on `dev` or
`master`. See the [Git workflow](docs/development/GIT_WORKFLOW.md).

## Development documentation

- [Canonical contributor/agent instructions](AGENTS.md)
- [Project skills and discovery](docs/development/AI_AGENTS.md)
- [Native development](docs/development/NATIVE.md)
- [Git workflow](docs/development/GIT_WORKFLOW.md)
- [Legacy removal record](docs/migration/LEGACY_REMOVAL.md)
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
