# 0010. Russian as the canonical documentation language

[Русская версия](0010-russian-canonical-documentation.md)

Status: accepted (2026-11-03)

## Context

The project initially followed the upstream scheme: canonical English documents in `.md` and Russian companions in `.ru.md`. The user works primarily in Russian and chose the opposite priority. The scheme must change across existing documents, links, and automated checks rather than only in a written rule.

## Decision

The canonical Russian version of every document lives in `.md`. Its English version lives next to it in `.en.md`. This applies to README, AGENTS, ADRs, architecture documents, the roadmap, and the pull-request companion template.

Documents contain cross-links. Existing English `.md` files were renamed to `.en.md`, and Russian `.ru.md` files became canonical `.md`. Missing English translations of `architecture` and `roadmap` were added.

The consistency test recursively requires a pair for every document. Special GitHub files keep the canonical name required by GitHub itself, while their English version receives `.en.md`.

## Alternatives

- **Keep English canonical.** Fewer changes, but contrary to the user's choice.
- **Russian only.** Easier to maintain, but loses existing translations and accessibility for English readers.
- **Both languages inline.** No companions are needed, but files become twice as long and synchronized editing is harder.
- **Apply the rule only to new documents.** Preserves old links but leaves two competing schemes in the repository.

## Consequences

- (+) Primary documentation uses the user's language while English translations remain available.
- (+) One naming scheme is automatically verified, and Git preserves rename history.
- (−) Every document needs synchronized maintenance in two languages.
- (−) External links to old English `.md` and Russian `.ru.md` paths must be updated; local links are fixed by this migration.
