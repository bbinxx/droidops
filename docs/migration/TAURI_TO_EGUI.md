# Native UI migration

## Current implementation

The native application is `apps/droidops-desktop`, built with eframe/egui 0.37.
It runs independently of Tauri, React, Node, and WebKitGTK. The root Cargo
workspace excludes `src-tauri`, preserving the legacy lockfile and target paths.

`droidops-core` owns device models, typed errors, parser contracts, and validation.
`droidops-device-services` owns process supervision, discovery, ADB/Fastboot,
filesystem operations, and scrcpy command construction. UI state, native dialogs,
image previews, runtime wiring, and event handling belong to the desktop package.
Separate platform/ADB/Fastboot/UI crates are unnecessary at this stage.

## Legacy audit

Legacy screens are Dashboard, AppManager, FileExplorer, Gallery, ShellPage,
FastbootPage, and SideloadPage. `App.jsx` owns navigation and selected device.
Device logic is JavaScript in `src/lib/adb.js`; Rust registers Tauri plugins and
the starter greet command. There was no Rust device backend to extract unchanged.

Tauri dependencies include shell execution, process events, native dialogs,
filesystem/cache/temp paths, asset URLs, webview events, configuration, and build
tooling. The native version replaces these with Rust processes, background events,
rfd dialogs, image decoding, and egui textures.

Important audited discrepancies:

- APK installation uses `adb install -r`; the Sideload screen applies recovery ZIPs.
- Shell is one-shot command execution, not a persistent interactive terminal.
- Fastboot supports discovery/reboot, not flashing, formatting, or unlocking.
- Gallery has placeholder tiles and pulls images for previews.
- File explorer's cache pulls full images and keys them by filename alone.
- App icon extraction is incomplete and labels may fall back to package names.
- Per-app scrcpy helper exists without a corresponding current App Manager button.
- First-device auto-selection, broad process capabilities, unbounded commands,
  and swallowed discovery errors were not carried forward as desired behavior.

## Native behavior

Discovery checks ADB and Fastboot every five seconds with bounded retries through
the next interval, retaining tool errors separately. Selection is explicit and
invalidated on disconnect or transport/state transition. Operations and dialogs
are bound to a selection generation; obsolete results are discarded.

A two-thread Tokio runtime handles asynchronous child processes. Channels have
bounded capacity, output is limited to 4 MiB, and commands have timeouts and
cancellation. Scrcpy has an independent tracked task so it does not block other
operations. CPU image decoding has size/allocation limits and a cancellable wait;
an already running decode can finish in the background after cancellation.

The UI follows the supplied visual reference: warm off-white, thin dividers,
rectangular black actions, blue metadata, and an adaptive tool grid. All icons are
drawn as vectors; they do not depend on private-use font glyphs. Dark mode and
interface scaling are persisted; device selection and shell history are not.

## Safety and limitations

Managed mutations are restricted to shared-storage descendants, require canonical
parent checks, and reject final symlink targets. The ADB sync upload needs a
separate preflight; a malicious concurrent filesystem change can still race it.
These checks are not a sandbox against a hostile/rooted device. Free-form shell
remains explicitly privileged and confirmation-gated.

Uploads and APK replacement, deletions, app data clearing, uninstall, reboot,
rename, shell, and recovery ZIP operations require confirmation. Rename rejects
existing destinations. File listings use NUL framing and direct-child checks;
unsupported newline/non-UTF-8 filenames fail explicitly rather than forging rows.

No privileged package installer is ported: native settings provide tool installation
guidance instead. Windows/macOS hardware acceptance, native installers, app-label
resource decoding, and bounded thumbnail caching remain outstanding. See the
[parity matrix](FEATURE_PARITY.md). Keep the legacy implementation until acceptance.

## Verification

Linux compilation, unit/integration tests, Clippy, formatting, and a live window
capture were performed during implementation. Only discovery was exercised against
the host tools. No install, transfer, delete, shell, reboot, or recovery operation
was executed against a physical device. Tests use fixtures and disposable host
directories. CI configuration does not establish unexecuted platform results.
