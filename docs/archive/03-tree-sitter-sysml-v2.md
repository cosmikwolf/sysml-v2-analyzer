# Tool: tree-sitter-sysml-v2

> **Note:** An existing tree-sitter grammar for SysML v2 exists at [gitlab.com/nomograph/tree-sitter-sysml](https://gitlab.com/nomograph/tree-sitter-sysml) (192 tests, 98% coverage, MIT license). It is not on crates.io but is usable for Neovim highlighting. Evaluate it before building our own. See [10-revised-plan.md](./10-revised-plan.md) decision D6.

Tree-sitter grammar for SysML v2, providing syntax highlighting and structural queries in Neovim.

## Purpose

Provide fast, incremental syntax highlighting for `.sysml` and `.kerml` files in Neovim via tree-sitter. Tree-sitter operates independently of the LSP server — it highlights code instantly on file open, before the LSP has finished indexing.

The LSP server provides **semantic tokens** (richer, type-aware highlighting), but tree-sitter provides the **baseline** that works without any server running.

## Depends On

- Nothing — tree-sitter grammars are standalone

## Scope

The grammar covers the syntactic structure of SysML v2's textual notation. It does not perform semantic analysis (name resolution, type checking). It needs to handle the same subset defined in the sysml-v2-parser spec, plus enough error recovery to highlight partially-typed code.

## Grammar Structure

Tree-sitter grammars are defined in JavaScript (`grammar.js`). The key node types:

```
source_file
├── package_declaration
│   ├── package_keyword
│   ├── qualified_name
│   └── package_body
│       ├── import_statement
│       ├── part_definition
│       │   ├── metadata_annotation  (@ISRSafe { ... })
│       │   ├── part_keyword
│       │   ├── def_keyword
│       │   ├── name
│       │   ├── specialization  (:> Base)
│       │   └── body
│       │       ├── attribute_usage
│       │       ├── port_usage
│       │       ├── part_usage
│       │       ├── state_definition
│       │       ├── action_usage
│       │       └── connection_usage
│       ├── port_definition
│       ├── enum_definition
│       │   └── enum_member
│       ├── metadata_definition
│       ├── state_definition
│       │   ├── state_usage
│       │   ├── transition
│       │   │   ├── accept_keyword
│       │   │   ├── event_reference
│       │   │   ├── guard_expression  (if ...)
│       │   │   ├── do_action         (do ...)
│       │   │   └── target_state      (then ...)
│       │   └── entry_transition
│       ├── action_definition
│       ├── constraint_definition
│       ├── connection_definition
│       ├── flow_usage
│       └── alias_declaration
├── comment
└── doc_comment
```

## Highlight Queries

File: `queries/highlights.scm`

```scheme
; Keywords
["part" "def" "port" "state" "action" "metadata" "constraint"
 "enum" "import" "alias" "connect" "flow" "perform" "satisfy"
 "require" "in" "out" "inout" "entry" "exit" "do" "then"
 "accept" "if" "parallel" "of" "from" "to" "for"
 "redefines" "subsets"] @keyword

; Type-related keywords
["attribute" "item"] @keyword.type

; Boolean literals
["true" "false"] @boolean

; Null
["null"] @constant.builtin

; Operators
[":>" ":>>" "~" "@" "::" "->" "=" ">" "<" ">=" "<=" "==" "!=" "+" "-" "*" "/"] @operator

; Punctuation
["{" "}" "(" ")" "[" "]" ";"] @punctuation.bracket
["," "."] @punctuation.delimiter

; Names in definitions
(part_definition name: (identifier) @type.definition)
(port_definition name: (identifier) @type.definition)
(enum_definition name: (identifier) @type.definition)
(metadata_definition name: (identifier) @type.definition)
(state_definition name: (identifier) @type.definition)
(action_definition name: (identifier) @type.definition)
(constraint_definition name: (identifier) @type.definition)

; Names in usages
(part_usage name: (identifier) @variable)
(port_usage name: (identifier) @variable)
(attribute_usage name: (identifier) @property)
(state_usage name: (identifier) @variable)

; Type references
(type_reference (qualified_name) @type)

; Metadata annotations
(metadata_annotation "@" @attribute (qualified_name) @attribute)

; Enum members
(enum_member name: (identifier) @constant)

; Specialization targets
(specialization (qualified_name) @type)

; String literals
(string_literal) @string

; Number literals
(integer_literal) @number
(real_literal) @number.float

; Comments
(comment) @comment
(doc_comment) @comment.documentation

; Import paths
(import_statement path: (qualified_name) @module)
```

## Additional Queries

### Locals (scoping)

File: `queries/locals.scm`

```scheme
; Package bodies create scopes
(package_body) @local.scope

; Part/port/state bodies create scopes
(part_definition body: (_) @local.scope)
(state_definition body: (_) @local.scope)

; Definitions create bindings
(part_definition name: (identifier) @local.definition)
(port_definition name: (identifier) @local.definition)
(part_usage name: (identifier) @local.definition)

; Type references are references
(type_reference (qualified_name) @local.reference)
```

### Folds

File: `queries/folds.scm`

```scheme
(package_body) @fold
(part_definition body: (_) @fold)
(port_definition body: (_) @fold)
(state_definition body: (_) @fold)
(enum_definition body: (_) @fold)
(metadata_definition body: (_) @fold)
(constraint_definition body: (_) @fold)
(comment) @fold
```

### Indents

File: `queries/indents.scm`

```scheme
(package_body) @indent
(part_definition) @indent
(port_definition) @indent
(state_definition) @indent
(enum_definition) @indent
"{" @indent
"}" @dedent
```

## Error Recovery

The grammar must use tree-sitter's `ERROR` and `MISSING` recovery mechanisms to handle:

- Incomplete definitions (typing `part def ` with no name yet)
- Missing semicolons
- Unclosed braces
- Invalid tokens inside otherwise valid structures

Tree-sitter's GLR parsing handles most ambiguities automatically, but we should define `extras` (comments, whitespace) and `conflicts` explicitly.

## Tests

### Highlighting tests

Tree-sitter test format: annotated source files in `test/highlight/`.

```
test_highlight_part_def         — `part def Engine;` highlights `part` and `def` as keyword, `Engine` as type.definition
test_highlight_metadata         — `@ISRSafe { safe = true; }` highlights `@ISRSafe` as attribute, `true` as boolean
test_highlight_state_machine    — states, transitions, `accept`/`then` all highlighted correctly
test_highlight_enum             — enum name as type.definition, members as constant
test_highlight_port_directions  — `in`, `out`, `inout` as keyword
test_highlight_specialization   — `:>` as operator, target as type
test_highlight_comments         — line comments, block comments, doc comments
test_highlight_string_literals  — single, double, triple-quoted strings
test_highlight_numbers          — integers and reals
test_highlight_import           — `import` as keyword, path as module
```

### Parsing tests

```
test_parse_minimal              — `part def X;` produces correct tree structure
test_parse_nested               — nested parts produce correct parent-child relationships
test_parse_full_module          — complete firmware module definition parses without ERROR nodes
test_parse_state_machine        — state machine with transitions, guards, actions
test_parse_error_recovery       — incomplete input produces partial tree, not total failure
test_parse_metadata_annotation  — @-prefixed metadata parses as annotation, not error
test_parse_conjugated_port      — `~PortType` parses conjugation
test_parse_qualified_names      — `A::B::C` parses as single qualified_name node
```

### Corpus tests

Tree-sitter convention: `test/corpus/*.txt` files with paired input/expected-tree blocks.

```
test_corpus_empty_file
test_corpus_package_with_imports
test_corpus_part_hierarchy
test_corpus_port_definitions
test_corpus_enum_definitions
test_corpus_metadata_definitions_and_usages
test_corpus_state_machines
test_corpus_action_definitions
test_corpus_connections_and_flows
test_corpus_constraints
test_corpus_specialization_chains
test_corpus_mixed_firmware_module     — a realistic firmware module with all constructs
```

### Performance tests

```
perf_parse_1000_lines           — parse completes in < 5ms
perf_incremental_edit           — single character edit re-highlights in < 1ms
perf_error_recovery_cost        — file with 50% errors does not degrade > 3x vs clean file
```

## Deliverables

1. `grammar.js` — tree-sitter grammar definition
2. `queries/highlights.scm` — syntax highlighting queries
3. `queries/locals.scm` — scope/reference queries
4. `queries/folds.scm` — code folding queries
5. `queries/indents.scm` — auto-indent queries
6. `test/corpus/*.txt` — tree-sitter corpus tests
7. `test/highlight/*.sysml` — highlighting tests
8. Neovim ftdetect/ftplugin for `.sysml` and `.kerml` file types

## Distribution

- Published to the [nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter) parser list
- Installable via `:TSInstall sysml`
- Grammar compiled to C (tree-sitter convention) and also usable from Rust via `tree-sitter` crate bindings

## Reference Materials

- [Tree-sitter documentation](https://tree-sitter.github.io/tree-sitter/)
- [Tree-sitter creating parsers](https://tree-sitter.github.io/tree-sitter/creating-parsers/)
- [nvim-treesitter adding parsers](https://github.com/nvim-treesitter/nvim-treesitter#adding-parsers)
- [SysML v2 Pilot Xtext Grammar](https://github.com/Systems-Modeling/SysML-v2-Pilot-Implementation) — reference for syntax
