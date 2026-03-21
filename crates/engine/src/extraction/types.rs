//! Serializable extraction types.
//!
//! Flat representations of adapter types with `Serialize`/`Deserialize` derives
//! for YAML/JSON output.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A fully extracted module from a SysML `PartDefinition`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedModule {
    pub name: String,
    pub qualified_name: String,
    pub source_file: PathBuf,
    pub layer: Option<String>,
    pub metadata: HashMap<String, HashMap<String, serde_json::Value>>,
    pub ports: Vec<ExtractedPort>,
    pub actions: Vec<ExtractedAction>,
    pub connections: Vec<ExtractedConnection>,
    pub state_machines: Vec<ExtractedStateMachine>,
}

/// A port extracted from a `PortUsage` symbol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedPort {
    pub name: String,
    pub port_type: Option<String>,
    pub conjugated: bool,
}

/// An action extracted from an `ActionDefinition` symbol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedAction {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ActionParameter>,
}

/// A parameter of an action definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionParameter {
    pub name: String,
    pub type_name: String,
    pub direction: ParameterDirection,
}

/// Direction of an action parameter.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterDirection {
    In,
    Out,
}

/// A connection extracted from adapter's `Connection` type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedConnection {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow_type: Option<String>,
}

/// A state machine extracted from adapter's `StateMachine` type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedStateMachine {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_state: Option<String>,
    pub states: Vec<String>,
    pub transitions: Vec<ExtractedTransition>,
}

/// A transition extracted from adapter's `Transition` type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedTransition {
    pub name: String,
    pub from_state: String,
    pub to_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

/// Result of extracting a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractionResult {
    pub modules: Vec<ExtractedModule>,
    pub architecture: ExtractedArchitecture,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<ExtractedUI>,
}

/// Extracted UI specification — None if no UI parts found in workspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedUI {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub displays: Vec<ExtractedDisplay>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_devices: Vec<ExtractedInputDevice>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub leds: Vec<ExtractedLed>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gestures: Vec<ExtractedGesture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing_defaults: Option<ExtractedTimingDefaults>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fonts: Vec<ExtractedFont>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub icons: Vec<ExtractedIcon>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub screens: Vec<ExtractedScreen>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub indicators: Vec<ExtractedIndicator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navigation: Option<ExtractedNavigation>,
}

/// A display hardware definition extracted from @DisplayHardware metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedDisplay {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_depth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}

/// An input device extracted from @InputDevice metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedInputDevice {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<String>,
    #[serde(default)]
    pub has_button: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detents: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}

/// An LED hardware definition extracted from @LedHardware metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedLed {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub led_type: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub colors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}

/// A gesture extracted from @Gesture metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedGesture {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_ms: Option<u32>,
}

/// Timing defaults extracted from @GestureTimingDefaults metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedTimingDefaults {
    pub debounce_ms: u32,
    pub long_press_ms: u32,
    pub double_tap_ms: u32,
    pub combo_window_ms: u32,
    pub sequence_timeout_ms: u32,
}

/// A font asset extracted from @FontAsset metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedFont {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    pub size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

/// An icon asset extracted from @IconAsset metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedIcon {
    pub name: String,
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

/// A screen extracted from @Screen metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedScreen {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_interval_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<ExtractedElement>,
}

/// A UI element extracted from @Element metadata within a screen.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedElement {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_type: Option<String>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<String>,
    #[serde(default)]
    pub scroll: bool,
    #[serde(default)]
    pub truncate: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_min: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_max: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible_when: Option<ExtractedVisibility>,
}

/// Visibility condition for a UI element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedVisibility {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// An indicator binding extracted from @IndicatorBinding metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedIndicator {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub led: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<ExtractedIndicatorState>,
}

/// A single indicator state extracted from @IndicatorState metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedIndicatorState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duty_percent: Option<u32>,
}

/// Navigation extracted from @Navigation metadata on a state machine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedNavigation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_screen: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub screens: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transitions: Vec<ExtractedTransition>,
}

/// Workspace-level architecture summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedArchitecture {
    pub source_files: Vec<PathBuf>,
    pub modules: Vec<ModuleSummary>,
    pub dependency_graph: Vec<(String, String)>,
}

/// Brief summary of a module for the architecture view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleSummary {
    pub name: String,
    pub layer: Option<String>,
    pub source_file: PathBuf,
}

/// Output serialization format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Yaml,
    Json,
}
