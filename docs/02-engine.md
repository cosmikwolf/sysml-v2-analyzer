# Component: engine

**Crate:** `sysml-v2-engine`
**Domain scope:** General-purpose pipeline framework — domain rules come from config + templates
**Status:** Not yet implemented

## Purpose

Domain-agnostic pipeline framework that loads a domain definition (`domain.toml` + templates) and runs validation, extraction, and code generation. The engine doesn't know about firmware, automotive, or any specific domain — it applies rules and templates defined externally.

## Modules

| Module | Purpose |
|---|---|
| `domain` | Load and merge `domain.toml` + `sysml.toml` into `DomainConfig` |
| `validation` | Rule engine: layer dependency checking, required metadata, FSM well-formedness |
| `extraction` | Flatten metadata annotations + connections + FSMs into structured output |
| `codegen` | MiniJinja template rendering with domain-specific context |
| `diagnostic` | Shared `Diagnostic` type used across all stages |

## Domain config loading (`domain.rs`)

```rust
pub struct DomainConfig {
    pub name: String,
    pub metadata_library: PathBuf,
    pub layers: LayerConfig,
    pub required_metadata: RequiredMetadataConfig,
    pub type_map: HashMap<String, HashMap<String, String>>,  // language → SysML type → target type
    pub validation_rules: HashMap<String, Severity>,
    pub template_dir: PathBuf,
}

pub struct LayerConfig {
    pub order: Vec<String>,
    pub allowed_deps: HashMap<String, Vec<String>>,
}

impl DomainConfig {
    /// Load domain.toml, then overlay sysml.toml project overrides.
    pub fn load(domain_dir: &Path, project_config: Option<&Path>) -> Result<Self, EngineError>;
}
```

## Validation engine (`validation.rs`)

The validation engine runs generic checks parameterized by `DomainConfig`:

| Check | Config source | Description |
|---|---|---|
| Layer dependencies | `layers.allowed_deps` | Element at layer X connects to element at layer Y — is Y in allowed_deps(X)? |
| Required metadata | `required_metadata.parts` | Does each part definition have the required annotations? |
| FSM well-formedness | Built-in (generic) | Initial state exists, all states reachable, transitions deterministic, targets valid |
| Port compatibility | Built-in (generic) | Connected ports have compatible types and directions |
| Workspace rules | Built-in (generic) | No duplicate qualified names, no unused definitions |

```rust
pub struct ValidationResult {
    pub diagnostics: Vec<Diagnostic>,
    pub parts_checked: usize,
    pub state_machines_checked: usize,
}

pub fn validate(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
) -> ValidationResult;
```

Rules have IDs (e.g., `FW001`). The ID prefix comes from the domain config. Severity can be overridden per-project in `sysml.toml`.

## Extraction engine (`extraction.rs`)

Flattens the adapter's structured types into serializable output:

```rust
pub struct ExtractedModule {
    pub name: String,
    pub qualified_name: String,
    pub layer: Option<String>,
    pub metadata: HashMap<String, HashMap<String, serde_json::Value>>,
    pub ports: Vec<ExtractedPort>,
    pub actions: Vec<ExtractedAction>,
    pub connections: Vec<ExtractedConnection>,
    pub state_machines: Vec<ExtractedStateMachine>,
}

pub fn extract(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
) -> Result<ExtractionResult, EngineError>;
```

Extraction only proceeds if validation passes (no errors). Warnings are allowed.

## Code generation engine (`codegen.rs`)

MiniJinja-based template rendering:

```rust
pub struct CodegenContext {
    pub module: ExtractedModule,
    pub domain: DomainConfig,
    pub target_language: String,
}

pub fn generate(
    extraction: &ExtractionResult,
    config: &DomainConfig,
    output_dir: &Path,
) -> Result<GenerationReport, EngineError>;
```

The engine:
1. Creates a MiniJinja `Environment` with `trim_blocks`, `lstrip_blocks`, no auto-escape
2. Loads templates from `domains/<name>/templates/<language>/`
3. Registers standard filters: `snake_case`, `pascal_case`, `screaming_snake`, `map_type`
4. Renders each template with the extracted module data as context
5. Writes output files, skipping unchanged files (spec-hash fingerprinting)

## Diagnostics (`diagnostic.rs`)

```rust
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub severity: Severity,
    pub rule_id: String,
    pub message: String,
    pub help: Option<String>,
}

pub enum Severity { Error, Warning, Info }
```

## Dependencies

- `sysml-v2-adapter` — workspace loading, metadata, connections, FSMs
- `minijinja` — template engine
- `petgraph` — graph algorithms for cycle detection, reachability
- `serde` + `toml` + `serde_json` + `serde_yaml` — config parsing, extraction output
- `thiserror` — error types

## Design principles

1. **The engine never imports domain-specific constants.** No `"MemoryModel"`, `"ISRSafe"`, or `"driver"` strings in engine code. All domain knowledge comes from `DomainConfig`.
2. **Validation rules are parameterized.** "Required metadata for parts" reads the list from config, not a hardcoded array.
3. **Templates are loaded at runtime.** The engine doesn't compile templates into the binary.
4. **Extraction output is a generic tree of metadata key-value pairs.** The engine doesn't know what the keys mean — it just flattens what the adapter gives it.
