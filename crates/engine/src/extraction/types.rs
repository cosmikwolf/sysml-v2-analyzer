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
