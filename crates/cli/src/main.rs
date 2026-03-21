use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use sysml_v2_adapter::SysmlWorkspace;
use sysml_v2_engine::audit;
use sysml_v2_engine::diagnostic::Severity;
use sysml_v2_engine::domain::{DomainConfig, WorkspaceConfig};
use sysml_v2_engine::extraction;
use sysml_v2_engine::validation;

// ── CLI definition ──────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "sysml-v2-analyzer", version, about = "SysML v2 analysis toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Path to sysml.toml (default: walk up from cwd)
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Domain override (default: from sysml.toml)
    #[arg(long, global = true)]
    domain: Option<String>,

    /// Output format
    #[arg(long, global = true, default_value = "text")]
    format: OutputFormat,

    /// Quiet mode (errors only)
    #[arg(long, short, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Parse SysML files and report errors
    Parse {
        /// Directory to scan for .sysml files
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Parse and validate against domain rules
    Validate,

    /// Parse and validate (alias for validate)
    Check,

    /// Parse, validate, and extract to YAML/JSON
    Extract {
        /// Output directory
        #[arg(long, short, default_value = "output")]
        output: PathBuf,

        /// Extract format
        #[arg(long, default_value = "yaml")]
        extract_format: ExtractFormat,
    },

    /// Compare spec against source code
    Audit {
        /// Show code not covered by spec
        #[arg(long)]
        uncovered: bool,

        /// Expand macros before parsing
        #[arg(long)]
        expand: bool,

        /// Audit a specific module
        module: Option<String>,
    },

    /// Show workspace status summary
    Status,

    /// Create a new sysml.toml in the current directory
    Init {
        /// Domain name
        #[arg(default_value = "firmware")]
        domain: String,
    },
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, ValueEnum)]
enum ExtractFormat {
    Yaml,
    Json,
}

// ── Main ────────────────────────────────────────────────────────────

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(&cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(3)
        }
    }
}

fn run(cli: &Cli) -> Result<ExitCode, CliError> {
    match &cli.command {
        Command::Parse { path } => cmd_parse(cli, path),
        Command::Validate | Command::Check => cmd_validate(cli),
        Command::Extract {
            output,
            extract_format,
        } => cmd_extract(cli, output, extract_format),
        Command::Audit {
            uncovered,
            expand,
            module,
        } => cmd_audit(cli, *uncovered, *expand, module.as_deref()),
        Command::Status => cmd_status(cli),
        Command::Init { domain } => cmd_init(domain),
    }
}

// ── Error type ──────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error("no sysml.toml found (searched from cwd upward)")]
    ConfigNotFound,

    #[error("domain directory not found: {0}")]
    DomainNotFound(PathBuf),

    #[error("configuration error: {0}")]
    Config(#[from] sysml_v2_engine::domain::ConfigError),

    #[error("workspace error: {0}")]
    Adapter(#[from] sysml_v2_adapter::AdapterError),

    #[error("extraction error: {0}")]
    Extraction(#[from] extraction::ExtractionError),

    #[error("audit error: {0}")]
    Audit(#[from] audit::AuditError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// ── Config discovery ────────────────────────────────────────────────

/// Walk up from `start` looking for `sysml.toml`.
fn find_config(start: &Path) -> Option<PathBuf> {
    let mut dir = std::fs::canonicalize(start).ok()?;
    loop {
        let candidate = dir.join("sysml.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Resolved configuration from sysml.toml and domain directory.
struct ResolvedConfig {
    domain_config: DomainConfig,
    workspace_config: WorkspaceConfig,
    workspace_root: PathBuf,
}

fn resolve_config(cli: &Cli) -> Result<ResolvedConfig, CliError> {
    // Find sysml.toml
    let config_path = match &cli.config {
        Some(p) => {
            if p.exists() {
                p.clone()
            } else {
                return Err(CliError::ConfigNotFound);
            }
        }
        None => {
            let cwd = std::env::current_dir()?;
            find_config(&cwd).ok_or(CliError::ConfigNotFound)?
        }
    };

    let workspace_root = config_path
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    // Load workspace config to get domain name
    let ws_config = WorkspaceConfig::load(&config_path)?;

    // Domain name: CLI flag overrides sysml.toml
    let domain_name = cli.domain.as_deref().unwrap_or(&ws_config.domain);

    // Find domain directory
    let domain_dir = workspace_root.join("domains").join(domain_name);
    if !domain_dir.exists() {
        return Err(CliError::DomainNotFound(domain_dir));
    }

    let domain_config = DomainConfig::load(&domain_dir, Some(&config_path))?;

    Ok(ResolvedConfig {
        domain_config,
        workspace_config: ws_config,
        workspace_root,
    })
}

fn load_workspace(resolved: &ResolvedConfig) -> Result<SysmlWorkspace, CliError> {
    let ws = &resolved.workspace_config;
    if !ws.include.is_empty() || !ws.exclude.is_empty() {
        Ok(SysmlWorkspace::load_filtered(
            &resolved.workspace_root,
            &ws.include,
            &ws.exclude,
        )?)
    } else {
        Ok(SysmlWorkspace::load(&resolved.workspace_root)?)
    }
}

// ── Command implementations ─────────────────────────────────────────

fn cmd_parse(cli: &Cli, path: &Path) -> Result<ExitCode, CliError> {
    let ws = SysmlWorkspace::load(path)?;

    let errors = ws.all_errors();

    if errors.is_empty() {
        if !cli.quiet {
            println!(
                "Parsed {} file(s) — no errors",
                ws.files().len()
            );
        }
        Ok(ExitCode::SUCCESS)
    } else {
        match cli.format {
            OutputFormat::Text => {
                for (file, err) in &errors {
                    eprintln!("{}:  {:?}", file.path.display(), err);
                }
                eprintln!(
                    "\n{} parse error(s) in {} file(s)",
                    errors.len(),
                    ws.files().len()
                );
            }
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "parse_errors": errors.iter().map(|(f, e)| {
                        serde_json::json!({
                            "file": f.path.display().to_string(),
                            "error": format!("{:?}", e),
                        })
                    }).collect::<Vec<_>>(),
                    "file_count": ws.files().len(),
                });
                println!("{}", serde_json::to_string_pretty(&json).unwrap());
            }
        }
        Ok(ExitCode::from(2))
    }
}

fn cmd_validate(cli: &Cli) -> Result<ExitCode, CliError> {
    let resolved = resolve_config(cli)?;
    let ws = load_workspace(&resolved)?;
    let result = validation::validate(&ws, &resolved.domain_config);

    let error_count = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warning_count = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    match cli.format {
        OutputFormat::Text => {
            for d in &result.diagnostics {
                if cli.quiet && d.severity != Severity::Error {
                    continue;
                }
                eprintln!("{d}");
            }
            if !cli.quiet {
                println!(
                    "\nValidated {} part(s), {} state machine(s), {} UI element(s)",
                    result.parts_checked, result.state_machines_checked,
                    result.ui_elements_checked,
                );
                println!(
                    "Result: {} error(s), {} warning(s)",
                    error_count, warning_count,
                );
            }
        }
        OutputFormat::Json => {
            let diagnostics: Vec<_> = result
                .diagnostics
                .iter()
                .filter(|d| !cli.quiet || d.severity == Severity::Error)
                .map(|d| {
                    serde_json::json!({
                        "file": d.file.display().to_string(),
                        "line": d.line,
                        "col": d.col,
                        "severity": d.severity.to_string(),
                        "rule_id": d.rule_id,
                        "message": d.message,
                        "help": d.help,
                    })
                })
                .collect();

            let json = serde_json::json!({
                "diagnostics": diagnostics,
                "summary": {
                    "parts_checked": result.parts_checked,
                    "state_machines_checked": result.state_machines_checked,
                    "connections_checked": result.connections_checked,
                    "ui_elements_checked": result.ui_elements_checked,
                    "errors": error_count,
                    "warnings": warning_count,
                }
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
    }

    if error_count > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn cmd_extract(
    cli: &Cli,
    output: &Path,
    extract_format: &ExtractFormat,
) -> Result<ExitCode, CliError> {
    let resolved = resolve_config(cli)?;
    let ws = load_workspace(&resolved)?;
    let validation_result = validation::validate(&ws, &resolved.domain_config);

    let error_count = validation_result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();

    if error_count > 0 {
        // Print validation errors first
        for d in &validation_result.diagnostics {
            if d.severity == Severity::Error {
                eprintln!("{d}");
            }
        }
        eprintln!("\nExtraction blocked: {error_count} validation error(s)");
        return Ok(ExitCode::from(1));
    }

    let result = extraction::extract(&ws, &resolved.domain_config, &validation_result)?;

    let format = match extract_format {
        ExtractFormat::Yaml => extraction::types::OutputFormat::Yaml,
        ExtractFormat::Json => extraction::types::OutputFormat::Json,
    };

    let written = extraction::write_extraction(&result, output, format)?;

    if !cli.quiet {
        println!(
            "Extracted {} module(s) to {}",
            result.modules.len(),
            output.display(),
        );
        for path in &written {
            println!("  {}", path.display());
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn cmd_audit(
    cli: &Cli,
    uncovered: bool,
    expand: bool,
    module: Option<&str>,
) -> Result<ExitCode, CliError> {
    let resolved = resolve_config(cli)?;
    let ws = load_workspace(&resolved)?;
    let validation_result = validation::validate(&ws, &resolved.domain_config);

    let error_count = validation_result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();

    if error_count > 0 {
        for d in &validation_result.diagnostics {
            if d.severity == Severity::Error {
                eprintln!("{d}");
            }
        }
        eprintln!("\nAudit blocked: {error_count} validation error(s)");
        return Ok(ExitCode::from(1));
    }

    let extraction_result =
        extraction::extract(&ws, &resolved.domain_config, &validation_result)?;

    // Find languages directory relative to the domain directory
    let languages_dir = resolved.workspace_root.join("domains").join("..").join("..").join("languages");
    // Also try relative to the binary
    let languages_dir = if languages_dir.join("rust").join("audit.scm").exists() {
        languages_dir
    } else {
        // Fall back: look next to the workspace root
        let alt = resolved.workspace_root.join("languages");
        if alt.join("rust").join("audit.scm").exists() {
            alt
        } else {
            // Last resort: look relative to the executable
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_default();
            exe_dir.join("languages")
        }
    };

    let report = audit::audit(
        &extraction_result,
        &resolved.domain_config,
        &resolved.workspace_root,
        &languages_dir,
        uncovered,
        expand,
        module,
    )?;

    let summary = report.summary();

    match cli.format {
        OutputFormat::Text => {
            print!("{}", audit::format_text(&report));
            if !cli.quiet {
                println!(
                    "Audit: {} match(es), {} missing, {} mismatch(es), {} uncovered",
                    summary.matches, summary.missing, summary.mismatches, summary.uncovered
                );
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&report).unwrap();
            println!("{json}");
        }
    }

    if summary.missing > 0 || summary.mismatches > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn cmd_status(cli: &Cli) -> Result<ExitCode, CliError> {
    let resolved = resolve_config(cli)?;
    let ws = load_workspace(&resolved)?;

    let file_count = ws.files().len();
    let part_defs = ws.symbols_of_kind(sysml_v2_adapter::SymbolKind::PartDefinition);
    let state_defs = ws.symbols_of_kind(sysml_v2_adapter::SymbolKind::StateDefinition);
    let port_defs = ws.symbols_of_kind(sysml_v2_adapter::SymbolKind::PortDefinition);
    let parse_errors: usize = ws.files().iter().map(|f| f.parse.errors.len()).sum();

    match cli.format {
        OutputFormat::Text => {
            println!("Workspace: {}", resolved.workspace_root.display());
            println!("Domain: {}", resolved.domain_config.name);
            println!();
            println!("Files:        {file_count}");
            println!("Parts:        {}", part_defs.len());
            println!("State machines: {}", state_defs.len());
            println!("Port defs:    {}", port_defs.len());
            if parse_errors > 0 {
                println!("Parse errors: {parse_errors}");
            }
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "workspace": resolved.workspace_root.display().to_string(),
                "domain": resolved.domain_config.name,
                "files": file_count,
                "parts": part_defs.len(),
                "state_machines": state_defs.len(),
                "port_defs": port_defs.len(),
                "parse_errors": parse_errors,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn cmd_init(domain: &str) -> Result<ExitCode, CliError> {
    let path = Path::new("sysml.toml");
    if path.exists() {
        eprintln!("sysml.toml already exists");
        return Ok(ExitCode::from(3));
    }

    let content = format!(
        r#"[workspace]
domain = "{domain}"
include = ["**/*.sysml"]
"#
    );

    std::fs::write(path, content)?;
    println!("Created sysml.toml (domain: {domain})");
    Ok(ExitCode::SUCCESS)
}
