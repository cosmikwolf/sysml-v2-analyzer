# Phase 2: Engine Scaffold + Domain Config

## Goal

Restructure the workspace from the old 6-crate layout to the new 3-crate layout (adapter, engine, cli). Create the firmware domain directory. Implement domain config loading and merging.

## Restructure tasks

### Remove old scaffold crates

Delete `crates/validate/`, `crates/extract/`, `crates/gencontract/`, `crates/codegen/` (all are empty scaffolds with no implementation code).

### Create engine crate

```
crates/engine/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── domain.rs         # DomainConfig loading + merging
    └── diagnostic.rs     # Shared Diagnostic type
```

Dependencies:
- `sysml-v2-adapter`
- `serde` + `toml` + `serde_json`
- `thiserror`

### Create template domain (for engine tests + starter)

```
domains/template/
├── domain.toml               # minimal: 2 layers, 1 required metadata
├── template_library.sysml    # minimal: 1 metadata def, 1 enum
└── templates/
    └── rust/
        └── module.rs.j2      # trivial template
```

Engine tests use this domain, not the firmware domain. This keeps engine tests domain-agnostic.

### Create firmware domain

```
domains/firmware/
├── domain.toml
├── firmware_library.sysml    # authoritative copy (not in tests/fixtures/)
└── templates/
    └── rust/                 # empty for now, populated in Phase 5
```

`firmware_library.sysml` lives here as the authoritative source. The copy in `tests/fixtures/` remains for adapter tests (which are domain-agnostic and don't load domains).

### Create example sysml.toml

At the workspace root (`tools/sysml-v2-analyzer/sysml.toml`):

```toml
[workspace]
domain = "firmware"
include = ["tests/fixtures/**/*.sysml"]
```

### Update workspace Cargo.toml

Change members from 6 crates to 3:

```toml
[workspace]
members = ["crates/adapter", "crates/engine", "crates/cli"]
```

### Update CLI crate

Update `crates/cli/Cargo.toml` to depend on `sysml-v2-engine` instead of the old crates.

## Implementation: domain.rs

### DomainConfig struct

```rust
pub struct DomainConfig {
    pub name: String,
    pub description: Option<String>,
    pub metadata_library: PathBuf,
    pub layers: LayerConfig,
    pub required_metadata: RequiredMetadataConfig,
    pub type_map: HashMap<String, HashMap<String, String>>,
    pub validation_rules: HashMap<String, Severity>,
    pub template_dir: PathBuf,
}

pub struct LayerConfig {
    pub order: Vec<String>,
    pub allowed_deps: HashMap<String, Vec<String>>,
}

pub struct RequiredMetadataConfig {
    pub parts: Vec<String>,
}
```

### Loading and merging

```rust
impl DomainConfig {
    /// Load from domain directory, optionally merging project overrides.
    pub fn load(domain_dir: &Path, project_config: Option<&Path>) -> Result<Self, EngineError>;
}
```

1. Parse `domain_dir/domain.toml` into `DomainConfig`
2. If `project_config` exists, parse `sysml.toml`
3. Merge: domain defaults ← project overrides (project wins for any key present in both)

### WorkspaceConfig (from sysml.toml)

```rust
pub struct WorkspaceConfig {
    pub domain: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub validation_overrides: HashMap<String, Severity>,
    pub required_metadata_overrides: Option<RequiredMetadataConfig>,
}
```

## Implementation: diagnostic.rs

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

pub enum Severity { Error, Warning, Info, Off }
```

## Tests

- Load `domains/firmware/domain.toml` → verify all fields parsed correctly
- Load with `sysml.toml` overrides → verify merge behavior
- Missing `domain.toml` → clear error
- Invalid TOML → clear error with file path
- Severity override: domain says `warning`, project says `error` → `error` wins
- Severity override: project says `off` → rule disabled

## Verification

```
cargo build                      # workspace compiles with new structure
cargo test -p sysml-v2-adapter   # adapter tests still pass (unchanged)
cargo test -p sysml-v2-engine    # domain config tests pass
cargo clippy --workspace         # clean
```
