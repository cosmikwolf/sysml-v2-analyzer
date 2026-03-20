# Component: domain plugin system

**Location:** `domains/<name>/`
**Domain scope:** Each directory is a self-contained domain definition
**Status:** Not yet implemented (firmware domain planned as first)

## Purpose

A domain plugin provides all the domain-specific knowledge the engine needs: what layers exist, what metadata is required, how to map types, and how to generate code. Domains are directories, not Rust crates — adding a domain requires no compilation.

## Domain directory structure

```
domains/<name>/
├── domain.toml              # domain definition (required)
├── <name>_library.sysml     # SysML metadata library (required)
└── templates/               # codegen templates (required for generation)
    ├── rust/
    │   ├── module.rs.j2
    │   ├── state_machine.rs.j2
    │   ├── trait_interface.rs.j2
    │   └── test.rs.j2
    ├── c/
    │   ├── module.c.j2
    │   ├── module.h.j2
    │   └── test.c.j2
    └── cpp/                 # optional, added when needed
        ├── module.cpp.j2
        └── module.hpp.j2
```

## domain.toml specification

```toml
[domain]
name = "firmware"
description = "Embedded firmware development"
metadata_library = "firmware_library.sysml"

# ── Layer hierarchy ──
# Defines element categories and allowed dependency directions.
# The engine validates that connections only flow in allowed directions.
[layers]
order = ["application", "middleware", "driver", "hal", "pac"]

[layers.allowed_deps]
application = ["middleware", "driver"]
middleware = ["driver", "hal"]
driver = ["hal"]
hal = ["pac"]
pac = []

# ── Required metadata ──
# Which annotations must be present on each element kind.
# The engine reports a diagnostic if any are missing.
[required_metadata]
parts = ["MemoryModel", "ConcurrencyModel", "ErrorHandling"]

# ── Validation rules ──
# Default severity for each rule ID. Projects can override in sysml.toml.
# "off" disables the rule entirely.
[validation.rules]
LAYER001 = "error"       # layer violation
LAYER002 = "warning"     # missing layer attribute
LAYER003 = "error"       # circular dependency
LAYER004 = "info"        # same-layer dependency
META010 = "warning"      # missing required metadata
FSM020 = "error"         # no initial state
FSM021 = "warning"       # unreachable state
FSM022 = "error"         # non-deterministic transitions
FSM024 = "error"         # transition targets non-existent state
PORT030 = "error"        # incompatible port types
PORT033 = "warning"      # unused port

# ── Type mappings ──
# SysML type → target language type, per language.
[type_map.rust]
Integer = "i32"
Boolean = "bool"
String = "&str"
Real = "f64"
"Integer[0..*]" = "&[i32]"

[type_map.c]
Integer = "int32_t"
Boolean = "bool"
String = "const char*"
Real = "double"
"Integer[0..*]" = "int32_t*"
```

## SysML metadata library

Each domain provides a `.sysml` file defining its metadata types. The engine parses this file as part of the workspace. Example for firmware:

```sysml
package Firmware {
    metadata def MemoryModel {
        attribute 'allocation' : AllocationKind;
        attribute maxInstances : Integer;
    }
    metadata def ISRSafe {
        attribute safe : Boolean;
    }
    // ... more metadata defs, enums, etc.
}
```

The metadata library defines what annotations are *available*. The `domain.toml` defines which are *required* and how to *validate* them.

## Template conventions

- **Extension:** `.j2` (Jinja2 standard)
- **Naming:** Double extension — `module.rs.j2` generates `.rs` files
- **Organization:** By target language (`rust/`, `c/`, `cpp/`)
- **Engine:** MiniJinja with `trim_blocks` + `lstrip_blocks`
- **Context:** Templates receive an `ExtractedModule` (or `ExtractedStateMachine`, etc.) as their data context
- **Filters available:** `snake_case`, `pascal_case`, `screaming_snake`, `map_type`

## Workspace config (`sysml.toml`)

Lives in the user's project root. Selects a domain and provides per-project overrides:

```toml
[workspace]
domain = "firmware"
include = ["spec/**/*.sysml"]
exclude = ["target/**"]

# Override domain defaults for this project
[validation.rules]
LAYER004 = "off"           # don't flag same-layer deps in this project

[required_metadata]
parts = ["MemoryModel"]    # only require MemoryModel (drop ConcurrencyModel, ErrorHandling)
```

The engine merges: `domain.toml` defaults ← `sysml.toml` overrides.

## Adding a new domain

1. Create `domains/<name>/`
2. Write `domain.toml` with layers, required metadata, type maps
3. Write `<name>_library.sysml` with metadata definitions
4. Write templates in `templates/<language>/`
5. In the user's project, set `domain = "<name>"` in `sysml.toml`

No Rust code. No recompilation.

## Future: domain-specific Rust code

If a domain needs validation logic that can't be expressed in config (e.g., cross-field analysis, semantic checks beyond layer/metadata/FSM rules), the domain directory can optionally contain a Rust crate:

```
domains/<name>/
├── domain.toml
├── <name>_library.sysml
├── templates/
└── src/                      # optional Rust crate
    ├── Cargo.toml
    └── lib.rs                # impl DomainPlugin for <Name>Domain
```

The `DomainPlugin` trait would be defined in the engine crate. This is not built yet — it will be added when a concrete need arises.
