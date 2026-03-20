# Tool: sysml-v2-extract

Rust crate and CLI that extracts structured data from a parsed SysML v2 workspace into JSON/YAML for the code generation pipeline.

## Purpose

Bridge the architecture layer (SysML v2) to the generation contract layer (custom YAML). This tool queries the parsed model and emits a normalized, flat representation of each module, interface, state machine, and connection — ready to be consumed by the code generator without needing to understand SysML v2 syntax.

This is the seam between the standardized architecture spec and the proprietary generation pipeline.

## Depends On

- **sysml-v2-adapter** — for workspace loading, metadata extraction, connection resolution, and state machine extraction (replaces sysml-v2-parser; see [01-sysml-v2-adapter.md](./01-sysml-v2-adapter.md))
- **sysml-v2-validate** — extraction should refuse to proceed if validation errors exist

## Extraction Targets

### Module extraction

For each `part def` with firmware metadata annotations, emit a module descriptor:

```yaml
# Output: extracted/modules/bt_a2dp_sink.yaml
module: BtA2dpSink
qualified_name: Firmware::Drivers::BtA2dpSink
source_file: spec/bt_a2dp_sink.sysml
source_span: { line: 5, col: 1, end_line: 45, end_col: 1 }

layer: driver

metadata:
  memory:
    allocation: static
    max_instances: 1
  concurrency:
    thread_safe: true
    isr_shared: false
    protection: mutex
  error_handling: result
  isr_safe: false

owns: [bluetooth_controller]    # from @Ownership metadata
borrows: [i2s_peripheral]      # from @Ownership metadata

ports:
  - name: audioOut
    direction: out
    type: AudioData
    conjugated: false
  - name: btStatus
    direction: in
    type: ConnectionStatus
    conjugated: false

actions:
  - name: init
    inputs: []
    outputs: [{ name: result, type: "Result<Self, A2dpError>" }]
    blocking: false
    isr_safe: false
  - name: start_discovery
    inputs: [{ name: config, type: A2dpConfig }]
    outputs: [{ name: result, type: "Result<(), A2dpError>" }]
    blocking: true
    isr_safe: false

depends_on:
  - { module: I2SOutput, via_port: audioOut, target_port: audioIn }
  - { module: StatusLed, via_port: btStatus, target_port: statusIn }

constraints:
  - "No dynamic allocation"
  - "Must not block for more than 100ms"

specializes: null   # or parent part def qualified name
notes: "A2DP Bluetooth audio sink driver for ESP32"
```

### State machine extraction

For each `state def` inside a part, emit a state machine descriptor:

```yaml
# Output: extracted/state_machines/bt_a2dp_sink_connection_fsm.yaml
name: ConnectionFSM
owner_module: BtA2dpSink
source_file: spec/bt_a2dp_sink.sysml
source_span: { line: 20, col: 5, end_line: 38, end_col: 5 }

states:
  - { name: disconnected, is_initial: true, is_error: false }
  - { name: connecting, is_initial: false, is_error: false }
  - { name: streaming, is_initial: false, is_error: false }
  - { name: error, is_initial: false, is_error: true }

events:
  - ConnectEvent
  - ConnectedEvent
  - DisconnectEvent
  - TimeoutEvent
  - ErrorEvent

transitions:
  - { from: disconnected, event: ConnectEvent, to: connecting, guard: null, action: start_discovery }
  - { from: connecting, event: ConnectedEvent, to: streaming, guard: null, action: on_connected }
  - { from: connecting, event: TimeoutEvent, to: disconnected, guard: null, action: on_timeout }
  - { from: connecting, event: ErrorEvent, to: error, guard: null, action: on_error }
  - { from: streaming, event: DisconnectEvent, to: disconnected, guard: null, action: on_disconnect }
  - { from: streaming, event: ErrorEvent, to: error, guard: null, action: on_error }

parallel_regions: []   # or list of region descriptors
```

### Interface extraction

For each `port def` that defines a shared contract, emit an interface descriptor:

```yaml
# Output: extracted/interfaces/audio_data_port.yaml
interface: AudioDataPort
source_file: spec/interfaces.sysml
source_span: { line: 3, col: 1, end_line: 8, end_col: 1 }

functions:
  - name: write_samples
    inputs: [{ name: samples, type: "&[i16]" }]
    outputs: [{ name: result, type: "Result<usize, AudioError>" }]
    direction: out
    blocking: true
    isr_safe: false
  - name: available_space
    inputs: []
    outputs: [{ name: space, type: usize }]
    direction: out
    blocking: false
    isr_safe: true
```

### Architecture extraction

Emit a workspace-level summary:

```yaml
# Output: extracted/architecture.yaml
project: esp32_bluetooth
source_files:
  - spec/bt_a2dp_sink.sysml
  - spec/audio_pipeline.sysml
  - spec/i2s_output.sysml
  - spec/status_led.sysml
  - spec/interfaces.sysml
  - lib/firmware.sysml

modules:
  - { name: BtA2dpSink, layer: driver, file: extracted/modules/bt_a2dp_sink.yaml }
  - { name: AudioPipeline, layer: middleware, file: extracted/modules/audio_pipeline.yaml }
  - { name: I2SOutput, layer: driver, file: extracted/modules/i2s_output.yaml }
  - { name: StatusLed, layer: driver, file: extracted/modules/status_led.yaml }

state_machines:
  - { name: ConnectionFSM, owner: BtA2dpSink, file: extracted/state_machines/bt_a2dp_sink_connection_fsm.yaml }

interfaces:
  - { name: AudioDataPort, file: extracted/interfaces/audio_data_port.yaml }
  - { name: StatusPort, file: extracted/interfaces/status_port.yaml }

dependency_graph:
  - { from: BtA2dpSink, to: I2SOutput }
  - { from: BtA2dpSink, to: StatusLed }
  - { from: AudioPipeline, to: BtA2dpSink }
  - { from: AudioPipeline, to: I2SOutput }

global_constraints:
  - "No dynamic allocation"
  - "All modules must have @MemoryModel annotation"
```

## Public API

```rust
pub struct ExtractionConfig {
    pub output_dir: PathBuf,
    pub format: OutputFormat,        // Yaml | Json
    pub include_source_spans: bool,
    pub include_notes: bool,
}

pub struct ExtractionResult {
    pub modules: Vec<ExtractedModule>,
    pub state_machines: Vec<ExtractedStateMachine>,
    pub interfaces: Vec<ExtractedInterface>,
    pub architecture: ExtractedArchitecture,
    pub warnings: Vec<String>,
}

/// Extract all firmware-relevant data from a validated workspace.
pub fn extract(
    workspace: &Workspace,
    config: &ExtractionConfig,
) -> Result<ExtractionResult, ExtractionError>;

/// Extract a single module by qualified name.
pub fn extract_module(
    workspace: &Workspace,
    qualified_name: &str,
) -> Result<ExtractedModule, ExtractionError>;

/// Write extraction results to disk.
pub fn write_extraction(
    result: &ExtractionResult,
    config: &ExtractionConfig,
) -> Result<(), io::Error>;
```

## CLI Interface

```
$ sysml-v2-extract [OPTIONS] [PATH]

Arguments:
  [PATH]  Workspace root (default: current directory)

Options:
  -o, --output <DIR>      Output directory (default: extracted/)
  -f, --format <FMT>      Output format: yaml (default), json
  -m, --module <NAME>     Extract only specific module(s)
      --no-validate       Skip validation before extraction (not recommended)
      --include-spans     Include source file spans in output
      --dry-run           Show what would be extracted without writing files
  -q, --quiet             Minimal output
```

## Extraction Rules

1. **Only `part def` elements with firmware metadata annotations are extracted as modules.** Library definitions, helper types, and unannotated parts are skipped.

2. **Metadata is flattened.** SysML v2 metadata annotations (`@MemoryModel { allocation = ... }`) are flattened into the YAML structure the code generator expects.

3. **Type names are preserved as strings.** The extracted YAML does not try to resolve SysML v2 types into target-language types — that mapping happens in the generation contract.

4. **Connections become `depends_on` entries.** SysML v2 `connect` and `flow` statements are translated into the module's dependency list with port information.

5. **Actions become interface functions.** SysML v2 `action` usages inside a part become the module's interface function list. Input/output parameters are preserved.

6. **Source spans are optional.** They enable traceability from generated code back to the SysML v2 source, but can be omitted for cleaner output.

7. **Extraction refuses to proceed if validation has errors.** Warnings are allowed; errors are not. Use `--no-validate` to override (at your own risk).

## Tests

### Module extraction tests

```
test_extract_simple_module              — part def with metadata → correct module YAML
test_extract_module_all_metadata        — all firmware metadata annotations correctly flattened
test_extract_module_ports               — ports with directions and types extracted
test_extract_module_actions             — action usages with params extracted as interface functions
test_extract_module_dependencies        — connections translated to depends_on entries
test_extract_module_constraints         — constraint defs on part extracted as constraint strings
test_extract_module_specialization      — specializing part records parent reference
test_extract_module_owns_borrows        — @Ownership metadata split into owns/borrows lists
test_extract_module_no_metadata_skipped — part def without firmware metadata not extracted
test_extract_module_notes               — doc comments extracted as notes field
```

### State machine extraction tests

```
test_extract_fsm_simple                 — 3-state FSM extracted with all transitions
test_extract_fsm_initial_state          — initial state marked correctly
test_extract_fsm_error_state            — error state marked correctly
test_extract_fsm_guards                 — guard expressions preserved as strings
test_extract_fsm_actions                — transition actions referenced by name
test_extract_fsm_parallel               — parallel regions extracted as separate region descriptors
test_extract_fsm_nested_in_part         — FSM extracted with correct owner_module reference
test_extract_fsm_time_trigger           — `accept after 60 [s]` extracted as timed event
```

### Interface extraction tests

```
test_extract_interface_simple           — port def with functions extracted
test_extract_interface_directions       — in/out directions preserved
test_extract_interface_types            — parameter types preserved as strings
test_extract_interface_blocking         — blocking annotation extracted from metadata
```

### Architecture extraction tests

```
test_extract_architecture_modules       — all modules listed with layers and file paths
test_extract_architecture_deps          — dependency graph correctly computed from connections
test_extract_architecture_constraints   — global constraints collected
test_extract_architecture_source_files  — all parsed .sysml files listed
```

### Round-trip tests

```
test_roundtrip_extract_matches_source   — extracted data matches what's in the .sysml source
test_roundtrip_yaml_parseable           — emitted YAML is valid and parseable
test_roundtrip_json_parseable           — emitted JSON is valid and parseable
test_roundtrip_deterministic            — extracting twice produces identical output
```

### Error handling tests

```
test_extract_refuses_on_validation_error    — validation error → ExtractionError, no output
test_extract_proceeds_on_validation_warning — validation warning → extraction proceeds with warning
test_extract_no_validate_flag               — --no-validate skips validation check
test_extract_unknown_module_name            — --module NonExistent → clear error message
test_extract_empty_workspace                — workspace with no firmware parts → empty extraction
```

### Integration tests

```
integration_full_workspace_extraction   — extract complete example workspace, verify all files
integration_incremental_extraction      — change one .sysml file, re-extract, only affected module changes
integration_extract_then_codegen        — extracted YAML is consumable by sysml-v2-codegen (end-to-end)
```

## Dependencies (Rust crates)

- `sysml-v2-adapter` — our adapter crate (workspace loading, metadata, connections, FSMs)
- `sysml-v2-validate` — our validation crate
- `serde` / `serde_yaml` / `serde_json` — serialization
- `clap` — CLI argument parsing
