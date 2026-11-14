# Legacy UI removal

## Decision and scope

The user explicitly requested removal of the previous UI and mandatory feature
branching. This cleanup is implemented on `refactor/remove-legacy-ui`, based on
`dev`. It retires obsolete source/build tooling without claiming full native
hardware acceptance or erasing Git history.

The historical reference is commit `08414ac`. For example:

```bash
git show 08414ac:src/lib/adb.js
git show 08414ac:src/components/FileExplorer.jsx
```

## Removed

- React/JSX application, styles, hooks, JavaScript services, and frontend assets.
- Tauri package, configuration, capabilities, icons, generated-output log, and
  separate Cargo.lock.
- Node manifests and lockfile, Vite entry/configuration, Tailwind/PostCSS config,
  and placeholder public assets.
- Obsolete Tauri build/install scripts, publisher/changelog configuration, and
  automatic merge/tag-rewriting workflow.
- Tauri editor recommendation and obsolete workspace exclusion.

The Rust workspace, native icons, ADB/Fastboot services, native CI, run helper,
and historical release notes remain. The optional scrcpy setup helper remains
because mirroring still uses scrcpy; the application does not invoke it silently.
Ignored local outputs in older checkouts are not source dependencies and are not
deleted by switching to this branch.

## Remaining acceptance work

Friendly app labels/icons, thumbnail caching, expanded media coverage, native
installers/publishing, and cross-platform hardware acceptance remain incomplete.
See [feature parity](FEATURE_PARITY.md). Removing the old UI does not upgrade any
feature state to Verified or imply that old release assets are native builds.

## Validation

Verify the native workspace formats, passes Clippy/tests, builds, and generates
documentation using [the native guide](../development/NATIVE.md). Validate agent
instructions and local links. Inspect tracked files for obsolete entry points and
check that no remaining code/workflow depends on removed paths.

No physical-device operations are needed for this cleanup. Preserve pre-existing
user edits in the original checkout and a separate backup before integration.
