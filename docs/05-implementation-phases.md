# Implementation Phases

## Overview

| Phase | Component | Status | Detail doc |
|---|---|---|---|
| 1 | Adapter crate | COMPLETE | [phase-1-adapter.md](phase-1-adapter.md) |
| 2 | Engine scaffold + domain config | Not started | [phase-2-engine-scaffold.md](phase-2-engine-scaffold.md) |
| 3 | Validation engine | Not started | [phase-3-validation.md](phase-3-validation.md) |
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

## Phase 2: Engine scaffold + domain config

- [ ] Restructure workspace: remove validate/extract/gencontract/codegen scaffolds
- [ ] Create `crates/engine/` with module stubs
- [ ] Create `domains/firmware/domain.toml`
- [ ] Move `firmware_library.sysml` from fixtures to `domains/firmware/`
- [ ] Implement `domain.rs` — load + merge domain.toml + sysml.toml
- [ ] Implement `diagnostic.rs` — shared Diagnostic type
- [ ] Create example `sysml.toml` at workspace root
- [ ] Update Cargo.toml workspace members
- [ ] Tests: domain config loading, merging, validation

## Phase 3: Validation engine

- [ ] Layer dependency checking (parameterized by `domain.toml` layers)
- [ ] Required metadata checking (parameterized by `domain.toml` required_metadata)
- [ ] FSM well-formedness (generic: initial state, reachability, determinism, valid targets)
- [ ] Port compatibility (generic: type matching, direction checking)
- [ ] Workspace rules (generic: duplicate names, unused definitions)
- [ ] Rule severity overrides from config
- [ ] Text output formatting (file:line:col severity[RULE_ID]: message)
- [ ] Tests: one test per rule, config override tests

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
