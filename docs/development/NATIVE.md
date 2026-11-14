# Native desktop development

The root workspace builds the egui application. Legacy web UI/build tooling has
been removed; its historical baseline is `08414ac`. See the
[cleanup record](../migration/LEGACY_REMOVAL.md).

## Prerequisites

Rust 1.95+ and native linker/toolchain dependencies are required. Implementation
was tested with Rust 1.100.0 on Linux; the complete minimum-version build is not
verified. Linux needs X11/Wayland/OpenGL development libraries and a working
graphics session. Native dialogs use the desktop portal service. Windows needs
MSVC build tools; macOS needs Xcode command line tools.

ADB/Fastboot must be on PATH for device features; scrcpy is optional for mirroring.
No tool is silently installed. Linux USB permissions and Android authorization
must be configured normally. The native app does not need Node or WebKitGTK.

## Run and build

```bash
cargo run --locked
cargo build --release --locked
```

The executable is `target/release/droidops-desktop` (`.exe` on Windows). The default
workspace member is the desktop app. `bash scripts/run-native.sh` resolves the
repository and runs it. Use a standard Rust installation for routine development;
the isolated `/tmp` toolchain used for initial validation is not a project dependency.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo doc --workspace --no-deps --locked
python3 scripts/check_agent_infrastructure.py --self-test
python3 scripts/check_agent_infrastructure.py
```

Rust tests use fake processes, disposable host files, and an offline UI worker.
No physical device operations are executed by tests. The gallery decoder accepts
PNG/JPEG/WebP with a 64 MiB source limit, 16384-pixel dimension limits, and a
256 MiB decoder allocation limit. Preview textures are resized to at most 1280×900.

## Manual acceptance

On each target OS, verify startup/close, window sizing, keyboard focus, theme and
scale persistence, missing tool errors, unauthorized/offline devices, deliberate
selection, reconnection, and selection changes during a background operation.
With explicit authorization and disposable data, verify APKs, app data actions,
uploads/downloads/rename/delete, quoted/Unicode filenames, gallery preview, shell,
mirroring alongside other operations, reboot, and recovery sideload. Destructive
tests are optional acceptance work, never ordinary CI requirements.

The native CI workflow adds compile/test checks for Linux, Windows, and macOS; it
does not publish artifacts or establish manual platform acceptance. The obsolete Tauri publisher and automatic merge workflow were removed. Native
release packaging/publishing still needs a separate reviewed implementation.
