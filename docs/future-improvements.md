# Future Improvements

Ideas and enhancements that are out of scope for the current implementation but worth tracking.

---

## Audit enhancements

- ~~**Type-aware parameter comparison**~~ — **Done.** `compare_module()` accepts a `type_map` to translate SysML types to target language types. Reference qualifiers (`&`, `&mut`) are stripped before comparison.
- ~~**Trait/interface matching**~~ — **Done.** Spec port definitions with `port_type` match against `trait` definitions in Rust source code. Tree-sitter query extracts trait names and method signatures.
- **Connection topology audit** — Verify that inter-module connections described in spec are reflected as actual function calls or trait bounds in code. *Deferred:* requires cross-file analysis and call graph construction, which is significantly more complex than single-file structural matching. Worth revisiting when multi-module audit is needed.
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
