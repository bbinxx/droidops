# Git workflow

## Mandatory feature branching

Always start work on a dedicated feature branch before editing. This applies to
features, fixes, documentation, cleanup, CI, and release preparation. Never
implement or commit directly on `dev` or `master`.

`dev` is the integration branch; `master` is stable production. They receive
reviewed, explicitly authorized integration only. Do not introduce a `main` branch.
Hotfix branches start from `master` and need fixes synchronized back into `dev`.

## Starting work

Inspect `git status --short --branch` and existing worktrees first. Preserve
uncommitted changes and local-only commits. Reuse suitable isolation or create a
worktree rather than moving unrelated edits onto the new branch.

Fetch remote state and create the feature branch from the current development
tip. For example, substituting the appropriate branch name:

```bash
git fetch origin
git switch -c refactor/example-change origin/dev
```

Use `feat/<issue>-<description>`, `fix/<issue>-<description>`,
`refactor/<description>`, `docs/<description>`, `test/<description>`,
`chore/<description>`, `ci/<description>`, `migration/<description>`,
`release/vX.Y.Z`, or `hotfix/<description>`.

## Implementation and review

1. Define scope and acceptance criteria; inspect affected code and project skills.
2. Make focused changes and test observable behavior with the applicable checks.
3. Inspect staged changes and update documentation/parity evidence.
4. Use Conventional Commits with the allowed types in [AGENTS.md](../../AGENTS.md).
5. Prepare a PR to `dev`; release PRs target `master`. Include problem, resulting
   behavior, validation evidence, remaining risks, and acceptance gaps.
6. Push or publish the PR only with explicit authorization. Review before merge.
7. Merge and clean up branches only when separately authorized. Preserve history.

No emojis are permitted in repository content, commits, PRs, logs, or agent replies.
Canonical policies live in `AGENTS.md`; tool-specific adapters only reference them.

## Pushing and stale production branches

Push the intended branch explicitly after authorization:

```bash
git push -u origin HEAD
```

Avoid `git push --all` for routine feature work: it also attempts unrelated local
branches. A stale local `master` can be rejected while feature branches succeed.
Inspect divergence; never force-push to bypass that rejection. When `master` has
no unique local commits and is not checked out, an authorized update can use
`git fetch origin master:master`, which refuses a non-fast-forward update.

The canonical remote URL is `https://github.com/bbinxx/droidops.git`. Keep remote
changes explicit and do not infer permission to publish releases or rewrite tags.
