//! Extraction engine: flattens adapter types into serializable output.
//!
//! Produces a generic tree of metadata key-value pairs without interpreting
//! domain-specific semantics. Outputs YAML or JSON.

use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use sysml_v2_adapter::connection_resolver::resolve_connections;
use sysml_v2_adapter::metadata_extractor::extract_metadata;
use sysml_v2_adapter::state_machine_extractor::extract_state_machines;
use sysml_v2_adapter::workspace::extract_definition_body;
use sysml_v2_adapter::{ConnectionKind, SysmlWorkspace, SymbolKind};

use crate::diagnostic::Severity;
use crate::domain::DomainConfig;
use crate::validation::ValidationResult;

mod flatten;
pub mod types;

pub use types::*;

use flatten::flatten_annotations;

/// Errors that can occur during extraction or serialization.
#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    #[error("extraction blocked: {0} validation error(s) must be resolved first")]
    ValidationFailed(usize),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML serialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Extract structured data from a validated workspace.
///
/// The validation gate requires no `Error`-severity diagnostics.
/// Warnings are permitted.
pub fn extract(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
    validation: &ValidationResult,
) -> Result<ExtractionResult, ExtractionError> {
    // Validation gate
    let error_count = validation
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    if error_count > 0 {
        return Err(ExtractionError::ValidationFailed(error_count));
    }

    let known_layers: HashSet<String> = config.layers.order.iter().cloned().collect();

    let mut modules = Vec::new();
    let mut source_files: HashSet<PathBuf> = HashSet::new();

    for (file, sym) in workspace.all_symbols() {
        if sym.kind != SymbolKind::PartDefinition {
            continue;
        }

        source_files.insert(file.path.clone());

        let annotations = extract_metadata(file, sym);
        let metadata = flatten_annotations(&annotations);
        let connections = resolve_connections(file, sym);
        let state_machines = extract_state_machines(file, sym);
        let layer = crate::util::extract_layer_for_part(file, sym, &known_layers);

        let extracted_connections: Vec<ExtractedConnection> = connections
            .iter()
            .map(|c| ExtractedConnection {
                name: c.name.clone(),
                kind: match c.kind {
                    ConnectionKind::Connect => "connect".to_string(),
                    ConnectionKind::Flow => "flow".to_string(),
                },
                source: c.source.clone(),
                target: c.target.clone(),
                flow_type: c.flow_type.clone(),
            })
            .collect();

        let extracted_fsms: Vec<ExtractedStateMachine> = state_machines
            .iter()
            .map(|fsm| ExtractedStateMachine {
                name: fsm.name.clone(),
                initial_state: fsm.initial_state.clone(),
                states: fsm.states.iter().map(|s| s.name.clone()).collect(),
                transitions: fsm
                    .transitions
                    .iter()
                    .map(|t| ExtractedTransition {
                        name: t.name.clone(),
                        from_state: t.from_state.clone(),
                        to_state: t.to_state.clone(),
                        event: t.event.clone(),
                        guard: t.guard.clone(),
                        action: t.action.clone(),
                    })
                    .collect(),
            })
            .collect();

        let ports = extract_ports(workspace, file, sym);
        let actions = extract_actions(workspace, file, sym);

        modules.push(ExtractedModule {
            name: sym.name.to_string(),
            qualified_name: sym.qualified_name.to_string(),
            source_file: file.path.clone(),
            layer,
            metadata,
            ports,
            actions,
            connections: extracted_connections,
            state_machines: extracted_fsms,
        });
    }

    // Sort modules by name for deterministic output
    modules.sort_by(|a, b| a.name.cmp(&b.name));

    // Build architecture summary
    let mut sorted_files: Vec<PathBuf> = source_files.into_iter().collect();
    sorted_files.sort();

    let module_summaries: Vec<ModuleSummary> = modules
        .iter()
        .map(|m| ModuleSummary {
            name: m.name.clone(),
            layer: m.layer.clone(),
            source_file: m.source_file.clone(),
        })
        .collect();

    let dependency_graph = build_dependency_graph(workspace);

    Ok(ExtractionResult {
        modules,
        architecture: ExtractedArchitecture {
            source_files: sorted_files,
            modules: module_summaries,
            dependency_graph,
        },
    })
}

/// Write extraction results to files in the given output directory.
///
/// Creates one file per module plus an `architecture` file.
/// Returns the paths of all files written.
pub fn write_extraction(
    result: &ExtractionResult,
    output_dir: &Path,
    format: OutputFormat,
) -> Result<Vec<PathBuf>, ExtractionError> {
    std::fs::create_dir_all(output_dir)?;
    let mut written = Vec::new();

    let ext = match format {
        OutputFormat::Yaml => "yaml",
        OutputFormat::Json => "json",
    };

    // Write each module
    for module in &result.modules {
        let path = output_dir.join(format!("{}.{}", module.name, ext));
        let contents = serialize(module, format)?;
        let mut f = std::fs::File::create(&path)?;
        f.write_all(contents.as_bytes())?;
        written.push(path);
    }

    // Write architecture summary
    let arch_path = output_dir.join(format!("architecture.{}", ext));
    let contents = serialize(&result.architecture, format)?;
    let mut f = std::fs::File::create(&arch_path)?;
    f.write_all(contents.as_bytes())?;
    written.push(arch_path);

    Ok(written)
}

fn serialize<T: serde::Serialize>(value: &T, format: OutputFormat) -> Result<String, ExtractionError> {
    match format {
        OutputFormat::Yaml => Ok(serde_yaml::to_string(value)?),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(value)?),
    }
}

/// Extract ports from a part definition.
fn extract_ports(
    workspace: &SysmlWorkspace,
    file: &sysml_v2_adapter::ParsedFile,
    part_symbol: &sysml_v2_adapter::HirSymbol,
) -> Vec<ExtractedPort> {
    let part_qn: &str = &part_symbol.qualified_name;

    // Find child PortUsage symbols
    let port_symbols: Vec<_> = workspace
        .all_symbols()
        .filter(|(f, s)| {
            f.path == file.path
                && s.kind == SymbolKind::PortUsage
                && s.qualified_name.starts_with(part_qn)
                && s.qualified_name.len() > part_qn.len()
        })
        .collect();

    // Get body text for conjugation detection
    let body = extract_definition_body(&file.source, part_symbol);

    port_symbols
        .iter()
        .map(|(_, sym)| {
            let name = sym.name.to_string();
            let port_type = sym.supertypes.first().map(|s| s.to_string());

            // Check conjugation: look for `port <name> : ~` in body text
            let conjugated = body
                .as_ref()
                .map(|b| {
                    // Pattern: `port <name> : ~` or `port <name> :~`
                    let pattern = format!("port {} :", name);
                    if let Some(pos) = b.find(&pattern) {
                        let after_colon = &b[pos + pattern.len()..];
                        let trimmed = after_colon.trim_start();
                        trimmed.starts_with('~')
                    } else {
                        false
                    }
                })
                .unwrap_or(false);

            ExtractedPort {
                name,
                port_type,
                conjugated,
            }
        })
        .collect()
}

/// Extract action definitions from a part definition's body text.
///
/// Actions appear as `action def Name { ... }` in SysML. The HIR doesn't
/// expose a dedicated `ActionDefinition` symbol kind, so we parse the body text.
fn extract_actions(
    _workspace: &SysmlWorkspace,
    file: &sysml_v2_adapter::ParsedFile,
    part_symbol: &sysml_v2_adapter::HirSymbol,
) -> Vec<ExtractedAction> {
    let Some(body) = extract_definition_body(&file.source, part_symbol) else {
        return Vec::new();
    };

    let mut actions = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("action def ") {
            // Extract name: everything before '{' or ';' or whitespace
            let name = rest
                .split(|c: char| c == '{' || c == ';' || c.is_whitespace())
                .next()
                .unwrap_or("")
                .trim();
            if !name.is_empty() {
                actions.push(ExtractedAction {
                    name: name.to_string(),
                });
            }
        }
    }
    actions
}

/// Build the dependency graph from PartUsage → PartDefinition relationships.
fn build_dependency_graph(workspace: &SysmlWorkspace) -> Vec<(String, String)> {
    // Collect all PartDefinition names
    let part_def_names: HashSet<String> = workspace
        .all_symbols()
        .filter(|(_, s)| s.kind == SymbolKind::PartDefinition)
        .map(|(_, s)| s.name.to_string())
        .collect();

    let mut edges = Vec::new();

    // For each PartDefinition, find child PartUsage symbols
    for (file, sym) in workspace.all_symbols() {
        if sym.kind != SymbolKind::PartDefinition {
            continue;
        }
        let container_name = sym.name.to_string();
        let container_qn: &str = &sym.qualified_name;

        for (f, child) in workspace.all_symbols() {
            if f.path != file.path || child.kind != SymbolKind::PartUsage {
                continue;
            }
            if !child.qualified_name.starts_with(container_qn)
                || child.qualified_name.len() <= container_qn.len()
            {
                continue;
            }
            // Check separator is `::`
            if child.qualified_name.as_bytes().get(container_qn.len()) != Some(&b':') {
                continue;
            }

            if let Some(type_name) = child.supertypes.first() {
                let type_str = type_name.to_string();
                if part_def_names.contains(&type_str) && type_str != container_name {
                    edges.push((container_name.clone(), type_str));
                }
            }
        }
    }

    edges.sort();
    edges.dedup();
    edges
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Diagnostic, Severity};
    use std::path::PathBuf;

    use crate::domain::DomainConfig;
    use crate::validation::{validate, ValidationResult};
    use sysml_v2_adapter::SysmlWorkspace;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("tests")
            .join("fixtures")
    }

    fn domains_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("domains")
    }

    fn load_fixture(name: &str) -> String {
        std::fs::read_to_string(fixtures_dir().join(name))
            .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", name, e))
    }

    const VALID_FIXTURES: &[&str] = &[
        "firmware_library.sysml",
        "interfaces.sysml",
        "bt_a2dp_sink.sysml",
        "audio_pipeline.sysml",
        "i2s_output.sysml",
        "status_led.sysml",
    ];

    fn load_valid_workspace() -> SysmlWorkspace {
        let sources: Vec<(PathBuf, String)> = VALID_FIXTURES
            .iter()
            .map(|name| (PathBuf::from(name), load_fixture(name)))
            .collect();
        SysmlWorkspace::from_sources(sources)
    }

    fn load_firmware_config() -> DomainConfig {
        DomainConfig::load(&domains_dir().join("firmware"), None).unwrap()
    }

    fn empty_validation() -> ValidationResult {
        ValidationResult {
            diagnostics: Vec::new(),
            parts_checked: 0,
            state_machines_checked: 0,
            connections_checked: 0,
        }
    }

    fn validation_with_error() -> ValidationResult {
        ValidationResult {
            diagnostics: vec![Diagnostic {
                file: PathBuf::from("test.sysml"),
                line: 1,
                col: 1,
                severity: Severity::Error,
                rule_id: "TEST001".to_string(),
                message: "test error".to_string(),
                help: None,
            }],
            parts_checked: 1,
            state_machines_checked: 0,
            connections_checked: 0,
        }
    }

    fn validation_with_warning() -> ValidationResult {
        ValidationResult {
            diagnostics: vec![Diagnostic {
                file: PathBuf::from("test.sysml"),
                line: 1,
                col: 1,
                severity: Severity::Warning,
                rule_id: "TEST002".to_string(),
                message: "test warning".to_string(),
                help: None,
            }],
            parts_checked: 1,
            state_machines_checked: 0,
            connections_checked: 0,
        }
    }

    #[test]
    fn test_extract_single_module() {
        let lib = load_fixture("firmware_library.sysml");
        let ifaces = load_fixture("interfaces.sysml");
        let src = load_fixture("bt_a2dp_sink.sysml");
        let ws = SysmlWorkspace::from_sources(vec![
            (PathBuf::from("firmware_library.sysml"), lib),
            (PathBuf::from("interfaces.sysml"), ifaces),
            (PathBuf::from("bt_a2dp_sink.sysml"), src),
        ]);
        let config = load_firmware_config();
        let validation = validate(&ws, &config);
        let result = extract(&ws, &config, &validation).unwrap();

        let bt = result
            .modules
            .iter()
            .find(|m| m.name == "BtA2dpSink")
            .expect("should extract BtA2dpSink");

        assert_eq!(bt.layer.as_deref(), Some("driver"));
        assert!(bt.metadata.contains_key("MemoryModel"));
        assert!(bt.metadata.contains_key("ConcurrencyModel"));
        assert!(!bt.state_machines.is_empty(), "should have state machines");
        assert!(!bt.ports.is_empty(), "should have ports");
        assert!(!bt.actions.is_empty(), "should have actions");
    }

    #[test]
    fn test_extract_workspace() {
        let ws = load_valid_workspace();
        let config = load_firmware_config();
        let validation = validate(&ws, &config);
        let result = extract(&ws, &config, &validation).unwrap();

        // 4 part definitions: BtA2dpSink, AudioPipeline, I2sOutput, StatusLed
        assert_eq!(
            result.modules.len(),
            4,
            "should extract 4 modules, got: {:?}",
            result.modules.iter().map(|m| &m.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_validation_gate_errors() {
        let ws = SysmlWorkspace::from_sources(vec![(
            PathBuf::from("test.sysml"),
            "package Test { part def A { } }".to_string(),
        )]);
        let config = load_firmware_config();
        let validation = validation_with_error();
        let result = extract(&ws, &config, &validation);
        assert!(result.is_err());
        match result.unwrap_err() {
            ExtractionError::ValidationFailed(count) => assert_eq!(count, 1),
            other => panic!("expected ValidationFailed, got: {other}"),
        }
    }

    #[test]
    fn test_validation_gate_warnings() {
        let ws = SysmlWorkspace::from_sources(vec![(
            PathBuf::from("test.sysml"),
            "package Test { part def A { } }".to_string(),
        )]);
        let config = load_firmware_config();
        let validation = validation_with_warning();
        let result = extract(&ws, &config, &validation);
        assert!(result.is_ok(), "warnings should not block extraction");
    }

    #[test]
    fn test_round_trip_yaml() {
        let ws = load_valid_workspace();
        let config = load_firmware_config();
        let validation = validate(&ws, &config);
        let result = extract(&ws, &config, &validation).unwrap();

        // Serialize to YAML and back
        for module in &result.modules {
            let yaml = serde_yaml::to_string(module).unwrap();
            let deserialized: ExtractedModule = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(module, &deserialized, "YAML round-trip failed for {}", module.name);
        }
    }

    #[test]
    fn test_round_trip_json() {
        let ws = load_valid_workspace();
        let config = load_firmware_config();
        let validation = validate(&ws, &config);
        let result = extract(&ws, &config, &validation).unwrap();

        for module in &result.modules {
            let json = serde_json::to_string(module).unwrap();
            let deserialized: ExtractedModule = serde_json::from_str(&json).unwrap();
            assert_eq!(module, &deserialized, "JSON round-trip failed for {}", module.name);
        }
    }

    #[test]
    fn test_determinism() {
        let ws = load_valid_workspace();
        let config = load_firmware_config();
        let validation = validate(&ws, &config);
        let result1 = extract(&ws, &config, &validation).unwrap();
        let result2 = extract(&ws, &config, &validation).unwrap();
        assert_eq!(result1, result2, "extraction should be deterministic");
    }

    #[test]
    fn test_module_without_metadata() {
        let source = r#"
package Test {
    part def Bare {
        attribute x : Integer;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let mut config = load_firmware_config();
        // Clear required metadata to avoid validation errors
        config.required_metadata.parts.clear();
        config.layers.order.clear();
        let validation = empty_validation();
        let result = extract(&ws, &config, &validation).unwrap();

        assert_eq!(result.modules.len(), 1);
        let bare = &result.modules[0];
        assert_eq!(bare.name, "Bare");
        assert!(bare.metadata.is_empty(), "should have empty metadata");
    }

    #[test]
    fn test_architecture_summary() {
        let ws = load_valid_workspace();
        let config = load_firmware_config();
        let validation = validate(&ws, &config);
        let result = extract(&ws, &config, &validation).unwrap();

        let arch = &result.architecture;
        assert!(!arch.source_files.is_empty());
        assert_eq!(arch.modules.len(), 4);
        // AudioPipeline depends on BtA2dpSink, I2sOutput, StatusLed
        assert!(
            !arch.dependency_graph.is_empty(),
            "should have dependency edges"
        );
        assert!(
            arch.dependency_graph
                .iter()
                .any(|(from, to)| from == "AudioPipeline" && to == "BtA2dpSink"),
            "AudioPipeline should depend on BtA2dpSink: {:?}",
            arch.dependency_graph
        );
    }

    #[test]
    fn test_write_yaml() {
        let ws = load_valid_workspace();
        let config = load_firmware_config();
        let validation = validate(&ws, &config);
        let result = extract(&ws, &config, &validation).unwrap();

        let tmp = std::env::temp_dir().join(format!("sysml-extract-yaml-{}", std::process::id()));
        let written = write_extraction(&result, &tmp, OutputFormat::Yaml).unwrap();

        // Should have 4 module files + 1 architecture file
        assert_eq!(written.len(), 5, "should write 5 files: {:?}", written);
        for path in &written {
            assert!(path.exists(), "file should exist: {}", path.display());
            assert_eq!(path.extension().unwrap(), "yaml");
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_write_json() {
        let ws = load_valid_workspace();
        let config = load_firmware_config();
        let validation = validate(&ws, &config);
        let result = extract(&ws, &config, &validation).unwrap();

        let tmp = std::env::temp_dir().join(format!("sysml-extract-json-{}", std::process::id()));
        let written = write_extraction(&result, &tmp, OutputFormat::Json).unwrap();

        assert_eq!(written.len(), 5);
        for path in &written {
            assert_eq!(path.extension().unwrap(), "json");
        }

        // Verify JSON is valid by parsing one file
        let bt_path = written.iter().find(|p| p.file_name().unwrap().to_str().unwrap().starts_with("BtA2dpSink")).unwrap();
        let json_str = std::fs::read_to_string(bt_path).unwrap();
        let _: ExtractedModule = serde_json::from_str(&json_str).unwrap();

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
