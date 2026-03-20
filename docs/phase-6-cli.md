# Phase 6: CLI

## Goal

Implement the `sysml-v2-analyzer` binary that discovers workspace config, loads the domain, and runs pipeline stages via subcommands.

## Implementation

### Argument parsing

Use `clap` derive macros:

```rust
#[derive(Parser)]
#[command(name = "sysml-v2-analyzer")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Workspace root (default: auto-detect from sysml.toml)
    #[arg(short, long, global = true)]
    workspace: Option<PathBuf>,

    /// Path to sysml.toml (default: walk up from cwd)
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Override domain (default: from sysml.toml)
    #[arg(short, long, global = true)]
    domain: Option<String>,

    /// Output format
    #[arg(long, global = true, default_value = "text")]
    format: OutputFormat,

    /// Quiet mode (errors only)
    #[arg(short, long, global = true)]
    quiet: bool,
}
```

### Config discovery

```rust
fn find_config(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join("sysml.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        if dir.join(".git").exists() || !dir.pop() {
            return None;
        }
    }
}
```

### Domain resolution

1. Find `sysml.toml` (from `--config`, or by walking up from cwd)
2. Read `workspace.domain` from `sysml.toml` (or use `--domain` override)
3. Locate `domains/<domain>/domain.toml` relative to the analyzer installation
4. Load + merge configs via `DomainConfig::load()`

### Command implementations

Each command follows the same pattern:

```rust
fn run_validate(cli: &Cli) -> Result<ExitCode> {
    let config = load_config(cli)?;
    let workspace = load_workspace(&config)?;
    let result = engine::validate(&workspace, &config.domain);
    print_diagnostics(&result.diagnostics, cli.format, cli.quiet);
    if result.diagnostics.iter().any(|d| d.severity == Severity::Error) {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}
```

`parse` is special — it only uses the adapter, no domain required.

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Validation errors |
| 2 | Parse errors |
| 3 | Configuration error |

## Output formatting

### Text (default)

```
spec/bt_a2dp_sink.sysml:8:1 warning[META010]: part 'BtA2dpSink' missing @ErrorHandling
  = help: add @ErrorHandling annotation with strategy field

Validated 4 parts, 2 state machines
Result: 0 errors, 1 warning
```

### JSON

```json
{
  "diagnostics": [
    {
      "file": "spec/bt_a2dp_sink.sysml",
      "line": 8,
      "col": 1,
      "severity": "warning",
      "rule_id": "META010",
      "message": "part 'BtA2dpSink' missing @ErrorHandling",
      "help": "add @ErrorHandling annotation with strategy field"
    }
  ],
  "summary": {
    "parts_checked": 4,
    "state_machines_checked": 2,
    "errors": 0,
    "warnings": 1
  }
}
```

## Tests

CLI integration tests using `assert_cmd` or direct process spawning:

- `test_parse_clean` — parse valid fixtures → exit 0, "0 errors"
- `test_parse_broken` — parse malformed file → exit 2, error in output
- `test_validate_clean` — validate well-formed workspace → exit 0
- `test_validate_errors` — validate workspace with issues → exit 1, diagnostics in output
- `test_config_missing` — no sysml.toml found → exit 3, helpful error
- `test_domain_missing` — sysml.toml references nonexistent domain → exit 3
- `test_json_format` — `--format json` → valid JSON output
- `test_quiet_mode` — `--quiet` → only errors shown
- `test_status` — status command → shows workspace summary
- `test_init` — init command → creates sysml.toml

## Verification

```
cargo build -p sysml-v2-analyzer  # binary compiles
cargo test -p sysml-v2-analyzer   # CLI tests pass
./target/debug/sysml-v2-analyzer parse tests/fixtures/  # manual smoke test
```
