//! Workspace-level validation rules.
//!
//! - WS050: Duplicate qualified names across the workspace
//! - WS051: Part definition never instantiated (unused)

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use sysml_v2_adapter::{SysmlWorkspace, SymbolKind};

use crate::diagnostic::{Diagnostic, Severity};
use crate::domain::DomainConfig;

use super::{effective_severity, to_display_line};

pub(crate) fn check_workspace(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    check_duplicate_names(workspace, config, &mut diagnostics);
    check_unused_parts(workspace, config, &mut diagnostics);

    diagnostics
}

/// Symbol kinds that represent definitions (duplicates are meaningful).
/// Package and import symbols are excluded — it is normal for multiple files
/// to contribute to the same package or import the same names.
fn is_definition_kind(kind: SymbolKind) -> bool {
    matches!(
        kind,
        SymbolKind::PartDefinition
            | SymbolKind::PortDefinition
            | SymbolKind::EnumerationDefinition
            | SymbolKind::StateDefinition
    )
}

/// WS050: Duplicate qualified names among definitions.
fn check_duplicate_names(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("WS050", Severity::Error, config) else {
        return;
    };

    // Collect definition symbols by qualified name
    let mut seen: HashMap<&str, Vec<(PathBuf, usize)>> = HashMap::new();

    for (file, sym) in workspace.all_symbols() {
        if !is_definition_kind(sym.kind) {
            continue;
        }
        let qn: &str = &sym.qualified_name;
        seen.entry(qn)
            .or_default()
            .push((file.path.clone(), to_display_line(sym.start_line)));
    }

    for (qn, locations) in &seen {
        if locations.len() > 1 {
            let first = &locations[0];
            for loc in &locations[1..] {
                diagnostics.push(Diagnostic {
                    file: loc.0.clone(),
                    line: loc.1,
                    col: 1,
                    severity,
                    rule_id: "WS050".to_string(),
                    message: format!(
                        "duplicate qualified name '{qn}' — first defined at {}:{}",
                        first.0.display(),
                        first.1,
                    ),
                    help: None,
                });
            }
        }
    }
}

/// WS051: Unused part definition (never instantiated via PartUsage).
fn check_unused_parts(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("WS051", Severity::Warning, config) else {
        return;
    };

    // Collect all type names referenced by PartUsage symbols
    let mut used_types: HashSet<String> = HashSet::new();
    for (_, sym) in workspace.all_symbols() {
        if sym.kind == SymbolKind::PartUsage {
            for st in &sym.supertypes {
                used_types.insert(st.to_string());
            }
        }
    }

    // Check each PartDefinition
    for (file, sym) in workspace.all_symbols() {
        if sym.kind == SymbolKind::PartDefinition {
            let name: &str = &sym.name;
            if !used_types.contains(name) {
                diagnostics.push(Diagnostic {
                    file: file.path.clone(),
                    line: to_display_line(sym.start_line),
                    col: 1,
                    severity,
                    rule_id: "WS051".to_string(),
                    message: format!("part definition '{name}' is never instantiated"),
                    help: Some("no PartUsage in the workspace references this type".to_string()),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DomainConfig;
    use std::path::PathBuf;
    use sysml_v2_adapter::SysmlWorkspace;

    fn minimal_config() -> DomainConfig {
        DomainConfig {
            name: "test".to_string(),
            description: None,
            metadata_library: PathBuf::new(),
            layers: crate::domain::LayerConfig {
                order: Vec::new(),
                allowed_deps: HashMap::new(),
            },
            required_metadata: crate::domain::RequiredMetadataConfig {
                parts: Vec::new(),
            },
            type_map: HashMap::new(),
            validation_rules: HashMap::new(),
            source: crate::domain::SourceConfig::default(),
        }
    }

    #[test]
    fn test_ws_no_duplicates() {
        let source = r#"
package Firmware {
    part def A { }
    part def B { }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let diags = check_workspace(&ws, &config);
        let ws050: Vec<_> = diags.iter().filter(|d| d.rule_id == "WS050").collect();
        assert!(ws050.is_empty(), "no duplicates expected: {:?}", ws050);
    }

    #[test]
    fn test_ws_duplicate() {
        let source_a = r#"
package Firmware {
    part def Widget { }
}
"#;
        let source_b = r#"
package Firmware {
    part def Widget { }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![
            (PathBuf::from("a.sysml"), source_a.into()),
            (PathBuf::from("b.sysml"), source_b.into()),
        ]);
        let config = minimal_config();
        let diags = check_workspace(&ws, &config);
        let ws050: Vec<_> = diags.iter().filter(|d| d.rule_id == "WS050").collect();
        assert!(!ws050.is_empty(), "should detect duplicate 'Firmware::Widget'");
    }

    #[test]
    fn test_ws_unused() {
        let source = r#"
package Firmware {
    part def Unused { }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let diags = check_workspace(&ws, &config);
        let ws051: Vec<_> = diags.iter().filter(|d| d.rule_id == "WS051").collect();
        assert!(!ws051.is_empty(), "should flag unused PartDefinition");
    }

    #[test]
    fn test_ws_used() {
        let source = r#"
package Firmware {
    part def Widget { }
    part def Container {
        part w : Widget;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let diags = check_workspace(&ws, &config);
        let ws051: Vec<_> = diags.iter().filter(|d| d.rule_id == "WS051").collect();
        // Container is unused but Widget is used
        let unused_names: Vec<_> = ws051.iter().map(|d| &d.message).collect();
        assert!(
            !unused_names.iter().any(|m| m.contains("Widget")),
            "Widget should NOT be flagged as unused: {:?}",
            unused_names
        );
    }
}
