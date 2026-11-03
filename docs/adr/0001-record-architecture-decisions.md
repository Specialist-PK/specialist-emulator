# 0001. Record architecture decisions

[Русская версия](0001-record-architecture-decisions.ru.md)

Status: accepted (2026-11-03)

## Context

Code shows what the project does but not why a solution was chosen or which alternatives were rejected. Without durable records, architectural constraints look arbitrary and are accidentally broken.

## Decision

Record every significant decision in `docs/adr/NNNN-short-name.md` with an English canonical document and a Russian `.ru.md` companion.

Each record contains:

1. title and number;
2. status (`accepted`, `replaced by ADR-NNNN`, or `rejected`) and date;
3. context and constraints;
4. decision;
5. real alternatives and why they were rejected;
6. positive and negative consequences.

Accepted records are not rewritten. A changed decision gets a new record and the old one links to its replacement. Numbers are sequential and never reused. Records stay short. Decisions that are expensive to reverse require an ADR; variable names and other local choices do not.

## Alternatives

- **Code comments only.** Local reasons fit well, but cross-cutting decisions have no single code location.
- **One architecture page.** It quickly becomes stale and loses decision history.
- **No records.** Cheaper immediately, more expensive during later changes.

## Consequences

- (+) Rationale and trade-offs remain available to people and agents.
- (+) Changes begin with explicit understanding of existing constraints.
- (−) Significant work requires disciplined documentation in two languages.
