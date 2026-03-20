# sysml-v2-analyzer — Project Instructions

## Overview

This is a Rust workspace that reads SysML v2 specs and transforms them through a domain-agnostic pipeline: parse → validate → extract → generate. See `docs/00-architecture.md` for the full system diagram.

## Workspace structure

```
crates/adapter/   — syster-base wrapper (parsing, symbol queries, metadata extraction)
crates/engine/    — domain-agnostic pipeline (validation, extraction, codegen)
crates/cli/       — binary entry point
domains/          — domain plugin directories (firmware/, template/)
docs/             — architecture docs, decision records, phase specs
```

## Build & test

```bash
cargo build                      # full workspace
cargo test --workspace           # all tests
cargo test -p sysml-v2-adapter   # adapter only
cargo test -p sysml-v2-engine    # engine only
cargo clippy --workspace         # lint — must be clean
```

## Key docs

| File | Purpose |
|---|---|
| `docs/00-architecture.md` | System diagram, crate roles, data flow |
| `docs/decisions.md` | Architecture Decision Records (D1–D7+) |
| `docs/05-implementation-phases.md` | Phase checklist with status tracking |
| `docs/phase-{1..6}-*.md` | Detailed spec for each phase |

## Phase completion protocol

**After implementing each phase**, before the final commit, you MUST:

1. **Update `docs/05-implementation-phases.md`**:
   - Change the phase status in the overview table (e.g. `Not started` → `COMPLETE`)
   - Check off all completed items in the phase's checklist section (`- [ ]` → `- [x]`)

2. **Update `docs/decisions.md`** (if applicable):
   - If any new architecture decisions were made during the phase, append a new `## D{N}` entry
   - If an existing decision was reconsidered or modified, update its status and add a note

3. **Update the phase spec doc** (`docs/phase-{N}-*.md`) if the implementation deviated from the spec in any material way (different approach, dropped items, added items).

4. **Commit all doc updates together with the implementation** in the phase's final commit, or as a separate doc-update commit immediately after — do not leave docs out of date across commits.

5. **Verify before committing**:
   - `cargo test --workspace` — all tests pass
   - `cargo clippy --workspace` — clean (no warnings)
