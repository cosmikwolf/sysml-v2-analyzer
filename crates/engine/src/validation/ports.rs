//! Port compatibility validation.
//!
//! - PORT030: Connected ports have incompatible types
//! - PORT033: Port defined but not connected

use std::collections::HashMap;

use sysml_v2_adapter::connection_resolver::resolve_connections;
use sysml_v2_adapter::{ConnectionKind, SysmlWorkspace, SymbolKind};

use crate::diagnostic::{Diagnostic, Severity};
use crate::domain::DomainConfig;

use super::{effective_severity, to_display_line};

/// Check port connectivity and type compatibility for all parts in the workspace.
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

        check_port_type_compatibility(
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

/// PORT030: Connected ports have incompatible types.
///
/// For each `connect` statement, resolves the port type of both endpoints
/// and checks they reference the same PortDefinition. Flow connections are
/// skipped (they carry data types, not port types).
fn check_port_type_compatibility(
    workspace: &SysmlWorkspace,
    file: &sysml_v2_adapter::ParsedFile,
    part_symbol: &sysml_v2_adapter::HirSymbol,
    connections: &[sysml_v2_adapter::Connection],
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("PORT030", Severity::Error, config) else {
        return;
    };

    let part_qn: &str = &part_symbol.qualified_name;
    let part_name: &str = &part_symbol.name;

    // Build port_name → type_name map for this part's direct ports
    let local_port_types: HashMap<String, String> = workspace
        .all_symbols()
        .filter(|(f, s)| {
            f.path == file.path
                && s.kind == SymbolKind::PortUsage
                && s.qualified_name.starts_with(part_qn)
                && s.qualified_name.len() > part_qn.len()
        })
        .filter_map(|(_, s)| {
            let type_name = s.supertypes.first()?.to_string();
            Some((s.name.to_string(), type_name))
        })
        .collect();

    // Build subpart_name → type_name map for child PartUsage symbols
    let subpart_types: HashMap<String, String> = workspace
        .all_symbols()
        .filter(|(f, s)| {
            f.path == file.path
                && s.kind == SymbolKind::PartUsage
                && s.qualified_name.starts_with(part_qn)
                && s.qualified_name.len() > part_qn.len()
        })
        .filter_map(|(_, s)| {
            let type_name = s.supertypes.first()?.to_string();
            Some((s.name.to_string(), type_name))
        })
        .collect();

    for conn in connections {
        // Only check connect statements, not flow
        if conn.kind != ConnectionKind::Connect {
            continue;
        }

        let source_type = resolve_endpoint_port_type(
            &conn.source,
            &local_port_types,
            &subpart_types,
            workspace,
        );
        let target_type = resolve_endpoint_port_type(
            &conn.target,
            &local_port_types,
            &subpart_types,
            workspace,
        );

        if let (Some(src_type), Some(tgt_type)) = (&source_type, &target_type) {
            if src_type != tgt_type {
                diagnostics.push(Diagnostic {
                    file: file.path.clone(),
                    line: to_display_line(part_symbol.start_line),
                    col: 1,
                    severity,
                    rule_id: "PORT030".to_string(),
                    message: format!(
                        "incompatible port types in '{}': '{}' has type '{}' but '{}' has type '{}'",
                        part_name, conn.source, src_type, conn.target, tgt_type,
                    ),
                    help: Some("connected ports should reference the same port definition type".to_string()),
                });
            }
        }
    }
}

/// Resolve the port type for a connection endpoint.
///
/// - Simple endpoint (`"audioIn"`) → look up in local port types
/// - Compound endpoint (`"bt.audioOut"`) → resolve subpart type, then
///   find the port on that type's PartDefinition
fn resolve_endpoint_port_type(
    endpoint: &str,
    local_port_types: &HashMap<String, String>,
    subpart_types: &HashMap<String, String>,
    workspace: &SysmlWorkspace,
) -> Option<String> {
    if let Some(dot_pos) = endpoint.find('.') {
        // Compound: "subpart.portName"
        let subpart_name = &endpoint[..dot_pos];
        let port_name = &endpoint[dot_pos + 1..];

        // Get the subpart's type (PartDefinition name)
        let part_type = subpart_types.get(subpart_name)?;

        // Find that PartDefinition in the workspace, then find the port on it
        for (_, sym) in workspace.all_symbols() {
            if sym.kind == SymbolKind::PartDefinition && *sym.name == **part_type {
                // Find PortUsage child of this PartDefinition with matching name
                let part_def_qn: &str = &sym.qualified_name;
                for (_, port_sym) in workspace.all_symbols() {
                    if port_sym.kind == SymbolKind::PortUsage
                        && *port_sym.name == *port_name
                        && port_sym.qualified_name.starts_with(part_def_qn)
                        && port_sym.qualified_name.len() > part_def_qn.len()
                    {
                        return port_sym.supertypes.first().map(|s| s.to_string());
                    }
                }
            }
        }
        None
    } else {
        // Simple: direct port on this part
        local_port_types.get(endpoint).cloned()
    }
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

    let mut connected_ports: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for conn in connections {
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
        assert!(
            port033.is_empty(),
            "AudioPipeline ports should all be connected: {:?}",
            port033
        );
    }

    #[test]
    fn test_port_compatible_types() {
        // AudioPipeline connects ports of matching types — no PORT030
        let lib = load_fixture("firmware_library.sysml");
        let ifaces = load_fixture("interfaces.sysml");
        let bt = load_fixture("bt_a2dp_sink.sysml");
        let i2s = load_fixture("i2s_output.sysml");
        let led = load_fixture("status_led.sysml");
        let src = load_fixture("audio_pipeline.sysml");
        let ws = SysmlWorkspace::from_sources(vec![
            (PathBuf::from("firmware_library.sysml"), lib),
            (PathBuf::from("interfaces.sysml"), ifaces),
            (PathBuf::from("bt_a2dp_sink.sysml"), bt),
            (PathBuf::from("i2s_output.sysml"), i2s),
            (PathBuf::from("status_led.sysml"), led),
            (PathBuf::from("audio_pipeline.sysml"), src),
        ]);
        let config = minimal_config();
        let (diags, _) = check_ports(&ws, &config);
        let port030: Vec<_> = diags.iter().filter(|d| d.rule_id == "PORT030").collect();
        assert!(
            port030.is_empty(),
            "AudioPipeline connections should have compatible types: {:?}",
            port030
        );
    }

    #[test]
    fn test_port_incompatible_types() {
        let source = r#"
package Test {
    port def TypeA { }
    port def TypeB { }
    part def Mismatch {
        port a : TypeA;
        port b : TypeB;
        connect a to b;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_ports(&ws, &config);
        let port030: Vec<_> = diags.iter().filter(|d| d.rule_id == "PORT030").collect();
        assert!(
            !port030.is_empty(),
            "should detect incompatible port types TypeA vs TypeB"
        );
    }

    #[test]
    fn test_port_type_resolution_compound() {
        let source = r#"
package Test {
    port def SharedPort { }
    part def Inner {
        port p : SharedPort;
    }
    part def Outer {
        part sub : Inner;
        port local : SharedPort;
        connect sub.p to local;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_ports(&ws, &config);
        let port030: Vec<_> = diags.iter().filter(|d| d.rule_id == "PORT030").collect();
        assert!(
            port030.is_empty(),
            "compound endpoint should resolve to same type: {:?}",
            port030
        );
    }

    #[test]
    fn test_port_flow_skipped() {
        // Flow connections don't trigger PORT030 (they carry data types, not port types)
        let source = r#"
package Test {
    port def TypeA { }
    port def TypeB { }
    part def FlowTest {
        port a : TypeA;
        port b : TypeB;
        flow of Integer from a.data to b.data;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_ports(&ws, &config);
        let port030: Vec<_> = diags.iter().filter(|d| d.rule_id == "PORT030").collect();
        assert!(
            port030.is_empty(),
            "flow connections should not trigger PORT030"
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
