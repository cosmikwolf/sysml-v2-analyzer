# Phase 4: Extraction Engine

## Goal

Implement the extraction engine that flattens adapter types into serializable output (YAML/JSON). The engine produces a generic tree of metadata key-value pairs — it doesn't interpret what the keys mean.

## Extracted types

### ExtractedModule

One per `PartDefinition` that has metadata annotations:

```rust
pub struct ExtractedModule {
    pub name: String,
    pub qualified_name: String,
    pub source_file: PathBuf,
    pub layer: Option<String>,                              // from layer attribute, if present
    pub metadata: HashMap<String, HashMap<String, Value>>,  // annotation name → field → value
    pub ports: Vec<ExtractedPort>,
    pub actions: Vec<ExtractedAction>,
    pub connections: Vec<ExtractedConnection>,
    pub state_machines: Vec<ExtractedStateMachine>,
}
```

### ExtractedStateMachine

```rust
pub struct ExtractedStateMachine {
    pub name: String,
    pub owner_module: String,
    pub states: Vec<ExtractedState>,
    pub transitions: Vec<ExtractedTransition>,
    pub initial_state: Option<String>,
}
```

### ExtractedPort, ExtractedAction, ExtractedConnection

Flat representations of adapter types with `serde::Serialize` derives.

### ExtractionResult

```rust
pub struct ExtractionResult {
    pub modules: Vec<ExtractedModule>,
    pub architecture: ExtractedArchitecture,  // workspace-level summary
}

pub struct ExtractedArchitecture {
    pub source_files: Vec<PathBuf>,
    pub modules: Vec<ModuleSummary>,  // name, layer, file reference
    pub dependency_graph: Vec<(String, String)>,  // (from, to) pairs
}
```

## Extraction pipeline

1. **Validation gate** — call `validate()` first. If any `Error`-severity diagnostics exist, refuse extraction. Warnings are allowed.
2. **Module extraction** — for each `PartDefinition` with metadata, call `extract_metadata()`, `resolve_connections()`, `extract_state_machines()`, flatten into `ExtractedModule`.
3. **Architecture extraction** — build workspace-level summary from all modules.
4. **Serialization** — write to YAML or JSON (configurable).

## Public API

```rust
pub fn extract(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
    validation: &ValidationResult,
) -> Result<ExtractionResult, EngineError>;

pub fn write_extraction(
    result: &ExtractionResult,
    output_dir: &Path,
    format: OutputFormat,  // Yaml | Json
) -> Result<(), io::Error>;
```

## Metadata flattening

Adapter `MetadataValue` → `serde_json::Value`:

| MetadataValue | JSON |
|---|---|
| `EnumRef { enum_type, variant }` | `"EnumType::variant"` (string) |
| `Boolean(b)` | `true` / `false` |
| `Integer(n)` | `42` |
| `String(s)` | `"hello"` |
| `Tuple(values)` | `["a", "b"]` |

## Output format

YAML example for a module:

```yaml
name: BtA2dpSink
qualified_name: Firmware::BtA2dpSink
source_file: spec/bt_a2dp_sink.sysml
layer: driver
metadata:
  MemoryModel:
    allocation: "AllocationKind::static_alloc"
    maxInstances: 1
  ConcurrencyModel:
    threadSafe: true
    protection: "ProtectionKind::mutex"
  ISRSafe:
    safe: false
ports:
  - name: audioOut
    direction: out
    type: AudioDataPort
    conjugated: true
connections:
  - source: bt.audioOut
    target: audioIn
    kind: connect
state_machines:
  - name: ConnectionFSM
    initial_state: disconnected
    states: [disconnected, discovering, connected, streaming]
    transitions:
      - from: disconnected
        event: StartDiscoveryEvent
        to: discovering
```

## Tests

- Extract single module → verify all fields populated
- Extract workspace → verify all 4 part defs extracted
- Round-trip: extract → serialize YAML → deserialize → compare
- Determinism: extract twice → identical output
- Validation gate: extraction with errors → `EngineError`
- Validation gate: extraction with warnings → proceeds
- Module without metadata → not extracted
- Module with partial metadata → extracted with available fields

## Verification

```
cargo test -p sysml-v2-engine    # extraction tests pass
cargo clippy --workspace         # clean
```
