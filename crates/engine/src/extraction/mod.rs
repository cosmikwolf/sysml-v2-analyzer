//! Extraction engine: flattens adapter types into serializable output.
//!
//! Produces a generic tree of metadata key-value pairs without interpreting
//! domain-specific semantics. Outputs YAML or JSON.

use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use sysml_v2_adapter::connection_resolver::resolve_connections;
use sysml_v2_adapter::metadata_extractor::{extract_all_metadata, extract_metadata};
use sysml_v2_adapter::state_machine_extractor::extract_state_machines;
use sysml_v2_adapter::workspace::extract_definition_body;
use sysml_v2_adapter::{
    ConnectionKind, MetadataAnnotation, MetadataValue, SysmlWorkspace, SymbolKind,
};

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
    Yaml(#[from] serde_yml::Error),

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

    let ui = extract_ui(workspace);

    Ok(ExtractionResult {
        modules,
        architecture: ExtractedArchitecture {
            source_files: sorted_files,
            modules: module_summaries,
            dependency_graph,
        },
        ui,
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
        OutputFormat::Yaml => Ok(serde_yml::to_string(value)?),
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
/// Parses `in`/`out` parameters from the action body block.
fn extract_actions(
    _workspace: &SysmlWorkspace,
    file: &sysml_v2_adapter::ParsedFile,
    part_symbol: &sysml_v2_adapter::HirSymbol,
) -> Vec<ExtractedAction> {
    let Some(body) = extract_definition_body(&file.source, part_symbol) else {
        return Vec::new();
    };

    let mut actions = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if let Some(rest) = trimmed.strip_prefix("action def ") {
            // Extract name: everything before '{' or ';' or whitespace
            let name = rest
                .split(|c: char| c == '{' || c == ';' || c.is_whitespace())
                .next()
                .unwrap_or("")
                .trim()
                .to_string();

            if name.is_empty() {
                i += 1;
                continue;
            }

            // Check if this action has a body block (contains '{')
            let mut parameters = Vec::new();
            if trimmed.contains('{') {
                // Parse parameters from subsequent lines until '}'
                let start = i + 1;
                let mut j = start;
                while j < lines.len() {
                    let param_line = lines[j].trim();
                    if param_line.contains('}') {
                        break;
                    }
                    if let Some(param) = parse_action_parameter(param_line) {
                        parameters.push(param);
                    }
                    j += 1;
                }
                i = j + 1;
            } else {
                i += 1;
            }

            actions.push(ExtractedAction { name, parameters });
        } else {
            i += 1;
        }
    }
    actions
}

/// Parse a single action parameter line like `in config : A2dpConfig;`
fn parse_action_parameter(line: &str) -> Option<ActionParameter> {
    // Strip inline comments
    let without_comment = if let Some(pos) = line.find("//") {
        &line[..pos]
    } else {
        line
    };
    let trimmed = without_comment.trim().trim_end_matches(';').trim();

    let (direction, rest) = if let Some(rest) = trimmed.strip_prefix("in ") {
        (ParameterDirection::In, rest.trim())
    } else if let Some(rest) = trimmed.strip_prefix("out ") {
        (ParameterDirection::Out, rest.trim())
    } else {
        return None;
    };

    // Split on ':' to get name and type
    let mut parts = rest.splitn(2, ':');
    let name = parts.next()?.trim().to_string();
    let type_name = parts.next()?.trim().to_string();

    if name.is_empty() || type_name.is_empty() {
        return None;
    }

    Some(ActionParameter {
        name,
        type_name,
        direction,
    })
}

// ── UI metadata helpers ─────────────────────────────────────────────

/// Extract a string field value from a metadata annotation.
fn ui_get_string(annotation: &MetadataAnnotation, field_name: &str) -> Option<String> {
    annotation.fields.iter().find_map(|f| {
        if f.name == field_name {
            match &f.value {
                MetadataValue::String(s) => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        }
    })
}

/// Extract an integer field value from a metadata annotation.
fn ui_get_integer(annotation: &MetadataAnnotation, field_name: &str) -> Option<i64> {
    annotation.fields.iter().find_map(|f| {
        if f.name == field_name {
            match &f.value {
                MetadataValue::Integer(n) => Some(*n),
                _ => None,
            }
        } else {
            None
        }
    })
}

/// Extract a boolean field value from a metadata annotation.
fn ui_get_bool(annotation: &MetadataAnnotation, field_name: &str) -> Option<bool> {
    annotation.fields.iter().find_map(|f| {
        if f.name == field_name {
            match &f.value {
                MetadataValue::Boolean(b) => Some(*b),
                _ => None,
            }
        } else {
            None
        }
    })
}

/// Extract the variant name from an enum reference field.
fn ui_get_enum_variant(annotation: &MetadataAnnotation, field_name: &str) -> Option<String> {
    annotation.fields.iter().find_map(|f| {
        if f.name == field_name {
            match &f.value {
                MetadataValue::EnumRef { variant, .. } => Some(variant.clone()),
                _ => None,
            }
        } else {
            None
        }
    })
}

/// Extract string values from a tuple field.
fn ui_get_tuple_strings(annotation: &MetadataAnnotation, field_name: &str) -> Vec<String> {
    annotation
        .fields
        .iter()
        .find_map(|f| {
            if f.name == field_name {
                match &f.value {
                    MetadataValue::Tuple(values) => Some(
                        values
                            .iter()
                            .filter_map(|v| match v {
                                MetadataValue::String(s) => Some(s.clone()),
                                _ => None,
                            })
                            .collect(),
                    ),
                    _ => None,
                }
            } else {
                None
            }
        })
        .unwrap_or_default()
}

// ── UI extraction ───────────────────────────────────────────────────

/// Extract UI specification from the workspace.
///
/// Scans all parts for UI metadata annotations (@DisplayHardware, @InputDevice,
/// @LedHardware, @Gesture, @FontAsset, @IconAsset, @Screen, @IndicatorBinding,
/// @Navigation) and builds an `ExtractedUI` structure.
///
/// Returns `None` if no UI-related parts are found.
fn extract_ui(workspace: &SysmlWorkspace) -> Option<ExtractedUI> {
    let mut displays = Vec::new();
    let mut input_devices = Vec::new();
    let mut leds = Vec::new();
    let mut gestures = Vec::new();
    let mut fonts = Vec::new();
    let mut icons = Vec::new();
    let mut screens = Vec::new();
    let mut indicators = Vec::new();
    let mut timing_defaults: Option<ExtractedTimingDefaults> = None;
    let mut navigation: Option<ExtractedNavigation> = None;
    let mut found_any = false;

    // Phase 1: Scan all PartDefinition symbols for UI metadata.
    for (file, sym) in workspace.all_symbols() {
        if sym.kind != SymbolKind::PartDefinition {
            continue;
        }

        let annotations = extract_metadata(file, sym);

        for ann in &annotations {
            match ann.name.as_str() {
                "DisplayHardware" => {
                    found_any = true;
                    displays.push(ExtractedDisplay {
                        name: sym.name.to_string(),
                        display_type: ui_get_enum_variant(ann, "type"),
                        driver: ui_get_string(ann, "driver"),
                        width: ui_get_integer(ann, "width").unwrap_or(0) as u32,
                        height: ui_get_integer(ann, "height").unwrap_or(0) as u32,
                        color_depth: ui_get_enum_variant(ann, "colorDepth"),
                        interface: ui_get_enum_variant(ann, "interface"),
                        orientation: ui_get_string(ann, "orientation"),
                        module: ui_get_string(ann, "module"),
                    });
                }
                "InputDevice" => {
                    found_any = true;
                    input_devices.push(ExtractedInputDevice {
                        name: sym.name.to_string(),
                        input_type: ui_get_enum_variant(ann, "type"),
                        active: ui_get_enum_variant(ann, "active"),
                        has_button: ui_get_bool(ann, "hasButton").unwrap_or(false),
                        detents: ui_get_integer(ann, "detents").map(|n| n as u32),
                        module: ui_get_string(ann, "module"),
                    });
                }
                "LedHardware" => {
                    found_any = true;
                    leds.push(ExtractedLed {
                        name: sym.name.to_string(),
                        led_type: ui_get_enum_variant(ann, "type"),
                        colors: ui_get_tuple_strings(ann, "colors"),
                        module: ui_get_string(ann, "module"),
                    });
                }
                "Gesture" => {
                    found_any = true;
                    gestures.push(ExtractedGesture {
                        name: sym.name.to_string(),
                        device: ui_get_string(ann, "device"),
                        trigger: ui_get_enum_variant(ann, "trigger"),
                        window_ms: ui_get_integer(ann, "window_ms").map(|n| n as u32),
                    });
                }
                "FontAsset" => {
                    found_any = true;
                    fonts.push(ExtractedFont {
                        name: sym.name.to_string(),
                        family: ui_get_string(ann, "family"),
                        size: ui_get_integer(ann, "size").unwrap_or(0) as u32,
                        source: ui_get_enum_variant(ann, "source"),
                        file: ui_get_string(ann, "file")
                            .filter(|s| !s.is_empty()),
                    });
                }
                "IconAsset" => {
                    found_any = true;
                    icons.push(ExtractedIcon {
                        name: sym.name.to_string(),
                        width: ui_get_integer(ann, "width").unwrap_or(0) as u32,
                        height: ui_get_integer(ann, "height").unwrap_or(0) as u32,
                        source: ui_get_enum_variant(ann, "source"),
                        format: ui_get_enum_variant(ann, "format"),
                        file: ui_get_string(ann, "file")
                            .filter(|s| !s.is_empty()),
                    });
                }
                "Screen" => {
                    found_any = true;
                    let display = ui_get_string(ann, "display");
                    let refresh_mode = ui_get_enum_variant(ann, "refreshMode");
                    let poll_interval_ms =
                        ui_get_integer(ann, "pollInterval_ms").map(|n| n as u32);

                    // Collect @Element annotations in this part's body
                    let elements = extract_ui_elements(&annotations);

                    screens.push(ExtractedScreen {
                        name: sym.name.to_string(),
                        display,
                        refresh_mode,
                        poll_interval_ms,
                        elements,
                    });
                }
                "IndicatorBinding" => {
                    found_any = true;
                    let led = ui_get_string(ann, "led");
                    let module = ui_get_string(ann, "module");
                    let field = ui_get_string(ann, "field");

                    // Collect @IndicatorState annotations in this part's body
                    let states: Vec<ExtractedIndicatorState> = annotations
                        .iter()
                        .filter(|a| a.name == "IndicatorState")
                        .map(|a| ExtractedIndicatorState {
                            name: ui_get_string(a, "name"),
                            color: ui_get_string(a, "color"),
                            pattern: ui_get_enum_variant(a, "pattern"),
                            period_ms: ui_get_integer(a, "period_ms").map(|n| n as u32),
                            duty_percent: ui_get_integer(a, "duty_percent").map(|n| n as u32),
                        })
                        .collect();

                    indicators.push(ExtractedIndicator {
                        name: sym.name.to_string(),
                        led,
                        module,
                        field,
                        states,
                    });
                }
                _ => {}
            }
        }
    }

    // Phase 2: Scan for package-level @GestureTimingDefaults.
    for (file, _) in workspace.all_symbols() {
        let all_annotations = extract_all_metadata(file);
        for ann in &all_annotations {
            if ann.name == "GestureTimingDefaults" {
                found_any = true;
                timing_defaults = Some(ExtractedTimingDefaults {
                    debounce_ms: ui_get_integer(ann, "debounce_ms").unwrap_or(0) as u32,
                    long_press_ms: ui_get_integer(ann, "long_press_ms").unwrap_or(0) as u32,
                    double_tap_ms: ui_get_integer(ann, "double_tap_ms").unwrap_or(0) as u32,
                    combo_window_ms: ui_get_integer(ann, "combo_window_ms").unwrap_or(0) as u32,
                    sequence_timeout_ms: ui_get_integer(ann, "sequence_timeout_ms").unwrap_or(0)
                        as u32,
                });
                break;
            }
        }
        if timing_defaults.is_some() {
            break;
        }
    }

    // Phase 3: Scan state machines for @Navigation metadata.
    for (file, sym) in workspace.all_symbols() {
        if sym.kind != SymbolKind::PartDefinition {
            continue;
        }

        let annotations = extract_metadata(file, sym);
        let has_navigation = annotations.iter().any(|a| a.name == "Navigation");

        if has_navigation {
            found_any = true;
            // Extract state machines from this part and use them for navigation
            let state_machines = extract_state_machines(file, sym);

            if let Some(fsm) = state_machines.first() {
                let nav_screens: Vec<String> =
                    fsm.states.iter().map(|s| s.name.clone()).collect();
                let nav_transitions: Vec<ExtractedTransition> = fsm
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
                    .collect();

                navigation = Some(ExtractedNavigation {
                    initial_screen: fsm.initial_state.clone(),
                    screens: nav_screens,
                    transitions: nav_transitions,
                });
            }
        }
    }

    if !found_any {
        return None;
    }

    // Sort collections for deterministic output.
    displays.sort_by(|a, b| a.name.cmp(&b.name));
    input_devices.sort_by(|a, b| a.name.cmp(&b.name));
    leds.sort_by(|a, b| a.name.cmp(&b.name));
    gestures.sort_by(|a, b| a.name.cmp(&b.name));
    fonts.sort_by(|a, b| a.name.cmp(&b.name));
    icons.sort_by(|a, b| a.name.cmp(&b.name));
    screens.sort_by(|a, b| a.name.cmp(&b.name));
    indicators.sort_by(|a, b| a.name.cmp(&b.name));

    Some(ExtractedUI {
        displays,
        input_devices,
        leds,
        gestures,
        timing_defaults,
        fonts,
        icons,
        screens,
        indicators,
        navigation,
    })
}

/// Extract @Element annotations from a part's metadata list and convert to ExtractedElements.
fn extract_ui_elements(annotations: &[MetadataAnnotation]) -> Vec<ExtractedElement> {
    annotations
        .iter()
        .filter(|a| a.name == "Element")
        .enumerate()
        .map(|(idx, ann)| {
            let visible_module = ui_get_string(ann, "visible_module");
            let visible_field = ui_get_string(ann, "visible_field");
            let visible_op = ui_get_string(ann, "visible_op");
            let visible_value = ui_get_string(ann, "visible_value");

            let visible_when = if visible_module.is_some() || visible_field.is_some() {
                Some(ExtractedVisibility {
                    module: visible_module,
                    field: visible_field,
                    op: visible_op,
                    value: visible_value,
                })
            } else {
                None
            };

            ExtractedElement {
                id: format!("element_{}", idx),
                element_type: ui_get_enum_variant(ann, "type"),
                x: ui_get_integer(ann, "x").unwrap_or(0) as i32,
                y: ui_get_integer(ann, "y").unwrap_or(0) as i32,
                width: ui_get_integer(ann, "width").unwrap_or(0) as u32,
                height: ui_get_integer(ann, "height").unwrap_or(0) as u32,
                font: ui_get_string(ann, "font"),
                icon: ui_get_string(ann, "icon"),
                align: ui_get_enum_variant(ann, "align"),
                scroll: ui_get_bool(ann, "scroll").unwrap_or(false),
                truncate: ui_get_bool(ann, "truncate").unwrap_or(false),
                binding_module: ui_get_string(ann, "binding_module"),
                binding_field: ui_get_string(ann, "binding_field"),
                range_min: ui_get_integer(ann, "range_min").map(|n| n as i32),
                range_max: ui_get_integer(ann, "range_max").map(|n| n as i32),
                visible_when,
            }
        })
        .collect()
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
            ui_elements_checked: 0,
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
            ui_elements_checked: 0,
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
            ui_elements_checked: 0,
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
            let yaml = serde_yml::to_string(module).unwrap();
            let deserialized: ExtractedModule = serde_yml::from_str(&yaml).unwrap();
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

    #[test]
    fn test_extract_action_params() {
        let ws = load_valid_workspace();
        let config = load_firmware_config();
        let validation = validate(&ws, &config);
        let result = extract(&ws, &config, &validation).unwrap();

        let status_led = result
            .modules
            .iter()
            .find(|m| m.name == "StatusLed")
            .expect("should extract StatusLed");

        let new_action = status_led
            .actions
            .iter()
            .find(|a| a.name == "New")
            .expect("should have New action");

        // New has: in dataPin : ScalarValues, in clockPin : ScalarValues, out led : StatusLed
        assert!(
            new_action.parameters.len() >= 3,
            "New should have at least 3 params, got: {:?}",
            new_action.parameters
        );

        let data_pin = new_action
            .parameters
            .iter()
            .find(|p| p.name == "dataPin")
            .expect("should have dataPin param");
        assert_eq!(data_pin.direction, ParameterDirection::In);
        assert_eq!(data_pin.type_name, "ScalarValues");

        let led_out = new_action
            .parameters
            .iter()
            .find(|p| p.name == "led")
            .expect("should have led param");
        assert_eq!(led_out.direction, ParameterDirection::Out);
        assert_eq!(led_out.type_name, "StatusLed");
    }

    #[test]
    fn test_parse_action_parameter_line() {
        let param = super::parse_action_parameter("in config : A2dpConfig;").unwrap();
        assert_eq!(param.name, "config");
        assert_eq!(param.type_name, "A2dpConfig");
        assert_eq!(param.direction, ParameterDirection::In);

        let param = super::parse_action_parameter("out result : BtA2dpSink;").unwrap();
        assert_eq!(param.name, "result");
        assert_eq!(param.type_name, "BtA2dpSink");
        assert_eq!(param.direction, ParameterDirection::Out);

        assert!(super::parse_action_parameter("attribute x : Integer;").is_none());
        assert!(super::parse_action_parameter("").is_none());
    }
}
