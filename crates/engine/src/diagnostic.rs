//! Shared diagnostic type for validation and analysis results.

use std::fmt;
use std::path::PathBuf;

/// Severity level for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
    Off,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
            Severity::Off => write!(f, "off"),
        }
    }
}

/// A diagnostic emitted during validation or analysis.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub severity: Severity,
    pub rule_id: String,
    pub message: String,
    pub help: Option<String>,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{} {}[{}]: {}",
            self.file.display(),
            self.line,
            self.col,
            self.severity,
            self.rule_id,
            self.message,
        )?;
        if let Some(help) = &self.help {
            write!(f, "\n  help: {help}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_display() {
        let d = Diagnostic {
            file: PathBuf::from("module.sysml"),
            line: 10,
            col: 5,
            severity: Severity::Error,
            rule_id: "LAYER001".to_string(),
            message: "illegal dependency from application to pac".to_string(),
            help: Some("application may only depend on middleware".to_string()),
        };
        let output = d.to_string();
        assert_eq!(
            output,
            "module.sysml:10:5 error[LAYER001]: illegal dependency from application to pac\n  help: application may only depend on middleware"
        );
    }

    #[test]
    fn test_diagnostic_display_no_help() {
        let d = Diagnostic {
            file: PathBuf::from("test.sysml"),
            line: 1,
            col: 0,
            severity: Severity::Warning,
            rule_id: "META010".to_string(),
            message: "missing required metadata".to_string(),
            help: None,
        };
        assert_eq!(
            d.to_string(),
            "test.sysml:1:0 warning[META010]: missing required metadata"
        );
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::Error.to_string(), "error");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Info.to_string(), "info");
        assert_eq!(Severity::Off.to_string(), "off");
    }
}
