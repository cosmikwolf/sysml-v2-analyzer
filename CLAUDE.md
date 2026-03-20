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
| `docs/decisions.md` | Architecture Decision Records (D1–D8) |
| `docs/05-implementation-phases.md` | Phase checklist with status tracking |
| `docs/archive/phase-{1..6}-*.md` | Detailed spec for each phase (archived) |

## Testing

Use **proptest** for property-based testing wherever it makes sense. Good candidates:
- Pure functions with clear invariants (string transformations, type conversions)
- Parsers (should never panic on arbitrary input)
- Round-trip properties (serialize → deserialize, extract → merge)
- Anything with tricky edge cases around string parsing or pattern matching

Proptest is a `[dev-dependencies]` in both the adapter and engine crates. Add `#[cfg(test)] mod property_tests` alongside existing unit test modules.

## Documentation protocol

**After implementing a feature or fix**, before the final commit:

1. **Update `docs/decisions.md`** if a new architecture decision was made (append `## D{N}`)

2. **Update `docs/05-implementation-phases.md`** if the change relates to a tracked item

3. **Keep READMEs accurate** — if test counts, command lists, or feature descriptions change, update the relevant README

4. **Commit docs with the implementation** — do not leave docs out of date across commits

5. **Verify before committing**:
   - `cargo test --workspace` — all tests pass
   - `cargo clippy --workspace` — clean (no warnings)
