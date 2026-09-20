# Domain docs

## Layout and reading rules

This is a single-context repository:

- `CONTEXT.md` at the repository root holds domain terms and context.
- `docs/adr/` holds architecture decision records.

Before exploring, read `CONTEXT.md` and ADRs relevant to the work.
If a root `CONTEXT-MAP.md` is introduced later, follow its pointers
to relevant context files and context-scoped ADRs.

If these files are absent, proceed silently. Do not suggest creating
them upfront. The domain-modeling skill creates them when terms or
decisions are resolved, including through grill-with-docs and
improve-codebase-architecture.

## Vocabulary

Use the terms defined in `CONTEXT.md` when naming concepts in issues,
proposals, hypotheses, and tests. Avoid synonyms the glossary rejects.

For a missing concept, reconsider whether it belongs in the domain
or note the gap for domain-modeling.

## ADR conflicts

Explicitly identify any proposal that contradicts an existing ADR,
and explain why the decision should be reconsidered.
