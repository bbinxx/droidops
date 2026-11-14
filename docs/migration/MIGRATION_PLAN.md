# egui replacement implementation plan

## Goal and scope

Provide a real Rust-native replacement UI for DroidOps, preserving the legacy
application as an acceptance reference and keeping unrelated user changes intact.
Use the supplied ARTCRAFT visual reference for light surfaces, dividers, grid
composition, typography, buttons, and compact metadata. Device-specific content
and workflows remain DroidOps' own.

## Reviewable implementation sequence

1. Introduce the three-package workspace, domain contracts, parsing, and injectable
   process services. Test targeting, failure paths, timeout/cancellation, quoting,
   NUL-framed paths, confirmation, and filesystem guards.
2. Add the eframe application, eight screens, runtime/events, persistent appearance,
   native dialogs, independent scrcpy lifecycle, and vector icons. Test UI selection
   state and render fixtures without devices. Inspect a live Linux window.
3. Update documentation and add native PR validation without replacing the Tauri
   publisher. Record actual acceptance gaps in the parity matrix.
4. Complete friendly application metadata, thumbnail caching, progress metrics,
   platform acceptance, packaging/signing, and release validation in focused work.
5. Remove legacy dependencies only after parity, acceptance, and explicit approval.

## Acceptance criteria

- Root Cargo commands build the native app without Node or Tauri dependencies.
- Device work never blocks a rendering frame; cancellation/errors are visible.
- Device selection is deliberate and stale responses/confirmations cannot cross it.
- Every legacy screen has a native route and implemented supported workflows;
  partial parity is stated rather than hidden.
- No physical destructive operation is part of tests or developer startup.
- Fonts do not provide UI pictograms; vector icons render at different scales.
- Formatting, Clippy, tests, and Linux launch have recorded evidence.
- Source changes remain isolated from pre-existing frontend/lockfile modifications.

## Decisions

Keep legacy Cargo outside the root workspace to avoid accidental lockfile/target
migration. Use Tokio for bounded process I/O and timers, rfd for native dialogs,
image for bounded decoding, tracing for redacted events, and eframe persistence for
appearance only. No platform/UI crate is added without a real dependency boundary.
Native package version stays at the current Tauri/frontend metadata value, 0.1.2;
this is a development preview, not a new published release or a tag-history reset.
