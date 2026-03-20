# sysml-v2-analyzer

A Rust toolchain for analyzing SysML v2 specifications and generating code from them. Domain-agnostic at its core — domain-specific rules, metadata, and codegen templates are provided as plugins. Built on [syster-base](https://crates.io/crates/syster-base) (0.4.0-alpha).

## What it does

```
.sysml files  →  parse  →  validate  →  extract  →  generate  →  source code
```

The analyzer reads SysML v2 specifications and transforms them through a pipeline. Domain knowledge (what to validate, what to extract, how to generate code) comes from a `domains/` directory — not compiled into the binary.

1. **Parse** — Load `.sysml` files via syster-base into a queryable workspace
2. **Validate** — Check domain rules (layer dependencies, required metadata, FSM well-formedness)
3. **Extract** — Flatten SysML models into YAML/JSON
4. **Generate** — Render MiniJinja templates into source files

## Architecture

```
syster-base (external)       ← SysML v2 parser + HIR
       │
  ┌────┴────┐
  │ adapter │                ← domain-agnostic SysML v2 query library
  └────┬────┘
  ┌────┴────┐
  │ engine  │                ← domain-agnostic pipeline framework
  └────┬────┘
       │
  domains/<name>/            ← domain-specific config + templates
       │
  ┌────┴────┐
  │   cli   │                ← loads domain, runs pipeline
  └─────────┘
```

## Domain boundary

| Layer | Component | Domain scope |
|---|---|---|
| Parsing + querying | `adapter` crate + syster-base | **General SysML v2** — works for any domain |
| Pipeline framework | `engine` crate | **General** — validation, extraction, codegen engines are domain-agnostic |
| Domain rules + templates | `domains/<name>/` | **Domain-specific** — config, metadata library, codegen templates |
| Orchestration | `cli` crate | **General** — loads domain from `sysml.toml`, runs pipeline |

Adding a new domain = creating a directory under `domains/` with a `domain.toml`, a `.sysml` metadata library, and MiniJinja templates. No Rust code required.

## Crates

| Crate | Status | Purpose |
|---|---|---|
| [`sysml-v2-adapter`](crates/adapter/) | Implemented | Domain-agnostic SysML v2 query library (metadata, connections, FSMs) |
| [`sysml-v2-engine`](crates/engine/) | Not started | Domain-agnostic pipeline framework (validation, extraction, codegen) |
| [`sysml-v2-analyzer`](crates/cli/) | Not started | CLI binary — discovers config, loads domain, runs pipeline |

## Domains

| Domain | Status | Location |
|---|---|---|
| firmware | Planned (first domain) | [`domains/firmware/`](domains/firmware/) |

## Quick start

```bash
cd tools/sysml-v2-analyzer

# Build
cargo build

# Run adapter tests (the only implemented crate so far)
cargo test -p sysml-v2-adapter

# Lint
cargo clippy --workspace
```

## Usage (adapter crate)

```rust
use sysml_v2_adapter::{SysmlWorkspace, SymbolKind};
use sysml_v2_adapter::metadata_extractor::extract_metadata;
use sysml_v2_adapter::connection_resolver::resolve_connections;
use sysml_v2_adapter::state_machine_extractor::extract_state_machines;

let ws = SysmlWorkspace::load("spec/".as_ref())?;

for (file, sym) in ws.symbols_of_kind(SymbolKind::PartDefinition) {
    let metadata = extract_metadata(file, sym);
    let connections = resolve_connections(file, sym);
    let machines = extract_state_machines(file, sym);
}
```

## Test fixtures

Fixtures in `tests/fixtures/` model a Bluetooth audio sink firmware system:

| Fixture | Description |
|---|---|
| `firmware_library.sysml` | Metadata defs, enum defs (LayerKind, AllocationKind, etc.) |
| `interfaces.sysml` | Port type definitions, data structures |
| `bt_a2dp_sink.sysml` | Full part def with metadata, ports, ConnectionFSM (4 states, 7 transitions) |
| `audio_pipeline.sysml` | Composition with 3 connections + 1 flow |
| `i2s_output.sysml` | Simple driver module |
| `status_led.sysml` | LED controller with LedFSM (3 states, 6 transitions) |
| `large_model.sysml` | 50-module stress test (1075 lines) |
| `malformed.sysml` | Intentional errors for error recovery testing |

## Build target

Runs on the **host machine** (not ESP32). `.cargo/config.toml` overrides the parent project's ESP32 target.

## Docs

- [Architecture Overview](docs/00-architecture.md)
- [Implementation Phases](docs/05-implementation-phases.md)
- [Archive](docs/archive/) — pre-restructure docs (syster-base evaluation, standards analysis)
