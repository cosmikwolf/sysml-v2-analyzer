# Tool: sysml-v2-fw

Unified CLI binary that orchestrates the entire firmware spec-driven workflow.

## Purpose

Single entry point for the complete pipeline: parse → validate → extract → contract → codegen. Instead of invoking each tool separately, developers use `fw` subcommands. This also provides workflow commands (init, status, migrate) that span multiple tools.

## Depends On

- **sysml-v2-parser** — parsing
- **sysml-v2-validate** — firmware validation
- **sysml-v2-extract** — model extraction
- **sysml-v2-gencontract** — generation contract management
- **sysml-v2-codegen** — code generation

All tools are compiled into a single binary as library dependencies. No subprocess spawning.

## Command Structure

```
sysml-v2-fw <COMMAND> [OPTIONS]

Commands:
  init          Initialize a new firmware project
  parse         Parse .sysml files and report diagnostics
  validate      Run firmware validation rules
  extract       Extract architecture data to YAML/JSON
  contract      Manage generation contracts
  generate      Generate source code
  status        Show pipeline status (what's stale, what needs regeneration)
  migrate       Migrate existing YAML specs to SysML v2
  clean         Remove generated/extracted artifacts
  check         Run full pipeline check (parse → validate → extract → contract validate)
  completions   Generate shell completions (bash, zsh, fish)

Aliases:
  sysml-v2-fw gen        → sysml-v2-fw generate
  sysml-v2-fw val        → fw validate
  sysml-v2-fw ext        → fw extract
```

### `sysml-v2-fw init`

```
$ sysml-v2-sysml-v2-fw init [OPTIONS]

Initialize a new firmware project with SysML v2 spec structure.

Options:
  --language <LANG>       Target language: rust, c, cpp (default: rust)
  --rtos <RTOS>           RTOS: none, embassy, freertos, zephyr (default: none)
  --mcu <MCU>             Target MCU (e.g., esp32, stm32f411, rp2040)
  --name <NAME>           Project name

Creates:
  spec/                   Directory for .sysml architecture files
  spec/firmware.sysml     Firmware metadata library (LayerKind, MemoryModel, etc.)
  spec/architecture.sysml Workspace package with project-level metadata
  gencontract.yaml        Generation contract with sensible defaults
  sysml.toml              Workspace configuration
  .sysml-workspace        Workspace marker file
```

### `fw parse`

```
$ sysml-v2-fw parse [OPTIONS] [FILES...]

Parse SysML v2 files and report diagnostics.

Options:
  --format <FMT>     Output: text (default), json, sarif
  --workspace        Parse entire workspace (default if no files specified)

Examples:
  sysml-v2-fw parse                           # parse all workspace files
  sysml-v2-fw parse spec/bt_a2dp_sink.sysml   # parse specific file
```

### `fw validate`

```
$ sysml-v2-fw validate [OPTIONS] [MODULE...]

Run firmware validation rules.

Options:
  -r, --rule <RULE_ID>    Run specific rule(s) only
  -s, --severity <LEVEL>  Minimum severity: error, warning, info
  --format <FMT>          Output: text (default), json, sarif
  --fix                   Apply auto-fixes

Examples:
  sysml-v2-fw validate                        # validate entire workspace
  sysml-v2-fw validate BtA2dpSink             # validate specific module
  sysml-v2-fw validate --rule FW001,FW003     # check layer rules only
```

### `fw extract`

```
$ sysml-v2-fw extract [OPTIONS] [MODULE...]

Extract architecture data to YAML/JSON.

Options:
  -o, --output <DIR>      Output directory (default: extracted/)
  -f, --format <FMT>      yaml (default), json
  --include-spans          Include source spans
  --dry-run               Show what would be extracted

Examples:
  sysml-v2-fw extract                         # extract all modules
  sysml-v2-fw extract BtA2dpSink              # extract specific module
```

### `fw contract`

```
$ sysml-v2-fw contract <SUBCOMMAND>

Subcommands:
  init          Create new gencontract.yaml
  validate      Validate contract against extracted architecture
  resolve       Show effective config for a module
  diff          Compare two contracts
  schema        Print JSON Schema
```

### `sysml-v2-fw generate`

```
$ sysml-v2-sysml-v2-fw generate [OPTIONS] [MODULE...]

Generate source code from specs.

Options:
  -o, --output <DIR>      Output directory (default: src/)
  --force                 Overwrite hand-modified files
  --dry-run               Show what would be generated
  --report                Print generation report
  --all                   Regenerate everything (ignore hashes)

Examples:
  sysml-v2-sysml-v2-fw generate                        # generate stale modules
  sysml-v2-sysml-v2-fw generate --all                  # regenerate everything
  sysml-v2-sysml-v2-fw generate BtA2dpSink             # generate specific module
  sysml-v2-sysml-v2-fw generate --dry-run              # preview changes
```

### `fw status`

```
$ sysml-v2-fw status [OPTIONS]

Show pipeline status.

Output:
  spec/bt_a2dp_sink.sysml          ✓ parsed, ✓ valid, ✓ extracted, ✓ generated
  spec/audio_pipeline.sysml        ✓ parsed, ✓ valid, ✗ extracted (spec changed)
  spec/i2s_output.sysml            ✓ parsed, ✗ valid (1 error), — extracted, — generated
  spec/status_led.sysml            ✓ parsed, ✓ valid, ✓ extracted, ✗ generated (contract changed)
  gencontract.yaml                 ✓ valid

  Summary: 2 modules need regeneration, 1 module has validation errors

Options:
  --format <FMT>     text (default), json
  --module <NAME>    Show status for specific module
```

### `sysml-v2-fw migrate`

```
$ sysml-v2-sysml-v2-fw migrate [OPTIONS]

Migrate existing YAML specs to SysML v2 format.

Reads:
  spec/architecture.yaml
  spec/modules/*.yaml
  spec/state_machines/*.yaml
  spec/interfaces/*.yaml

Writes:
  spec/*.sysml (SysML v2 equivalents)
  gencontract.yaml (extracted from architecture.yaml platform fields)

Options:
  --dry-run               Show what would be generated
  --keep-yaml             Don't delete original YAML files after migration
  --output <DIR>          Output directory for .sysml files (default: spec/)
```

### `sysml-v2-fw check`

```
$ sysml-v2-sysml-v2-fw check [OPTIONS]

Run full pipeline check without generating code.

Equivalent to: sysml-v2-fw parse && fw validate && fw extract --dry-run && fw contract validate

Options:
  --format <FMT>     text (default), json, sarif
  --strict           Treat warnings as errors
```

### `fw clean`

```
$ sysml-v2-fw clean [OPTIONS]

Remove generated artifacts.

Options:
  --extracted       Remove extracted/ directory only
  --generated       Remove generated source files only (identified by @generated header)
  --all             Remove both extracted and generated artifacts
  --dry-run         Show what would be removed
```

## Exit Codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Validation/generation errors (fixable) |
| 2 | Configuration error (bad sysml.toml, bad gencontract.yaml) |
| 3 | Parse error (invalid .sysml syntax) |
| 4 | I/O error (file not found, permission denied) |
| 5 | Internal error (bug in the tool) |

## Global Options

```
Options available on all commands:
  --workspace <DIR>       Workspace root (default: auto-detect from cwd)
  --config <FILE>         Path to sysml.toml
  --color <WHEN>          Color output: auto (default), always, never
  --verbose               Increase output verbosity
  --quiet                 Suppress non-error output
  --log-level <LEVEL>     Log level: error, warn, info, debug, trace
```

## Shell Completions

```
$ sysml-v2-sysml-v2-fw completions fish > ~/.config/fish/completions/fw.fish
$ sysml-v2-sysml-v2-fw completions zsh > ~/.zsh/completions/_fw
$ sysml-v2-sysml-v2-fw completions bash > /etc/bash_completion.d/fw
```

## Tests

### Init tests

```
test_init_creates_structure         — sysml-v2-fw init creates expected directory tree
test_init_rust_defaults             — sysml-v2-fw init --language rust produces correct gencontract
test_init_c_defaults                — sysml-v2-fw init --language c produces correct gencontract
test_init_mcu_esp32                 — sysml-v2-fw init --mcu esp32 sets correct target in architecture.sysml
test_init_firmware_library          — generated firmware.sysml defines all metadata types
test_init_idempotent_fail           — sysml-v2-fw init on existing project fails with helpful message
```

### Parse command tests

```
test_parse_success                  — valid workspace parses with exit code 0
test_parse_syntax_error             — syntax error → exit code 3, diagnostic on stderr
test_parse_specific_file            — parsing single file works
test_parse_json_output              — --format json produces valid JSON diagnostics
```

### Validate command tests

```
test_validate_success               — valid workspace → exit code 0
test_validate_errors                — validation errors → exit code 1
test_validate_specific_rule         — --rule FW001 runs only layer checks
test_validate_specific_module       — module argument filters validation scope
test_validate_severity_filter       — --severity error hides warnings
```

### Generate command tests

```
test_generate_success               — generates files, exit code 0
test_generate_stale_only            — only regenerates modules with changed hashes
test_generate_force                 — --force overwrites hand-modified files
test_generate_dry_run               — --dry-run produces no files
test_generate_specific_module       — module argument generates only that module
test_generate_report                — --report prints generation report
test_generate_all                   — --all ignores hashes, regenerates everything
```

### Status command tests

```
test_status_all_clean               — all modules up-to-date → clean summary
test_status_stale_module            — changed spec → shows needs-regeneration
test_status_validation_error        — invalid module → shows error count
test_status_json_output             — --format json produces valid JSON
```

### Migrate command tests

```
test_migrate_modules                — YAML module specs → .sysml files
test_migrate_state_machines         — YAML FSM specs → state defs in .sysml
test_migrate_interfaces             — YAML interface specs → port defs in .sysml
test_migrate_architecture           — architecture.yaml platform fields → gencontract.yaml
test_migrate_dry_run                — --dry-run shows what would be created
test_migrate_keep_yaml              — --keep-yaml preserves original files
test_migrate_round_trip             — migrate → extract → compare with original YAML
```

### Check command tests

```
test_check_success                  — valid workspace → exit code 0
test_check_parse_failure            — syntax error → exit code 3
test_check_validation_failure       — validation error → exit code 1
test_check_strict                   — --strict treats warnings as errors
```

### Clean command tests

```
test_clean_extracted                — --extracted removes extracted/ only
test_clean_generated                — --generated removes files with @generated header only
test_clean_all                      — --all removes both
test_clean_dry_run                  — --dry-run lists but doesn't delete
test_clean_preserves_handwritten    — does not delete files without @generated header
```

### End-to-end pipeline tests

```
e2e_init_to_codegen                 — sysml-v2-fw init → write .sysml → sysml-v2-fw generate → compilable code
e2e_edit_and_regenerate             — edit .sysml → sysml-v2-fw generate → only changed module regenerated
e2e_add_module                      — add new .sysml → sysml-v2-fw generate → new module files created
e2e_contract_change                 — change gencontract.yaml → sysml-v2-fw generate → all modules regenerated
e2e_migrate_existing_project        — sysml-v2-fw migrate on current YAML project → sysml-v2-fw check passes
```

## Dependencies (Rust crates)

- `sysml-v2-parser`, `sysml-v2-validate`, `sysml-v2-extract`, `sysml-v2-gencontract`, `sysml-v2-codegen` — workspace crates
- `clap` — CLI framework (with derive macros)
- `clap_complete` — shell completion generation
- `console` / `indicatif` — terminal UI (colors, progress bars)
- `tracing` / `tracing-subscriber` — structured logging
- `anyhow` — error handling in CLI context
- `human-panic` — friendly panic messages
