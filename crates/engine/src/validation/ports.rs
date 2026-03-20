//! Port compatibility validation.
//!
//! - PORT030: Connected ports have incompatible types (DEFERRED — requires type resolution)
//! - PORT033: Port defined but not connected

use sysml_v2_adapter::connection_resolver::resolve_connections;
use sysml_v2_adapter::{SysmlWorkspace, SymbolKind};

use crate::diagnostic::{Diagnostic, Severity};
use crate::domain::DomainConfig;

use super::{effective_severity, to_display_line};

/// Check port connectivity for all parts in the workspace.
///
/// Returns diagnostics and the number of connections checked.
pub(crate) fn check_ports(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
) -> (Vec<Diagnostic>, usize) {
    let mut diagnostics = Vec::new();
    let mut connections_checked = 0;

    for (file, sym) in workspace.all_symbols() {
        if sym.kind != SymbolKind::PartDefinition {
            continue;
        }

        let connections = resolve_connections(file, sym);
        connections_checked += connections.len();

        check_unconnected_ports(
            workspace,
            file,
            sym,
            &connections,
            config,
            &mut diagnostics,
        );
    }

    (diagnostics, connections_checked)
}

/// PORT033: Port defined but not connected.
fn check_unconnected_ports(
    workspace: &SysmlWorkspace,
    file: &sysml_v2_adapter::ParsedFile,
    part_symbol: &sysml_v2_adapter::HirSymbol,
    connections: &[sysml_v2_adapter::Connection],
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("PORT033", Severity::Warning, config) else {
        return;
    };

    // Find child PortUsage symbols for this part
    let part_qn: &str = &part_symbol.qualified_name;
    let port_names: Vec<&str> = workspace
        .all_symbols()
        .filter(|(f, s)| {
            f.path == file.path
                && s.kind == SymbolKind::PortUsage
                && s.qualified_name.starts_with(part_qn)
                && s.qualified_name.len() > part_qn.len()
        })
        .map(|(_, s)| s.name.as_ref())
        .collect();

    if port_names.is_empty() {
        return;
    }

    // Build a set of port names referenced in connections
    let mut connected_ports: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for conn in connections {
        // Connection endpoints can be "portName" or "subpart.portName"
        // Extract the port name from both source and target
        for endpoint in [&conn.source, &conn.target] {
            if let Some(dot_pos) = endpoint.rfind('.') {
                connected_ports.insert(&endpoint[dot_pos + 1..]);
            }
            connected_ports.insert(endpoint);
        }
    }

    let part_name: &str = &part_symbol.name;
    for port_name in &port_names {
        if !connected_ports.contains(port_name) {
            diagnostics.push(Diagnostic {
                file: file.path.clone(),
                line: to_display_line(part_symbol.start_line),
                col: 1,
                severity,
                rule_id: "PORT033".to_string(),
                message: format!(
                    "port '{}' in '{}' is defined but not connected",
                    port_name, part_name,
                ),
                help: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::domain::{DomainConfig, LayerConfig, RequiredMetadataConfig};
    use sysml_v2_adapter::SysmlWorkspace;

    fn minimal_config() -> DomainConfig {
        DomainConfig {
            name: "test".to_string(),
            description: None,
            metadata_library: PathBuf::new(),
            layers: LayerConfig {
                order: Vec::new(),
                allowed_deps: HashMap::new(),
            },
            required_metadata: RequiredMetadataConfig {
                parts: Vec::new(),
            },
            type_map: HashMap::new(),
            validation_rules: HashMap::new(),
            template_dir: PathBuf::new(),
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
    fn test_port_all_connected() {
        // AudioPipeline has ports that are all connected
        let lib = load_fixture("firmware_library.sysml");
        let ifaces = load_fixture("interfaces.sysml");
        let src = load_fixture("audio_pipeline.sysml");
        let ws = SysmlWorkspace::from_sources(vec![
            (PathBuf::from("firmware_library.sysml"), lib),
            (PathBuf::from("interfaces.sysml"), ifaces),
            (PathBuf::from("audio_pipeline.sysml"), src),
        ]);
        let config = minimal_config();
        let (diags, conns) = check_ports(&ws, &config);
        assert!(conns > 0, "should find connections");
        let port033: Vec<_> = diags.iter().filter(|d| d.rule_id == "PORT033").collect();
        // AudioPipeline connects all 3 ports
        assert!(
            port033.is_empty(),
            "AudioPipeline ports should all be connected: {:?}",
            port033
        );
    }

    #[test]
    fn test_port_unused() {
        let source = r#"
package Test {
    port def TestPort { }
    part def Lonely {
        port p : TestPort;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_ports(&ws, &config);
        let port033: Vec<_> = diags.iter().filter(|d| d.rule_id == "PORT033").collect();
        assert!(
            !port033.is_empty(),
            "should detect unconnected port 'p': {:?}",
            port033
        );
    }

    #[test]
    fn test_port_no_ports() {
        let source = r#"
package Test {
    part def NoPorts {
        attribute x : Integer;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_ports(&ws, &config);
        assert!(diags.is_empty(), "no ports → no diagnostics");
    }
}
