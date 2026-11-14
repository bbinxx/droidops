---
name: github-actions
description: Build or review DroidOps GitHub Actions validation matrices, dependency checks, caching, artifacts, permissions, and release automation.
---

# GitHub Actions

## Purpose

Provide reliable PR/build evidence and one controlled release path with minimal
token and secret exposure.

## When to use

Use for CI workflows, cross-platform Rust/legacy builds, cache design, dependency
updates, artifact handling, release jobs, and workflow security review.

## Required context

Read [AGENTS.md](../../../AGENTS.md), `.github/workflows/release.yml`,
`.github/workflows/auto-merge-main.yml`, `.github/changelog-config.json`, manifests,
and build scripts. The branches are `master`/`dev`, not `main`. Existing automation
includes unsafe auto-merge/tag rewriting; do not reuse those behaviors.

## Implementation workflow

1. Identify event, trust boundary, required checks, OS/architecture targets, and
   whether the workflow validates or publishes. Audit existing publishers first.
2. Use PR checks with read-only permissions and unique stable job names. Never run
   untrusted PR code with release secrets or privileged `pull_request_target` jobs.
3. Retain legacy build validation until retirement. Match system packages and
   runner architecture to each target; verify runner/image availability at edit time.
4. Add fmt, Clippy, tests, docs/build checks, dependency review, and version checks
   as their infrastructure exists. Do not add commands targeting absent packages.
5. Cache by OS, toolchain, and lockfile; verify caches cannot bypass validation.
   Prefer reviewed immutable action references and reproducible dependency installs.
6. Gate publishing on validated tags/versions/ancestry and all required artifacts.
   Scope write permissions/secrets to publishing jobs and preserve one publisher.
7. Validate workflow syntax and observe actual jobs; distinguish local inspection
   from CI success. Record dependency-update and artifact retention policies.

## Coding standards

Keep matrices explicit, permissions least-privilege, scripts portable, and shell
inputs structured. Do not interpolate untrusted titles/ref names into executable
shell text. Avoid duplicate changelog configurations or implicit floating toolchains.

## Safety constraints

Do not enable automatic merges, force pushes, tag rewriting, or publication.
Repository settings/environment changes need authorization. Fork checks must not
require secrets. Never add real-device destructive tests to hosted runners.

## Validation commands

Current local checks:

```bash
python3 scripts/check_agent_infrastructure.py
git diff --check
```

Use `actionlint` for workflow edits when available and report if unavailable; it is
not installed by this skill. Run applicable job commands locally where supported,
then record actual CI matrix results. The native-ci workflow validates the workspace; legacy PR validation still needs work.

## Definition of done

Events, permissions, runner requirements, dependencies, check names, and publishing
gates are reviewed; syntax and executed jobs have evidence. Untested platforms and
manual environment setup remain explicit.

## Relevant documentation

See [native development](../../../docs/development/NATIVE.md) and
[feature parity](../../../docs/migration/FEATURE_PARITY.md) for current evidence.

See [the agent guide](../../../docs/development/AI_AGENTS.md) and
[GitHub Actions security guidance](https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions).
Build/release guides and required-check configuration are planned.
