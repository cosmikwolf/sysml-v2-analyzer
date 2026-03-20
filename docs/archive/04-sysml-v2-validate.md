# Tool: sysml-v2-validate

Rust crate providing firmware-specific model validation on top of the parsed SysML v2 workspace.

## Purpose

The sysml-v2-parser performs general SysML v2 validation (syntax, name resolution, type checking). This tool adds **firmware domain rules** — the constraints that make the architecture specification useful for embedded code generation. It enforces layer dependency rules, metadata completeness, state machine well-formedness, port compatibility, and constraint satisfaction.

This is the equivalent of the current `/fw-validate-spec` and `/fw-audit-deps` commands, operating on SysML v2 models instead of YAML specs.

## Depends On

- **sysml-v2-adapter** — for workspace loading, metadata extraction, connection resolution, and state machine extraction (replaces sysml-v2-parser; see [01-sysml-v2-adapter.md](./01-sysml-v2-adapter.md))

## Validation Rules

### Layer dependency rules

Each part can declare a `layer` attribute using the `LayerKind` enum from the firmware metadata library. Connections and dependencies between parts must respect the layer hierarchy.

```
Rule: A part at layer L may only connect to / depend on parts at layers in allowed_deps(L).

Default layer rules (overridable in workspace config):
  application  → [middleware, driver]
  middleware   → [driver, hal]
  driver       → [hal]
  hal          → [pac]
  pac          → []
```

| Rule ID | Severity | Description |
|---|---|---|
| `FW001` | Error | Part at layer X connects to part at layer Y where Y is not in allowed_deps(X) |
| `FW002` | Warning | Part has no `layer` attribute — cannot verify dependency rules |
| `FW003` | Error | Circular dependency detected between parts |
| `FW004` | Info | Part depends on a part in the same layer (allowed but flagged for review) |

### Metadata completeness

Every part intended for code generation should have certain firmware metadata annotations.

| Rule ID | Severity | Description |
|---|---|---|
| `FW010` | Warning | Part definition has no `@MemoryModel` annotation |
| `FW011` | Warning | Part definition has no `@ConcurrencyModel` annotation |
| `FW012` | Warning | Part definition has no `@ErrorHandling` annotation |
| `FW013` | Info | Part definition has no `@ISRSafe` annotation (defaults to false) |
| `FW014` | Error | `@MemoryModel` specifies `heap` allocation but workspace global constraint forbids dynamic allocation |
| `FW015` | Error | `@ConcurrencyModel` specifies `none` but part has ISR-shared ports |

### State machine well-formedness

| Rule ID | Severity | Description |
|---|---|---|
| `FW020` | Error | State machine has no initial state (no `entry; then X;`) |
| `FW021` | Warning | State is unreachable (no incoming transitions and not initial) |
| `FW022` | Error | Non-deterministic transitions: same event triggers two transitions from same state without distinct guards |
| `FW023` | Warning | No error/fallback state defined |
| `FW024` | Error | Transition targets a state not defined in this state machine |
| `FW025` | Warning | State has no outgoing transitions (terminal state) — flagged unless explicitly marked |
| `FW026` | Error | Parallel region contains nested parallel regions (not supported in firmware codegen) |

### Port compatibility

| Rule ID | Severity | Description |
|---|---|---|
| `FW030` | Error | Connection between ports with incompatible types |
| `FW031` | Error | Connection between two output ports (no consumer) |
| `FW032` | Error | Connection between two input ports (no producer) |
| `FW033` | Warning | Port defined but not connected to anything |
| `FW034` | Error | Conjugated port used where non-conjugated is expected (direction mismatch) |

### Constraint satisfaction

| Rule ID | Severity | Description |
|---|---|---|
| `FW040` | Error | Constraint expression references attribute that does not exist on the constrained element |
| `FW041` | Warning | Constraint expression could not be statically evaluated — will need runtime check |
| `FW042` | Error | Contradictory constraints on the same element |

### Workspace-level rules

| Rule ID | Severity | Description |
|---|---|---|
| `FW050` | Error | Two part definitions have the same qualified name |
| `FW051` | Warning | Part definition exists but is never instantiated (dead code) |
| `FW052` | Error | Workspace declares a global constraint that is violated by a part's metadata |
| `FW053` | Warning | Import brings names into scope that shadow local definitions |

## Configuration

Validation rules are configurable via `sysml.toml`:

```toml
[validation]
# Override default layer dependency rules
[validation.layer_rules]
application = ["middleware", "driver"]
middleware = ["driver", "hal"]
driver = ["hal"]
hal = ["pac"]
pac = []

# Enable/disable specific rules
[validation.rules]
FW002 = "error"     # Upgrade "no layer" from warning to error
FW004 = "off"       # Disable same-layer dependency info
FW051 = "off"       # Allow unused definitions (library files)

# Required metadata annotations for code generation
[validation.required_metadata]
parts = ["MemoryModel", "ConcurrencyModel", "ErrorHandling"]
```

## Public API

```rust
pub struct ValidationConfig {
    pub layer_rules: HashMap<String, Vec<String>>,
    pub rule_overrides: HashMap<String, Severity>,
    pub required_metadata: RequiredMetadata,
}

pub struct ValidationResult {
    pub diagnostics: Vec<Diagnostic>,
    pub summary: ValidationSummary,
}

pub struct ValidationSummary {
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
    pub rules_checked: usize,
    pub parts_validated: usize,
    pub state_machines_validated: usize,
}

/// Run all firmware validation rules on a resolved workspace.
pub fn validate_firmware(
    workspace: &Workspace,
    config: &ValidationConfig,
) -> ValidationResult;

/// Run a specific rule category only.
pub fn validate_layers(workspace: &Workspace, config: &ValidationConfig) -> Vec<Diagnostic>;
pub fn validate_metadata(workspace: &Workspace, config: &ValidationConfig) -> Vec<Diagnostic>;
pub fn validate_state_machines(workspace: &Workspace) -> Vec<Diagnostic>;
pub fn validate_ports(workspace: &Workspace) -> Vec<Diagnostic>;
pub fn validate_constraints(workspace: &Workspace) -> Vec<Diagnostic>;
pub fn validate_workspace(workspace: &Workspace, config: &ValidationConfig) -> Vec<Diagnostic>;
```

## CLI Interface

```
$ sysml-v2-validate [OPTIONS] [PATH]

Arguments:
  [PATH]  Workspace root (default: current directory)

Options:
  -c, --config <FILE>     Path to sysml.toml (default: auto-detect)
  -r, --rule <RULE_ID>    Run only specific rule(s)
  -s, --severity <LEVEL>  Minimum severity to report (error, warning, info)
      --format <FMT>      Output format: text (default), json, sarif
      --fix               Apply auto-fixes where available
  -q, --quiet             Only output errors, suppress warnings and info
```

Output format example (text):
```
spec/bt_a2dp_sink.sysml:15:3 error[FW001]: layer violation: driver-layer part 'BtA2dpSink'
  connects to application-layer part 'MainApp'
  |
  15 |   connect self.btStatus to mainApp.statusIn;
  |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  = help: driver-layer parts may only connect to hal-layer parts

spec/audio_pipeline.sysml:8:1 warning[FW010]: part 'AudioPipeline' has no @MemoryModel annotation
  |
  8 | part def AudioPipeline {
  | ^^^^^^^^^^^^^^^^^^^^^^^^
  = help: add `@MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }`

Validated 4 parts, 2 state machines, 6 connections
Result: 1 error, 1 warning, 0 info
```

## Tests

### Layer dependency tests

```
test_layer_valid_driver_to_hal          — driver → hal connection passes
test_layer_valid_app_to_middleware      — application → middleware passes
test_layer_invalid_driver_to_app       — driver → application produces FW001
test_layer_invalid_hal_to_driver       — hal → driver produces FW001
test_layer_same_layer                  — driver → driver produces FW004 (info)
test_layer_missing_attribute           — part with no layer produces FW002
test_layer_custom_rules                — custom layer_rules in config are respected
test_layer_circular_dependency         — A→B→C→A produces FW003
test_layer_transitive                  — A→B→C where A→C would violate, but A→B and B→C are fine
test_layer_pac_no_deps                 — pac-layer part with any dependency produces FW001
```

### Metadata completeness tests

```
test_metadata_all_present               — part with all required annotations passes
test_metadata_missing_memory            — part without @MemoryModel produces FW010
test_metadata_missing_concurrency       — part without @ConcurrencyModel produces FW011
test_metadata_heap_forbidden            — heap allocation with no-dynamic-alloc constraint produces FW014
test_metadata_isr_no_protection         — ISR-shared port with no protection produces FW015
test_metadata_custom_required           — custom required_metadata config is respected
test_metadata_library_parts_skipped     — parts in library files not flagged (FW051 off)
```

### State machine tests

```
test_fsm_valid_simple                   — 3-state FSM with all transitions passes
test_fsm_no_initial_state               — missing `entry; then X;` produces FW020
test_fsm_unreachable_state              — state with no incoming transition produces FW021
test_fsm_nondeterministic               — same event, two transitions, no guards → FW022
test_fsm_nondeterministic_with_guards   — same event, two transitions, distinct guards → passes
test_fsm_no_error_state                 — no designated error state produces FW023
test_fsm_invalid_target                 — transition to nonexistent state produces FW024
test_fsm_terminal_state                 — state with no outgoing transitions produces FW025
test_fsm_nested_parallel                — parallel inside parallel produces FW026
test_fsm_valid_parallel                 — single-level parallel regions pass
```

### Port compatibility tests

```
test_port_valid_connection              — matching in/out ports with same type passes
test_port_type_mismatch                 — connecting Integer port to String port produces FW030
test_port_both_output                   — connecting two out ports produces FW031
test_port_both_input                    — connecting two in ports produces FW032
test_port_unused                        — defined but unconnected port produces FW033
test_port_conjugation_correct           — conjugated port reverses directions correctly
test_port_conjugation_mismatch          — wrong conjugation produces FW034
```

### Workspace-level tests

```
test_workspace_duplicate_names          — two parts with same qualified name produces FW050
test_workspace_unused_part              — defined but never instantiated part produces FW051
test_workspace_global_constraint        — global no-heap constraint + heap part produces FW052
test_workspace_shadow_import            — import shadows local definition produces FW053
```

### Configuration tests

```
test_config_rule_override_severity      — FW002 overridden to error is reported as error
test_config_rule_disabled               — disabled rule produces no diagnostics
test_config_custom_layer_rules          — non-default layer hierarchy is enforced
test_config_default_when_missing        — no config file uses sensible defaults
test_config_invalid_rule_id             — unknown rule ID in config produces config warning
```

### Output format tests

```
test_output_text                        — text format has file:line:col, severity, rule ID, message, context
test_output_json                        — JSON format has structured fields matching diagnostic struct
test_output_sarif                       — SARIF format is valid per SARIF 2.1.0 schema
test_output_quiet                       — quiet mode suppresses warnings and info
test_output_empty                       — no issues produces "Validated N parts... 0 errors" summary
```

### Integration tests

```
integration_full_workspace              — validate the complete firmware example workspace end-to-end
integration_clean_workspace_passes      — a correctly-authored workspace produces 0 errors
integration_mixed_errors                — workspace with known issues produces exactly expected diagnostics
integration_fix_and_revalidate          — fix a flagged issue, revalidate, confirm it's gone
```

## Dependencies (Rust crates)

- `sysml-v2-adapter` — our adapter crate (workspace loading, metadata, connections, FSMs)
- `petgraph` — graph algorithms for dependency cycle detection
- `serde` / `toml` — configuration parsing
- `codespan-reporting` — diagnostic rendering
- `serde_json` — JSON output format
