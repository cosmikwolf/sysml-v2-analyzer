# Architecture Overview

## What this project does

sysml-v2-analyzer reads SysML v2 specifications and transforms them through a pipeline:

```
.sysml files  →  parse  →  validate  →  extract  →  generate  →  source code
```

The pipeline is **domain-agnostic at its core**. Domain-specific knowledge (what to validate, what to extract, how to generate code) lives in a separate `domains/` directory as configuration and templates — not compiled into the binary.

## System diagram

```
   ┌──────────────────────────────────────────────────────────┐
   │  cli crate (sysml-v2-analyzer binary)                    │
   │  Reads sysml.toml → selects domain → runs pipeline       │
   └────────────────────────────┬─────────────────────────────┘
                                │
   ┌────────────────────────────┴─────────────────────────────┐
   │  engine crate                                             │
   │  Domain-agnostic pipeline framework                       │
   │  ├── validation engine (layer deps, required metadata,    │
   │  │                      FSM checks, port compat)          │
   │  ├── extraction engine (flatten to YAML/JSON)             │
   │  └── codegen engine (MiniJinja template rendering)        │
   └───────────┬──────────────────────────┬───────────────────┘
               │                          │
   ┌───────────┴───────────┐   ┌──────────┴──────────────────┐
   │  adapter crate         │   │  domains/<name>/             │
   │  Domain-agnostic       │   │  ├── domain.toml (config)    │
   │  SysML v2 query library│   │  ├── *.sysml (metadata lib)  │
   │  (syster-base wrapper) │   │  └── templates/*.j2           │
   └───────────┬───────────┘   │                               │
               │                │  firmware/ auto/ template/ ...│
   ┌───────────┴───────────┐   └───────────────────────────────┘
   │  syster-base (external)│
   │  SysML v2 parser + HIR │
   └────────────────────────┘
```

## Domain boundary

| Layer | Crate | Domain scope |
|---|---|---|
| Parsing + querying | `adapter` + syster-base | **General SysML v2** — works for any domain |
| Pipeline framework | `engine` | **General** — validation engine, extraction engine, codegen engine are domain-agnostic |
| Domain rules + templates | `domains/<name>/` | **Domain-specific** — config, metadata library, codegen templates |
| Orchestration | `cli` | **General** — loads domain from `sysml.toml`, runs pipeline |

Adding a new domain = creating a new directory under `domains/` with:
- `domain.toml` — layer hierarchy, required metadata, type maps
- `*.sysml` — metadata library defining domain-specific annotations
- `templates/<language>/*.j2` — MiniJinja codegen templates

No Rust code required for most domains.

## Crate structure

```
tools/sysml-v2-analyzer/
├── Cargo.toml                   # workspace root
├── .cargo/config.toml           # host target (not ESP32)
├── sysml.toml                   # example workspace config
├── crates/
│   ├── adapter/                 # general SysML v2 query library
│   ├── engine/                  # domain-agnostic pipeline framework
│   └── cli/                     # unified binary
├── domains/
│   ├── template/                # minimal example domain (also used by engine tests)
│   │   ├── domain.toml
│   │   ├── template_library.sysml
│   │   └── templates/
│   │       └── rust/
│   │           └── module.rs.j2
│   └── firmware/                # firmware domain plugin
│       ├── domain.toml
│       ├── firmware_library.sysml
│       └── templates/
│           └── rust/
│               ├── module.rs.j2
│               └── ...
└── tests/
    └── fixtures/                # SysML v2 test fixtures (adapter tests)
```

## Configuration layering

Two config files with different purposes:

**`domains/<name>/domain.toml`** — shared domain definition (checked into the domain directory):
- Layer hierarchy and allowed dependencies
- Required metadata annotations per element kind
- Type mappings (SysML → target language)
- Default validation rule severities

**`sysml.toml`** — per-project workspace config (checked into the user's project):
- Which domain to use (`domain = "firmware"`)
- File include/exclude patterns
- Per-project rule overrides (disable rules, change severities)
- Per-project metadata overrides

The engine merges: domain defaults ← project overrides.

## Template engine

MiniJinja (Jinja2-compatible) with:
- `trim_blocks` + `lstrip_blocks` enabled for clean codegen output
- Auto-escape disabled (code generation, not HTML)
- Templates loaded from `domains/<name>/templates/<language>/`
- `.j2` extension with double-extension naming (`module.rs.j2`)

## Key dependencies

| Dependency | Version | Purpose |
|---|---|---|
| syster-base | `=0.4.0-alpha` (pinned) | SysML v2 parser + HIR |
| minijinja | latest | Template engine for code generation |
| petgraph | 0.7 | Graph algorithms (cycle detection, reachability) |
| clap | 4 | CLI argument parsing |
| serde + toml | latest | Configuration parsing |
| thiserror | 2 | Error types |

## Related documents

| Document | Purpose |
|---|---|
| [decisions.md](decisions.md) | Architecture decision record (D1–D8) |
| [01-adapter.md](01-adapter.md) | Adapter crate architecture |
| [02-engine.md](02-engine.md) | Engine crate architecture |
| [03-domains.md](03-domains.md) | Domain plugin system |
| [04-cli.md](04-cli.md) | CLI design |
| [05-implementation-phases.md](05-implementation-phases.md) | Phased implementation tracker |
| [archive/phase-1-adapter.md](archive/phase-1-adapter.md) | Phase 1: Adapter |
| [archive/phase-2-engine-scaffold.md](archive/phase-2-engine-scaffold.md) | Phase 2: Engine scaffold + domain config |
| [archive/phase-3-validation.md](archive/phase-3-validation.md) | Phase 3: Validation engine |
| [archive/phase-4-extraction.md](archive/phase-4-extraction.md) | Phase 4: Extraction engine |
| [archive/phase-5-codegen.md](archive/phase-5-codegen.md) | Phase 5: Code generation |
| [archive/phase-6-cli.md](archive/phase-6-cli.md) | Phase 6: CLI |

## Archive

Pre-restructure documentation (original 8-crate plan, syster-base evaluation, standards analysis) is preserved in [`archive/`](archive/).
