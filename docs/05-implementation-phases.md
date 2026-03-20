# Implementation Phases

## Overview

| Phase | Component | Status | Detail doc |
|---|---|---|---|
| 1 | Adapter crate | COMPLETE | [phase-1-adapter.md](phase-1-adapter.md) |
| 2 | Engine scaffold + domain config | COMPLETE | [phase-2-engine-scaffold.md](phase-2-engine-scaffold.md) |
| 3 | Validation engine | COMPLETE | [phase-3-validation.md](phase-3-validation.md) |
| 4 | Extraction engine | COMPLETE | [phase-4-extraction.md](phase-4-extraction.md) |
| 5 | Code generation engine | COMPLETE | [phase-5-codegen.md](phase-5-codegen.md) |
| 6 | CLI | COMPLETE | [phase-6-cli.md](phase-6-cli.md) |

## Phase 1: Adapter crate — COMPLETE

- [x] Workspace loading (`SysmlWorkspace::load`, `from_sources`)
- [x] Symbol querying (`all_symbols`, `symbols_of_kind`, `find_by_qualified_name`)
- [x] Metadata extraction via CST (`extract_metadata`, `MetadataValue` variants)
- [x] Connection resolution (`resolve_connections`, `ConnectionKind`)
- [x] State machine extraction (`extract_state_machines`, states/transitions/initial)
- [x] Symbol kind mapping (`classify_symbol`, `MetadataDefinition` detection)
- [x] Integration tests (9 tests on full fixture workspace)
- [x] Unit tests (36 tests across all modules)
- [x] `cargo clippy` clean

## Phase 2: Engine scaffold + domain config — COMPLETE

- [x] Restructure workspace: remove validate/extract/gencontract/codegen scaffolds
- [x] Create `crates/engine/` with module stubs
- [x] Create `domains/template/` — minimal domain for engine tests + starter
- [x] Create `domains/firmware/` — firmware domain with `domain.toml` + `firmware_library.sysml`
- [x] Implement `domain.rs` — load + merge domain.toml + sysml.toml
- [x] Implement `diagnostic.rs` — shared Diagnostic type
- [x] Create example `sysml.toml` at workspace root
- [x] Update Cargo.toml workspace members
- [x] Update CLI crate to depend on engine instead of old crates
- [x] Tests: domain config loading, merging, severity overrides (11 tests)

## Phase 3: Validation engine — COMPLETE

- [x] Layer dependency checking: LAYER001 (illegal dep), LAYER002 (missing layer), LAYER003 (cycle via petgraph), LAYER004 (same-layer)
- [x] Required metadata checking: META010 (missing required annotation)
- [x] FSM well-formedness: FSM020 (no initial), FSM021 (unreachable), FSM022 (non-deterministic), FSM024 (invalid target), FSM025 (terminal)
- [x] Port connectivity: PORT033 (unconnected port). PORT030 (type incompatibility) deferred — adapter lacks structured port type info
- [x] Workspace rules: WS050 (duplicate qualified names among definitions), WS051 (unused part definition)
- [x] Rule severity overrides from config (`effective_severity()` helper)
- [x] Text output formatting (Diagnostic Display impl from Phase 2)
- [x] Tests: 25 unit tests (one+ per rule) + 2 integration tests + 2 config override tests

## Phase 4: Extraction engine — COMPLETE

- [x] Module extraction (PartDefinition → `ExtractedModule` with flattened metadata, ports, actions, connections, FSMs)
- [x] State machine extraction (adapter `StateMachine` → serializable `ExtractedStateMachine`)
- [x] Port extraction (PortUsage symbols + body text conjugation detection)
- [x] Action extraction (body text parsing for `action def` names)
- [x] Architecture summary (source files, module summaries, dependency graph from PartUsage→supertypes)
- [x] YAML output (`write_extraction` with `OutputFormat::Yaml`)
- [x] JSON output (`write_extraction` with `OutputFormat::Json`)
- [x] Validation gate (refuse extraction if Error-severity diagnostics exist, warnings allowed)
- [x] Shared `extract_layer_for_part` utility (moved from validation/layers.rs to util.rs)
- [x] Metadata flattening (MetadataValue → serde_json::Value)
- [x] Tests: 17 unit tests (flatten, extraction, gates, round-trip, determinism, write) + existing validation/integration tests

## Phase 5: Code generation engine — COMPLETE

- [x] MiniJinja environment setup (trim_blocks, lstrip_blocks, keep_trailing_newline, no auto-escape)
- [x] Template loading from `domains/<name>/templates/<language>/`
- [x] Standard filters: `snake_case`, `pascal_case`, `screaming_snake`, `map_type` (domain-aware type mapping)
- [x] Module template rendering (`module.<ext>.j2` → per-module source files)
- [x] State machine template rendering (`state_machine.<ext>.j2` → per-FSM source files)
- [x] Test template rendering (`test.<ext>.j2` → per-module test stubs)
- [x] Spec-hash fingerprinting (SHA256 of serialized input, skip unchanged files)
- [x] Generation report (generated/skipped file lists)
- [x] Firmware Rust templates: `module.rs.j2`, `state_machine.rs.j2`, `test.rs.j2`
- [x] Tests: 15 codegen tests (filters, hash, generation, skip, report) + existing validation/extraction tests

## Phase 6: CLI — COMPLETE

- [x] `clap` derive for argument parsing (global options: --config, --domain, --format, --quiet)
- [x] `sysml.toml` discovery (walk up from cwd, or --config flag)
- [x] `parse` command (parse only, no domain required, exit 0/2)
- [x] `validate` command (parse + validate, exit 0/1)
- [x] `extract` command (validate + extract to YAML/JSON, exit 0/1)
- [x] `generate` command (full pipeline: validate → extract → codegen, exit 0/1)
- [x] `status` command (workspace summary: files, parts, FSMs, ports)
- [x] `check` command (alias for validate)
- [x] `init` command (create sysml.toml with domain name)
- [x] Exit codes (0=success, 1=validation errors, 2=parse errors, 3=config errors)
- [x] Text + JSON output formats (--format text|json)
- [x] Known limitation: include/exclude globs from sysml.toml not yet applied (adapter scans full directory)
