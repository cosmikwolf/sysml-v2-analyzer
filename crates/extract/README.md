# sysml-v2-extract

Extract SysML v2 models into YAML/JSON for the code generation pipeline.

**Status: Scaffold** — Crate structure exists, implementation pending.

## Domain scope

This crate is **firmware-specific**. The extraction output schema assumes the SysML model contains firmware metadata annotations (`@MemoryModel`, `@ConcurrencyModel`, `@ISRSafe`, `@Ownership`) and produces YAML/JSON structured around firmware concepts: modules with layers, memory allocation strategies, concurrency protection, and hardware ownership. A different domain would need a different extractor with its own output schema.

## Purpose

Bridges the architecture layer (SysML v2) to the generation contract layer (firmware-specific YAML/JSON). Queries the parsed and validated model, then emits a normalized, flat representation of each module, interface, state machine, and connection — structured around firmware concerns like layer classification, memory models, and ISR safety.

## Planned extraction targets

| Target | Input | Output |
|---|---|---|
| Module | `part def` with firmware metadata | `extracted/modules/<name>.yaml` — ports, actions, metadata, dependencies |
| State machine | `state def` inside a part | `extracted/state_machines/<name>.yaml` — states, transitions, events |
| Interface | `port def` with shared contract | `extracted/interfaces/<name>.yaml` — function signatures, directions |
| Architecture | Entire workspace | `extracted/architecture.yaml` — module list, dependency graph, constraints |

## Key design decisions

- Only `part def` elements with firmware metadata annotations are extracted as modules
- Metadata is flattened from SysML v2 annotation syntax into the YAML structure the codegen expects
- Type names are preserved as strings — mapping to target-language types happens in gencontract
- Extraction refuses to proceed if validation has errors (override with `--no-validate`)

## Dependencies

- `sysml-v2-adapter` — Workspace loading, metadata/connection/FSM extraction
- `sysml-v2-validate` — Pre-extraction validation gate
- `serde` / `serde_yaml` / `serde_json` — Serialization

## Design spec

See [`docs/sysml-toolchain/05-sysml-v2-extract.md`](../../../../docs/sysml-toolchain/05-sysml-v2-extract.md) for the full design.
