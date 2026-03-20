//! Code generation engine using MiniJinja templates.
//!
//! Renders extracted module data through domain-provided templates
//! to produce source code files. Supports incremental generation
//! via spec-hash fingerprinting.

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use minijinja::Environment;

use crate::domain::DomainConfig;
use crate::extraction::types::{ExtractionResult, ExtractedModule};

pub mod filters;
mod hash;

// ── Error type ──────────────────────────────────────────────────────

/// Errors that can occur during code generation.
#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("template not found: {0}")]
    TemplateNotFound(String),

    #[error("template render error in '{template}': {source}")]
    Render {
        template: String,
        source: minijinja::Error,
    },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// ── Report types ────────────────────────────────────────────────────

/// Report of what the generation pipeline produced.
#[derive(Debug, Clone)]
pub struct GenerationReport {
    pub generated: Vec<GeneratedFile>,
    pub skipped: Vec<SkippedFile>,
}

/// A file that was generated (written to disk).
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub template: String,
    pub output_path: PathBuf,
    pub module_name: String,
}

/// A file that was skipped (spec-hash matched or template missing).
#[derive(Debug, Clone)]
pub struct SkippedFile {
    pub output_path: PathBuf,
    pub module_name: String,
    pub reason: String,
}

// ── Language → file extension mapping ───────────────────────────────

fn language_extension(language: &str) -> &str {
    match language {
        "rust" => "rs",
        "c" => "c",
        "cpp" => "cpp",
        _ => language,
    }
}

// ── Generation pipeline ─────────────────────────────────────────────

/// Generate source code from extracted module data using domain templates.
///
/// Templates are loaded from `config.template_dir / language /`.
/// Files with matching spec-hashes are skipped for incremental generation.
pub fn generate(
    extraction: &ExtractionResult,
    config: &DomainConfig,
    language: &str,
    output_dir: &Path,
) -> Result<GenerationReport, CodegenError> {
    let template_dir = config.template_dir.join(language);
    let ext = language_extension(language);

    // Build the type map for this language
    let type_map: HashMap<String, String> = config
        .type_map
        .get(language)
        .cloned()
        .unwrap_or_default();

    let mut report = GenerationReport {
        generated: Vec::new(),
        skipped: Vec::new(),
    };

    std::fs::create_dir_all(output_dir)?;

    // Module template
    let module_template_name = format!("module.{ext}.j2");
    let module_template = load_template(&template_dir, &module_template_name);

    // State machine template
    let fsm_template_name = format!("state_machine.{ext}.j2");
    let fsm_template = load_template(&template_dir, &fsm_template_name);

    // Test template
    let test_template_name = format!("test.{ext}.j2");
    let test_template = load_template(&template_dir, &test_template_name);

    for module in &extraction.modules {
        // ── Module template ──
        if let Some(template_src) = &module_template {
            let output_path = output_dir.join(format!(
                "{}.{}",
                filters::snake_case(&module.name),
                ext,
            ));

            let spec_hash = hash::compute_spec_hash(module);

            if hash::check_spec_hash(&output_path, &spec_hash) {
                report.skipped.push(SkippedFile {
                    output_path,
                    module_name: module.name.clone(),
                    reason: "spec-hash unchanged".to_string(),
                });
            } else {
                let rendered = render_module_template(
                    template_src,
                    &module_template_name,
                    module,
                    config,
                    language,
                    &type_map,
                )?;

                let content = format!("{}{}", hash::spec_hash_header(&spec_hash), rendered);
                write_file(&output_path, &content)?;

                report.generated.push(GeneratedFile {
                    template: module_template_name.clone(),
                    output_path,
                    module_name: module.name.clone(),
                });
            }
        }

        // ── State machine templates ──
        if let Some(template_src) = &fsm_template {
            for fsm in &module.state_machines {
                let output_path = output_dir.join(format!(
                    "{}_{}.{}",
                    filters::snake_case(&module.name),
                    filters::snake_case(&fsm.name),
                    ext,
                ));

                let fsm_hash_input = (&module.name, fsm);
                let spec_hash = hash::compute_spec_hash(&fsm_hash_input);

                if hash::check_spec_hash(&output_path, &spec_hash) {
                    report.skipped.push(SkippedFile {
                        output_path,
                        module_name: module.name.clone(),
                        reason: "spec-hash unchanged".to_string(),
                    });
                } else {
                    let rendered = render_fsm_template(
                        template_src,
                        &fsm_template_name,
                        module,
                        fsm,
                        config,
                        language,
                        &type_map,
                    )?;

                    let content =
                        format!("{}{}", hash::spec_hash_header(&spec_hash), rendered);
                    write_file(&output_path, &content)?;

                    report.generated.push(GeneratedFile {
                        template: fsm_template_name.clone(),
                        output_path,
                        module_name: module.name.clone(),
                    });
                }
            }
        }

        // ── Test template ──
        if let Some(template_src) = &test_template {
            let output_path = output_dir.join(format!(
                "{}_test.{}",
                filters::snake_case(&module.name),
                ext,
            ));

            let spec_hash = hash::compute_spec_hash(module);

            if hash::check_spec_hash(&output_path, &spec_hash) {
                report.skipped.push(SkippedFile {
                    output_path,
                    module_name: module.name.clone(),
                    reason: "spec-hash unchanged".to_string(),
                });
            } else {
                let rendered = render_module_template(
                    template_src,
                    &test_template_name,
                    module,
                    config,
                    language,
                    &type_map,
                )?;

                let content = format!("{}{}", hash::spec_hash_header(&spec_hash), rendered);
                write_file(&output_path, &content)?;

                report.generated.push(GeneratedFile {
                    template: test_template_name.clone(),
                    output_path,
                    module_name: module.name.clone(),
                });
            }
        }
    }

    Ok(report)
}

// ── Template helpers ────────────────────────────────────────────────

/// Load a template file from disk, returning None if not found.
fn load_template(template_dir: &Path, name: &str) -> Option<String> {
    let path = template_dir.join(name);
    std::fs::read_to_string(path).ok()
}

/// Create a MiniJinja environment with standard settings and filters.
fn create_environment(type_map: &HashMap<String, String>) -> Environment<'_> {
    let mut env = Environment::new();
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);
    env.set_keep_trailing_newline(true);

    filters::register_filters(&mut env);

    // Register map_type as a filter using a closure over the type map
    let type_map = type_map.clone();
    env.add_filter("map_type", move |value: String| -> String {
        filters::map_type(&value, &type_map)
    });

    env
}

/// Render a module template with the standard context.
fn render_module_template(
    template_src: &str,
    template_name: &str,
    module: &ExtractedModule,
    config: &DomainConfig,
    language: &str,
    type_map: &HashMap<String, String>,
) -> Result<String, CodegenError> {
    let env = create_environment(type_map);
    let tmpl = env
        .template_from_str(template_src)
        .map_err(|e| CodegenError::Render {
            template: template_name.to_string(),
            source: e,
        })?;

    let module_value = serde_json::to_value(module).unwrap_or_default();

    let ctx = minijinja::context! {
        module => module_value,
        domain => config.name,
        language => language,
    };

    tmpl.render(ctx).map_err(|e| CodegenError::Render {
        template: template_name.to_string(),
        source: e,
    })
}

/// Render a state machine template with FSM-specific context.
fn render_fsm_template(
    template_src: &str,
    template_name: &str,
    module: &ExtractedModule,
    fsm: &crate::extraction::types::ExtractedStateMachine,
    config: &DomainConfig,
    language: &str,
    type_map: &HashMap<String, String>,
) -> Result<String, CodegenError> {
    let env = create_environment(type_map);
    let tmpl = env
        .template_from_str(template_src)
        .map_err(|e| CodegenError::Render {
            template: template_name.to_string(),
            source: e,
        })?;

    let module_value = serde_json::to_value(module).unwrap_or_default();
    let fsm_value = serde_json::to_value(fsm).unwrap_or_default();

    // Collect unique event names from transitions
    let events: Vec<&str> = fsm
        .transitions
        .iter()
        .filter_map(|t| t.event.as_deref())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let ctx = minijinja::context! {
        module => module_value,
        fsm => fsm_value,
        events => events,
        domain => config.name,
        language => language,
    };

    tmpl.render(ctx).map_err(|e| CodegenError::Render {
        template: template_name.to_string(),
        source: e,
    })
}

fn write_file(path: &Path, content: &str) -> Result<(), CodegenError> {
    let mut f = std::fs::File::create(path)?;
    f.write_all(content.as_bytes())?;
    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DomainConfig;
    use crate::extraction::types::*;
    use crate::validation::validate;
    use std::path::PathBuf;
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

    fn extract_firmware() -> ExtractionResult {
        let ws = load_valid_workspace();
        let config = load_firmware_config();
        let validation = validate(&ws, &config);
        crate::extraction::extract(&ws, &config, &validation).unwrap()
    }

    fn tmpdir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "sysml-codegen-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn test_generate_module() {
        let extraction = extract_firmware();
        let config = load_firmware_config();
        let out = tmpdir("gen-module");

        let report = generate(&extraction, &config, "rust", &out).unwrap();

        assert!(
            !report.generated.is_empty(),
            "should generate at least one file"
        );

        // Check that BtA2dpSink module file exists and has content
        let bt_file = out.join("bt_a2dp_sink.rs");
        assert!(bt_file.exists(), "bt_a2dp_sink.rs should exist");
        let content = std::fs::read_to_string(&bt_file).unwrap();
        assert!(content.contains("BtA2dpSink"), "should contain module name");
        assert!(content.contains("spec-hash:"), "should have spec-hash header");

        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn test_generate_state_machine() {
        let extraction = extract_firmware();
        let config = load_firmware_config();
        let out = tmpdir("gen-fsm");

        let report = generate(&extraction, &config, "rust", &out).unwrap();

        // Report should include FSM files
        assert!(
            report.generated.iter().any(|g| g.template.contains("state_machine")),
            "should generate state_machine template files: {:?}",
            report.generated.iter().map(|g| &g.template).collect::<Vec<_>>()
        );

        // BtA2dpSink has ConnectionFSM
        let fsm_file = out.join("bt_a2dp_sink_connection_fsm.rs");
        assert!(fsm_file.exists(), "FSM file should exist");
        let content = std::fs::read_to_string(&fsm_file).unwrap();
        assert!(
            content.contains("ConnectionFSM") || content.contains("connection_fsm"),
            "should reference FSM name"
        );

        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn test_spec_hash_skip() {
        let extraction = extract_firmware();
        let config = load_firmware_config();
        let out = tmpdir("gen-skip");

        // First generation
        let report1 = generate(&extraction, &config, "rust", &out).unwrap();
        let gen_count = report1.generated.len();
        assert!(gen_count > 0, "first run should generate files");

        // Second generation — same input
        let report2 = generate(&extraction, &config, "rust", &out).unwrap();
        assert_eq!(
            report2.skipped.len(),
            gen_count,
            "second run should skip all {} files, but skipped {} and generated {}",
            gen_count,
            report2.skipped.len(),
            report2.generated.len(),
        );
        assert!(
            report2.generated.is_empty(),
            "second run should generate nothing"
        );

        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn test_generation_report() {
        let extraction = extract_firmware();
        let config = load_firmware_config();
        let out = tmpdir("gen-report");

        let report = generate(&extraction, &config, "rust", &out).unwrap();

        // 4 modules × (module + test) = 8 files, plus FSM files for BtA2dpSink and StatusLed
        assert!(
            report.generated.len() >= 8,
            "should generate at least 8 files (4 modules + 2 FSMs + tests), got {}",
            report.generated.len()
        );

        for gf in &report.generated {
            assert!(
                gf.output_path.exists(),
                "generated file should exist: {}",
                gf.output_path.display()
            );
        }

        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn test_missing_template_dir() {
        let extraction = extract_firmware();
        let mut config = load_firmware_config();
        // Point to nonexistent template dir
        config.template_dir = PathBuf::from("/nonexistent/templates");
        let out = tmpdir("gen-missing");

        // Should succeed but generate nothing (no templates found)
        let report = generate(&extraction, &config, "rust", &out).unwrap();
        assert!(report.generated.is_empty(), "no templates → nothing generated");

        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn test_template_domain() {
        let extraction = extract_firmware();
        let config = DomainConfig::load(&domains_dir().join("template"), None).unwrap();
        let out = tmpdir("gen-template-domain");

        let report = generate(&extraction, &config, "rust", &out).unwrap();

        // Template domain only has module.rs.j2, no FSM or test templates
        assert!(
            !report.generated.is_empty(),
            "should generate module files from template domain"
        );

        let _ = std::fs::remove_dir_all(&out);
    }
}
