# ADR-0001: Record architecture decisions

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

This project has accumulated non-trivial architectural choices -- in its
language/runtime, its dependencies, its public interfaces, and its release
process -- but the *reasoning* behind them lives only in commit messages,
PR discussion, or individual memory. Without a durable record, contributors
cannot tell an intentional design decision from an accident, and settled
trade-offs get re-litigated.

## Decision

We will use Architecture Decision Records (ADRs), in the lightweight format
popularised by Michael Nygard. Each significant decision gets a numbered
Markdown file under `docs/adr/`, named `NNNN-kebab-case-title.md`, based on
[`template.md`](template.md).

ADRs are immutable once **Accepted**: to change a decision, add a new ADR
that *supersedes* the old one and update the old ADR's status. Trivial or
easily reversible changes do not need an ADR.

## Consequences

- Architectural intent is captured next to the code and reviewed in the same
  pull request that implements it.
- The history of decisions is append-only, so the evolution of the design is
  preserved rather than overwritten.
- A small amount of process overhead per significant decision, and one more
  document to keep in sync with reality.
