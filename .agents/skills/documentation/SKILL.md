---
name: documentation
description: Write source-backed DroidOps architecture, setup, contribution, feature, migration, release, API, and ADR documentation with accurate implementation status.
---

# Documentation

## Purpose

Give humans and agents one accurate place to find each policy and implementation
fact, with evidence and clear distinctions between plans and completed work.

## When to use

Use for architecture/development guides, onboarding, API docs, feature specs,
parity records, ADRs, changelogs, and documentation maintenance.

## Required context

Read [AGENTS.md](../../../AGENTS.md), [README.md](../../../README.md),
[RELEASES.md](../../../RELEASES.md), and affected source. README/release claims
currently exceed implementation in several places; verify features directly.

## Implementation workflow

1. Identify audience, question, authoritative source, and owning document.
2. Inspect implementation and tests; label proposed/implemented/tested/verified
   behavior separately. Link evidence instead of claiming undocumented support.
3. Keep canonical policy in AGENTS and link from adapters/skills. Keep parity
   status, audit details, and migration sequencing in their respective documents.
4. Write runnable commands with prerequisites and working directory. Distinguish
   current commands from future workspace/packaging checks and platform caveats.
5. For ADRs include Context, Decision, Alternatives, Consequences, and Status.
   For feature specs include the problem, goal, scope, UX, architecture, safety,
   acceptance criteria, and test plan.
6. Check local links and examples; review historical claims without inventing
   release artifacts, legal ownership, maintainers, dates, or validation evidence.

## Coding standards

Use concise Markdown, relative file links, fenced commands, and explicit ownership
of facts. Avoid duplicating full guides across skills. Mark future documents as
planned text until created; do not link to nonexistent files.

## Safety constraints

Never include personal serials, device dumps, screenshots, credentials, or private
paths. Dangerous device commands are not onboarding smoke tests. Do not add license
copyright text or CODEOWNERS without confirmed ownership/maintainers.

## Validation commands

```bash
python3 scripts/check_agent_infrastructure.py --self-test
python3 scripts/check_agent_infrastructure.py
git diff --check
```

The checker covers agent documents only, not all repository docs or URL/fragment
availability. Review other links manually or with an existing scoped checker.
Use `cargo doc --manifest-path src-tauri/Cargo.toml --no-deps --locked` for relevant
Rust API docs when the toolchain is available; workspace docs are future checks.

## Definition of done

Documentation matches inspected source, commands state prerequisites, links resolve,
plans are labeled, and no unexecuted validation is described as passing. Changed
policies have one canonical owner.

## Relevant documentation

See [native development](../../../docs/development/NATIVE.md) and
[feature parity](../../../docs/migration/FEATURE_PARITY.md) for current evidence.

See [the agent guide](../../../docs/development/AI_AGENTS.md). Architecture, setup,
testing, migration, security, release, contribution, and ADR guides are planned.
