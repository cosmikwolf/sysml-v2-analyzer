//! UI well-formedness validation.
//!
//! - UI001: Element bounds exceed display resolution
//! - UI002: Element overlap without visible_when guard
//! - UI003: Font/icon asset reference not found
//! - UI004: Data binding references non-existent module
//! - UI005: Indicator color not achievable by LED hardware
//! - UI006: Gesture device reference not found
//! - UI007: Screen display reference not found
//! - UI008: Indicator LED reference not found

use std::collections::HashMap;

use sysml_v2_adapter::metadata_extractor::extract_metadata;
use sysml_v2_adapter::{MetadataAnnotation, MetadataValue, SysmlWorkspace, SymbolKind};

use crate::diagnostic::{Diagnostic, Severity};
use crate::domain::DomainConfig;

use super::{effective_severity, to_display_line};

// ── Helper functions ────────────────────────────────────────────────

/// Extract a string field value from a metadata annotation.
fn get_string_field(annotation: &MetadataAnnotation, field_name: &str) -> Option<String> {
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
fn get_integer_field(annotation: &MetadataAnnotation, field_name: &str) -> Option<i64> {
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

/// Extract the variant name from an enum reference field.
fn get_enum_variant(annotation: &MetadataAnnotation, field_name: &str) -> Option<String> {
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
fn get_tuple_strings(annotation: &MetadataAnnotation, field_name: &str) -> Vec<String> {
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

// ── Index types ─────────────────────────────────────────────────────

/// Display hardware info: (width, height).
struct DisplayInfo {
    width: i64,
    height: i64,
}

/// LED hardware info: type and available colors.
struct LedInfo {
    led_type: String,
    colors: Vec<String>,
}

/// Bounding box for an element on a screen.
struct ElementBox {
    x: i64,
    y: i64,
    width: i64,
    height: i64,
    has_visible_guard: bool,
    font: Option<String>,
    icon: Option<String>,
    binding_module: Option<String>,
}

// ── Main entry point ────────────────────────────────────────────────

/// Check all UI metadata in the workspace for well-formedness.
///
/// Returns diagnostics and the number of UI elements checked.
pub(crate) fn check_ui_wellformedness(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
) -> (Vec<Diagnostic>, usize) {
    let mut diagnostics = Vec::new();
    let mut ui_elements_checked: usize = 0;

    // Phase 1: Build lookup indexes by scanning all parts for UI metadata.
    let mut displays: HashMap<String, DisplayInfo> = HashMap::new();
    let mut inputs: HashMap<String, String> = HashMap::new(); // name → type
    let mut leds: HashMap<String, LedInfo> = HashMap::new();
    let mut fonts: HashMap<String, bool> = HashMap::new(); // name → exists
    let mut icons: HashMap<String, bool> = HashMap::new(); // name → exists
    let mut part_names: Vec<String> = Vec::new();

    for (file, sym) in workspace.all_symbols() {
        if sym.kind != SymbolKind::PartDefinition {
            continue;
        }
        part_names.push(sym.name.to_string());

        let annotations = extract_metadata(file, sym);
        for ann in &annotations {
            match ann.name.as_str() {
                "DisplayHardware" => {
                    if let (Some(w), Some(h)) =
                        (get_integer_field(ann, "width"), get_integer_field(ann, "height"))
                    {
                        displays.insert(sym.name.to_string(), DisplayInfo { width: w, height: h });
                    }
                }
                "InputDevice" => {
                    let input_type =
                        get_enum_variant(ann, "type").unwrap_or_else(|| "unknown".to_string());
                    inputs.insert(sym.name.to_string(), input_type);
                }
                "LedHardware" => {
                    let led_type =
                        get_enum_variant(ann, "type").unwrap_or_else(|| "unknown".to_string());
                    let colors = get_tuple_strings(ann, "colors");
                    leds.insert(sym.name.to_string(), LedInfo { led_type, colors });
                }
                "FontAsset" => {
                    fonts.insert(sym.name.to_string(), true);
                }
                "IconAsset" => {
                    icons.insert(sym.name.to_string(), true);
                }
                _ => {}
            }
        }
    }

    // Early return if no UI-related parts found at all.
    if displays.is_empty() && inputs.is_empty() && leds.is_empty() {
        return (diagnostics, 0);
    }

    // Phase 2: Validate UI rules by scanning parts again.
    for (file, sym) in workspace.all_symbols() {
        if sym.kind != SymbolKind::PartDefinition {
            continue;
        }

        let annotations = extract_metadata(file, sym);
        let file_path = &file.path;
        let line = to_display_line(sym.start_line);

        // Collect screen-level info.
        let screen_ann = annotations.iter().find(|a| a.name == "Screen");

        if let Some(screen) = screen_ann {
            ui_elements_checked += 1;
            let display_name = get_string_field(screen, "display");

            // UI007: Screen display reference not found.
            if let Some(ref dname) = display_name {
                check_screen_display_ref(dname, &displays, file_path, line, config, &mut diagnostics);
            }

            // Collect all @Element annotations in this screen's body.
            let elements: Vec<ElementBox> = annotations
                .iter()
                .filter(|a| a.name == "Element")
                .map(|a| {
                    ui_elements_checked += 1;
                    ElementBox {
                        x: get_integer_field(a, "x").unwrap_or(0),
                        y: get_integer_field(a, "y").unwrap_or(0),
                        width: get_integer_field(a, "width").unwrap_or(0),
                        height: get_integer_field(a, "height").unwrap_or(0),
                        has_visible_guard: get_string_field(a, "visible_module").is_some(),
                        font: get_string_field(a, "font"),
                        icon: get_string_field(a, "icon"),
                        binding_module: get_string_field(a, "binding_module"),
                    }
                })
                .collect();

            // UI001: Element bounds exceed display resolution.
            if let Some(ref dname) = display_name {
                if let Some(display) = displays.get(dname) {
                    check_element_bounds(
                        &elements, display, &sym.name, file_path, line, config, &mut diagnostics,
                    );
                }
            }

            // UI002: Element overlap without visible_when guard.
            check_element_overlap(&elements, &sym.name, file_path, line, config, &mut diagnostics);

            // UI003: Font/icon asset reference not found.
            check_asset_refs(
                &elements, &fonts, &icons, &sym.name, file_path, line, config, &mut diagnostics,
            );

            // UI004: Data binding references non-existent module.
            check_data_bindings(
                &elements, &part_names, &sym.name, file_path, line, config, &mut diagnostics,
            );
        }

        // UI006: Gesture device reference not found.
        for ann in annotations.iter().filter(|a| a.name == "Gesture") {
            ui_elements_checked += 1;
            check_gesture_device(ann, &inputs, &sym.name, file_path, line, config, &mut diagnostics);
        }

        // UI004 (IndicatorBinding variant) & UI005 & UI008.
        for ann in annotations.iter().filter(|a| a.name == "IndicatorBinding") {
            ui_elements_checked += 1;

            // UI008: Indicator LED reference not found.
            if let Some(led_name) = get_string_field(ann, "led") {
                check_indicator_led_ref(
                    &led_name, &leds, &sym.name, file_path, line, config, &mut diagnostics,
                );
            }

            // UI004: Binding module reference.
            if let Some(module_name) = get_string_field(ann, "module") {
                check_binding_module_ref(
                    &module_name,
                    &part_names,
                    &sym.name,
                    file_path,
                    line,
                    config,
                    &mut diagnostics,
                );
            }
        }

        // UI005: Indicator color not achievable by LED hardware.
        // Check state machines that have @IndicatorBinding — their states
        // should have colors achievable by the referenced LED.
        let indicator_binding = annotations.iter().find(|a| a.name == "IndicatorBinding");
        if let Some(binding) = indicator_binding {
            if let Some(led_name) = get_string_field(binding, "led") {
                if let Some(led_info) = leds.get(&led_name) {
                    // Check @IndicatorState annotations on states.
                    for state_ann in annotations.iter().filter(|a| a.name == "IndicatorState") {
                        check_indicator_color(
                            state_ann, led_info, &led_name, &sym.name, file_path, line, config,
                            &mut diagnostics,
                        );
                    }
                }
            }
        }
    }

    (diagnostics, ui_elements_checked)
}

// ── Individual rule checks ──────────────────────────────────────────

/// UI001: Element bounds exceed display resolution.
fn check_element_bounds(
    elements: &[ElementBox],
    display: &DisplayInfo,
    screen_name: &str,
    file: &std::path::Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("UI001", Severity::Error, config) else {
        return;
    };

    for elem in elements {
        let right = elem.x + elem.width;
        let bottom = elem.y + elem.height;

        if right > display.width || bottom > display.height {
            diagnostics.push(Diagnostic {
                file: file.to_path_buf(),
                line,
                col: 1,
                severity,
                rule_id: "UI001".to_string(),
                message: format!(
                    "element at ({},{}) with size {}x{} exceeds display resolution {}x{} in screen '{}'",
                    elem.x, elem.y, elem.width, elem.height,
                    display.width, display.height, screen_name,
                ),
                help: Some(format!(
                    "element extends to ({},{}) but display is {}x{}",
                    right, bottom, display.width, display.height,
                )),
            });
        }
    }
}

/// UI002: Element overlap without visible_when guard.
fn check_element_overlap(
    elements: &[ElementBox],
    screen_name: &str,
    file: &std::path::Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("UI002", Severity::Warning, config) else {
        return;
    };

    for i in 0..elements.len() {
        for j in (i + 1)..elements.len() {
            let a = &elements[i];
            let b = &elements[j];

            // Check axis-aligned bounding box intersection.
            let overlaps = a.x < b.x + b.width
                && a.x + a.width > b.x
                && a.y < b.y + b.height
                && a.y + a.height > b.y;

            if overlaps && !a.has_visible_guard && !b.has_visible_guard {
                diagnostics.push(Diagnostic {
                    file: file.to_path_buf(),
                    line,
                    col: 1,
                    severity,
                    rule_id: "UI002".to_string(),
                    message: format!(
                        "elements at ({},{}) and ({},{}) overlap in screen '{}' without visible_when guards",
                        a.x, a.y, b.x, b.y, screen_name,
                    ),
                    help: Some(
                        "add visible_module/visible_field to at least one element to prevent overlap"
                            .to_string(),
                    ),
                });
            }
        }
    }
}

/// UI003: Font/icon asset reference not found.
#[allow(clippy::too_many_arguments)]
fn check_asset_refs(
    elements: &[ElementBox],
    fonts: &HashMap<String, bool>,
    icons: &HashMap<String, bool>,
    screen_name: &str,
    file: &std::path::Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("UI003", Severity::Error, config) else {
        return;
    };

    for elem in elements {
        if let Some(ref font_name) = elem.font {
            if !fonts.contains_key(font_name) {
                diagnostics.push(Diagnostic {
                    file: file.to_path_buf(),
                    line,
                    col: 1,
                    severity,
                    rule_id: "UI003".to_string(),
                    message: format!(
                        "font '{}' referenced in screen '{}' is not defined as a FontAsset",
                        font_name, screen_name,
                    ),
                    help: Some(
                        "define a part with @FontAsset metadata for this font".to_string(),
                    ),
                });
            }
        }
        if let Some(ref icon_name) = elem.icon {
            if !icons.contains_key(icon_name) {
                diagnostics.push(Diagnostic {
                    file: file.to_path_buf(),
                    line,
                    col: 1,
                    severity,
                    rule_id: "UI003".to_string(),
                    message: format!(
                        "icon '{}' referenced in screen '{}' is not defined as an IconAsset",
                        icon_name, screen_name,
                    ),
                    help: Some(
                        "define a part with @IconAsset metadata for this icon".to_string(),
                    ),
                });
            }
        }
    }
}

/// UI004: Data binding references non-existent module.
fn check_data_bindings(
    elements: &[ElementBox],
    part_names: &[String],
    screen_name: &str,
    file: &std::path::Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("UI004", Severity::Error, config) else {
        return;
    };

    for elem in elements {
        if let Some(ref module) = elem.binding_module {
            if !part_names.iter().any(|p| p == module) {
                diagnostics.push(Diagnostic {
                    file: file.to_path_buf(),
                    line,
                    col: 1,
                    severity,
                    rule_id: "UI004".to_string(),
                    message: format!(
                        "binding_module '{}' in screen '{}' does not match any part definition",
                        module, screen_name,
                    ),
                    help: None,
                });
            }
        }
    }
}

/// UI004 variant for IndicatorBinding module field.
fn check_binding_module_ref(
    module_name: &str,
    part_names: &[String],
    part_name: &str,
    file: &std::path::Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("UI004", Severity::Error, config) else {
        return;
    };

    if !part_names.iter().any(|p| p == module_name) {
        diagnostics.push(Diagnostic {
            file: file.to_path_buf(),
            line,
            col: 1,
            severity,
            rule_id: "UI004".to_string(),
            message: format!(
                "IndicatorBinding module '{}' in '{}' does not match any part definition",
                module_name, part_name,
            ),
            help: None,
        });
    }
}

/// UI005: Indicator color not achievable by LED hardware.
#[allow(clippy::too_many_arguments)]
fn check_indicator_color(
    state_ann: &MetadataAnnotation,
    led_info: &LedInfo,
    led_name: &str,
    part_name: &str,
    file: &std::path::Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("UI005", Severity::Error, config) else {
        return;
    };

    if let Some(color) = get_string_field(state_ann, "color") {
        // RGB LEDs can produce any color.
        if led_info.led_type == "rgb" {
            return;
        }
        if !led_info.colors.iter().any(|c| c == &color) {
            diagnostics.push(Diagnostic {
                file: file.to_path_buf(),
                line,
                col: 1,
                severity,
                rule_id: "UI005".to_string(),
                message: format!(
                    "indicator color '{}' in '{}' is not achievable by LED '{}' (type: {}, colors: {:?})",
                    color, part_name, led_name, led_info.led_type, led_info.colors,
                ),
                help: Some(format!(
                    "LED '{}' supports colors: {:?}",
                    led_name, led_info.colors,
                )),
            });
        }
    }
}

/// UI006: Gesture device reference not found.
fn check_gesture_device(
    gesture_ann: &MetadataAnnotation,
    inputs: &HashMap<String, String>,
    part_name: &str,
    file: &std::path::Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("UI006", Severity::Error, config) else {
        return;
    };

    if let Some(device) = get_string_field(gesture_ann, "device") {
        if !inputs.contains_key(&device) {
            diagnostics.push(Diagnostic {
                file: file.to_path_buf(),
                line,
                col: 1,
                severity,
                rule_id: "UI006".to_string(),
                message: format!(
                    "gesture device '{}' in '{}' does not match any InputDevice part",
                    device, part_name,
                ),
                help: Some(format!(
                    "defined input devices: {:?}",
                    inputs.keys().collect::<Vec<_>>(),
                )),
            });
        }
    }
}

/// UI007: Screen display reference not found.
fn check_screen_display_ref(
    display_name: &str,
    displays: &HashMap<String, DisplayInfo>,
    file: &std::path::Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("UI007", Severity::Error, config) else {
        return;
    };

    if !displays.contains_key(display_name) {
        diagnostics.push(Diagnostic {
            file: file.to_path_buf(),
            line,
            col: 1,
            severity,
            rule_id: "UI007".to_string(),
            message: format!(
                "screen references display '{}' which is not defined as a DisplayHardware part",
                display_name,
            ),
            help: Some(format!(
                "defined displays: {:?}",
                displays.keys().collect::<Vec<_>>(),
            )),
        });
    }
}

/// UI008: Indicator LED reference not found.
fn check_indicator_led_ref(
    led_name: &str,
    leds: &HashMap<String, LedInfo>,
    part_name: &str,
    file: &std::path::Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("UI008", Severity::Error, config) else {
        return;
    };

    if !leds.contains_key(led_name) {
        diagnostics.push(Diagnostic {
            file: file.to_path_buf(),
            line,
            col: 1,
            severity,
            rule_id: "UI008".to_string(),
            message: format!(
                "IndicatorBinding references LED '{}' in '{}' which is not defined as a LedHardware part",
                led_name, part_name,
            ),
            help: Some(format!(
                "defined LEDs: {:?}",
                leds.keys().collect::<Vec<_>>(),
            )),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::domain::{DomainConfig, LayerConfig, RequiredMetadataConfig, SourceConfig};
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
            source: SourceConfig::default(),
        }
    }

    #[test]
    fn test_no_ui_parts_returns_empty() {
        let source = r#"
package Test {
    part def Plain {
        attribute x : Integer;
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, count) = check_ui_wellformedness(&ws, &config);
        assert!(diags.is_empty(), "no UI parts → no diagnostics");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_ui001_bounds_exceeded() {
        let source = r#"
package Test {
    part def MyDisplay {
        @DisplayHardware {
            type = DisplayKind::oled;
            width = 128;
            height = 64;
            colorDepth = ColorDepthKind::mono;
            interface = InterfaceKind::spi;
            driver = "ssd1306";
            orientation = "landscape";
            module = "display_driver";
        }
    }
    part def HomeScreen {
        @Screen {
            display = "MyDisplay";
            refreshMode = RefreshMode::event;
            pollInterval_ms = 0;
        }
        attribute title {
            @Element {
                type = ElementKind::text;
                x = 100;
                y = 50;
                width = 50;
                height = 20;
                font = "DefaultFont";
                binding_module = "HomeScreen";
                binding_field = "title";
            }
        }
    }
    part def DefaultFont {
        @FontAsset {
            family = "mono";
            size = 8;
            source = AssetSource::builtin;
            file = "";
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, count) = check_ui_wellformedness(&ws, &config);
        assert!(count > 0, "should check UI elements");
        let ui001: Vec<_> = diags.iter().filter(|d| d.rule_id == "UI001").collect();
        assert!(
            !ui001.is_empty(),
            "element at (100,50) + 50x20 exceeds 128x64: {:?}",
            diags
        );
    }

    #[test]
    fn test_ui002_overlap_without_guard() {
        let source = r#"
package Test {
    part def MyDisplay {
        @DisplayHardware {
            type = DisplayKind::oled;
            width = 128;
            height = 64;
            colorDepth = ColorDepthKind::mono;
            interface = InterfaceKind::spi;
            driver = "ssd1306";
            orientation = "landscape";
            module = "display_driver";
        }
    }
    part def OverlapScreen {
        @Screen {
            display = "MyDisplay";
            refreshMode = RefreshMode::event;
            pollInterval_ms = 0;
        }
        attribute elem1 {
            @Element {
                type = ElementKind::text;
                x = 0;
                y = 0;
                width = 50;
                height = 20;
            }
        }
        attribute elem2 {
            @Element {
                type = ElementKind::text;
                x = 10;
                y = 5;
                width = 50;
                height = 20;
            }
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_ui_wellformedness(&ws, &config);
        let ui002: Vec<_> = diags.iter().filter(|d| d.rule_id == "UI002").collect();
        assert!(
            !ui002.is_empty(),
            "overlapping elements without visible guards should trigger UI002: {:?}",
            diags
        );
    }

    #[test]
    fn test_ui003_missing_font() {
        let source = r#"
package Test {
    part def MyDisplay {
        @DisplayHardware {
            type = DisplayKind::oled;
            width = 128;
            height = 64;
            colorDepth = ColorDepthKind::mono;
            interface = InterfaceKind::spi;
            driver = "ssd1306";
            orientation = "landscape";
            module = "display_driver";
        }
    }
    part def FontScreen {
        @Screen {
            display = "MyDisplay";
            refreshMode = RefreshMode::event;
            pollInterval_ms = 0;
        }
        attribute label {
            @Element {
                type = ElementKind::text;
                x = 0;
                y = 0;
                width = 50;
                height = 10;
                font = "NonExistentFont";
            }
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_ui_wellformedness(&ws, &config);
        let ui003: Vec<_> = diags.iter().filter(|d| d.rule_id == "UI003").collect();
        assert!(
            ui003.iter().any(|d| d.message.contains("NonExistentFont")),
            "should detect missing font asset: {:?}",
            ui003
        );
    }

    #[test]
    fn test_ui004_missing_binding_module() {
        let source = r#"
package Test {
    part def MyDisplay {
        @DisplayHardware {
            type = DisplayKind::oled;
            width = 128;
            height = 64;
            colorDepth = ColorDepthKind::mono;
            interface = InterfaceKind::spi;
            driver = "ssd1306";
            orientation = "landscape";
            module = "display_driver";
        }
    }
    part def BindScreen {
        @Screen {
            display = "MyDisplay";
            refreshMode = RefreshMode::event;
            pollInterval_ms = 0;
        }
        attribute val {
            @Element {
                type = ElementKind::text;
                x = 0;
                y = 0;
                width = 50;
                height = 10;
                binding_module = "GhostModule";
                binding_field = "value";
            }
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_ui_wellformedness(&ws, &config);
        let ui004: Vec<_> = diags.iter().filter(|d| d.rule_id == "UI004").collect();
        assert!(
            ui004.iter().any(|d| d.message.contains("GhostModule")),
            "should detect non-existent binding module: {:?}",
            ui004
        );
    }

    #[test]
    fn test_ui007_missing_display() {
        let source = r#"
package Test {
    part def SomeInput {
        @InputDevice {
            type = InputKind::button;
            active = ActiveLevel::low;
            hasButton = false;
            detents = 0;
            module = "button_driver";
        }
    }
    part def BadScreen {
        @Screen {
            display = "NonExistentDisplay";
            refreshMode = RefreshMode::event;
            pollInterval_ms = 0;
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_ui_wellformedness(&ws, &config);
        let ui007: Vec<_> = diags.iter().filter(|d| d.rule_id == "UI007").collect();
        assert!(
            ui007.iter().any(|d| d.message.contains("NonExistentDisplay")),
            "should detect non-existent display reference: {:?}",
            ui007
        );
    }

    #[test]
    fn test_ui008_missing_led() {
        let source = r#"
package Test {
    part def SomeInput {
        @InputDevice {
            type = InputKind::button;
            active = ActiveLevel::low;
            hasButton = false;
            detents = 0;
            module = "button_driver";
        }
    }
    part def StatusIndicator {
        @IndicatorBinding {
            led = "GhostLED";
            module = "StatusIndicator";
            field = "state";
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_ui_wellformedness(&ws, &config);
        let ui008: Vec<_> = diags.iter().filter(|d| d.rule_id == "UI008").collect();
        assert!(
            ui008.iter().any(|d| d.message.contains("GhostLED")),
            "should detect non-existent LED reference: {:?}",
            ui008
        );
    }

    #[test]
    fn test_ui_rule_disabled() {
        let source = r#"
package Test {
    part def MyDisplay {
        @DisplayHardware {
            type = DisplayKind::oled;
            width = 128;
            height = 64;
            colorDepth = ColorDepthKind::mono;
            interface = InterfaceKind::spi;
            driver = "ssd1306";
            orientation = "landscape";
            module = "display_driver";
        }
    }
    part def HomeScreen {
        @Screen {
            display = "MyDisplay";
            refreshMode = RefreshMode::event;
            pollInterval_ms = 0;
        }
        attribute title {
            @Element {
                type = ElementKind::text;
                x = 200;
                y = 200;
                width = 50;
                height = 20;
            }
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let mut config = minimal_config();
        config
            .validation_rules
            .insert("UI001".to_string(), Severity::Off);
        let (diags, _) = check_ui_wellformedness(&ws, &config);
        let ui001: Vec<_> = diags.iter().filter(|d| d.rule_id == "UI001").collect();
        assert!(
            ui001.is_empty(),
            "UI001 should be disabled but got: {:?}",
            ui001
        );
    }

    #[test]
    fn test_ui005_color_not_achievable() {
        let source = r#"
package Test {
    part def SomeInput {
        @InputDevice {
            type = InputKind::button;
            active = ActiveLevel::low;
            hasButton = false;
            detents = 0;
            module = "button_driver";
        }
    }
    part def StatusLED {
        @LedHardware {
            type = LedType::bicolor;
            colors = ("red", "green");
            module = "led_driver";
        }
    }
    part def StatusIndicator {
        @IndicatorBinding {
            led = "StatusLED";
            module = "StatusIndicator";
            field = "state";
        }
        @IndicatorState {
            color = "blue";
            pattern = LedPattern::solid;
            period_ms = 0;
            duty_percent = 100;
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_ui_wellformedness(&ws, &config);
        let ui005: Vec<_> = diags.iter().filter(|d| d.rule_id == "UI005").collect();
        assert!(
            ui005.iter().any(|d| d.message.contains("blue")),
            "bicolor LED (red, green) cannot produce blue: {:?}",
            ui005
        );
    }

    #[test]
    fn test_ui006_gesture_device_not_found() {
        let source = r#"
package Test {
    part def SomeInput {
        @InputDevice {
            type = InputKind::button;
            active = ActiveLevel::low;
            hasButton = false;
            detents = 0;
            module = "button_driver";
        }
    }
    part def PressGesture {
        @Gesture {
            device = "NonExistentInput";
            trigger = TriggerKind::press;
            window_ms = 50;
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_ui_wellformedness(&ws, &config);
        let ui006: Vec<_> = diags.iter().filter(|d| d.rule_id == "UI006").collect();
        assert!(
            ui006.iter().any(|d| d.message.contains("NonExistentInput")),
            "should detect non-existent gesture device: {:?}",
            ui006
        );
    }
}
