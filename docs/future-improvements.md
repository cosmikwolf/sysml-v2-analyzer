# Future Improvements

Ideas and enhancements that are out of scope for the current implementation but worth tracking.

---

## Audit enhancements

- **Type-aware parameter comparison** — Currently parameter comparison is count-based. Could use domain type maps to compare spec types (SysML) against code types (Rust/C) semantically.
- **Trait/interface matching** — Match spec port definitions against Rust trait definitions or C header declarations.
- **Connection topology audit** — Verify that inter-module connections described in spec are reflected as actual function calls or trait bounds in code.
- **Incremental audit** — Cache previous audit results and only re-audit modules whose source files changed (using file hashes).
- **Custom query patterns** — Allow domains to provide additional `.scm` query files for domain-specific constructs.

## Language support

- **C++ support** — Add `tree-sitter-cpp` grammar and `languages/cpp/audit.scm` queries.
- **Python support** — For firmware tools/scripts that are part of the spec.
- **Assembly support** — For HAL/PAC layer audit of inline assembly or standalone `.s` files.

## Extraction enhancements

- **Constraint extraction** — Extract SysML constraint blocks and verify them as assertions in code.
- **Requirement traceability** — Map SysML requirements to test functions.
- **Allocation extraction** — Extract SysML allocations (logical → physical component mapping).

## CLI enhancements

- **Watch mode** — Re-run audit on file changes (`audit --watch`).
- **CI integration** — Machine-readable output format for CI/CD pipeline checks.
- **Diff mode** — Show what changed between two audit runs.

## Code generation (optional, re-add if needed)

If a domain emerges where code generation from specs is useful, the infrastructure from D4/D6/D8 could be re-added as an optional pipeline stage. The MiniJinja template approach worked well for the scaffold; the issue was that firmware implementation code can't be generated from specs alone.
