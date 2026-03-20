# Tool: sysml-v2-adapter

Domain-agnostic SysML v2 query library built on `syster-base`, providing structured APIs for metadata extraction, connection topology, and state machine analysis.

> **Note:** This document replaces the original `01-sysml-v2-parser.md` which specified a from-scratch parser. Following the syster-base evaluation (2.65/3.0 — GO), we use syster-base as a dependency instead. See [10-revised-plan.md](./10-revised-plan.md) for the decision rationale.

## Domain scope

This crate is **general-purpose SysML v2 tooling**. It operates on standard SysML v2 constructs (metadata annotations, connections, state machines, symbol kinds) without any knowledge of firmware concepts. The same adapter could serve an automotive, robotics, or aerospace SysML v2 project. Firmware-specific meaning is assigned by downstream crates (validate, extract, codegen).

## Purpose

Bridge the gap between syster-base's general-purpose HIR and the structured data our pipeline needs. The adapter:

1. Loads a multi-file SysML workspace and exposes parsed files with both CST and HIR
2. Extracts metadata annotation field values via CST traversal (not exposed in HIR)
3. Resolves connection topology from `ConnectionUsage` / `FlowConnectionUsage` symbols
4. Extracts state machine structure (states, transitions, guards, actions)
5. Maps `SymbolKind::Other` → `MetadataDefinition` for metadata def symbols

## Dependency

```toml
[dependencies]
syster-base = "=0.4.0-alpha"   # pinned — alpha API may break
thiserror = "2"
```

The `=0.4.0-alpha` pin is intentional. syster-base is alpha software; even patch bumps could break our adapter. We control upgrades explicitly.

## Modules

### `workspace.rs`

Loads a directory of `.sysml` files into a queryable workspace.

```rust
pub struct SysmlWorkspace { /* parsed files, symbol index */ }
pub struct ParsedFile {
    pub path: PathBuf,
    pub source: String,
    pub parse: Parse,
    pub syntax_file: SyntaxFile,
    pub symbols: Vec<HirSymbol>,
    pub file_id: FileId,
}

impl SysmlWorkspace {
    pub fn load(root: &Path) -> Result<Self, AdapterError>;
    pub fn all_symbols(&self) -> impl Iterator<Item = (&ParsedFile, &HirSymbol)>;
    pub fn symbols_of_kind(&self, kind: SymbolKind) -> Vec<(&ParsedFile, &HirSymbol)>;
    pub fn find_by_qualified_name(&self, name: &str) -> Option<(&ParsedFile, &HirSymbol)>;
    pub fn files(&self) -> &[ParsedFile];
}
```

### `metadata_extractor.rs`

Extracts structured metadata annotation values via CST traversal.

```rust
pub struct MetadataAnnotation {
    pub name: String,           // e.g. "MemoryModel"
    pub fields: Vec<MetadataField>,
}

pub struct MetadataField {
    pub name: String,           // e.g. "allocation"
    pub value: MetadataValue,
}

pub enum MetadataValue {
    EnumRef { enum_type: String, variant: String },  // AllocationKind::static_alloc
    Boolean(bool),
    Integer(i64),
    String(String),
    Tuple(Vec<MetadataValue>),
}

pub fn extract_metadata(file: &ParsedFile, part_symbol: &HirSymbol) -> Vec<MetadataAnnotation>;
```

**How it works:** The HIR identifies metadata usages as `AttributeUsage` symbols with supertypes like `["MemoryModel"]`. The adapter locates the corresponding CST node by source span, walks descendants to find `field = value;` assignments, and parses values into the `MetadataValue` enum.

### `connection_resolver.rs`

Resolves connection topology between parts.

```rust
pub struct Connection {
    pub name: String,
    pub kind: ConnectionKind,
    pub source: String,         // e.g. "bt.audioOut"
    pub target: String,         // e.g. "audioIn"
    pub flow_type: Option<String>,  // e.g. "Integer" for flow connections
}

pub enum ConnectionKind {
    Connect,
    Flow,
}

pub fn resolve_connections(file: &ParsedFile, part_symbol: &HirSymbol) -> Vec<Connection>;
```

**How it works:** `ConnectionUsage` and `FlowConnectionUsage` symbols encode source/target in their name and supertypes. The adapter parses these plus CST text for the `connect A to B` / `flow of T from A to B` patterns.

### `state_machine_extractor.rs`

Extracts state machine structure.

```rust
pub struct StateMachine {
    pub name: String,
    pub qualified_name: String,
    pub states: Vec<State>,
    pub transitions: Vec<Transition>,
    pub initial_state: Option<String>,
}

pub struct State {
    pub name: String,
    pub is_parallel: bool,
}

pub struct Transition {
    pub name: String,
    pub from_state: String,
    pub event: Option<String>,
    pub to_state: String,
    pub guard: Option<String>,
    pub action: Option<String>,
}

pub fn extract_state_machines(
    file: &ParsedFile,
    part_symbol: &HirSymbol,
) -> Vec<StateMachine>;
```

**How it works:** HIR provides `StateDefinition`, `StateUsage`, and `TransitionUsage` symbols with qualified names preserving the parent-child hierarchy. The adapter uses CST traversal for details: `entry; then X;` for initial state, `first X` for from-state, `accept E` for event, `then Y` for to-state, `if G` for guard.

### `symbol_kind_mapper.rs`

Maps `SymbolKind::Other` to more specific kinds.

```rust
pub enum MappedSymbolKind {
    Known(SymbolKind),
    MetadataDefinition,
}

pub fn classify_symbol(file: &ParsedFile, symbol: &HirSymbol) -> MappedSymbolKind;
```

**How it works:** syster-base maps `metadata def` to `SymbolKind::Other` rather than a dedicated variant. The adapter checks the CST at the symbol's source span for the `metadata def` keyword pair to correctly identify these.

## Known Workarounds

| Issue | Workaround |
|---|---|
| `metadata def` → `SymbolKind::Other` | CST keyword check in `symbol_kind_mapper.rs` |
| Metadata field values not in HIR | CST traversal in `metadata_extractor.rs` |
| Reserved keywords as identifiers | Source files must quote with single quotes: `'allocation'` |
| 45% documentation coverage | Read syster-base source for API discovery |
| Alpha version instability | Pin `=0.4.0-alpha` exactly |

## Tests

Tests are ported from `eval-syster/tests/` with structured assertions replacing the exploratory style.

### Workspace tests (from Phase 1+2)

```
test_load_workspace              — load all valid fixtures, no parse errors
test_all_symbols_count           — workspace has expected symbol count
test_symbols_of_kind_part_def    — finds BtA2dpSink, AudioPipeline, I2sOutput, StatusLed
test_symbols_of_kind_port_def    — finds AudioDataPort, StatusPort, LedCommandPort, I2sWritePort
test_symbols_of_kind_state_def   — finds ConnectionFSM, LedFSM
test_find_by_qualified_name      — "Firmware::BtA2dpSink" resolves
test_parse_error_recovery        — malformed.sysml loads with errors, doesn't panic
test_cst_round_trip              — source text preserved through parse
```

### Metadata tests (from Phase 3)

```
test_extract_memory_model        — allocation=static_alloc, maxInstances=1
test_extract_concurrency_model   — threadSafe=true, protection=mutex
test_extract_error_handling      — strategy=result
test_extract_isr_safe            — safe=false
test_extract_ownership           — owns=("Bluetooth controller",), borrows=()
test_missing_metadata            — I2sOutput has no @LayerConstraint
test_enum_ref_value              — AllocationKind::static_alloc parsed as EnumRef
test_boolean_value               — true parsed as Boolean(true)
test_integer_value               — 1 parsed as Integer(1)
test_tuple_value                 — ("x", "y") parsed as Tuple
```

### Connection tests (from Phase 4)

```
test_resolve_connect_statements  — 3 connect statements in AudioPipeline
test_resolve_flow_statement      — 1 flow of Integer
test_connection_source_target    — bt.audioOut → audioIn
test_flow_type                   — flow type is "Integer"
test_no_connections              — I2sOutput has no connections
```

### State machine tests (from Phase 5)

```
test_extract_connection_fsm      — ConnectionFSM found with 4 states, 7 transitions
test_initial_state               — initial state is "disconnected"
test_transition_from_state       — first transition from disconnected
test_transition_event            — accept StartDiscoveryEvent
test_transition_to_state         — then discovering
test_guard_expression            — if count > 0 (inline test)
test_parallel_regions            — parallel state parses (inline test)
test_led_fsm                     — LedFSM: 3 states, 6 transitions
```

### Symbol kind mapper tests

```
test_classify_metadata_def       — MemoryModel classified as MetadataDefinition
test_classify_part_def           — BtA2dpSink classified as Known(PartDefinition)
test_classify_enum_def           — LayerKind classified as Known(EnumerationDefinition)
```

### Integration test

```
test_full_workspace_extraction   — load workspace, extract all parts with metadata/connections/FSMs
                                   Assert ConnectionFSM: 4 states, 7 transitions, initial=disconnected
                                   Assert AudioPipeline: 3 connections, 1 flow
                                   Assert all 4 part defs have MemoryModel metadata
```

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("failed to read SysML file: {path}")]
    FileRead { path: PathBuf, source: std::io::Error },

    #[error("parse errors in {path}: {count} error(s)")]
    ParseErrors { path: PathBuf, count: usize },

    #[error("no .sysml files found in {path}")]
    EmptyWorkspace { path: PathBuf },
}
```

## Reference

- [syster-base on docs.rs](https://docs.rs/syster-base/) — API documentation
- [Evaluation results](./09-evaluation-syster-base.md) — hands-on evaluation with API patterns
- [Revised plan](./10-revised-plan.md) — architecture overview and decision log
