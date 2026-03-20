# sysml-v2-codegen

Code generation from SysML v2 models and generation contracts.

**Status: Scaffold** — Crate structure exists, implementation pending.

## Domain scope

This crate is **firmware-specific**. The generated code targets embedded systems: `no_std` Rust with static allocation, ESP-IDF C with RTOS primitives, ISR-safe patterns, and hardware peripheral ownership. Renderers enforce firmware constraints from the generation contract — for example, a module with `@MemoryModel { allocation = static_alloc }` will never generate heap allocation code.

## Purpose

Transforms extracted firmware models + generation contract into embedded source files (`.rs`, `.c`, `.h`, `.cpp`, `.hpp`) with test stubs. This is the final stage of the pipeline — it takes the fully validated, extracted, and contract-resolved data and writes firmware code that respects memory, concurrency, and ISR safety constraints.

## Planned components

| Component | Purpose |
|---|---|
| `render/rust.rs` | Rust source renderer (primary target) |
| `render/c.rs` | C source renderer (deferred) |
| `render/cpp.rs` | C++ source renderer (deferred) |
| `incremental.rs` | Spec-hash fingerprinting to skip unchanged files |
| `report.rs` | Generation report (files created, skipped, warnings) |

## Output per module

For each firmware module, the codegen produces:
- One header/interface file (`.h` for C, trait in `.rs` for Rust)
- One implementation file (`.c` / `.rs`)
- One test stub file (`_test.c` / `tests/*.rs`)

State machine definitions produce additional enum types and transition functions.

## Dependencies

- `sysml-v2-extract` — Extracted model types
- `sysml-v2-gencontract` — Generation contract resolution

## Design spec

See [`docs/sysml-toolchain/07-sysml-v2-codegen.md`](../../../../docs/sysml-toolchain/07-sysml-v2-codegen.md) for the full design.
