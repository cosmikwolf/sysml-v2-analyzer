//! Layer dependency validation.
//!
//! - LAYER001: Illegal dependency (connection from layer X to layer Y not in allowed_deps)
//! - LAYER002: Part has no layer attribute — can't verify dependencies
//! - LAYER003: Circular dependency between parts
//! - LAYER004: Same-layer dependency (allowed but flagged)

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use petgraph::graph::{DiGraph, NodeIndex};
use sysml_v2_adapter::{SysmlWorkspace, SymbolKind};

use crate::diagnostic::{Diagnostic, Severity};
use crate::domain::DomainConfig;

use super::{effective_severity, to_display_line};
use crate::util::extract_layer_for_part;

/// Information about a part definition with layer assignment.
#[derive(Debug, Clone)]
struct PartInfo {
    name: String,
    qualified_name: String,
    layer: Option<String>,
    file: PathBuf,
    line: usize,
}

/// Check layer dependency rules for all parts in the workspace.
///
/// Returns diagnostics, parts checked, and connections (edges) checked.
pub(crate) fn check_layer_deps(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
) -> (Vec<Diagnostic>, usize, usize) {
    let mut diagnostics = Vec::new();

    // Skip layer checks if no layers configured
    if config.layers.order.is_empty() {
        return (diagnostics, 0, 0);
    }

    let known_layers: HashSet<String> = config.layers.order.iter().cloned().collect();

    // Collect all PartDefinitions and their layer assignments
    let mut parts: Vec<PartInfo> = Vec::new();
    let mut part_index_by_name: HashMap<String, usize> = HashMap::new();

    for (file, sym) in workspace.all_symbols() {
        if sym.kind != SymbolKind::PartDefinition {
            continue;
        }

        let layer = extract_layer_for_part(file, sym, &known_layers);
        let name: String = sym.name.to_string();
        let idx = parts.len();
        part_index_by_name.insert(name.clone(), idx);

        parts.push(PartInfo {
            name,
            qualified_name: sym.qualified_name.to_string(),
            layer,
            file: file.path.clone(),
            line: to_display_line(sym.start_line),
        });
    }

    let parts_checked = parts.len();

    // LAYER002: Check for missing layer attributes
    for part in &parts {
        if part.layer.is_none() {
            if let Some(severity) = effective_severity("LAYER002", Severity::Warning, config) {
                diagnostics.push(Diagnostic {
                    file: part.file.clone(),
                    line: part.line,
                    col: 1,
                    severity,
                    rule_id: "LAYER002".to_string(),
                    message: format!(
                        "part definition '{}' has no layer attribute — layer dependency checking skipped for this part",
                        part.name,
                    ),
                    help: Some(format!(
                        "add an attribute with a value matching one of: {}",
                        config.layers.order.join(", ")
                    )),
                });
            }
        }
    }

    // Build dependency graph from sub-part usages
    let mut graph: DiGraph<usize, ()> = DiGraph::new();
    let mut node_indices: HashMap<usize, NodeIndex> = HashMap::new();

    // Create nodes for all parts
    for (idx, _) in parts.iter().enumerate() {
        let node = graph.add_node(idx);
        node_indices.insert(idx, node);
    }

    // Add edges from sub-part usages (PartUsage symbols)
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for (_, sym) in workspace.all_symbols() {
        if sym.kind != SymbolKind::PartUsage {
            continue;
        }

        // Find which PartDefinition contains this usage (by qualified name prefix)
        let usage_qn: &str = &sym.qualified_name;
        let container_idx = parts.iter().position(|p| {
            usage_qn.starts_with(&p.qualified_name)
                && usage_qn.len() > p.qualified_name.len()
                && usage_qn.as_bytes().get(p.qualified_name.len()) == Some(&b':')
        });

        // Find which PartDefinition is the type of this usage
        let type_name = sym.supertypes.first().map(|s| s.as_ref());
        let target_idx = type_name.and_then(|tn| part_index_by_name.get(tn).copied());

        if let (Some(from), Some(to)) = (container_idx, target_idx) {
            if from != to {
                let from_node = node_indices[&from];
                let to_node = node_indices[&to];
                graph.add_edge(from_node, to_node, ());
                edges.push((from, to));
            }
        }
    }

    let connections_checked = edges.len();

    // LAYER001: Illegal dependency (cross-layer violation)
    // LAYER004: Same-layer dependency
    for &(from_idx, to_idx) in &edges {
        let from_part = &parts[from_idx];
        let to_part = &parts[to_idx];

        let (Some(from_layer), Some(to_layer)) = (&from_part.layer, &to_part.layer) else {
            continue; // Skip if either part has no layer (LAYER002 already flagged)
        };

        if from_layer == to_layer {
            // LAYER004: Same-layer dependency
            if let Some(severity) = effective_severity("LAYER004", Severity::Info, config) {
                diagnostics.push(Diagnostic {
                    file: from_part.file.clone(),
                    line: from_part.line,
                    col: 1,
                    severity,
                    rule_id: "LAYER004".to_string(),
                    message: format!(
                        "same-layer dependency: '{}' and '{}' are both in layer '{}'",
                        from_part.name, to_part.name, from_layer,
                    ),
                    help: None,
                });
            }
        } else {
            // Check if the dependency is allowed
            let allowed = config
                .layers
                .allowed_deps
                .get(from_layer)
                .map(|deps| deps.iter().any(|d| d == to_layer))
                .unwrap_or(false);

            if !allowed {
                if let Some(severity) = effective_severity("LAYER001", Severity::Error, config) {
                    let allowed_list = config
                        .layers
                        .allowed_deps
                        .get(from_layer)
                        .map(|deps| deps.join(", "))
                        .unwrap_or_else(|| "none".to_string());

                    diagnostics.push(Diagnostic {
                        file: from_part.file.clone(),
                        line: from_part.line,
                        col: 1,
                        severity,
                        rule_id: "LAYER001".to_string(),
                        message: format!(
                            "part '{}' (layer: {}) depends on '{}' (layer: {}) which is not in allowed_deps[{}]",
                            from_part.name, from_layer, to_part.name, to_layer, from_layer,
                        ),
                        help: Some(format!(
                            "allowed dependencies for {}: [{}]",
                            from_layer, allowed_list
                        )),
                    });
                }
            }
        }
    }

    // LAYER003: Circular dependency
    if let Some(severity) = effective_severity("LAYER003", Severity::Error, config) {
        let sccs = petgraph::algo::tarjan_scc(&graph);
        for scc in &sccs {
            if scc.len() > 1 {
                // Found a cycle — report it
                let cycle_names: Vec<&str> = scc
                    .iter()
                    .map(|&node| parts[*graph.node_weight(node).unwrap()].name.as_str())
                    .collect();

                let first_node = scc[0];
                let first_part = &parts[*graph.node_weight(first_node).unwrap()];

                diagnostics.push(Diagnostic {
                    file: first_part.file.clone(),
                    line: first_part.line,
                    col: 1,
                    severity,
                    rule_id: "LAYER003".to_string(),
                    message: format!(
                        "circular dependency detected: {}",
                        cycle_names.join(" -> "),
                    ),
                    help: None,
                });
            }
        }
    }

    (diagnostics, parts_checked, connections_checked)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DomainConfig, LayerConfig, RequiredMetadataConfig};
    use std::path::PathBuf;
    use sysml_v2_adapter::SysmlWorkspace;

    fn two_layer_config() -> DomainConfig {
        let mut allowed_deps = HashMap::new();
        allowed_deps.insert("upper".to_string(), vec!["lower".to_string()]);
        allowed_deps.insert("lower".to_string(), Vec::new());

        DomainConfig {
            name: "test".to_string(),
            description: None,
            metadata_library: PathBuf::new(),
            layers: LayerConfig {
                order: vec!["upper".to_string(), "lower".to_string()],
                allowed_deps,
            },
            required_metadata: RequiredMetadataConfig {
                parts: Vec::new(),
            },
            type_map: HashMap::new(),
            validation_rules: HashMap::new(),
            template_dir: PathBuf::new(),
        }
    }

    fn firmware_config() -> DomainConfig {
        let domains_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("domains");
        DomainConfig::load(&domains_dir.join("firmware"), None).unwrap()
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
    fn test_layer_valid_connection() {
        let source = r#"
package Test {
    enum def LayerKind {
        enum upper;
        enum lower;
    }
    part def Server {
        attribute layer : LayerKind = LayerKind::lower;
    }
    part def App {
        attribute layer : LayerKind = LayerKind::upper;
        part s : Server;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = two_layer_config();
        let (diags, parts, conns) = check_layer_deps(&ws, &config);
        assert!(parts > 0);
        assert!(conns > 0);
        let layer001: Vec<_> = diags.iter().filter(|d| d.rule_id == "LAYER001").collect();
        assert!(
            layer001.is_empty(),
            "upper -> lower is allowed: {:?}",
            layer001
        );
    }

    #[test]
    fn test_layer_violation() {
        let source = r#"
package Test {
    enum def LayerKind {
        enum upper;
        enum lower;
    }
    part def App {
        attribute layer : LayerKind = LayerKind::upper;
    }
    part def Server {
        attribute layer : LayerKind = LayerKind::lower;
        part a : App;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = two_layer_config();
        let (diags, _, _) = check_layer_deps(&ws, &config);
        let layer001: Vec<_> = diags.iter().filter(|d| d.rule_id == "LAYER001").collect();
        assert!(
            !layer001.is_empty(),
            "lower -> upper should be a violation"
        );
    }

    #[test]
    fn test_layer_missing_attribute() {
        let source = r#"
package Test {
    part def NoLayer {
        attribute x : Integer;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = two_layer_config();
        let (diags, _, _) = check_layer_deps(&ws, &config);
        let layer002: Vec<_> = diags.iter().filter(|d| d.rule_id == "LAYER002").collect();
        assert!(
            !layer002.is_empty(),
            "should flag missing layer attribute"
        );
    }

    #[test]
    fn test_layer_cycle() {
        // A depends on B, B depends on A → cycle
        let source = r#"
package Test {
    enum def LayerKind {
        enum upper;
        enum lower;
    }
    part def A {
        attribute layer : LayerKind = LayerKind::upper;
        part b : B;
    }
    part def B {
        attribute layer : LayerKind = LayerKind::lower;
        part a : A;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = two_layer_config();
        let (diags, _, _) = check_layer_deps(&ws, &config);
        let layer003: Vec<_> = diags.iter().filter(|d| d.rule_id == "LAYER003").collect();
        assert!(
            !layer003.is_empty(),
            "should detect circular dependency A <-> B"
        );
    }

    #[test]
    fn test_layer_same_layer() {
        let source = r#"
package Test {
    enum def LayerKind {
        enum upper;
        enum lower;
    }
    part def Widget {
        attribute layer : LayerKind = LayerKind::lower;
    }
    part def Gadget {
        attribute layer : LayerKind = LayerKind::lower;
        part w : Widget;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = two_layer_config();
        let (diags, _, _) = check_layer_deps(&ws, &config);
        let layer004: Vec<_> = diags.iter().filter(|d| d.rule_id == "LAYER004").collect();
        assert!(
            !layer004.is_empty(),
            "should flag same-layer dependency"
        );
    }

    #[test]
    fn test_layer_no_config() {
        let source = r#"
package Test {
    part def Whatever { }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let mut config = two_layer_config();
        config.layers.order.clear();
        let (diags, parts, _) = check_layer_deps(&ws, &config);
        assert_eq!(parts, 0, "no layers configured → skip");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_firmware_fixtures_valid_layers() {
        // Real firmware fixtures: middleware → driver is allowed
        let lib = load_fixture("firmware_library.sysml");
        let ifaces = load_fixture("interfaces.sysml");
        let bt = load_fixture("bt_a2dp_sink.sysml");
        let audio = load_fixture("audio_pipeline.sysml");
        let i2s = load_fixture("i2s_output.sysml");
        let led = load_fixture("status_led.sysml");
        let ws = SysmlWorkspace::from_sources(vec![
            (PathBuf::from("firmware_library.sysml"), lib),
            (PathBuf::from("interfaces.sysml"), ifaces),
            (PathBuf::from("bt_a2dp_sink.sysml"), bt),
            (PathBuf::from("audio_pipeline.sysml"), audio),
            (PathBuf::from("i2s_output.sysml"), i2s),
            (PathBuf::from("status_led.sysml"), led),
        ]);
        let config = firmware_config();
        let (diags, parts, _) = check_layer_deps(&ws, &config);
        assert!(parts > 0, "should find parts");
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.rule_id == "LAYER001" && d.severity == Severity::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "firmware fixtures should have no layer violations: {:?}",
            errors
        );
    }
}
