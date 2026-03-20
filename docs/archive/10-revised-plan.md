# Revised Architecture Plan — Post syster-base Evaluation

## Status: ACTIVE — Implementation Phase

This document supersedes the original 8-crate from-scratch plan. Following the syster-base evaluation (score 2.65/3.0 — GO), the architecture shifts from building a parser to building an adapter layer on top of syster-base.

## Post-Evaluation Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  syster-base 0.4.0-alpha (REUSE — 33K lines, MIT)           │
│  logos lexer → rowan CST → salsa incremental → HIR          │
│  parse_sysml() / SyntaxFile::sysml() / file_symbols()      │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────────┐
│  sysml-v2-adapter (BUILD — ~500 lines)                      │
│  Thin wrapper: metadata CST extraction, connection          │
│  topology, state machine structure, symbol kind mapping     │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────────┐
│  sysml-v2-validate (BUILD)                                  │
│  Firmware-specific validation: layer rules, metadata        │
│  completeness, FSM well-formedness, port compat             │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────────┐
│  sysml-v2-extract (BUILD)                                   │
│  SysML models → YAML/JSON for codegen pipeline              │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────────┐
│  sysml-v2-gencontract (BUILD)                               │
│  Generation contract schema + cross-validation              │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────────┐
│  sysml-v2-codegen (BUILD)                                   │
│  Model + contract → .rs/.c/.h source files + test stubs     │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────────┐
│  sysml-v2-analyzer CLI (BUILD)                              │
│  Unified binary: parse, validate, extract, generate         │
└─────────────────────────────────────────────────────────────┘
```

## Crate Dependency Graph

```
                    syster-base (external)
                         │
                    ┌────┴────┐
                    │ adapter │
                    └────┬────┘
                   ┌─────┼──────┐
                   │     │      │
              ┌────┴──┐  │  ┌───┴───┐
              │validate│  │  │extract│
              └────┬──┘  │  └───┬───┘
                   │     │      │
                   │  ┌──┴──────┴──┐
                   │  │gencontract │
                   │  └──────┬─────┘
                   │         │
                   │    ┌────┴───┐
                   │    │ codegen│
                   │    └────┬───┘
                   │         │
                   └────┬────┘
                     ┌──┴──┐
                     │ cli │
                     └─────┘
```

## Domain Boundary

The toolchain separates **domain-agnostic SysML v2 tooling** from **firmware-specific logic**:

- **syster-base + adapter** — General-purpose SysML v2 parsing, querying, metadata extraction, connection resolution, and state machine analysis. These crates work for any SysML v2 domain (firmware, automotive, robotics, aerospace). The adapter could be extracted as a standalone library.

- **validate → extract → gencontract → codegen → CLI** — Firmware-specific. These crates understand firmware concepts: layer hierarchies (driver/hal/middleware/application), ISR safety, static memory allocation, concurrency protection, peripheral ownership. They are driven by the firmware metadata library (`firmware_library.sysml`).

A different domain would reuse syster-base + adapter and replace everything from validate upward with its own domain rules, extraction schema, and code generators.

## What We Build vs. Reuse

| Component | Source | Effort | Lines (est.) |
|---|---|---|---|
| Lexer, parser, CST | syster-base | Reuse | 0 |
| HIR, symbol extraction | syster-base | Reuse | 0 |
| Salsa-cached queries | syster-base | Reuse | 0 |
| Name resolution | syster-base (partial) | Reuse | 0 |
| **Metadata value extraction** | **Build** (adapter) | ~100 lines | ~100 |
| **Connection topology** | **Build** (adapter) | ~100 lines | ~100 |
| **FSM extraction** | **Build** (adapter) | ~150 lines | ~150 |
| **Symbol kind mapping** | **Build** (adapter) | ~50 lines | ~50 |
| **Workspace loading** | **Build** (adapter) | ~100 lines | ~100 |
| **Firmware validation** | **Build** (validate) | Medium | ~1500 |
| **Model extraction** | **Build** (extract) | Medium | ~1200 |
| **Generation contract** | **Build** (gencontract) | Medium | ~800 |
| **Code generator** | **Build** (codegen) | Large | ~2000 |
| **CLI** | **Build** (cli) | Small | ~500 |
| LSP server | syster-lsp (evaluate later) | Potential reuse | — |
| Tree-sitter grammar | nomograph/tree-sitter-sysml | Reuse | 0 |

**Total custom code:** ~6,500 lines (vs. ~40K lines in original plan)

## Implementation Phases

| Phase | Crate | Depends On | Description |
|---|---|---|---|
| 1 | `sysml-v2-adapter` | syster-base | Foundation: workspace loading, metadata/connection/FSM extraction |
| 2 | `sysml-v2-validate` | adapter | Firmware validation rules (FW001–FW053) |
| 3 | `sysml-v2-extract` | adapter, validate | SysML → YAML/JSON extraction |
| 4 | `sysml-v2-gencontract` | extract | Generation contract schema + validation |
| 5 | `sysml-v2-codegen` | extract, gencontract | Code generation (Rust primary, C/C++ deferred) |
| 6 | `sysml-v2-analyzer` CLI | all crates | Unified binary |

## Workspace Structure

```
tools/sysml-v2-analyzer/
├── Cargo.toml                    # [workspace]
├── .cargo/config.toml            # target = host (not ESP32)
├── crates/
│   ├── adapter/                  # Phase 1
│   ├── validate/                 # Phase 2
│   ├── extract/                  # Phase 3
│   ├── gencontract/              # Phase 4
│   ├── codegen/                  # Phase 5
│   └── cli/                      # Phase 6
└── tests/
    └── fixtures/                 # SysML v2 test fixtures (from eval-syster)
```

## Decision Log

### D1: Adapter over fork

**Decision:** Use syster-base as a dependency, build a thin adapter layer.

**Why:** Evaluation scored 2.65/3.0 (above 2.50 "use as dependency" threshold). The HIR extracts all needed symbol types. Only metadata field values and some symbol kind mappings need CST-level workarounds. Forking would mean maintaining 33K lines of parser code. The adapter layer is ~500 lines.

### D2: Same repository

**Decision:** Place the toolchain in `tools/sysml-v2-analyzer/` within the existing firmware repo.

**Why:** The toolchain is purpose-built for this firmware project's spec-driven workflow. Co-locating it:
- Keeps fixtures and specs together
- Allows CI to run toolchain tests alongside firmware tests
- Avoids cross-repo dependency management during development
- Can be extracted to a standalone repo later if it gains broader use

### D3: Cargo workspace

**Decision:** Use a Cargo workspace with one crate per pipeline stage.

**Why:** Clean dependency boundaries prevent cycles. Each crate can be tested independently. The CLI crate composes all others into a single binary. Workspace-level `Cargo.lock` ensures reproducible builds.

### D4: Host target override

**Decision:** Override the build target to host (not ESP32) via `.cargo/config.toml`.

**Why:** The parent project targets `xtensa-esp32-espidf`. The analyzer runs on the developer's machine, not on the microcontroller. The `.cargo/config.toml` override prevents the workspace from inheriting the parent's ESP32 target.

### D5: LSP deferred

**Decision:** Defer LSP server work. Note existence of syster-lsp.

**Why:** syster-lsp already provides a working LSP server built on syster-base. Evaluate it separately when we reach the editor integration phase. Building our own LSP would duplicate effort.

### D6: tree-sitter grammar exists

**Decision:** Note existence of nomograph/tree-sitter-sysml on GitLab. Do not build our own.

**Why:** The grammar has 192 tests and 98% coverage. It's not on crates.io but is usable for Neovim highlighting. Evaluate for integration when we reach the editor tooling phase.

## References

- [syster-base evaluation](./09-evaluation-syster-base.md) — hands-on evaluation results
- [Architecture analysis](./ANALYSIS.md) — original analysis and standard comparisons
- [Adapter spec](./01-sysml-v2-adapter.md) — detailed adapter crate design
- [Validation spec](./04-sysml-v2-validate.md) — firmware validation rules
- [Extraction spec](./05-sysml-v2-extract.md) — model extraction pipeline
