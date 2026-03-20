//! Source file resolution for audit.
//!
//! Maps module names to source file paths using either explicit
//! metadata overrides or convention-based resolution.

use std::path::{Path, PathBuf};

use crate::domain::DomainConfig;
use crate::extraction::ExtractedModule;
use crate::util::{language_extension, snake_case};

/// Resolve the source file path for a module.
///
/// 1. Check `module.metadata.SourceMapping.file` override.
/// 2. Fall back to convention: `workspace_root / config.source.root / snake_case(name).ext`.
pub fn resolve_source_path(
    module: &ExtractedModule,
    config: &DomainConfig,
    workspace_root: &Path,
) -> Option<PathBuf> {
    // Check metadata override
    if let Some(source_mapping) = module.metadata.get("SourceMapping") {
        if let Some(file_value) = source_mapping.get("file") {
            if let Some(file_str) = file_value.as_str() {
                let path = workspace_root.join(file_str);
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    // Convention-based resolution
    let ext = language_extension(&config.source.language);
    let file_name = format!("{}.{}", snake_case(&module.name), ext);

    let path = workspace_root
        .join(&config.source.root)
        .join(&file_name);

    if path.exists() {
        return Some(path);
    }

    // For "flat" layout, also try module directory pattern: name/mod.ext
    if config.source.layout == "flat" {
        let dir_path = workspace_root
            .join(&config.source.root)
            .join(snake_case(&module.name))
            .join(format!("mod.{}", ext));
        if dir_path.exists() {
            return Some(dir_path);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::domain::SourceConfig;

    fn test_module(name: &str) -> ExtractedModule {
        ExtractedModule {
            name: name.to_string(),
            qualified_name: format!("Firmware::{}", name),
            source_file: PathBuf::from("test.sysml"),
            layer: None,
            metadata: HashMap::new(),
            ports: Vec::new(),
            actions: Vec::new(),
            connections: Vec::new(),
            state_machines: Vec::new(),
        }
    }

    fn test_config() -> DomainConfig {
        DomainConfig {
            name: "test".to_string(),
            description: None,
            metadata_library: PathBuf::new(),
            layers: crate::domain::LayerConfig {
                order: Vec::new(),
                allowed_deps: HashMap::new(),
            },
            required_metadata: crate::domain::RequiredMetadataConfig {
                parts: Vec::new(),
            },
            type_map: HashMap::new(),
            validation_rules: HashMap::new(),
            source: SourceConfig {
                root: "src".to_string(),
                language: "rust".to_string(),
                layout: "flat".to_string(),
            },
        }
    }

    #[test]
    fn test_source_map_convention() {
        let module = test_module("BtA2dpSink");
        let config = test_config();

        // Create a temp dir with a source file
        let tmp = std::env::temp_dir().join(format!("audit-test-{}", std::process::id()));
        let src_dir = tmp.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("bt_a2dp_sink.rs"), "// stub").unwrap();

        let path = resolve_source_path(&module, &config, &tmp);
        assert_eq!(path, Some(src_dir.join("bt_a2dp_sink.rs")));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_source_map_override() {
        let mut module = test_module("BtA2dpSink");

        // Add SourceMapping metadata
        let mut source_mapping = HashMap::new();
        source_mapping.insert(
            "file".to_string(),
            serde_json::Value::String("lib/bluetooth.rs".to_string()),
        );
        module
            .metadata
            .insert("SourceMapping".to_string(), source_mapping);

        let config = test_config();
        let tmp = std::env::temp_dir().join(format!("audit-test-override-{}", std::process::id()));
        let lib_dir = tmp.join("lib");
        std::fs::create_dir_all(&lib_dir).unwrap();
        std::fs::write(lib_dir.join("bluetooth.rs"), "// stub").unwrap();

        let path = resolve_source_path(&module, &config, &tmp);
        assert_eq!(path, Some(lib_dir.join("bluetooth.rs")));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_source_map_mod_pattern() {
        let module = test_module("StatusLed");
        let config = test_config();

        let tmp = std::env::temp_dir().join(format!("audit-test-mod-{}", std::process::id()));
        let mod_dir = tmp.join("src").join("status_led");
        std::fs::create_dir_all(&mod_dir).unwrap();
        std::fs::write(mod_dir.join("mod.rs"), "// stub").unwrap();

        let path = resolve_source_path(&module, &config, &tmp);
        assert_eq!(path, Some(mod_dir.join("mod.rs")));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_source_map_not_found() {
        let module = test_module("Nonexistent");
        let config = test_config();
        let tmp = std::env::temp_dir().join(format!("audit-test-none-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let path = resolve_source_path(&module, &config, &tmp);
        assert!(path.is_none());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
