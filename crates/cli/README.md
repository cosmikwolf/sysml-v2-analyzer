# sysml-v2-analyzer (CLI)

Unified CLI binary for the SysML v2 analyzer toolchain.

Discovers workspace config (`sysml.toml`), loads the selected domain, and runs pipeline stages via subcommands. The CLI itself has no domain knowledge — it delegates to the engine.

## Commands

```
sysml-v2-analyzer <command> [options]

  parse       Parse .sysml files and report syntax errors
  validate    Validate against domain rules (layer deps, metadata, FSMs)
  check       Alias for validate
  extract     Extract models to YAML/JSON files
  audit       Compare spec against source code (tree-sitter)
  status      Show workspace summary (files, parts, FSMs, ports)
  init        Create sysml.toml in current directory
```

## Examples

```bash
# Parse a directory of .sysml files
sysml-v2-analyzer parse spec/

# Validate with the firmware domain
sysml-v2-analyzer validate

# Validate with JSON output
sysml-v2-analyzer --format json validate

# Extract to YAML (default) or JSON
sysml-v2-analyzer extract -o output/
sysml-v2-analyzer extract -o output/ --extract-format json

# Audit spec against source code
sysml-v2-analyzer audit

# Audit with JSON output
sysml-v2-analyzer --format json audit

# Show workspace info
sysml-v2-analyzer status

# Override domain from command line
sysml-v2-analyzer --domain template validate

# Create a new project config
sysml-v2-analyzer init firmware
```

## Global options

| Flag | Description |
|---|---|
| `--config <path>` | Path to `sysml.toml` (default: walk up from cwd) |
| `--domain <name>` | Domain override (default: from `sysml.toml`) |
| `--format text\|json` | Output format (default: `text`) |
| `-q, --quiet` | Errors only |

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Validation errors |
| 2 | Parse errors |
| 3 | Configuration error (missing `sysml.toml`, bad domain, etc.) |

## Dependencies

- `sysml-v2-adapter` — workspace loading (for `parse` command)
- `sysml-v2-engine` — validation, extraction, audit
- `clap` — argument parsing

## Design

See [docs/phase-6-cli.md](../../docs/phase-6-cli.md) for the full design spec.
