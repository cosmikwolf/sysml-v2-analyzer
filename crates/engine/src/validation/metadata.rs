//! Required metadata validation.
//!
//! - META010: Part definition missing a required metadata annotation

use std::collections::HashSet;

use sysml_v2_adapter::metadata_extractor::extract_metadata;
use sysml_v2_adapter::{SysmlWorkspace, SymbolKind};

use crate::diagnostic::{Diagnostic, Severity};
use crate::domain::DomainConfig;

use super::{effective_severity, to_display_line};

/// Check that all PartDefinitions have the required metadata annotations.
///
/// Returns diagnostics and the number of parts checked.
pub(crate) fn check_required_metadata(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
) -> (Vec<Diagnostic>, usize) {
    let mut diagnostics = Vec::new();
    let mut parts_checked = 0;

    if config.required_metadata.parts.is_empty() {
        return (diagnostics, parts_checked);
    }

    let Some(severity) = effective_severity("META010", Severity::Warning, config) else {
        return (diagnostics, parts_checked);
    };

    let required: HashSet<&str> = config
        .required_metadata
        .parts
        .iter()
        .map(|s| s.as_str())
        .collect();

    for (file, sym) in workspace.all_symbols() {
        if sym.kind != SymbolKind::PartDefinition {
            continue;
        }
        parts_checked += 1;

        let annotations = extract_metadata(file, sym);
        let present: HashSet<&str> = annotations.iter().map(|a| a.name.as_str()).collect();

        for req in &required {
            if !present.contains(req) {
                diagnostics.push(Diagnostic {
                    file: file.path.clone(),
                    line: to_display_line(sym.start_line),
                    col: 1,
                    severity,
                    rule_id: "META010".to_string(),
                    message: format!(
                        "part definition '{}' is missing required metadata annotation '@{}'",
                        sym.name, req,
                    ),
                    help: None,
                });
            }
        }
    }

    (diagnostics, parts_checked)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::domain::{DomainConfig, LayerConfig, RequiredMetadataConfig};
    use sysml_v2_adapter::SysmlWorkspace;

    fn config_requiring(names: &[&str]) -> DomainConfig {
        DomainConfig {
            name: "test".to_string(),
            description: None,
            metadata_library: PathBuf::new(),
            layers: LayerConfig {
                order: Vec::new(),
                allowed_deps: HashMap::new(),
            },
            required_metadata: RequiredMetadataConfig {
                parts: names.iter().map(|s| s.to_string()).collect(),
            },
            type_map: HashMap::new(),
            validation_rules: HashMap::new(),
            source: crate::domain::SourceConfig::default(),
        }
    }

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("tests")
            .join("fixtures")
    }

    fn load_fixture(name: &str) -> String {
        std::fs::read_to_string(fixtures_dir().join(name))
            .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", name, e))
    }

    #[test]
    fn test_meta_all_present() {
        // BtA2dpSink has @MemoryModel, @ConcurrencyModel, @ErrorHandling
        let lib = load_fixture("firmware_library.sysml");
        let src = load_fixture("bt_a2dp_sink.sysml");
        let ws = SysmlWorkspace::from_sources(vec![
            (PathBuf::from("firmware_library.sysml"), lib),
            (PathBuf::from("bt_a2dp_sink.sysml"), src),
        ]);
        let config = config_requiring(&["MemoryModel", "ConcurrencyModel", "ErrorHandling"]);
        let (diags, parts) = check_required_metadata(&ws, &config);
        let meta010: Vec<_> = diags.iter().filter(|d| d.rule_id == "META010").collect();
        assert!(
            meta010.is_empty(),
            "BtA2dpSink has all required metadata, but got: {:?}",
            meta010
        );
        assert!(parts > 0);
    }

    #[test]
    fn test_meta_missing() {
        let source = r#"
package Test {
    part def Bare {
        attribute x : Integer;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = config_requiring(&["MemoryModel", "ErrorHandling"]);
        let (diags, _) = check_required_metadata(&ws, &config);
        let meta010: Vec<_> = diags.iter().filter(|d| d.rule_id == "META010").collect();
        assert_eq!(meta010.len(), 2, "should flag 2 missing annotations: {:?}", meta010);
    }

    #[test]
    fn test_meta_empty_required_list() {
        let source = r#"
package Test {
    part def Bare { }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = config_requiring(&[]);
        let (diags, _) = check_required_metadata(&ws, &config);
        assert!(diags.is_empty(), "no required metadata → no diagnostics");
    }
}
