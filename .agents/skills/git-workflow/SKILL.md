---
name: git-workflow
description: Prepare DroidOps branches, Conventional Commits, pull requests, conflict resolutions, and release or hotfix integration without rewriting history.
---

# Git workflow

## Purpose

Keep changes focused and reviewable while protecting user work and release history.

## When to use

Use for branch creation, commit preparation, PR review, conflict resolution,
integration planning, and authorized branch cleanup.

## Required context

Read [AGENTS.md](../../../AGENTS.md), current status, remotes, branch tips, recent
commits, and the issue/specification. `master` is production and `dev` integration.
The remote URL may retain the old ADBplus name; do not silently rewrite it.

## Implementation workflow

1. Inspect status and existing isolation. Preserve unstaged/staged changes and
   local-only commits; do not automatically stash, reset, or amend them.
2. Verify the intended base against remote state when network access is available.
   Use a focused branch from `dev`; hotfixes start from `master`.
3. Use the branch prefixes and Conventional Commit types in canonical instructions.
   Link issue/specification, scope, and acceptance criteria in the review description.
4. Inspect conflicts and both sides' intent before resolving. Validate the final
   combined behavior; do not use blanket conflict-side selection.
5. Prepare a PR to `dev` with problem, behavior, validation, safety, and limitations.
   Production releases use reviewed PRs to `master`; sync hotfixes back to `dev`.
6. Squash merge and merged-branch cleanup are maintainer actions requiring explicit
   authorization. Do not auto-merge or force-move release tags.

## Coding standards

Stage only intended paths. Keep commits logical and avoid unrelated formatting or
dependency churn. Use `!` or a `BREAKING CHANGE` footer when appropriate. Describe
the final implementation, not abandoned approaches, in PR text.

## Safety constraints

No push, merge, deletion, history rewrite, settings change, or publication without
explicit authorization. Skill selection is not authorization. Inspect new branches
for copied secrets/device fixtures before proposing publication.

## Validation commands

```bash
git status --short --branch
git diff --check
git diff --stat
git diff --cached --check
```

Before review, compare `git diff <verified-base-sha>...HEAD` for committed changes
and inspect staged/untracked additions too. Run the affected code's scoped checks;
a clean Git diff is not an application test.

## Definition of done

The base and scope are verified, unrelated work is preserved, validation evidence
is recorded, and the PR is reviewable. Report unpublished/uncommitted state plainly.

## Relevant documentation

See [native development](../../../docs/development/NATIVE.md) and
[feature parity](../../../docs/migration/FEATURE_PARITY.md) for current evidence.

See [the agent guide](../../../docs/development/AI_AGENTS.md). The detailed Git
workflow, contribution guide, and PR templates are planned for the next phase.
