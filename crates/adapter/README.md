# sysml-v2-adapter

Domain-agnostic SysML v2 query library built on [syster-base](https://crates.io/crates/syster-base). Provides structured APIs for metadata extraction, connection topology, and state machine analysis.

## Domain scope

This crate is **general-purpose SysML v2 tooling**. It knows nothing about firmware, embedded systems, or any specific domain. It extracts `@Annotation { field = value; }` bodies, resolves `connect`/`flow` topology, and parses `state def` structures — all of which are standard SysML v2 constructs that exist in any domain model. Firmware-specific meaning is assigned by the engine crate, driven by domain config.

## Why this exists

syster-base provides a general-purpose SysML v2 parser with an HIR layer, but three things aren't directly exposed:

1. **Metadata annotation values** — The HIR identifies *which* annotations exist but doesn't expose `@M { field = value; }` field values. This crate extracts them via CST traversal.
2. **Symbol kind correction** — `metadata def` declarations map to `SymbolKind::Other` in the HIR. This crate reclassifies them as `MetadataDefinition`.
3. **Structured extraction** — Connection topology and state machine details require combining HIR symbols with CST text parsing. This crate provides clean APIs for both.

## Modules

### `workspace` — Workspace loading and querying

```rust
use sysml_v2_adapter::{SysmlWorkspace, SymbolKind};

let ws = SysmlWorkspace::load("spec/".as_ref())?;

// Iterate all part definitions
for (file, sym) in ws.symbols_of_kind(SymbolKind::PartDefinition) {
    println!("{}", sym.name);
}

// Find by name
let (file, sym) = ws.find_by_qualified_name("BtA2dpSink").unwrap();
```

Key types:
- `SysmlWorkspace` — Loaded workspace with parsed files and symbol index
- `ParsedFile` — A single `.sysml` file with CST (`parse`), HIR (`symbols`), and source text
- `AdapterError` — Error type for file I/O, parse errors, empty workspace

### `metadata_extractor` — Annotation value extraction

```rust
use sysml_v2_adapter::metadata_extractor::{extract_metadata, MetadataValue};

let annotations = extract_metadata(file, part_symbol);
for ann in &annotations {
    for field in &ann.fields {
        match &field.value {
            MetadataValue::EnumRef { enum_type, variant } => { /* ... */ }
            MetadataValue::Boolean(b) => { /* ... */ }
            MetadataValue::Integer(n) => { /* ... */ }
            MetadataValue::String(s) => { /* ... */ }
            MetadataValue::Tuple(values) => { /* ... */ }
        }
    }
}
```

Key types:
- `MetadataAnnotation` — Name + fields from a `@Name { ... }` block
- `MetadataField` — A single `field = value` assignment
- `MetadataValue` — Parsed value (enum ref, bool, int, string, tuple)

### `connection_resolver` — Connection topology

```rust
use sysml_v2_adapter::connection_resolver::{resolve_connections, ConnectionKind};

let connections = resolve_connections(file, part_symbol);
for conn in &connections {
    match conn.kind {
        ConnectionKind::Connect => println!("connect {} to {}", conn.source, conn.target),
        ConnectionKind::Flow => println!("flow of {:?} from {} to {}",
            conn.flow_type, conn.source, conn.target),
    }
}
```

Key types:
- `Connection` — Source, target, kind, and optional flow type
- `ConnectionKind` — `Connect` or `Flow`

### `state_machine_extractor` — FSM structure

```rust
use sysml_v2_adapter::state_machine_extractor::extract_state_machines;

let machines = extract_state_machines(file, part_symbol);
for fsm in &machines {
    println!("FSM: {} (initial: {:?})", fsm.name, fsm.initial_state);
    for state in &fsm.states {
        println!("  state {}{}", state.name, if state.is_parallel { " [parallel]" } else { "" });
    }
    for t in &fsm.transitions {
        println!("  {} --{:?}--> {}", t.from_state, t.event, t.to_state);
    }
}
```

Key types:
- `StateMachine` — Name, states, transitions, initial state
- `State` — Name and parallel flag
- `Transition` — From, event, to, guard, action

### `symbol_kind_mapper` — Corrected symbol classification

```rust
use sysml_v2_adapter::symbol_kind_mapper::{classify_symbol, MappedSymbolKind};

match classify_symbol(file, symbol) {
    MappedSymbolKind::MetadataDefinition => println!("metadata def"),
    MappedSymbolKind::Known(kind) => println!("{:?}", kind),
}
```

## Known quirks

| Issue | Detail |
|---|---|
| 0-indexed lines | syster-base HIR uses 0-indexed `start_line`/`end_line` |
| Name-only spans | HIR spans cover the *name* only, not the full body — use `extract_definition_body()` |
| Reserved keywords | `allocation`, `action`, `state`, `port`, `part`, `flow`, `import` must be quoted with `'` in SysML source |
| Alpha API | Pinned to `=0.4.0-alpha` — even patch bumps could break |

## Tests

```bash
cargo test -p sysml-v2-adapter
```

45 tests: 36 unit tests across all modules + 9 integration tests exercising the full pipeline on fixture data.
