---
name: release-management
description: Reconcile DroidOps versions and release history, prepare SemVer changes and changelogs, and validate tags, packages, checksums, and release artifacts.
---

# Release management

## Purpose

Prepare reproducible, traceable releases without resetting or rewriting history.

## When to use

Use for version synchronization, release branches, changelog generation, tag
validation, artifact inspection, GitHub Release preparation, and release checks.

## Required context

Read [AGENTS.md](../../../AGENTS.md), `package.json`, `package-lock.json`,
`src-tauri/Cargo.toml`, its lockfile, `src-tauri/tauri.conf.json`, README,
[RELEASES.md](../../../RELEASES.md), and the release workflows. Check live tags,
release records, and actual assets separately from local history.

## Implementation workflow

1. Reconcile current versions and local/remote tags; do not assume a release-note
   heading or workflow success means matching artifacts are published.
2. Choose a SemVer change based on compatibility and user-visible scope. Preserve
   the existing sequence during migration; distinguish native previews explicitly.
3. Prepare a release branch if needed and a PR to `master`. Prefer workspace Cargo
   metadata as the native authority once available, generating required duplicates.
4. Synchronize package/config/lock metadata and changelog; verify tag format,
   intended commit ancestry, version equality, and release notes before publishing.
5. Build supported OS/architecture packages, inspect contents, test installation,
   generate checksums, and document signing/notarization status and reproducibility
   limits. Keep one publisher; stage complete artifacts before an approved release.
6. Synchronize release fixes back to `dev` through the approved workflow.

## Coding standards

Keep version updates mechanical and scoped. Use deterministic validation scripts
and locked dependencies. Preserve historical notes; qualify unsupported claims
rather than inventing dates, assets, build results, or changelog entries.

## Safety constraints

Creating/pushing release tags, merging, publishing, and changing repository settings
require explicit authorization. Never force-update an existing release tag.
Never run the current auto-merge workflow as a release validation step.

## Validation commands

Current read-only local review:

```bash
git tag --list
git log -10 --oneline
python3 scripts/check_agent_infrastructure.py
git diff --check
```

Version consistency, changelog, tag-ancestry, checksums, and native packaging
validators are planned; introduce and test them before claiming release readiness.
Use scoped legacy build checks from `AGENTS.md` until the application is retired.

## Definition of done

Metadata/history agree, artifacts and checksums have platform evidence, notes state
limitations, and the release is ready for the final approval boundary. Preparation
is not publication; report each separately.

## Relevant documentation

See [native development](../../../docs/development/NATIVE.md) and
[feature parity](../../../docs/migration/FEATURE_PARITY.md) for current evidence.

See [the agent guide](../../../docs/development/AI_AGENTS.md). Versioning policy,
release guide, and a canonical changelog are planned for subsequent work.
