# Tool: sysml-v2-lsp

> **Status: DEFERRED** — syster-lsp (alpha) already provides an LSP server built on syster-base. Evaluate it for our workflow before building a custom server. See [10-revised-plan.md](./10-revised-plan.md) decision D5.

Rust binary implementing the Language Server Protocol for SysML v2 files, targeting Neovim.

## Purpose

Provide a first-class editing experience for `.sysml` and `.kerml` files in Neovim (and any other LSP-capable editor). This replaces the need for Eclipse or the archived VS Code extension.

## Depends On

- **sysml-v2-adapter** — for workspace loading and firmware-specific queries (replaces sysml-v2-parser; see [01-sysml-v2-adapter.md](./01-sysml-v2-adapter.md))

## LSP Features

### Phase 1 (Core)

| Feature | LSP Method | Description |
|---|---|---|
| Diagnostics | `textDocument/publishDiagnostics` | Syntax errors, unresolved names, type errors, firmware validation warnings |
| Go to definition | `textDocument/definition` | Jump to part def, port def, metadata def, enum def from any usage |
| Find references | `textDocument/references` | Find all usages of a definition |
| Hover | `textDocument/hover` | Show definition signature, documentation, metadata annotations |
| Completion | `textDocument/completion` | Keywords, element names in scope, metadata fields, enum members, port types |
| Document symbols | `textDocument/documentSymbol` | Outline view: parts, ports, states, actions, constraints |
| Workspace symbols | `workspace/symbol` | Find any definition across the workspace |
| Semantic tokens | `textDocument/semanticTokens/full` | Rich syntax highlighting (keywords, types, metadata, annotations) |

### Phase 2 (Enhanced)

| Feature | LSP Method | Description |
|---|---|---|
| Rename | `textDocument/rename` | Rename a definition and all references across workspace |
| Code actions | `textDocument/codeAction` | Quick fixes: add missing metadata fields, add missing import |
| Formatting | `textDocument/formatting` | Auto-format `.sysml` files using consistent style |
| Signature help | `textDocument/signatureHelp` | Show action parameter info while typing |
| Folding ranges | `textDocument/foldingRange` | Collapse part bodies, state machines, constraint blocks |
| Inlay hints | `textDocument/inlayHint` | Show inferred types, resolved metadata values |

### Phase 3 (Firmware-specific)

| Feature | LSP Method | Description |
|---|---|---|
| Layer violation diagnostic | `publishDiagnostics` | Warn when a driver-layer part connects to an application-layer part |
| Metadata completeness | `publishDiagnostics` | Warn when a part is missing required firmware metadata (e.g., `@MemoryModel`) |
| State machine visualization | Custom notification | Send Mermaid diagram of state machine to a Neovim side panel |
| Dependency graph | Custom notification | Send module dependency graph as DOT/Mermaid |

## Architecture

```
┌────────────────────────────────────────────┐
│  Neovim                                    │
│  (nvim-lspconfig + tree-sitter-sysml-v2)      │
│                                            │
│  ← stdio JSON-RPC →                        │
└────────────────┬───────────────────────────┘
                 │
┌────────────────▼───────────────────────────┐
│  sysml-v2-lsp binary                          │
│                                            │
│  ┌──────────────────────────────────────┐  │
│  │  LSP Protocol Layer                  │  │
│  │  (tower-lsp or lsp-server)           │  │
│  └──────────────┬───────────────────────┘  │
│                 │                           │
│  ┌──────────────▼───────────────────────┐  │
│  │  Document Manager                    │  │
│  │  - Open file tracking                │  │
│  │  - Change application (incremental)  │  │
│  │  - Version management                │  │
│  └──────────────┬───────────────────────┘  │
│                 │                           │
│  ┌──────────────▼───────────────────────┐  │
│  │  sysml-v2-parser (library)              │  │
│  │  - Incremental reparsing             │  │
│  │  - Name resolution                   │  │
│  │  - Type checking                     │  │
│  │  - Firmware validation               │  │
│  └──────────────────────────────────────┘  │
└────────────────────────────────────────────┘
```

### Key design considerations

1. **Startup time** — the LSP server must be usable within 500ms of Neovim opening a `.sysml` file. Background-index the workspace; serve immediate requests from the open file only.

2. **Incremental processing** — when the user types a character, do not reparse the entire workspace. Use the salsa-style incremental computation from sysml-v2-parser to invalidate only affected queries.

3. **Error tolerance** — the parser must always produce a partial tree. The LSP must provide completions and go-to-definition even in files with syntax errors.

4. **Stdio transport** — Neovim communicates via JSON-RPC over stdio. No HTTP server needed.

## Neovim Configuration

The LSP server should work with this minimal `nvim-lspconfig` setup:

```lua
-- In user's Neovim config
local lspconfig = require('lspconfig')

lspconfig.sysml_v2.setup {
  cmd = { 'sysml-v2-lsp' },
  filetypes = { 'sysml', 'kerml' },
  root_dir = lspconfig.util.root_pattern('.sysml-workspace', '.git'),
  settings = {
    sysml = {
      firmwareLibrary = './lib/firmware.sysml',
      validation = {
        layerRules = true,
        metadataCompleteness = true,
      },
    },
  },
}
```

## Workspace Discovery

The LSP server discovers workspace files by:

1. Looking for a `.sysml-workspace` marker file (or `sysml.toml` config) in parent directories
2. Falling back to the Git repository root
3. Scanning for `*.sysml` and `*.kerml` files recursively
4. Respecting `.gitignore` patterns

The workspace config file (`sysml.toml`) can specify:
```toml
[workspace]
include = ["spec/**/*.sysml", "lib/**/*.sysml"]
exclude = ["target/**"]

[firmware]
metadata_library = "lib/firmware.sysml"
layer_rules = true
metadata_completeness = true
```

## Tests

### Protocol tests

```
test_initialize             — server responds to initialize with correct capabilities
test_shutdown               — server shuts down cleanly on shutdown/exit
test_did_open               — opening a file triggers parsing and publishes diagnostics
test_did_change             — editing a file triggers incremental reparse and updated diagnostics
test_did_close              — closing a file clears its diagnostics
test_did_open_invalid       — opening a file with syntax errors publishes error diagnostics
```

### Diagnostics tests

```
diag_syntax_error           — missing semicolon produces diagnostic at correct position
diag_unresolved_name        — reference to nonexistent part produces error diagnostic
diag_type_error             — wrong metadata field type produces error diagnostic
diag_multiple_errors        — file with 3 errors produces exactly 3 diagnostics
diag_clear_on_fix           — fixing an error removes the diagnostic on next change
diag_cross_file             — error in file A caused by change in file B is reported
diag_firmware_layer         — layer violation produces warning diagnostic
diag_firmware_metadata      — missing @MemoryModel produces info diagnostic
```

### Navigation tests

```
goto_def_part_usage         — go-to-definition on `engine : Engine` jumps to `part def Engine`
goto_def_port_type          — go-to-definition on port type jumps to port def
goto_def_metadata_type      — go-to-definition on `@ISRSafe` jumps to metadata def
goto_def_enum_member        — go-to-definition on `LayerKind::driver` jumps to enum member
goto_def_import             — go-to-definition on imported name jumps to source file
goto_def_cross_file         — go-to-definition across files in workspace
find_refs_part_def          — find-references on `part def Engine` lists all usages
find_refs_metadata_def      — find-references on metadata def lists all `@M` annotations
```

### Completion tests

```
complete_keywords           — typing `par` suggests `part`, `parallel`
complete_part_names         — inside `: ` after a part usage, suggest all part defs in scope
complete_port_types         — inside port usage, suggest port defs
complete_metadata_fields    — inside `@M { }`, suggest M's attribute names
complete_enum_members       — after `EnumName::`, suggest all members
complete_import_path        — after `import `, suggest package names
complete_state_targets      — after `then `, suggest sibling state names
complete_after_error        — completion works even when file has syntax errors elsewhere
```

### Hover tests

```
hover_part_def              — hovering part def shows its doc comment and metadata summary
hover_port_usage            — hovering port shows port def signature and direction
hover_metadata_usage        — hovering @M shows metadata def fields and current values
hover_enum_member           — hovering enum member shows parent enum name
hover_action_param          — hovering action parameter shows type
```

### Document symbol tests

```
symbols_empty_file          — empty file produces no symbols
symbols_part_defs           — file with 3 part defs produces 3 symbols
symbols_nested              — nested parts appear as children in symbol tree
symbols_state_machine       — states appear as children of state def
symbols_mixed               — file with parts, ports, enums, metadata produces correct hierarchy
```

### Semantic token tests

```
tokens_keywords             — `part`, `def`, `port`, `state` etc. get keyword token type
tokens_types                — type references get type token type
tokens_metadata             — `@ISRSafe` gets decorator token type
tokens_comments             — comments get comment token type
tokens_strings              — string literals get string token type
```

### Performance tests

```
perf_startup_time           — server ready to serve requests within 500ms on a 50-file workspace
perf_completion_latency     — completion response within 50ms
perf_goto_def_latency       — go-to-definition response within 20ms
perf_diagnostic_latency     — diagnostics published within 200ms of file change
perf_large_file             — 2000-line file does not degrade responsiveness
```

### Integration tests

```
integration_neovim_stdio    — full round-trip: spawn server, send initialize, open file, get diagnostics via stdio
integration_multi_file_edit — edit file A, verify diagnostics update in file B
integration_workspace_reload — add new .sysml file to workspace, verify it gets indexed
```

## Dependencies (Rust crates)

- `tower-lsp` — LSP protocol implementation (async, tower-based)
- `tokio` — async runtime
- `serde` / `serde_json` — JSON-RPC serialization
- `sysml-v2-parser` — our parser crate (workspace dependency)
- `dashmap` — concurrent hash map for document store
- `notify` — file system watcher for workspace changes

## Neovim Plugin Requirements

A minimal Neovim plugin (Lua) should be distributed alongside the binary:

- Register `sysml` and `kerml` filetypes
- Configure `nvim-lspconfig` entry
- Optional: keybindings for firmware-specific actions (show dependency graph, show state machine diagram)
- Optional: integration with a Mermaid renderer for diagram previews
