# Changelog

## 0.2.0 — Unreleased

### Added

- Independent Android app windows in the native Applications screen. Each session
  owns a scrcpy process, virtual display, cancellation token, and device identity.
- Available application names, per-app window titles and SDL identities, an Active
  App Windows section, independent Stop controls, and owned shutdown cleanup.
- Phone-resolution and DPI matching, custom display/window settings, optional audio,
  content-preservation controls, and confirmed force-stop launching.
- Fake-runner and UI tests plus an explicit opt-in hardware smoke example.
- Arch packaging and installation documentation integrated from dev. Its recipe
  still pins the previously reviewed 0.1.2 snapshot; it does not package 0.2.0 yet.

### Fixed

- Launch verification requires the target app on its non-primary virtual display,
  rather than reporting success when a scrcpy window merely exists.
- Phone-matching mode preserves Android resolution and density while scaling the
  whole picture in a resizable desktop window, preventing tiny high-DPI layouts.
- Explicit letterboxing prevents full-resolution frames from being cropped.
- Flex mode and fixed desktop size flags are mutually exclusive.

### Versioning and validation

- Set workspace and transition manifests/lockfiles to 0.2.0. This additive native
  preview advances beyond existing v0.1.5 history without resetting old tags.
- Local Arch evidence: Spotify, Chrome, and Gmail simultaneously verified on
  distinct virtual displays alongside normal mirroring; independent stop and
  cleanup passed. Keep launch checks exercised the rendering configuration.
- 34 workspace tests and native validation checks passed before integration;
  see feature documentation for the exact evidence and remaining acceptance work.

### Known limitations

- Live Android layout resizing is opt-in in custom-resolution mode. Phone mode
  scales a fixed layout. Some apps restrict secondary displays or resizing.
- APK app icons, desktop shortcuts, custom egui video containers, Windows/macOS
  acceptance, and 0.2.0 native release artifacts remain follow-up work.
- No release or tag is published by this development version update.

Earlier release history remains in [RELEASES.md](RELEASES.md); its historical
feature claims are not evidence of native feature parity.
