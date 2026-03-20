//! Integration test: run full validation on real fixtures with firmware domain config.

use std::path::PathBuf;

use sysml_v2_adapter::SysmlWorkspace;
use sysml_v2_engine::diagnostic::Severity;
use sysml_v2_engine::domain::DomainConfig;
use sysml_v2_engine::validation::validate;

fn domains_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("domains")
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

#[test]
fn test_full_firmware_validation() {
    let ws = load_valid_workspace();
    let config = DomainConfig::load(&domains_dir().join("firmware"), None).unwrap();
    let result = validate(&ws, &config);

    // The firmware fixtures should have no errors
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();

    assert!(
        errors.is_empty(),
        "firmware fixtures should produce no errors, got {} error(s):\n{}",
        errors.len(),
        errors
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Should have checked some parts and state machines
    assert!(
        result.parts_checked > 0,
        "should have checked at least 1 part"
    );
    assert!(
        result.state_machines_checked > 0,
        "should have checked at least 1 state machine"
    );
}

#[test]
fn test_firmware_validation_counts() {
    let ws = load_valid_workspace();
    let config = DomainConfig::load(&domains_dir().join("firmware"), None).unwrap();
    let result = validate(&ws, &config);

    // 4 part definitions: BtA2dpSink, AudioPipeline, I2sOutput, StatusLed
    assert!(
        result.parts_checked >= 4,
        "should check at least 4 parts, got {}",
        result.parts_checked
    );

    // 2 state machines: ConnectionFSM, LedFSM
    assert!(
        result.state_machines_checked >= 2,
        "should check at least 2 state machines, got {}",
        result.state_machines_checked
    );
}
