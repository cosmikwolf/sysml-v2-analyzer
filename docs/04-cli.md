# Component: CLI

**Crate:** `sysml-v2-analyzer`
**Domain scope:** General orchestration — domain-specific behavior comes from the loaded domain
**Status:** Implemented

## Purpose

Single binary that discovers the workspace config (`sysml.toml`), loads the selected domain, and runs the pipeline stages. The CLI itself has no domain knowledge — it delegates to the engine.

## Commands

```
sysml-v2-analyzer <command> [options]

Commands:
  parse       Parse .sysml files and report errors
  validate    Run domain validation rules
  extract     Extract models to YAML/JSON
  audit       Compare spec against source code
  status      Show workspace status (files, modules, diagnostics summary)
  check       Parse + validate (no extraction or audit)
  init        Initialize a new sysml.toml in the current directory
```

## Global options

```
  -w, --workspace <DIR>    Workspace root (default: auto-detect from sysml.toml)
  -c, --config <FILE>      Path to sysml.toml (default: walk up from cwd)
  -d, --domain <NAME>      Override domain (default: from sysml.toml)
      --format <FMT>       Output format: text (default), json
  -q, --quiet              Errors only
  -v, --verbose            Detailed output
```

## Config discovery

The CLI finds `sysml.toml` by walking up from the current directory, similar to how Cargo finds `Cargo.toml`. The search stops at the filesystem root or a `.git` directory. Can be overridden with `--config`.

## Command details

### `parse`

Loads the workspace and reports parse errors. Does not require a domain — this is pure SysML v2 syntax checking via the adapter.

```
$ sysml-v2-analyzer parse
Parsed 6 files (0 errors, 0 warnings)

$ sysml-v2-analyzer parse
spec/broken.sysml:12:5 error: expected ';' after attribute declaration
Parsed 6 files (1 error)
```

### `validate`

Runs domain validation rules from `domain.toml`. Requires a domain.

```
$ sysml-v2-analyzer validate
spec/bt_a2dp_sink.sysml:8:1 warning[META010]: part 'BtA2dpSink' missing @ErrorHandling
Validated 4 parts, 2 state machines
Result: 0 errors, 1 warning
```

### `extract`

Validates first, then extracts models to `extracted/` (or `--output <dir>`).

```
$ sysml-v2-analyzer extract -o build/extracted
Extracted 4 modules, 2 state machines, 4 interfaces
Output: build/extracted/
```

### `audit`

Full pipeline: validate → extract → compare spec against source code.

```
$ sysml-v2-analyzer audit
BtA2dpSink (src/bt_a2dp_sink.rs):
  ✓ struct BtA2dpSink
  ✓ action Init
  ~ action Start — spec: (self: BtA2dpSink), code: (self: &mut Self)

$ sysml-v2-analyzer audit --uncovered
# also shows code items with no spec counterpart

$ sysml-v2-analyzer audit BtA2dpSink
# audit a single module
```

### `status`

Quick overview of the workspace.

```
$ sysml-v2-analyzer status
Workspace: /path/to/project
Domain: firmware
Files: 6 .sysml files
Modules: BtA2dpSink, AudioPipeline, I2sOutput, StatusLed
State machines: ConnectionFSM, LedFSM
Diagnostics: 0 errors, 1 warning
```

### `check`

Parse + validate without extraction. Fast feedback loop.

### `init`

Interactive setup: creates `sysml.toml` in the current directory, asks which domain to use.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success (no errors) |
| 1 | Validation errors found |
| 2 | Parse errors found |
| 3 | Configuration error (missing sysml.toml, invalid domain, etc.) |

## Dependencies

- `sysml-v2-adapter` — workspace loading (for `parse`)
- `sysml-v2-engine` — validation, extraction, audit
- `clap` — argument parsing with derive macros
- `serde_json` — JSON output format
