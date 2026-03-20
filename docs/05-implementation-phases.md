# Implementation Phases

## Overview

| Phase | Component | Status | Detail doc |
|---|---|---|---|
| 1 | Adapter crate | COMPLETE | [phase-1-adapter.md](phase-1-adapter.md) |
| 2 | Engine scaffold + domain config | COMPLETE | [phase-2-engine-scaffold.md](phase-2-engine-scaffold.md) |
| 3 | Validation engine | COMPLETE | [phase-3-validation.md](phase-3-validation.md) |
| 4 | Extraction engine | Not started | [phase-4-extraction.md](phase-4-extraction.md) |
| 5 | Code generation engine | Not started | [phase-5-codegen.md](phase-5-codegen.md) |
| 6 | CLI | Not started | [phase-6-cli.md](phase-6-cli.md) |

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

## Phase 4: Extraction engine

- [ ] Module extraction (part def → `ExtractedModule` with flattened metadata)
- [ ] State machine extraction (adapter `StateMachine` → serializable `ExtractedStateMachine`)
- [ ] Interface extraction (port defs → function signatures)
- [ ] Architecture summary (workspace-level: modules, deps, constraints)
- [ ] YAML output
- [ ] JSON output
- [ ] Validation gate (refuse extraction if errors exist)
- [ ] Tests: round-trip, determinism, validation gate

## Phase 5: Code generation engine

- [ ] MiniJinja environment setup (trim_blocks, lstrip_blocks, no auto-escape)
- [ ] Template loading from `domains/<name>/templates/<language>/`
- [ ] Standard filters: `snake_case`, `pascal_case`, `screaming_snake`, `map_type`
- [ ] Module template rendering
- [ ] State machine template rendering
- [ ] Spec-hash fingerprinting (skip unchanged files)
- [ ] Generation report
- [ ] Create firmware Rust templates (`module.rs.j2`, `state_machine.rs.j2`, `test.rs.j2`)
- [ ] Tests: template rendering, filter correctness, incremental skip

## Phase 6: CLI

- [ ] `clap` derive for argument parsing
- [ ] `sysml.toml` discovery (walk up from cwd)
- [ ] `parse` command
- [ ] `validate` command
- [ ] `extract` command
- [ ] `generate` command (full pipeline)
- [ ] `status` command
- [ ] `check` command (parse + validate)
- [ ] `init` command
- [ ] Exit codes
- [ ] Text + JSON output formats
- [ ] Tests: CLI integration tests
