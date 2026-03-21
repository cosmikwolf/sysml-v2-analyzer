# Implementation Phases

## Overview

| Phase | Component | Status | Detail doc |
|---|---|---|---|
| 1 | Adapter crate | COMPLETE | [phase-1-adapter.md](archive/phase-1-adapter.md) |
| 2 | Engine scaffold + domain config | COMPLETE | [phase-2-engine-scaffold.md](archive/phase-2-engine-scaffold.md) |
| 3 | Validation engine | COMPLETE | [phase-3-validation.md](archive/phase-3-validation.md) |
| 4 | Extraction engine | COMPLETE | [phase-4-extraction.md](archive/phase-4-extraction.md) |
| 5 | Code generation engine | COMPLETE | [phase-5-codegen.md](archive/phase-5-codegen.md) |
| 6 | CLI | COMPLETE | [phase-6-cli.md](archive/phase-6-cli.md) |
| 7 | Audit engine | COMPLETE | — |
| 8 | UI validation + extraction | COMPLETE | — |

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
- [x] Port connectivity: PORT033 (unconnected port), PORT030 (type incompatibility via PortUsage supertypes resolution)
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

## Phase 5: Code generation engine — REMOVED (superseded by D9)

Code generation was replaced by the audit-driven workflow. See D9 in `decisions.md`.

## Phase 6: CLI — COMPLETE

- [x] `clap` derive for argument parsing (global options: --config, --domain, --format, --quiet)
- [x] `sysml.toml` discovery (walk up from cwd, or --config flag)
- [x] `parse` command (parse only, no domain required, exit 0/2)
- [x] `validate` command (parse + validate, exit 0/1)
- [x] `extract` command (validate + extract to YAML/JSON, exit 0/1)
- [x] `audit` command (validate → extract → audit against source code, exit 0/1)
- [x] `status` command (workspace summary: files, parts, FSMs, ports)
- [x] `check` command (alias for validate)
- [x] `init` command (create sysml.toml with domain name)
- [x] Exit codes (0=success, 1=validation errors, 2=parse errors, 3=config errors)
- [x] Text + JSON output formats (--format text|json)
- [x] Include/exclude glob filtering from sysml.toml (`SysmlWorkspace::load_filtered` via globset)

## Phase 7: Audit engine — COMPLETE

- [x] `SourceConfig` replaces `template_dir` in `DomainConfig` (root, language, layout)
- [x] `ExtractedAction` enriched with parameters (`ActionParameter`, `ParameterDirection`)
- [x] Tree-sitter dependencies (`tree-sitter`, `tree-sitter-rust`, `tree-sitter-c`)
- [x] Language query files (`languages/rust/audit.scm`, `languages/c/audit.scm`)
- [x] `code_parser` module: tree-sitter parsing → `Vec<CodeConstruct>` (functions, structs, enums, impl blocks)
- [x] `source_map` module: resolve module name → file path (metadata override or convention-based)
- [x] `compare` module: diff `ExtractedModule` vs `Vec<CodeConstruct>` → `Vec<AuditItem>`
- [x] `audit` entry point: full pipeline with `--expand`, `--uncovered`, `--module` options
- [x] Text and JSON output formats for audit results
- [x] `snake_case` utility moved to `util.rs`
- [x] Tests: 16 unit tests (code_parser, source_map, compare) + 2 extraction param tests

## Phase 8: UI validation + extraction — COMPLETE

- [x] Extended `firmware_library.sysml` with 15 enum definitions (DisplayType, InputType, ElementKind, etc.) + 11 metadata definitions (@Display, @Input, @Screen, @Element, @Indicator, @IndicatorState, etc.)
- [x] Added UI validation rules in `validation/ui.rs`:
  - UI001 (display resolution) — width/height must be positive integers
  - UI002 (element bounds) — element x/y/w/h within display resolution
  - UI003 (font reference) — font names must match a declared @Font
  - UI004 (icon reference) — icon names must match a declared @Icon
  - UI005 (data binding module) — binding_module must reference an extracted module
  - UI006 (indicator LED reference) — led_id must match a declared @LED
  - UI007 (screen reference in navigation) — transition targets must be declared @Screen parts
  - UI008 (input reference in navigation) — input triggers must match declared @Input parts
- [x] Added UI extraction types (`ExtractedUI`, `ExtractedDisplay`, `ExtractedInput`, `ExtractedScreen`, `ExtractedElement`, `ExtractedIndicator`, `ExtractedIndicatorState`, `ExtractedFont`, `ExtractedIcon`) in `extraction/types.rs`
- [x] Added `extract_ui()` function in `extraction/mod.rs`
- [x] Updated `domains/firmware/domain.toml` with UI rule severities (UI001-008 all default to Error)
- [x] Added test fixtures: `ui_hardware.sysml` (valid UI spec), `ui_bad_bounds.sysml` (out-of-bounds elements), `ui_bad_refs.sysml` (invalid references)
