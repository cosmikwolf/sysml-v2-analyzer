# Tool: sysml-v2-codegen

Rust crate and CLI that generates source code from extracted architecture data combined with a generation contract.

## Purpose

The final stage of the pipeline. Takes the extracted module/interface/FSM descriptors (from sysml-v2-extract) and the generation contract (from sysml-v2-gencontract), and produces target-language source files, test stubs, and build system fragments.

This replaces the current `/fw-gen-module`, `/fw-gen-statemachine`, `/fw-gen-architecture`, and `/fw-gen-all` commands.

## Depends On

- **sysml-v2-extract** — extracted YAML/JSON descriptors
- **sysml-v2-gencontract** — generation contract with platform config, type mappings, naming, layout

## What It Generates

### Per module

| Language | Files | Description |
|---|---|---|
| Rust | `src/<module>/mod.rs` | Module implementation skeleton |
| Rust | `src/<module>/tests.rs` | Test stubs (if `tests.test_structure: separate`) |
| C | `src/include/<module>.h` | Public header with opaque types, function prototypes, error enum |
| C | `src/<module>/<module>.c` | Implementation skeleton |
| C | `tests/<module>_test.c` | Test stubs |
| C++ | `src/include/<module>.hpp` | Public header with class declaration |
| C++ | `src/<module>/<module>.cpp` | Implementation skeleton |
| C++ | `tests/<module>_test.cpp` | Test stubs |

### Per state machine

| Language | Output |
|---|---|
| Rust | State enum, event enum, transition function, embedded in owner module |
| C | State/event enums in header, transition table + dispatch function in `.c` |
| C++ | State/event enums, state machine class with transition method |

### Per interface

| Language | Output |
|---|---|
| Rust | `trait` definition with associated types and methods |
| C | Function pointer typedefs or vtable struct |
| C++ | Abstract base class with pure virtual methods |

### Workspace-level

| Output | Description |
|---|---|
| `src/lib.rs` or `src/main.rs` | Module declarations (`mod` statements) for Rust |
| `CMakeLists.txt` fragment | Source file list and include directories for C/C++ |
| `Cargo.toml` fragment | Dependencies and features for Rust |
| Build system integration | Whatever `gencontract.yaml` specifies |

## Generation Process

```
1. Load extracted architecture (extracted/architecture.yaml)
2. Load generation contract (gencontract.yaml)
3. Validate contract against architecture (sysml-v2-gencontract validate)
4. For each module:
   a. Resolve effective config (defaults + overrides)
   b. Map SysML types to target language types
   c. Apply naming conventions
   d. Render module template
   e. Render test template
   f. Compute spec-hash fingerprint
   g. Check if existing file has matching hash (skip if up-to-date)
   h. Write file with header comment
5. For each state machine:
   a. Render state/event enums
   b. Render transition table or match arms
   c. Embed in owner module file
6. For each interface:
   a. Render trait/vtable/abstract class
7. Render workspace-level files (lib.rs, CMakeLists, Cargo.toml)
8. Output generation report
```

## Incremental Regeneration

Every generated file includes a fingerprint header:

```rust
// @generated from spec/bt_a2dp_sink.sysml
// @spec-hash a1b2c3d4e5f6...
// @contract-hash f6e5d4c3b2a1...
// DO NOT EDIT — regenerate with sysml-v2-codegen
```

The spec-hash is computed from the extracted module descriptor YAML. The contract-hash is computed from the resolved module config. If both hashes match, the file is up-to-date and skipped.

If a file has no hash header (hand-written or modified), the tool warns before overwriting and requires `--force` to proceed.

## Template System

Code generation uses structured Rust code (not string templates) to build source files. This ensures:

- Generated code is always syntactically valid
- Type mappings are applied consistently
- Naming conventions are enforced
- The generator itself can be tested with unit tests

For each target language, there is a `render` module:

```rust
mod render {
    pub mod rust;     // Rust code renderer
    pub mod c;        // C code renderer
    pub mod cpp;      // C++ code renderer
}
```

Each renderer implements a common trait:

```rust
pub trait CodeRenderer {
    fn render_module(&self, module: &ExtractedModule, config: &ResolvedModuleConfig) -> RenderedFile;
    fn render_test(&self, module: &ExtractedModule, config: &ResolvedModuleConfig) -> RenderedFile;
    fn render_state_machine(&self, fsm: &ExtractedStateMachine, config: &ResolvedModuleConfig) -> String;
    fn render_interface(&self, iface: &ExtractedInterface, config: &ResolvedModuleConfig) -> RenderedFile;
    fn render_workspace(&self, arch: &ExtractedArchitecture, contract: &GenContract) -> Vec<RenderedFile>;
}

pub struct RenderedFile {
    pub path: PathBuf,
    pub content: String,
    pub spec_hash: String,
    pub contract_hash: String,
}
```

## Public API

```rust
pub struct CodegenConfig {
    pub extracted_dir: PathBuf,
    pub contract_path: PathBuf,
    pub output_dir: PathBuf,
    pub force: bool,           // overwrite hand-modified files
    pub dry_run: bool,         // show what would be generated
    pub modules: Option<Vec<String>>,  // specific modules only
}

pub struct CodegenResult {
    pub files_generated: Vec<PathBuf>,
    pub files_skipped: Vec<PathBuf>,     // up-to-date
    pub files_warned: Vec<PathBuf>,      // hand-modified, not overwritten
    pub report: GenerationReport,
}

pub struct GenerationReport {
    pub modules_generated: usize,
    pub state_machines_generated: usize,
    pub interfaces_generated: usize,
    pub spec_coverage: Coverage,         // Complete | Partial
    pub ambiguities: Vec<String>,
    pub assumptions: Vec<String>,
    pub constraints_verified: Vec<String>,
}

pub fn generate(config: &CodegenConfig) -> Result<CodegenResult, CodegenError>;
```

## CLI Interface

```
$ sysml-v2-codegen [OPTIONS] [PATH]

Arguments:
  [PATH]  Workspace root (default: current directory)

Options:
  -e, --extracted <DIR>    Extracted data directory (default: extracted/)
  -c, --contract <FILE>    Generation contract (default: gencontract.yaml)
  -o, --output <DIR>       Output directory (default: src/)
  -m, --module <NAME>      Generate only specific module(s)
      --force              Overwrite hand-modified files
      --dry-run            Show what would be generated without writing
      --report             Print generation report to stdout
      --format <FMT>       Report format: text (default), json
```

## Tests

### Rust renderer tests

```
test_rust_module_skeleton           — generates mod.rs with struct, impl, constructor
test_rust_module_nostd              — no_std: true → #![no_std] attribute
test_rust_module_error_type         — generates ModuleError enum from interface outputs
test_rust_module_result_returns     — all fallible functions return Result<T, ModuleError>
test_rust_module_static_allocation  — static allocation → no Box, no Vec, no heap
test_rust_module_mutex_protection   — mutex protection → Mutex wrapper on shared state
test_rust_module_critical_section   — critical_section → CriticalSection guard usage
test_rust_module_async              — async_runtime: embassy → async fn signatures
test_rust_module_ports_as_fields    — ports become struct fields with correct types
test_rust_module_actions_as_methods — actions become impl methods with correct signatures
test_rust_module_doc_comments       — doc_comments: true → /// comments from notes
test_rust_module_header_comment     — spec-hash and contract-hash in header
test_rust_module_naming             — snake_case module name, PascalCase type name
test_rust_test_skeleton             — generates test file with #[test] stubs for each action
test_rust_test_trait_mock           — mock_strategy: trait_objects → mock struct implementing trait
test_rust_fsm_enum                  — state machine → State enum + Event enum
test_rust_fsm_transition            — transition table → match expression
test_rust_fsm_initial_state         — initial state set in constructor
test_rust_fsm_error_state           — error state handled with logging/recovery
test_rust_interface_trait           — port def → trait definition
test_rust_interface_associated      — interface outputs → associated types
test_rust_workspace_lib_rs          — generates lib.rs with mod declarations
test_rust_workspace_cargo_toml      — generates Cargo.toml dependency fragment
```

### C renderer tests

```
test_c_module_header                — generates .h with include guard, opaque struct, prototypes
test_c_module_impl                  — generates .c with struct definition, function skeletons
test_c_module_error_enum            — generates error code enum in header
test_c_module_return_codes          — all functions return int (0 = success, negative = error)
test_c_module_opaque_struct         — struct forward-declared in .h, defined in .c
test_c_module_volatile_isr          — ISR-shared vars declared volatile
test_c_module_irq_protection        — disable_irq protection wraps ISR-shared access
test_c_module_static_allocation     — no malloc/free calls
test_c_module_naming                — snake_case everything
test_c_test_skeleton                — generates test file with test function stubs
test_c_fsm_enums                    — state/event enums in header
test_c_fsm_transition_table         — transition table as const array
test_c_fsm_dispatch                 — dispatch function with switch/case
test_c_interface_vtable             — port def → function pointer struct
test_c_workspace_cmake              — generates CMakeLists.txt fragment
```

### C++ renderer tests

```
test_cpp_module_header              — generates .hpp with class declaration
test_cpp_module_impl                — generates .cpp with method implementations
test_cpp_module_raii                — RAII resource management in constructor/destructor
test_cpp_module_error_handling      — error_code or exception based on config
test_cpp_module_atomic_isr          — std::atomic for ISR-shared variables
test_cpp_module_no_heap             — static allocation → no new/delete/make_unique
test_cpp_interface_abstract         — port def → abstract base class with pure virtual methods
test_cpp_test_skeleton              — generates gtest or catch2 test file
```

### Incremental regeneration tests

```
test_incremental_skip_uptodate      — file with matching hashes is skipped
test_incremental_regen_changed      — file with different spec-hash is regenerated
test_incremental_regen_contract     — file with different contract-hash is regenerated
test_incremental_warn_handmodified  — file without hash header → warning, not overwritten
test_incremental_force_overwrite    — --force overwrites hand-modified files
test_incremental_new_module         — new module creates new files
test_incremental_removed_module     — module removed from architecture → warning (file not deleted)
```

### Type mapping tests

```
test_typemap_builtin                — Integer → i32, Boolean → bool per contract
test_typemap_custom                 — AudioData → &[i16] per contract type_map
test_typemap_unmapped               — passes through as-is with warning
test_typemap_language_specific      — same SysML type maps differently for Rust vs C
```

### Naming convention tests

```
test_naming_module_snake            — BtA2dpSink module → bt_a2dp_sink file/mod name
test_naming_type_pascal             — bt_a2dp_sink → BtA2dpSink struct name
test_naming_function_snake          — StartDiscovery action → start_discovery method
test_naming_constant_screaming      — MaxRetries → MAX_RETRIES constant
test_naming_enum_variant_pascal     — disconnected state → Disconnected variant
```

### Generation report tests

```
test_report_complete                — all modules generated → spec_coverage: Complete
test_report_partial                 — some modules skipped → spec_coverage: Partial
test_report_ambiguities             — unmapped types listed as ambiguities
test_report_constraints_verified    — checked constraints listed
test_report_json_format             — --format json produces valid JSON report
```

### Dry run tests

```
test_dry_run_no_files_written       — --dry-run creates no files
test_dry_run_lists_files            — --dry-run lists all files that would be generated
test_dry_run_shows_changes          — --dry-run shows which files would change
```

### Integration tests

```
integration_rust_full_pipeline      — extract → contract → codegen produces compilable Rust
integration_c_full_pipeline         — extract → contract → codegen produces compilable C
integration_cpp_full_pipeline       — extract → contract → codegen produces compilable C++
integration_specific_module         — --module BtA2dpSink generates only that module
integration_empty_architecture      — no modules → only workspace-level files generated
integration_report_output           — generation report matches expected for known input
```

## Dependencies (Rust crates)

- `sysml-v2-extract` — extracted data types (or just serde structs matching the YAML schema)
- `sysml-v2-gencontract` — generation contract types
- `serde` / `serde_yaml` — data loading
- `sha2` — spec-hash computation
- `clap` — CLI
- `heck` — case conversion (snake_case, PascalCase, SCREAMING_SNAKE)
- `codespan-reporting` — for report output
