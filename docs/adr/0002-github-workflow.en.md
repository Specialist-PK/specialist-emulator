# 0002. Issue/branch/PR workflow and protected `main`

[Русская версия](0002-github-workflow.md)

Status: accepted (2026-11-03)

## Context

The initial changes went directly to `main` and combined independent ideas. The upstream process instead makes plans visible, separates reviewable units, and requires green cross-platform checks before merging.

## Decision

Work follows this sequence:

1. every task starts with a Task or Bug issue linked to the pinned Roadmap;
2. one branch per issue is created from fresh `main` using `<type>/<summary>`;
3. commits are small English Conventional Commits, one idea each;
4. a pull request fills the template, includes `Closes #N`, uses an English title and Russian description, and lists unverified items;
5. only squash merge is allowed after the aggregate `ci` check passes; merged branches are deleted automatically.

GitHub settings enforce the process: pull requests are required for `main`, `ci` is the only required status check, admins are included, linear history and conversation resolution are required, force-push and branch deletion are disabled. Merge commits and rebase merges are disabled. No approvals are required while the repository has one maintainer.

Agent confirmation rules still apply: following the process does not implicitly authorize a merge.

## Alternatives

- **Direct work on `main`.** Faster for one change, but loses review boundaries and makes broken cross-platform states visible only after publication.
- **Convention without GitHub enforcement.** Easier to configure, but can be violated accidentally.
- **Required approvals.** Valuable for a team, but self-approval is impossible for a solo maintainer.
- **Require up-to-date branches.** Stronger freshness, but unnecessary repeated CI for a small solo project.

## Consequences

- (+) Plans, implementation, checks, and history form one traceable process.
- (+) Adding CI jobs or operating systems does not require branch-protection changes.
- (−) Even small fixes need an issue, branch, and PR.
- (−) Repository settings must be recreated if the project is moved.
