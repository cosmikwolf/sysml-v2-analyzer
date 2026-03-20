# Tool: sysml-v2-gencontract

Rust crate and CLI for defining, validating, and managing generation contracts — the platform-specific configuration that controls how extracted architecture data becomes source code.

## Purpose

The generation contract is the **"how to generate"** layer. It is separate from the architecture model ("what exists") and answers questions like:

- What target language and standard? (Rust 2021, C11, C++20)
- What error handling pattern? (Result, return codes, exceptions)
- What memory allocation strategy? (static only, allow heap)
- What naming conventions? (snake_case, camelCase)
- What file layout? (one module per directory, flat)
- What test framework? (built-in, Unity, Google Test)
- What RTOS integration? (bare-metal, FreeRTOS, Embassy)

No existing standard covers this (see ANALYSIS.md). We define our own schema, inspired by:
- **EMF `.genmodel`** — decorator model pattern
- **Smithy `smithy-build.json`** — projections + plugin config
- **Terraform `codegen-spec`** — versioned intermediate representation
- **JHipster `.yo-rc.json`** — comprehensive generation parameters
- **OMG MDA** — PIM/PSM conceptual separation

## Depends On

- **sysml-v2-extract** — reads extracted module/interface/FSM descriptors

## Contract Schema

File: `gencontract.yaml` (or `gencontract.toml`)

```yaml
# Generation Contract v1
version: 1

# Target platform
platform:
  language: rust               # rust | c | cpp
  standard: "2021"             # Rust edition, C standard, C++ standard
  no_std: true                 # Rust: #![no_std]
  target_triple: "xtensa-esp32-none-elf"  # optional
  rtos: embassy                # none | freertos | zephyr | rtic | embassy

# Error handling
errors:
  strategy: result             # result | panic_never | return_code | error_code | exception
  error_trait: null             # Rust: custom error trait path (e.g., "defmt::Format")
  propagation: question_mark   # Rust: question_mark | match_explicit
                               # C: return_check | goto_cleanup
                               # C++: exception | error_code_check

# Memory
memory:
  allocation: static           # static | stack | heap | mixed
  global_max_instances: 1      # default max_instances if not specified per-module
  buffer_strategy: caller      # caller | module — who owns intermediate buffers

# Concurrency
concurrency:
  default_protection: mutex    # mutex | critical_section | disable_irq | none
  isr_shared_qualifier: volatile  # C: volatile, Rust: AtomicXxx, C++: std::atomic
  async_runtime: embassy       # Rust only: embassy | tokio | none
  task_model: async            # async | thread | bare_loop

# Naming conventions
naming:
  modules: snake_case          # snake_case | PascalCase | camelCase
  types: PascalCase
  functions: snake_case
  constants: SCREAMING_SNAKE
  enum_variants: PascalCase    # Rust convention
  file_names: snake_case

# File layout
layout:
  source_dir: src
  test_dir: tests
  include_dir: src/include        # C/C++ only
  module_structure: directory     # directory (src/module/mod.rs) | flat (src/module.rs)
  test_structure: separate        # separate (tests/module_test.rs) | inline (mod tests {})
  one_file_per: module            # module | definition (finer-grained)

# Test generation
tests:
  generate: true
  framework: builtin              # Rust: builtin, C: unity | cmocka, C++: gtest | catch2
  mock_strategy: trait_objects    # Rust: trait_objects | mockall, C: function_pointers
  coverage_target: null           # optional percentage target

# Code style
style:
  max_line_length: 100
  indent: spaces_4               # spaces_2 | spaces_4 | tabs
  doc_comments: true              # generate doc comments from SysML notes
  header_comment: |               # prepended to every generated file
    // @generated from {source_file}
    // @spec-hash {spec_hash}
    // DO NOT EDIT — regenerate with sysml-v2-codegen

# Module overrides — per-module settings that override defaults
overrides:
  BtA2dpSink:
    errors:
      strategy: result
    concurrency:
      protection: critical_section
  StatusLed:
    tests:
      mock_strategy: function_pointers

# Type mappings — map SysML v2 type names to target language types
type_map:
  Integer: i32
  Real: f32
  Boolean: bool
  String: "&str"
  Natural: u32
  AudioData: "&[i16]"
  ConnectionStatus: "ConnectionState"
  # Custom types resolve via this map; unmapped types pass through as-is

# External dependencies (build system integration)
external_deps:
  - name: esp-idf-hal
    version: "0.44"
    source: crate
    features: ["bluetooth", "i2s"]
  - name: embassy-executor
    version: "0.7"
    source: crate
```

## Validation Rules

The generation contract must be internally consistent and compatible with the extracted architecture.

| Rule ID | Severity | Description |
|---|---|---|
| `GC001` | Error | `language` does not match extracted module action parameter types |
| `GC002` | Error | `no_std: true` but a module uses heap allocation |
| `GC003` | Error | `errors.strategy: panic_never` but a module's metadata says `error_handling: exception` |
| `GC004` | Warning | `type_map` is missing a type referenced in extracted modules |
| `GC005` | Error | `rtos: none` but `concurrency.task_model: async` without `async_runtime` |
| `GC006` | Error | `layout.module_structure: flat` but module names would collide |
| `GC007` | Warning | Module override references a module not found in extracted architecture |
| `GC008` | Error | `tests.framework` not compatible with `language` (e.g., `gtest` with `rust`) |
| `GC009` | Error | `version` field is missing or not a supported schema version |
| `GC010` | Warning | `external_deps` lists a package not referenced by any extracted module |

## Public API

```rust
pub struct GenContract {
    pub version: u32,
    pub platform: PlatformConfig,
    pub errors: ErrorConfig,
    pub memory: MemoryConfig,
    pub concurrency: ConcurrencyConfig,
    pub naming: NamingConfig,
    pub layout: LayoutConfig,
    pub tests: TestConfig,
    pub style: StyleConfig,
    pub overrides: HashMap<String, ModuleOverride>,
    pub type_map: HashMap<String, String>,
    pub external_deps: Vec<ExternalDep>,
}

/// Load and validate a generation contract.
pub fn load_contract(path: &Path) -> Result<GenContract, ContractError>;

/// Validate contract against extracted architecture.
pub fn validate_contract(
    contract: &GenContract,
    architecture: &ExtractedArchitecture,
) -> Vec<Diagnostic>;

/// Resolve effective config for a specific module (defaults + overrides).
pub fn resolve_module_config(
    contract: &GenContract,
    module_name: &str,
) -> ResolvedModuleConfig;

/// Map a SysML v2 type name to the target language type.
pub fn map_type(contract: &GenContract, sysml_type: &str) -> String;
```

## CLI Interface

```
$ sysml-v2-gencontract [COMMAND] [OPTIONS]

Commands:
  validate    Validate a generation contract against extracted architecture
  init        Create a new gencontract.yaml with sensible defaults
  diff        Show differences between two generation contracts
  resolve     Show effective config for a specific module (with overrides applied)
  schema      Print the JSON Schema for gencontract.yaml

Options:
  -c, --contract <FILE>   Path to gencontract.yaml (default: auto-detect)
  -a, --architecture <FILE>  Path to extracted/architecture.yaml
```

## Tests

### Schema validation tests

```
test_load_valid_contract            — well-formed contract loads without error
test_load_minimal_contract          — contract with only required fields loads with defaults
test_load_missing_version           — missing version field produces GC009
test_load_unknown_version           — version: 99 produces GC009
test_load_invalid_yaml              — malformed YAML produces parse error
test_load_unknown_field             — unknown field produces warning (forward compat)
test_load_all_languages             — rust, c, cpp all load correctly
test_load_all_rtos                  — none, freertos, embassy, zephyr, rtic all load
test_load_all_error_strategies      — result, panic_never, return_code, exception all load
```

### Cross-validation tests

```
test_validate_compatible            — valid contract + valid architecture → no errors
test_validate_type_map_missing      — unmapped type produces GC004
test_validate_heap_nostd            — no_std + heap produces GC002
test_validate_panic_never_exception — panic_never + exception metadata produces GC003
test_validate_async_no_runtime      — async task model without runtime produces GC005
test_validate_flat_collision        — flat layout + colliding names produces GC006
test_validate_unknown_override      — override for nonexistent module produces GC007
test_validate_wrong_test_framework  — gtest with rust produces GC008
test_validate_unused_dep            — external dep not referenced produces GC010
```

### Module resolution tests

```
test_resolve_defaults               — module with no override gets default config
test_resolve_override_errors        — module override for errors.strategy applies
test_resolve_override_concurrency   — module override for concurrency.protection applies
test_resolve_override_partial       — partial override merges with defaults correctly
test_resolve_override_nested        — nested override field (errors.propagation) applies
```

### Type mapping tests

```
test_map_builtin_types              — Integer → i32, Boolean → bool, etc.
test_map_custom_types               — AudioData → "&[i16]" per type_map
test_map_unmapped_passthrough       — unknown type passes through as-is
test_map_empty_map                  — no type_map → all types pass through
```

### Init tests

```
test_init_rust_defaults             — `sysml-v2-gencontract init --language rust` produces sensible Rust defaults
test_init_c_defaults                — `sysml-v2-gencontract init --language c` produces sensible C defaults
test_init_cpp_defaults              — `sysml-v2-gencontract init --language cpp` produces sensible C++ defaults
test_init_file_created              — init creates gencontract.yaml in expected location
test_init_no_overwrite              — init refuses to overwrite existing file without --force
```

### Diff tests

```
test_diff_identical                 — two identical contracts → no diff
test_diff_language_change           — language change highlighted
test_diff_added_override            — new module override shown
test_diff_removed_dep               — removed external dep shown
test_diff_type_map_change           — changed type mapping highlighted
```

### Integration tests

```
integration_load_and_validate       — load contract, load architecture, validate, no errors
integration_resolve_all_modules     — resolve config for every module in architecture
integration_schema_output           — schema command produces valid JSON Schema
```

## Dependencies (Rust crates)

- `serde` / `serde_yaml` / `toml` — config deserialization
- `schemars` — JSON Schema generation from Rust types
- `clap` — CLI
- `similar` — text diffing for the `diff` command
- `codespan-reporting` — diagnostic rendering
