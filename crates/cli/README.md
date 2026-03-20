# sysml-v2-analyzer (CLI)

Unified CLI binary for the SysML v2 analyzer toolchain.

**Status: Not started**

## Purpose

Discovers workspace config (`sysml.toml`), loads the selected domain, and runs pipeline stages via subcommands. The CLI itself has no domain knowledge — it delegates to the engine.

## Commands

```
sysml-v2-analyzer <command> [options]

  parse       Parse .sysml files and report errors
  validate    Run domain validation rules
  extract     Extract models to YAML/JSON
  generate    Full pipeline: validate → extract → generate source code
  status      Show workspace status
  check       Parse + validate
  init        Create sysml.toml in current directory
```

## Dependencies

- `sysml-v2-adapter` — workspace loading (for `parse` command)
- `sysml-v2-engine` — validation, extraction, code generation
- `clap` — argument parsing

## Design

See [docs/04-cli.md](../../docs/04-cli.md) for the full design spec.
