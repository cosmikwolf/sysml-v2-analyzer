# Component: adapter

**Crate:** `sysml-v2-adapter`
**Domain scope:** General-purpose SysML v2 — no domain knowledge
**Status:** Implemented (45 tests passing)

## Purpose

Domain-agnostic SysML v2 query library built on syster-base. Provides structured APIs for metadata extraction, connection topology, and state machine analysis that work for any SysML v2 project regardless of domain.

## Modules

| Module | Purpose |
|---|---|
| `workspace` | Load `.sysml` files into a queryable workspace with CST + HIR |
| `metadata_extractor` | Extract `@Annotation { field = value; }` bodies via CST traversal |
| `connection_resolver` | Resolve `connect`/`flow` topology into structured `Connection` types |
| `state_machine_extractor` | Extract `state def` structure: states, transitions, guards, actions |
| `symbol_kind_mapper` | Reclassify `SymbolKind::Other` → `MetadataDefinition` via CST inspection |

## Public API

### Workspace

```rust
SysmlWorkspace::load(root: &Path) -> Result<Self, AdapterError>
SysmlWorkspace::from_sources(sources: Vec<(PathBuf, String)>) -> Self
SysmlWorkspace::files() -> &[ParsedFile]
SysmlWorkspace::all_symbols() -> impl Iterator<Item = (&ParsedFile, &HirSymbol)>
SysmlWorkspace::symbols_of_kind(kind: SymbolKind) -> Vec<(&ParsedFile, &HirSymbol)>
SysmlWorkspace::find_by_qualified_name(name: &str) -> Option<(&ParsedFile, &HirSymbol)>
extract_definition_body(source: &str, symbol: &HirSymbol) -> Option<String>
```

### Metadata

```rust
extract_metadata(file: &ParsedFile, part_symbol: &HirSymbol) -> Vec<MetadataAnnotation>
extract_all_metadata(file: &ParsedFile) -> Vec<MetadataAnnotation>
```

`MetadataValue` variants: `EnumRef`, `Boolean`, `Integer`, `String`, `Tuple`.

### Connections

```rust
resolve_connections(file: &ParsedFile, part_symbol: &HirSymbol) -> Vec<Connection>
```

`ConnectionKind` variants: `Connect`, `Flow`.

### State machines

```rust
extract_state_machines(file: &ParsedFile, part_symbol: &HirSymbol) -> Vec<StateMachine>
```

`StateMachine` contains `states: Vec<State>`, `transitions: Vec<Transition>`, `initial_state: Option<String>`.

### Symbol classification

```rust
classify_symbol(file: &ParsedFile, symbol: &HirSymbol) -> MappedSymbolKind
```

## Known quirks (syster-base 0.4.0-alpha)

| Quirk | Detail |
|---|---|
| 0-indexed lines | HIR `start_line`/`end_line` are 0-indexed |
| Name-only spans | HIR spans cover the name only, not the full body — use `extract_definition_body()` |
| `metadata def` → `Other` | `SymbolKind::Other` instead of `MetadataDefinition` — use `classify_symbol()` |
| Reserved keywords | `allocation`, `action`, `state`, `port`, `part`, `flow`, `import` must be quoted with `'` |
| Alpha API | Pinned to `=0.4.0-alpha` — even patch bumps could break |

## Dependencies

- `syster-base = "=0.4.0-alpha"` — SysML v2 parser + HIR
- `thiserror` — error types

No domain-specific dependencies.

## Tests

36 unit tests + 9 integration tests covering:
- Workspace loading and symbol querying
- Metadata value extraction (enum refs, booleans, integers, tuples)
- Connection resolution (connect + flow statements)
- State machine extraction (states, transitions, initial state, events)
- Symbol kind classification (metadata def detection)
- Error recovery on malformed input
- CST round-trip losslessness
