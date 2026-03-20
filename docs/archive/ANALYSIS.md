# SysML v2 Firmware Toolchain — Architecture Analysis

## Problem Statement

Our firmware development workflow uses a custom YAML spec format (`FORMAT.md`) that conflates three concerns:

1. **Architecture description** — what modules exist, their layers, dependencies, interfaces
2. **Implementation contracts** — memory allocation, error handling, concurrency, language-specific rules
3. **Hardware/UI modeling** — datasheet summaries, pixel-level screen layouts, LED patterns

We want to separate these concerns and adopt a standardized specification language for the architecture layer, while keeping a purpose-built format for code generation contracts.

## Standards Evaluated

### Architecture Description Languages

| Standard | Fit | Why |
|---|---|---|
| **AADL (SAE AS5506)** | Excellent for embedded | Native threads, ports, memory binding, ISR, scheduling. But Eclipse/Java tooling is heavy. |
| **SysML v2 (OMG)** | Strong with AADL library | Textual notation, formal semantics via KerML, parts/ports/states/constraints as primitives. AADL library bridges embedded concepts. Adopted July 2025. |
| **LikeC4** | Low | Visualization-focused. Custom element kinds but untyped metadata, no validation, no firmware semantics. |
| **Structurizr DSL** | Low | C4 hierarchy fixed at 4 levels. Properties are unvalidated strings. Diagram generation only. |
| **AaC (Architecture-as-Code)** | Moderate | YAML-based, Python-extensible schemas. But immature, small community. |
| **C4 Model** | None | Pure visualization framework. No typed interfaces, no code generation. |
| **arc42** | None | Documentation template, not a machine-readable format. |

### Code Generation Contract Standards

No single standard exists. The concept appears under different names across ecosystems:

| System | Name | Format |
|---|---|---|
| OMG MDA | PSM + marks | Conceptual only — never standardized |
| Eclipse EMF | `.genmodel` | XML decorator model |
| Smithy | `smithy-build.json` | JSON (projections + plugins) |
| Buf (protobuf) | `buf.gen.yaml` | YAML (plugins + managed mode) |
| OpenAPI Generator | config options | YAML/JSON per language target |
| AUTOSAR | ECUC | ARXML |
| Terraform | `codegen-spec` | JSON Schema — most explicit example |
| JHipster | `.yo-rc.json` | JSON (~40 generation params) |

Common pattern across all:
```
Architecture Model (WHAT) → Generation Contract (HOW) → Generated Code
     (stable)                 (platform-specific)        (derived)
```

## Recommendation: SysML v2

### Why SysML v2 over AADL

- **Textual notation** that is diffable, version-controllable, and parseable
- **Formal semantics** via KerML — parts, ports, connections, state machines, constraints have mathematical meaning
- **Composition and specialization** — parts specialize, ports are typed and conjugated
- **State machines are first-class** — transitions with formal guard/trigger/action syntax
- **Requirements traceability** — `satisfy R1 by module` is built into the language
- **Metadata extensibility** — `metadata def` replaces UML stereotypes for domain-specific annotations
- **AADL bridge** — the SysML v2 AADL Library maps thread semantics, port typing, memory binding into SysML v2
- **It is an actual standard** — OMG-adopted, SAE-aligned, growing ecosystem
- **More general** — can model full systems, not just real-time threads

### What SysML v2 provides natively

```sysml
part def BtA2dpSink {
    // Metadata annotations (firmware-specific, from our library)
    @ISRSafe { safe = false; }
    @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }

    attribute layer : LayerKind = LayerKind::driver;

    // Typed ports
    port audioOut : ~AudioDataPort;
    port btStatus : StatusPort;

    // State machine — first-class citizen
    state def ConnectionFSM {
        entry; then disconnected;
        state disconnected;
            accept ConnectEvent then connecting;
        state connecting;
            accept ConnectedEvent then streaming;
            accept TimeoutEvent then disconnected;
        state streaming;
            accept DisconnectEvent then disconnected;
    }

    // Interface actions
    perform action init { out result : InitResult; }
    perform action startDiscovery { in config : A2dpConfig; }
}
```

### What we must build

SysML v2's existing tooling (Eclipse pilot, archived VS Code extension) does not fit our workflow (Neovim/CLI, Rust, no Java dependency). We need a native Rust toolchain:

1. **sysml-v2-parser** — SysML v2 / KerML parser producing typed AST
2. **sysml-v2-lsp** — Language Server Protocol for Neovim
3. **sysml-v2-validate** — Firmware-specific model validation
4. **sysml-v2-extract** — Model → structured JSON/YAML for code gen pipeline
5. **sysml-v2-gencontract** — Generation contract processor
6. **sysml-v2-codegen** — Code generator from model + contract → source files
7. **sysml-v2-fw** — Unified CLI orchestrating the pipeline
8. **tree-sitter-sysml-v2** — Tree-sitter grammar for Neovim syntax highlighting

### Two-layer architecture

```
┌─────────────────────────────────────────────────────┐
│  Architecture Layer (SysML v2)                      │
│  .sysml files — parts, ports, states, constraints   │
│  firmware metadata library — ISR, memory, layers    │
│  Standard: OMG SysML v2.0 / KerML 1.0              │
└──────────────────────┬──────────────────────────────┘
                       │ sysml-v2-extract
                       ▼
┌─────────────────────────────────────────────────────┐
│  Generation Contract Layer (custom YAML)            │
│  target language, error handling, naming, file       │
│  layout, test generation, platform constraints      │
│  No existing standard — our own schema              │
└──────────────────────┬──────────────────────────────┘
                       │ sysml-v2-codegen
                       ▼
┌─────────────────────────────────────────────────────┐
│  Generated Source Code                              │
│  .rs / .c / .h / .cpp / .hpp + test stubs           │
└─────────────────────────────────────────────────────┘
```

### Risk assessment

| Risk | Severity | Mitigation |
|---|---|---|
| SysML v2 spec is large (~1000 pages) | High | Implement subset: parts, ports, metadata, state machines, constraints, imports. Skip: requirements diagrams, parametric models, analysis cases. |
| AADL library is CC-BY-ND (no derivatives) | Medium | Build our own firmware metadata library inspired by AADL concepts, not derived from their files. |
| Parser correctness | High | Test against Eclipse pilot implementation output. Use the official KerML/SysML v2 grammar as reference. |
| Adoption barrier (learning SysML v2) | Medium | Good CLI tooling + LSP + tree-sitter highlighting reduces friction. Migration tool from existing YAML specs. |
| Spec evolves (SysML 2.1 in progress) | Low | Our parser subset is small; tracking changes is manageable. |

### Ecosystem findings (March 2026)

After evaluating available tooling, we identified a Rust-native SysML v2 parsing ecosystem that eliminates the need to build a parser from scratch:

| Crate | Version | Purpose | License |
|---|---|---|---|
| **syster-base** | 0.4.0-alpha | SysML v2 / KerML parser + HIR + salsa queries | MIT |
| syster-lsp | alpha | LSP server built on syster-base | MIT |
| syster-cli | alpha | CLI built on syster-base | MIT |
| tree-sitter-sysml | — | Tree-sitter grammar (GitLab, not on crates.io) | MIT |
| sysand | — | SysML v2 package manager | MIT |

**syster-base** was evaluated hands-on (see `docs/sysml-toolchain/09-evaluation-syster-base.md`) and scored **2.65/3.0** — above the "use as dependency" threshold. Key results:

- Parses all firmware SysML constructs (parts, ports, metadata, state machines, connections, flows)
- HIR extracts typed symbols with qualified names, supertypes, and relationships
- Sub-millisecond parse times for our workspace size
- Metadata annotation values require a thin CST adapter (~100 lines)
- Saves ~33K lines and ~6 months of parser development

### Revised architecture (post-evaluation)

Instead of building **sysml-v2-parser** from scratch (documented in `01-sysml-v2-parser.md`), we use syster-base as a dependency and build a ~500-line adapter layer on top. The remaining toolchain components (validate, extract, gencontract, codegen, CLI) are still custom.

```
syster-base (reuse)  →  sysml-v2-adapter (build, ~500 lines)
                     →  sysml-v2-validate (build)
                     →  sysml-v2-extract (build)
                     →  sysml-v2-gencontract (build)
                     →  sysml-v2-codegen (build)
                     →  sysml-v2-fw CLI (build)
```

### References

- [SysML v2 Pilot Implementation](https://github.com/Systems-Modeling/SysML-v2-Pilot-Implementation)
- [SysML v2 AADL Library](https://github.com/Systems-Modeling/SysML-v2-AADL-Release)
- [SysML v2 API Services](https://github.com/Systems-Modeling/SysML-v2-API-Services)
- [SysML v2 API Python Client](https://github.com/Systems-Modeling/SysML-v2-API-Python-Client)
- [MontiCore SysML v2 Parser](https://github.com/MontiCore/sysmlv2)
- [OSATE (AADL Tooling)](https://github.com/osate)
- [OMG SysML v2 Specification](https://www.omg.org/sysml/sysmlv2/)
- [SysML v2 Review — Kobryn](https://sysml.org/sysml-v2/reviews/good-bad-ugly/)
- [Terraform Plugin Codegen Spec](https://github.com/hashicorp/terraform-plugin-codegen-spec)
- [Smithy Build Config](https://smithy.io/2.0/guides/building-models/build-config.html)
- [Sensmetry SysML v2 Cheatsheet](https://sensmetry.com/sysml-cheatsheet/)
- **[syster-base on crates.io](https://crates.io/crates/syster-base)** — Rust SysML v2 parser (evaluated)
- **[syster-base on docs.rs](https://docs.rs/syster-base/)** — API documentation
- **[tree-sitter-sysml on GitLab](https://gitlab.com/nomograph/tree-sitter-sysml)** — tree-sitter grammar
