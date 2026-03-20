# Tool: sysml-v2-parser

Rust crate providing a SysML v2 / KerML parser that produces a typed abstract syntax tree.

## Purpose

Parse `.sysml` and `.kerml` source files into a structured, queryable AST. This is the foundation of the entire toolchain — every other tool depends on it.

## Scope

We do **not** need to implement the full SysML v2 / KerML specification (~1000 pages). We implement a **firmware-relevant subset**:

### Must implement (Phase 1)

| Construct | KerML/SysML | Why |
|---|---|---|
| Packages / namespaces | KerML | File organization, imports |
| Part definitions | SysML | Module/component definitions |
| Part usages | SysML | Module instantiation and composition |
| Port definitions | SysML | Interface boundaries |
| Port usages | SysML | Port instantiation on parts |
| Attribute definitions | SysML | Typed properties on parts |
| Attribute usages | SysML | Property values |
| Enum definitions | SysML | Finite value types (LayerKind, AllocationKind) |
| Metadata definitions | SysML | `metadata def ISRSafe { ... }` — domain extensions |
| Metadata usages | SysML | `@ISRSafe { safe = false; }` — annotations on elements |
| State definitions | SysML | `state def ConnectionFSM { ... }` |
| State usages | SysML | States, transitions, triggers, guards, actions |
| Action definitions | SysML | Interface function signatures |
| Action usages | SysML | `perform action init { ... }` |
| Connection definitions | SysML | `connect source.port to target.port` |
| Flow definitions | SysML | `flow of Data from a to b` |
| Constraint definitions | SysML | `constraint def LayerRule { ... }` |
| Specialization | KerML | `:>` (subtyping), `:>>` (redefinition) |
| Import statements | KerML | `import FirmwareLibrary::*;` |
| Comments / documentation | KerML | `doc /* ... */` and line comments |
| Alias | KerML | `alias` declarations |

### Deferred (Phase 2+)

- Requirement definitions (`requirement def`)
- Use case definitions
- Analysis / verification cases
- Parametric constraints with solver bindings
- Rendering / view definitions
- Allocation definitions
- Temporal / occurrence modeling
- Item definitions (unless needed for flow typing)

### Explicitly out of scope

- Graphical notation rendering
- SysML v2 API server compatibility (we query the AST directly)
- Full KerML metamodel (we implement the surface syntax, not the abstract metamodel)

## Architecture

```
                   Source text (.sysml / .kerml)
                           │
                           ▼
                   ┌───────────────┐
                   │    Lexer      │  Token stream
                   └───────┬───────┘
                           ▼
                   ┌───────────────┐
                   │    Parser     │  Concrete Syntax Tree (lossless)
                   └───────┬───────┘
                           ▼
                   ┌───────────────┐
                   │  AST Builder  │  Typed AST (lossy — drops whitespace/comments)
                   └───────┬───────┘
                           ▼
                   ┌───────────────┐
                   │  Name Resolver│  Resolved AST (cross-references linked)
                   └───────┬───────┘
                           ▼
                   ┌───────────────┐
                   │  Type Checker │  Validated AST
                   └───────────────┘
```

### Lexer

- Hand-written or generated from the KerML/SysML v2 grammar
- Must produce a **lossless token stream** (preserving whitespace and comments) for the LSP tool
- Tokens: keywords (`part`, `def`, `port`, `state`, `action`, `metadata`, `constraint`, `enum`, `import`, `alias`, `connect`, `flow`, `accept`, `then`, `if`, `do`, `entry`, `exit`, `in`, `out`, `inout`, `perform`, `satisfy`, `require`, `redefines`, `subsets`, `:>`, `:>>`, `@`, `::`, etc.), identifiers, literals (integer, real, string, boolean), punctuation, comments
- Track source spans (byte offset, line, column) on every token for error reporting and LSP

### Parser

- Recursive descent or Pratt parser
- Produces a **Concrete Syntax Tree (CST)** that preserves all tokens (including trivia) — needed for LSP features (formatting, code actions)
- Inspired by rust-analyzer's rowan/ungrammar approach: green tree (immutable, interned) + red tree (lazy, with parent pointers)
- Error recovery: the parser must produce a partial tree even when the input has syntax errors. This is critical for LSP (users type incomplete code constantly).

### AST Builder

- Transforms CST into a typed AST with strongly-typed Rust structs
- Drops whitespace and comment tokens (but they remain accessible in the CST for LSP)
- Each AST node carries a `Span` back to the source

### Name Resolver

- Resolves `import` statements across files in a workspace
- Resolves qualified names (`FirmwareLibrary::MemoryModel`)
- Links specialization targets (`:>` references)
- Links metadata annotation types (`@ISRSafe` → `metadata def ISRSafe`)
- Links port types, attribute types, enum references
- Reports unresolved names as diagnostics

### Type Checker

- Verifies specialization compatibility (a part specializing another part, not a port)
- Verifies metadata field types match their definitions
- Verifies port conjugation (`~` operator) correctness
- Verifies state machine well-formedness (reachable states, no duplicate transitions for same event)
- Verifies constraint expression types (boolean result)

## Public API

```rust
// Core types
pub struct SourceFile { pub path: PathBuf, pub text: String }
pub struct Workspace { /* collection of parsed files */ }
pub struct Diagnostic { pub span: Span, pub severity: Severity, pub message: String }
pub struct Span { pub file: FileId, pub start: usize, pub end: usize }

// Parsing
pub fn parse_file(source: &SourceFile) -> (CstNode, Vec<Diagnostic>);
pub fn build_ast(cst: &CstNode) -> (AstModule, Vec<Diagnostic>);

// Workspace (multi-file)
pub fn load_workspace(root: &Path) -> (Workspace, Vec<Diagnostic>);
pub fn resolve_names(workspace: &mut Workspace) -> Vec<Diagnostic>;
pub fn check_types(workspace: &Workspace) -> Vec<Diagnostic>;

// Querying
impl Workspace {
    pub fn all_parts(&self) -> impl Iterator<Item = &PartDef>;
    pub fn all_ports(&self) -> impl Iterator<Item = &PortDef>;
    pub fn all_states(&self) -> impl Iterator<Item = &StateDef>;
    pub fn all_metadata_defs(&self) -> impl Iterator<Item = &MetadataDef>;
    pub fn find_element(&self, qualified_name: &str) -> Option<&Element>;
    pub fn dependents_of(&self, element: &Element) -> Vec<&Element>;
    pub fn dependencies_of(&self, element: &Element) -> Vec<&Element>;
}
```

## Key Design Decisions

1. **Lossless CST + typed AST dual layer** — the CST preserves all tokens for LSP; the AST is ergonomic for semantic analysis. This is the rust-analyzer pattern.
2. **Incremental reparsing** — when a file changes, only reparse that file and re-resolve names that could be affected. Essential for LSP responsiveness.
3. **Salsa-style incremental computation** — consider using `salsa` or a similar incremental computation framework for caching parse/resolve/check results.
4. **No Java dependency** — this is a pure Rust implementation. We reference the official Xtext grammar and the OMG spec for correctness, but do not depend on Java tooling at runtime.
5. **Grammar source** — the authoritative grammar is in the [SysML v2 Pilot Implementation](https://github.com/Systems-Modeling/SysML-v2-Pilot-Implementation) as Xtext `.xtext` files. We transliterate the relevant productions into Rust.

## Tests

### Lexer tests

```
lex_keywords          — each SysML v2 keyword produces the correct token kind
lex_identifiers       — simple, qualified (A::B::C), and escaped (`backtick-id`)
lex_literals          — integers, reals, strings (single/double/triple-quoted), booleans
lex_operators         — :>, :>>, ~, @, ->, ::, all punctuation
lex_comments          — line comments (//), block comments (/* */), doc comments (doc /*)
lex_spans             — every token has correct byte offset, line, and column
lex_unicode           — identifiers with unicode characters
lex_error_recovery    — unterminated string produces error token + continues lexing
```

### Parser tests

```
parse_empty_package         — `package P;` produces a package node
parse_part_def              — `part def Engine;` produces PartDef with name "Engine"
parse_part_def_with_body    — `part def V { attribute mass : Real; }` produces PartDef with attribute
parse_part_usage            — `part engine : Engine;` produces PartUsage
parse_nested_parts          — `part def A { part b : B { part c : C; } }` correct nesting
parse_port_def              — `port def P { in x : Integer; out y : Real; }` with directions
parse_port_usage            — `port p : P;` on a part
parse_conjugated_port       — `port p : ~P;` conjugation marker
parse_enum_def              — `enum def Color { red; green; blue; }` with members
parse_metadata_def          — `metadata def M { attribute x : Boolean; }` fields
parse_metadata_usage        — `@M { x = true; }` applied to a part def
parse_state_def             — full state machine with entry, states, transitions
parse_state_transition      — `accept Event if guard do action then target;`
parse_state_parallel        — `state mission parallel { state a; state b; }` concurrent regions
parse_action_def            — `action def Init { in cfg : Config; out result : Result; }`
parse_action_usage          — `perform action init : Init;`
parse_connection             — `connect a.p1 to b.p2;`
parse_flow                  — `flow of Data from a.out to b.in;`
parse_constraint_def        — `constraint def C { in x : Integer; x > 0 }`
parse_specialization        — `part def Sub :> Base { ... }`
parse_redefinition          — `:>> attr = value;`
parse_import                — `import Pkg::*;` and `import Pkg::Element;`
parse_alias                 — `alias A for Pkg::LongName;`
parse_multiple_files        — workspace with cross-file references
parse_error_recovery        — incomplete `part def { ` produces partial tree + diagnostic
parse_error_missing_semi    — `part def Foo` (no semicolon) recovers gracefully
parse_error_nested          — error inside a body does not destroy the enclosing definition
parse_preserves_trivia      — CST retains comments and whitespace
parse_span_accuracy         — AST node spans match source text exactly
```

### Name resolution tests

```
resolve_local_reference         — part usage references part def in same file
resolve_import_wildcard         — `import Pkg::*;` makes Pkg's members visible
resolve_import_specific         — `import Pkg::Foo;` makes only Foo visible
resolve_qualified_name          — `Pkg::Sub::Element` resolves through nesting
resolve_metadata_type           — `@ISRSafe` resolves to `metadata def ISRSafe`
resolve_port_type               — `port p : AudioPort` resolves to port def
resolve_specialization_target   — `:> Base` resolves to the correct part def
resolve_enum_member             — `AllocationKind::static_alloc` resolves
resolve_cross_file              — reference to definition in another .sysml file
resolve_circular_import_error   — `A imports B imports A` produces diagnostic, no hang
resolve_unresolved_name_error   — reference to nonexistent name produces diagnostic
resolve_shadowing               — local name shadows imported name
resolve_alias                   — alias resolves to its target
```

### Type checking tests

```
typecheck_metadata_field_type       — `@M { x = 42; }` where x is Boolean → error
typecheck_metadata_missing_field    — `@M {}` where M has required field → error
typecheck_specialization_kind       — `port def P :> SomePartDef` → error (kind mismatch)
typecheck_redefinition_compatible   — `:>> attr = value` where value matches attr type
typecheck_port_conjugation          — conjugated port reverses in/out directions
typecheck_state_unreachable         — state with no incoming transition → warning
typecheck_state_duplicate_trigger   — same event on two transitions from same state → error
typecheck_constraint_boolean        — constraint body must evaluate to boolean type
typecheck_enum_member_valid         — enum usage references actual member
```

### Integration tests

```
integration_firmware_library    — parse the firmware metadata library (our .sysml files) without errors
integration_bt_a2dp_module      — parse a complete BT A2DP module definition, resolve all names
integration_workspace_10_files  — load a 10-file workspace, resolve cross-file references
integration_round_trip          — parse → pretty-print → reparse produces equivalent AST
integration_error_count         — known-bad file produces exactly the expected diagnostics
```

### Performance tests

```
perf_parse_1000_lines       — parse a 1000-line .sysml file in < 10ms
perf_parse_10_files         — parse 10 files totaling 5000 lines in < 50ms
perf_incremental_reparse    — changing one file in a 10-file workspace re-resolves in < 20ms
perf_memory_usage           — a 50-file workspace uses < 50MB of memory
```

## Dependencies (Rust crates)

- `logos` or hand-written lexer — tokenization
- `rowan` — lossless CST (red-green tree, used by rust-analyzer)
- `salsa` — incremental computation framework (optional, for LSP performance)
- `smol_str` — interned strings for identifiers
- `text-size` — text offset types compatible with rowan
- `codespan-reporting` — diagnostic rendering to terminal

## Reference Materials

- [SysML v2 Pilot Xtext Grammar](https://github.com/Systems-Modeling/SysML-v2-Pilot-Implementation) — authoritative grammar productions
- [OMG SysML v2 Specification](https://www.omg.org/sysml/sysmlv2/) — formal language spec
- [rust-analyzer architecture](https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/architecture.md) — CST/AST dual-layer pattern
- [Rowan crate](https://github.com/rust-analyzer/rowan) — lossless syntax trees
