//! Domain-agnostic validation engine.
//!
//! Applies validation rules parameterized by [`DomainConfig`] to a SysML workspace.
//! Rules are grouped by category: layer deps, required metadata, FSM well-formedness,
//! port compatibility, and workspace-level checks.

use sysml_v2_adapter::SysmlWorkspace;

use crate::diagnostic::{Diagnostic, Severity};
use crate::domain::DomainConfig;

mod fsm;
mod layers;
mod metadata;
mod ports;
mod ui;
mod workspace_rules;

/// Result of running validation on a workspace.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub diagnostics: Vec<Diagnostic>,
    pub parts_checked: usize,
    pub state_machines_checked: usize,
    pub connections_checked: usize,
    pub ui_elements_checked: usize,
}

/// Validate a workspace against domain configuration rules.
///
/// Runs all enabled validation rules and returns collected diagnostics.
/// Rules with severity `Off` in the config are skipped entirely.
pub fn validate(workspace: &SysmlWorkspace, config: &DomainConfig) -> ValidationResult {
    let mut diagnostics = Vec::new();

    // Workspace-level rules (duplicates, unused defs)
    diagnostics.extend(workspace_rules::check_workspace(workspace, config));

    // Required metadata
    let (meta_diags, meta_parts) = metadata::check_required_metadata(workspace, config);
    diagnostics.extend(meta_diags);

    // FSM well-formedness
    let (fsm_diags, state_machines_checked) = fsm::check_fsm_wellformedness(workspace, config);
    diagnostics.extend(fsm_diags);

    // Port checks
    let (port_diags, port_conns) = ports::check_ports(workspace, config);
    diagnostics.extend(port_diags);

    // Layer dependency checks
    let (layer_diags, layer_parts, layer_conns) = layers::check_layer_deps(workspace, config);
    diagnostics.extend(layer_diags);

    // UI well-formedness
    let (ui_diags, ui_elements_checked) = ui::check_ui_wellformedness(workspace, config);
    diagnostics.extend(ui_diags);

    let parts_checked = meta_parts.max(layer_parts);
    let connections_checked = port_conns.max(layer_conns);

    ValidationResult {
        diagnostics,
        parts_checked,
        state_machines_checked,
        connections_checked,
        ui_elements_checked,
    }
}

/// Resolve the effective severity for a rule.
///
/// If the config has an override, use it; otherwise use the default.
/// Returns `None` if the rule is disabled (`Severity::Off`).
pub(crate) fn effective_severity(
    rule_id: &str,
    default: Severity,
    config: &DomainConfig,
) -> Option<Severity> {
    match config.validation_rules.get(rule_id) {
        Some(Severity::Off) => None,
        Some(sev) => Some(*sev),
        None => Some(default),
    }
}

/// Convert a 0-indexed HIR line to a 1-indexed diagnostic line.
pub(crate) fn to_display_line(hir_line: u32) -> usize {
    (hir_line as usize) + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DomainConfig;
    use std::path::PathBuf;
    use sysml_v2_adapter::SysmlWorkspace;

    fn domains_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("domains")
    }

    fn load_firmware_config() -> DomainConfig {
        DomainConfig::load(&domains_dir().join("firmware"), None).unwrap()
    }

    fn has_rule(result: &ValidationResult, rule_id: &str) -> bool {
        result.diagnostics.iter().any(|d| d.rule_id == rule_id)
    }

    #[test]
    fn test_severity_override() {
        // Create a config where META010 is overridden to error
        let mut config = load_firmware_config();
        config
            .validation_rules
            .insert("META010".to_string(), Severity::Error);

        // Create a part missing required metadata
        let source = r#"
package Test {
    part def Bare {
        attribute x : Integer;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let result = validate(&ws, &config);

        let meta_diags: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| d.rule_id == "META010")
            .collect();
        assert!(!meta_diags.is_empty(), "should have META010 diagnostics");
        for d in &meta_diags {
            assert_eq!(d.severity, Severity::Error, "META010 should be overridden to error");
        }
    }

    #[test]
    fn test_rule_disabled() {
        // Create a config where META010 is off
        let mut config = load_firmware_config();
        config
            .validation_rules
            .insert("META010".to_string(), Severity::Off);

        // Create a part missing required metadata — should NOT produce META010
        let source = r#"
package Test {
    part def Bare {
        attribute x : Integer;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let result = validate(&ws, &config);

        assert!(
            !has_rule(&result, "META010"),
            "META010 should be disabled, but found diagnostics: {:?}",
            result.diagnostics
        );
    }
}
