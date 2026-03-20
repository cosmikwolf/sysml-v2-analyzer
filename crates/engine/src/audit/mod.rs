//! Audit engine: compare spec against source code.
//!
//! Uses tree-sitter to parse source files and compares the structural
//! constructs against the extracted SysML modules.

pub mod code_parser;
pub mod compare;
pub mod source_map;

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::domain::DomainConfig;
use crate::extraction::ExtractionResult;

/// Errors that can occur during audit.
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("tree-sitter error: {0}")]
    TreeSitter(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("macro expansion failed: {0}")]
    ExpandFailed(String),
}

/// Result of auditing the workspace.
#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    pub modules: Vec<ModuleAudit>,
}

/// Audit result for a single module.
#[derive(Debug, Clone, Serialize)]
pub struct ModuleAudit {
    pub module_name: String,
    pub source_file: Option<PathBuf>,
    pub items: Vec<AuditItem>,
}

/// A single audit finding.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum AuditItem {
    /// Spec item matches code item.
    Match {
        kind: String,
        name: String,
    },
    /// Spec item has no corresponding code item.
    Missing {
        kind: String,
        name: String,
        detail: String,
    },
    /// Spec item exists in code but differs.
    Mismatch {
        kind: String,
        name: String,
        spec_detail: String,
        code_detail: String,
    },
    /// Code item has no corresponding spec item.
    Uncovered {
        kind: String,
        name: String,
        line: usize,
    },
}

/// Run an audit comparing spec extraction results against source code.
pub fn audit(
    extraction: &ExtractionResult,
    config: &DomainConfig,
    workspace_root: &Path,
    languages_dir: &Path,
    show_uncovered: bool,
    expand: bool,
    module_filter: Option<&str>,
) -> Result<AuditReport, AuditError> {
    let mut modules = Vec::new();

    for module in &extraction.modules {
        // Apply module filter if specified
        if let Some(filter) = module_filter {
            if module.name != filter {
                continue;
            }
        }

        let source_path =
            source_map::resolve_source_path(module, config, workspace_root);

        let items = if let Some(ref path) = source_path {
            let source = if expand {
                try_expand(path, &config.source.language).unwrap_or_else(|e| {
                    eprintln!(
                        "warning: macro expansion failed for {}, using raw source: {}",
                        path.display(),
                        e
                    );
                    std::fs::read_to_string(path).unwrap_or_default()
                })
            } else {
                std::fs::read_to_string(path)?
            };

            let constructs =
                code_parser::parse_source(&source, &config.source.language, languages_dir)?;
            compare::compare_module(module, &constructs, show_uncovered)
        } else {
            // No source file found — all spec items are "missing from code"
            vec![AuditItem::Missing {
                kind: "file".to_string(),
                name: module.name.clone(),
                detail: "source file not found".to_string(),
            }]
        };

        modules.push(ModuleAudit {
            module_name: module.name.clone(),
            source_file: source_path,
            items,
        });
    }

    Ok(AuditReport { modules })
}

/// Try to expand macros in a source file.
fn try_expand(path: &Path, language: &str) -> Result<String, AuditError> {
    match language {
        "rust" => {
            // Use `cargo expand` for Rust — requires nightly + cargo-expand
            let output = std::process::Command::new("cargo")
                .args(["expand", "--lib"])
                .output()
                .map_err(|e| AuditError::ExpandFailed(format!("cargo expand: {}", e)))?;

            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(AuditError::ExpandFailed(format!(
                    "cargo expand failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )))
            }
        }
        "c" => {
            // Use C preprocessor
            let output = std::process::Command::new("cc")
                .args(["-E", &path.to_string_lossy()])
                .output()
                .map_err(|e| AuditError::ExpandFailed(format!("cc -E: {}", e)))?;

            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(AuditError::ExpandFailed(format!(
                    "cc -E failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )))
            }
        }
        other => Err(AuditError::ExpandFailed(format!(
            "no expander for language: {}",
            other
        ))),
    }
}

/// Format an audit report as human-readable text.
pub fn format_text(report: &AuditReport) -> String {
    let mut out = String::new();

    for module_audit in &report.modules {
        let file_display = module_audit
            .source_file
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "not found".to_string());

        out.push_str(&format!("{} ({}):\n", module_audit.module_name, file_display));

        for item in &module_audit.items {
            match item {
                AuditItem::Match { kind, name } => {
                    out.push_str(&format!("  ✓ {} {}\n", kind, name));
                }
                AuditItem::Missing { kind, name, detail } => {
                    out.push_str(&format!("  + {} {} — {}\n", kind, name, detail));
                }
                AuditItem::Mismatch {
                    kind,
                    name,
                    spec_detail,
                    code_detail,
                } => {
                    out.push_str(&format!(
                        "  ~ {} {} — spec: {}, code: {}\n",
                        kind, name, spec_detail, code_detail
                    ));
                }
                AuditItem::Uncovered { kind, name, line } => {
                    out.push_str(&format!("  ? {} {} (line {})\n", kind, name, line));
                }
            }
        }

        out.push('\n');
    }

    out
}

/// Summary statistics for an audit report.
pub struct AuditSummary {
    pub matches: usize,
    pub missing: usize,
    pub mismatches: usize,
    pub uncovered: usize,
}

impl AuditReport {
    pub fn summary(&self) -> AuditSummary {
        let mut matches = 0;
        let mut missing = 0;
        let mut mismatches = 0;
        let mut uncovered = 0;

        for module_audit in &self.modules {
            for item in &module_audit.items {
                match item {
                    AuditItem::Match { .. } => matches += 1,
                    AuditItem::Missing { .. } => missing += 1,
                    AuditItem::Mismatch { .. } => mismatches += 1,
                    AuditItem::Uncovered { .. } => uncovered += 1,
                }
            }
        }

        AuditSummary {
            matches,
            missing,
            mismatches,
            uncovered,
        }
    }
}
