# Native migration plan

## Current implementation

The three-package Rust workspace and egui desktop screens are implemented. The
native UI uses the supplied light, flat grid reference and font-independent
vector icons. Processes run through bounded background services, explicit device
selection, confirmation gates, timeouts, and cancellation.

The original implementation retained Tauri/React for comparison. The user has now
explicitly requested its removal. [The cleanup record](LEGACY_REMOVAL.md) owns that
decision and scope; commit `08414ac` remains the historical comparison baseline.

## Remaining focused work

1. Complete friendly application labels/icons and bounded thumbnail caching.
2. Expand media coverage and improve transfer/progress feedback using measurements.
3. Record non-destructive device and GUI acceptance on Linux, Windows, and macOS.
4. Add native packaging/signing, artifact checksums, version consistency checks,
   and a reviewed release workflow. No publisher is currently configured.
5. Resolve the gaps in [feature parity](FEATURE_PARITY.md) with evidence rather
   than treating source removal or compilation as acceptance.

## Development constraints

Always create a dedicated feature branch before editing, including cleanup and
documentation. Never implement or commit directly on `dev` or `master`. Preserve
unrelated user changes. Do not push, merge, delete branches, change repository
settings, rewrite history/tags, or publish without explicit authorization.

Keep domain policy, device services, UI state, and platform effects separate.
Ordinary tests use fixtures and controlled host processes, never destructive
physical-device actions. Apply the canonical no-emoji rule everywhere.

## Architecture and version decisions

The root workspace is now the complete application source and dependency graph.
Tokio owns process I/O/timers; rfd provides native dialogs; image decoding is bounded;
tracing records redacted events; persisted settings contain appearance only.
Additional platform/UI/transport crates require a concrete dependency benefit.

Workspace package version `0.1.2` is inherited development metadata, not a new
release or a reset of tag history. Reconcile existing tags, release records, and
artifact versions before publishing a native release.
