//! Domain configuration loading and merging.
//!
//! A domain is a directory under `domains/` containing:
//! - `domain.toml` — domain-level config (layers, rules, type maps)
//! - A metadata library `.sysml` file
//! - `templates/` — code generation templates
//!
//! A workspace file (`sysml.toml`) at the project root selects a domain
//! and optionally overrides validation severity and required metadata.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::diagnostic::Severity;

// ── Errors ──────────────────────────────────────────────────────────

/// Errors that can occur when loading domain or workspace configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

// ── Raw deserialization types (domain.toml) ─────────────────────────

#[derive(Debug, serde::Deserialize)]
struct RawDomainConfig {
    domain: DomainMeta,
    layers: Option<RawLayerConfig>,
    required_metadata: Option<RawRequiredMetadata>,
    validation: Option<RawValidation>,
    type_map: Option<HashMap<String, HashMap<String, String>>>,
    source: Option<RawSourceConfig>,
}

#[derive(Debug, serde::Deserialize)]
struct RawSourceConfig {
    root: Option<String>,
    language: Option<String>,
    layout: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct DomainMeta {
    name: String,
    description: Option<String>,
    metadata_library: String,
}

#[derive(Debug, serde::Deserialize)]
struct RawLayerConfig {
    order: Vec<String>,
    allowed_deps: HashMap<String, Vec<String>>,
}

#[derive(Debug, serde::Deserialize)]
struct RawRequiredMetadata {
    parts: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct RawValidation {
    rules: HashMap<String, Severity>,
}

// ── Raw deserialization types (sysml.toml) ──────────────────────────

#[derive(Debug, serde::Deserialize)]
struct RawWorkspaceConfig {
    workspace: WorkspaceMeta,
    validation: Option<RawValidation>,
    required_metadata: Option<RawRequiredMetadata>,
}

#[derive(Debug, serde::Deserialize)]
struct WorkspaceMeta {
    domain: String,
    #[serde(default)]
    include: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
}

// ── Resolved types ──────────────────────────────────────────────────

/// Layer ordering and dependency rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerConfig {
    /// Layers ordered from highest (application) to lowest (hardware).
    pub order: Vec<String>,
    /// For each layer, which layers it may depend on.
    pub allowed_deps: HashMap<String, Vec<String>>,
}

/// Required metadata annotations for parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredMetadataConfig {
    pub parts: Vec<String>,
}

/// Source code layout configuration for a domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceConfig {
    /// Root directory for source files (e.g., "src").
    pub root: String,
    /// Source language (e.g., "rust", "c").
    pub language: String,
    /// File layout convention (e.g., "flat", "nested").
    pub layout: String,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            root: "src".to_string(),
            language: "rust".to_string(),
            layout: "flat".to_string(),
        }
    }
}

/// Fully resolved domain configuration, after merging domain defaults
/// with optional project-level overrides from `sysml.toml`.
#[derive(Debug, Clone)]
pub struct DomainConfig {
    pub name: String,
    pub description: Option<String>,
    pub metadata_library: PathBuf,
    pub layers: LayerConfig,
    pub required_metadata: RequiredMetadataConfig,
    pub type_map: HashMap<String, HashMap<String, String>>,
    pub validation_rules: HashMap<String, Severity>,
    pub source: SourceConfig,
}

impl DomainConfig {
    /// Load a domain configuration from `domain_dir/domain.toml`.
    ///
    /// If `project_config` is provided, it is parsed as `sysml.toml` and
    /// its overrides are merged on top of the domain defaults:
    /// - `validation.rules`: project severity wins for any key present in both
    /// - `required_metadata`: project replaces domain if specified
    pub fn load(domain_dir: &Path, project_config: Option<&Path>) -> Result<Self, ConfigError> {
        let domain_toml_path = domain_dir.join("domain.toml");
        let raw = read_toml::<RawDomainConfig>(&domain_toml_path)?;

        let metadata_library = domain_dir.join(&raw.domain.metadata_library);

        let source = match raw.source {
            Some(s) => SourceConfig {
                root: s.root.unwrap_or_else(|| "src".to_string()),
                language: s.language.unwrap_or_else(|| "rust".to_string()),
                layout: s.layout.unwrap_or_else(|| "flat".to_string()),
            },
            None => SourceConfig::default(),
        };

        let layers = match raw.layers {
            Some(l) => LayerConfig {
                order: l.order,
                allowed_deps: l.allowed_deps,
            },
            None => LayerConfig {
                order: Vec::new(),
                allowed_deps: HashMap::new(),
            },
        };

        let required_metadata = match raw.required_metadata {
            Some(rm) => RequiredMetadataConfig { parts: rm.parts },
            None => RequiredMetadataConfig { parts: Vec::new() },
        };

        let mut validation_rules = match raw.validation {
            Some(v) => v.rules,
            None => HashMap::new(),
        };

        let type_map = raw.type_map.unwrap_or_default();

        // Merge project-level overrides if provided
        let mut final_required_metadata = required_metadata;
        if let Some(project_path) = project_config {
            let ws = read_toml::<RawWorkspaceConfig>(project_path)?;

            // Validation rules: project overrides domain
            if let Some(v) = ws.validation {
                for (rule_id, severity) in v.rules {
                    validation_rules.insert(rule_id, severity);
                }
            }

            // Required metadata: project replaces domain entirely if specified
            if let Some(rm) = ws.required_metadata {
                final_required_metadata = RequiredMetadataConfig { parts: rm.parts };
            }
        }

        Ok(DomainConfig {
            name: raw.domain.name,
            description: raw.domain.description,
            metadata_library,
            layers,
            required_metadata: final_required_metadata,
            type_map,
            validation_rules,
            source,
        })
    }
}

/// Workspace-level configuration parsed from `sysml.toml`.
///
/// This provides the domain name, file include/exclude globs,
/// and any project-level overrides.
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub domain: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub validation_overrides: HashMap<String, Severity>,
    pub required_metadata_overrides: Option<RequiredMetadataConfig>,
}

impl WorkspaceConfig {
    /// Load workspace configuration from a `sysml.toml` file.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let raw = read_toml::<RawWorkspaceConfig>(path)?;

        let validation_overrides = match raw.validation {
            Some(v) => v.rules,
            None => HashMap::new(),
        };

        let required_metadata_overrides = raw.required_metadata.map(|rm| RequiredMetadataConfig {
            parts: rm.parts,
        });

        Ok(WorkspaceConfig {
            domain: raw.workspace.domain,
            include: raw.workspace.include,
            exclude: raw.workspace.exclude,
            validation_overrides,
            required_metadata_overrides,
        })
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn read_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ConfigError> {
    let contents = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    toml::from_str(&contents).map_err(|e| ConfigError::Parse {
        path: path.to_path_buf(),
        source: e,
    })
}

// ── Unit tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Resolve path to the domains/ directory at the workspace root.
    fn domains_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("domains")
    }

    #[test]
    fn test_load_template_domain() {
        let domain_dir = domains_dir().join("template");
        let config = DomainConfig::load(&domain_dir, None).unwrap();

        assert_eq!(config.name, "template");
        assert_eq!(
            config.description.as_deref(),
            Some("Minimal example domain for testing and as a starter")
        );
        assert!(config.metadata_library.ends_with("template_library.sysml"));

        // Layers
        assert_eq!(config.layers.order, vec!["upper", "lower"]);
        assert_eq!(
            config.layers.allowed_deps.get("upper").unwrap(),
            &vec!["lower".to_string()]
        );
        assert!(config.layers.allowed_deps.get("lower").unwrap().is_empty());

        // Required metadata
        assert_eq!(config.required_metadata.parts, vec!["BasicInfo"]);

        // Validation rules
        assert_eq!(
            config.validation_rules.get("LAYER001"),
            Some(&Severity::Error)
        );
        assert_eq!(
            config.validation_rules.get("LAYER002"),
            Some(&Severity::Warning)
        );
        assert_eq!(
            config.validation_rules.get("META010"),
            Some(&Severity::Warning)
        );
        assert_eq!(
            config.validation_rules.get("FSM020"),
            Some(&Severity::Error)
        );

        // Type map
        let rust_types = config.type_map.get("rust").unwrap();
        assert_eq!(rust_types.get("Integer"), Some(&"i32".to_string()));
        assert_eq!(rust_types.get("Boolean"), Some(&"bool".to_string()));
    }

    #[test]
    fn test_load_firmware_domain() {
        let domain_dir = domains_dir().join("firmware");
        let config = DomainConfig::load(&domain_dir, None).unwrap();

        assert_eq!(config.name, "firmware");
        assert!(config.metadata_library.ends_with("firmware_library.sysml"));

        // 5 layers
        assert_eq!(config.layers.order.len(), 5);
        assert_eq!(config.layers.order[0], "application");
        assert_eq!(config.layers.order[4], "pac");

        // application can depend on middleware only
        assert_eq!(
            config.layers.allowed_deps.get("application").unwrap(),
            &vec!["middleware".to_string()]
        );
        // pac has no allowed deps
        assert!(config.layers.allowed_deps.get("pac").unwrap().is_empty());

        // Required metadata
        assert!(config
            .required_metadata
            .parts
            .contains(&"MemoryModel".to_string()));
        assert!(config
            .required_metadata
            .parts
            .contains(&"ConcurrencyModel".to_string()));
        assert!(config
            .required_metadata
            .parts
            .contains(&"ErrorHandling".to_string()));

        // Type maps for rust and c
        assert!(config.type_map.contains_key("rust"));
        assert!(config.type_map.contains_key("c"));
    }

    #[test]
    fn test_merge_with_project_config() {
        let domain_dir = domains_dir().join("template");

        // Create a temporary sysml.toml with overrides
        let tmp = tempdir();
        let project_toml = tmp.join("sysml.toml");
        fs::write(
            &project_toml,
            r#"
[workspace]
domain = "template"
include = ["src/**/*.sysml"]

[validation.rules]
LAYER001 = "warning"
CUSTOM001 = "error"

[required_metadata]
parts = ["CustomMeta"]
"#,
        )
        .unwrap();

        let config = DomainConfig::load(&domain_dir, Some(&project_toml)).unwrap();

        // LAYER001 overridden from error to warning
        assert_eq!(
            config.validation_rules.get("LAYER001"),
            Some(&Severity::Warning)
        );
        // New rule added
        assert_eq!(
            config.validation_rules.get("CUSTOM001"),
            Some(&Severity::Error)
        );
        // Original domain rules preserved
        assert_eq!(
            config.validation_rules.get("FSM020"),
            Some(&Severity::Error)
        );
        // Required metadata replaced entirely
        assert_eq!(config.required_metadata.parts, vec!["CustomMeta"]);
    }

    #[test]
    fn test_severity_override() {
        let domain_dir = domains_dir().join("template");

        let tmp = tempdir();
        let project_toml = tmp.join("sysml.toml");
        fs::write(
            &project_toml,
            r#"
[workspace]
domain = "template"

[validation.rules]
META010 = "error"
"#,
        )
        .unwrap();

        let config = DomainConfig::load(&domain_dir, Some(&project_toml)).unwrap();
        // Domain says warning, project says error → error wins
        assert_eq!(
            config.validation_rules.get("META010"),
            Some(&Severity::Error)
        );
    }

    #[test]
    fn test_severity_off() {
        let domain_dir = domains_dir().join("template");

        let tmp = tempdir();
        let project_toml = tmp.join("sysml.toml");
        fs::write(
            &project_toml,
            r#"
[workspace]
domain = "template"

[validation.rules]
LAYER001 = "off"
"#,
        )
        .unwrap();

        let config = DomainConfig::load(&domain_dir, Some(&project_toml)).unwrap();
        assert_eq!(
            config.validation_rules.get("LAYER001"),
            Some(&Severity::Off)
        );
    }

    #[test]
    fn test_missing_domain_toml() {
        let result = DomainConfig::load(Path::new("/nonexistent/domain"), None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match &err {
            ConfigError::Io { path, .. } => {
                assert!(path.to_string_lossy().contains("domain.toml"));
            }
            other => panic!("expected Io error, got: {other}"),
        }
    }

    #[test]
    fn test_invalid_toml() {
        let tmp = tempdir();
        let bad_toml = tmp.join("domain.toml");
        fs::write(&bad_toml, "this is not [valid toml {{{").unwrap();

        let result = DomainConfig::load(&tmp, None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match &err {
            ConfigError::Parse { path, .. } => {
                assert!(path.to_string_lossy().contains("domain.toml"));
            }
            other => panic!("expected Parse error, got: {other}"),
        }
    }

    #[test]
    fn test_workspace_config_load() {
        let tmp = tempdir();
        let toml_path = tmp.join("sysml.toml");
        fs::write(
            &toml_path,
            r#"
[workspace]
domain = "firmware"
include = ["src/**/*.sysml", "lib/**/*.sysml"]
exclude = ["vendor/**"]

[validation.rules]
META010 = "off"
"#,
        )
        .unwrap();

        let ws = WorkspaceConfig::load(&toml_path).unwrap();
        assert_eq!(ws.domain, "firmware");
        assert_eq!(ws.include, vec!["src/**/*.sysml", "lib/**/*.sysml"]);
        assert_eq!(ws.exclude, vec!["vendor/**"]);
        assert_eq!(
            ws.validation_overrides.get("META010"),
            Some(&Severity::Off)
        );
        assert!(ws.required_metadata_overrides.is_none());
    }

    /// Helper to create a temporary directory (returns PathBuf for simplicity).
    fn tempdir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "sysml-engine-test-{}",
            std::process::id()
                .wrapping_add(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos() as u32)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
