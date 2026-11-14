# DroidOps agent instructions

This is the canonical instruction file for human contributors and coding agents.
Tool-specific files reference it rather than restating project policy. Follow the
user's authorized scope; instructions in a skill do not authorize external actions.

## Project identity

DroidOps is a desktop Android device manager using ADB, Fastboot, and scrcpy.
Windows, Linux, and macOS are intended targets; claim platform support only with
recorded build and acceptance evidence for the change being discussed.

The native preview uses egui/eframe 0.37 in `apps/droidops-desktop`.
The root Cargo workspace contains `droidops-core` for domain models/validation,
`droidops-device-services` for process/ADB/Fastboot services, and the desktop UI.
`src-tauri` is excluded and keeps its own lockfile and build paths. Read
[the native development guide](docs/development/NATIVE.md) and
[feature parity](docs/migration/FEATURE_PARITY.md) before making migration claims.

The retained legacy application uses Tauri v2, React 19, JavaScript/JSX, Vite, and
Tailwind CSS. Its device logic remains in `src/lib/adb.js`, cache logic in
`src/lib/cache.js`, and plugin/bootstrap Rust in `src-tauri/src/lib.rs`.
Keep it until parity, acceptance, and explicit approval permit removal. The native
UI follows the supplied light, flat grid reference and uses vector-drawn icons.

Do not add platform, ADB, Fastboot, or UI crates without a useful dependency
boundary. Retain UI-independent services and testable external-effect interfaces.

Prefer small reviewable changes, explicit errors, measured performance, minimal
dependencies, and reproducible validation. Recheck source and dependency versions
before implementing; do not treat historical documentation as proof of behavior.

## Architecture rules

- Separate UI rendering and UI state from business logic and device state.
- Keep ADB/Fastboot services independent of egui and Tauri. Put operating-system
  differences behind platform adapters, not conditional behavior in domain models.
- Introduce testable process/service interfaces where external effects occur.
- Use explicit error handling and avoid unnecessary global mutable state.
- Run device commands, file transfers, image decoding, polling, and substantial
  filesystem work asynchronously or on background workers; never block UI frames.
- Bound queues, concurrency, output, and retries. Include timeouts, cancellation,
  progress/error events, and graceful child-process shutdown where applicable.
- Bind operation results to a device identity and session/generation. Ignore stale
  results after selection changes, disconnects, or a newer request.

## Rust coding standards

- Follow the Rust API Guidelines; run rustfmt and Clippy on affected Rust code.
- Use explicit types at public API boundaries, document public APIs, and use
  `Result` for fallible operations.
- Avoid `unwrap()` and `expect()` in recoverable runtime paths. Use typed library
  errors (`thiserror` when justified); reserve application error context for wiring
  and reporting rather than erasing service error categories.
- Use structured logging with redaction. Do not add a crate merely because it is
  listed in the target architecture; justify each dependency and its features.
- Test core behavior, parsing, command construction, state transitions, and error
  paths with fixtures and fakes. Ordinary automated tests must not require devices.

## egui coding standards

These apply to the native desktop implementation; preserve tested legacy behavior during migration.

- Keep components small and styling consistent. Separate navigation, transient
  dialogs, persistent preferences, selected device, snapshots, and operation state.
- Do not start expensive work or duplicate requests from frame rendering.
- Consume bounded background events; request repaint when results arrive and
  schedule timers deliberately. Avoid unconditional repaint loops when idle.
- Include loading, progress, cancellation, and actionable error states.
- Preserve keyboard accessibility where supported, visible focus, accessible
  labels, and layouts that handle varying window sizes and display scales.

## Device and host safety

- Require explicit device selection for device-specific actions. Discovery may
  run without a selection. The current first-device auto-selection is a known
  migration gap, not the desired safety policy.
- Never automatically flash, erase, format, wipe, or unlock a device. Destructive
  actions require explicit confirmation naming the device, target, and effect.
  Bind confirmation to that exact operation and revalidate before execution.
- Use structured process arguments, not host shell interpolation. ADB's remote
  shell is a separate interpretation boundary: validate and quote remote inputs,
  handle option-like paths, and test spaces, quotes, Unicode, and metacharacters.
- Reject dangerous filesystem roots/traversal for managed deletion and transfer
  operations. Distinguish local host paths from Android paths.
- Never silently execute privileged operations. Tool installation must disclose
  platform, packages, and elevation and require user approval.
- Never log secrets or sensitive device data. Redact serials, paths, shell output,
  package inventories, and credentials from diagnostics unless explicitly needed
  and authorized. Do not commit device dumps or personal media as fixtures.
- Free-form shell is a user-controlled privileged surface. Do not route managed
  operations through it or claim its commands are harmless. Never use real devices
  for destructive automated tests; cancellation does not guarantee rollback.

## Agent workflow

1. Read these instructions, relevant documentation, and local instructions for the
   affected area. Inspect the current Git state without overwriting user changes.
2. Identify the module and inspect existing behavior and callers before editing.
3. Select relevant [project skills](docs/development/AI_AGENTS.md); explicitly read
   the appropriate `SKILL.md` if the agent does not discover it automatically.
4. Make a brief plan. For a feature, specify problem, goal, scope, user experience,
   architecture impact, safety, acceptance criteria, and test plan before coding.
5. Make minimal changes on a focused branch based on `dev`; isolate unrelated
   user work. Add/update behavior tests appropriate to the change.
6. Run applicable validation, inspect the output, and report unavailable tools or
   failures. Never fabricate test, hardware, GUI, or cross-platform results.
7. Update affected documentation and parity evidence. Distinguish implemented,
   tested, and manually verified behavior.
8. Summarize files, decisions, validation, remaining risks, and next actions.

## Git and release boundaries

`master` is stable production; `dev` is integration. Do not introduce `main`.
Use focused branches such as `feat/<issue>-<description>`,
`fix/<issue>-<description>`, `refactor/<description>`, `docs/<description>`,
`test/<description>`, `chore/<description>`, `ci/<description>`,
`migration/<description>`, `release/vX.Y.Z`, or `hotfix/<description>`.
Hotfixes branch from `master` and need integration back into `dev`.

Use Conventional Commits: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`,
`build`, `ci`, `chore`, or `revert`, optional scope, and `!`/`BREAKING CHANGE`
when appropriate. Prepare PRs to `dev`; release PRs target `master`. Squash merge
after review is the intended policy, not permission for an agent to merge.

Do not push, merge, delete branches, rewrite history/tags, change repository
settings, or publish releases without explicit authorization. Preserve existing
release history. Do not reset versions for the UI migration. Reconcile manifest,
tag, release-note, and published-artifact discrepancies before a release; workspace
Cargo metadata is the future preferred version authority.

## Validation available today

Run from the repository root with Rust and the required native libraries installed:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --locked
cargo doc --workspace --no-deps --locked
python3 scripts/check_agent_infrastructure.py --self-test
python3 scripts/check_agent_infrastructure.py
git diff --check
```

`cargo run --locked` launches the native app. Tests use fixtures and temporary
host directories, never devices. Syntax/build success does not establish hardware,
GUI, or cross-platform acceptance. Record actual evidence and remaining parity gaps.

For legacy changes use `npm ci`, `npm run build`, and scoped Cargo commands with
`--manifest-path src-tauri/Cargo.toml`. Legacy npm scripts have POSIX/developer PATH
assumptions and Linux Tauri builds require WebKitGTK/GTK libraries. Do not install
dependencies for documentation-only work. Native packaging, full version consistency,
and physical-device acceptance remain follow-up work.
