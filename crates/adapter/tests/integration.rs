//! Integration tests for sysml-v2-adapter.
//!
//! Loads the full fixture workspace and exercises all adapter modules
//! together: workspace loading, metadata extraction, connection resolution,
//! and state machine extraction.

use std::path::PathBuf;

use sysml_v2_adapter::connection_resolver::{resolve_connections, ConnectionKind};
use sysml_v2_adapter::metadata_extractor::{extract_metadata, MetadataValue};
use sysml_v2_adapter::state_machine_extractor::extract_state_machines;
use sysml_v2_adapter::symbol_kind_mapper::{classify_symbol, MappedSymbolKind};
use sysml_v2_adapter::workspace::SysmlWorkspace;
use sysml_v2_adapter::SymbolKind;

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

/// Valid fixture file names (excludes malformed.sysml and large_model.sysml).
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
fn test_full_workspace_no_errors() {
    let ws = load_valid_workspace();

    for file in ws.files() {
        assert!(
            file.parse.errors.is_empty(),
            "{} had parse errors: {:?}",
            file.path.display(),
            file.parse.errors
        );
    }
}

#[test]
fn test_all_part_defs_found() {
    let ws = load_valid_workspace();
    let part_defs = ws.symbols_of_kind(SymbolKind::PartDefinition);

    let names: Vec<&str> = part_defs.iter().map(|(_, sym)| sym.name.as_ref()).collect();

    for expected in &["BtA2dpSink", "AudioPipeline", "I2sOutput", "StatusLed"] {
        assert!(
            names.contains(expected),
            "missing part def '{}' in {:?}",
            expected,
            names
        );
    }
}

#[test]
fn test_all_part_defs_have_memory_model() {
    let ws = load_valid_workspace();
    let part_defs = ws.symbols_of_kind(SymbolKind::PartDefinition);

    for (file, sym) in &part_defs {
        let annotations = extract_metadata(file, sym);
        let has_mm = annotations.iter().any(|a| a.name == "MemoryModel");
        assert!(
            has_mm,
            "part def '{}' should have @MemoryModel annotation",
            sym.name
        );
    }
}

#[test]
fn test_connection_fsm_structure() {
    let ws = load_valid_workspace();
    let (file, part) = ws
        .all_symbols()
        .find(|(_, sym)| sym.kind == SymbolKind::PartDefinition && *sym.name == *"BtA2dpSink")
        .expect("should find BtA2dpSink");

    let machines = extract_state_machines(file, part);
    let fsm = machines
        .iter()
        .find(|m| m.name == "ConnectionFSM")
        .expect("should find ConnectionFSM");

    // 4 states: disconnected, discovering, connected, streaming
    assert_eq!(fsm.states.len(), 4, "ConnectionFSM states: {:?}", fsm.states);

    // 7 transitions
    assert_eq!(
        fsm.transitions.len(),
        7,
        "ConnectionFSM transitions: {:?}",
        fsm.transitions
    );

    // Initial state is disconnected
    assert_eq!(fsm.initial_state.as_deref(), Some("disconnected"));
}

#[test]
fn test_audio_pipeline_connections() {
    let ws = load_valid_workspace();
    let (file, part) = ws
        .all_symbols()
        .find(|(_, sym)| sym.kind == SymbolKind::PartDefinition && *sym.name == *"AudioPipeline")
        .expect("should find AudioPipeline");

    let connections = resolve_connections(file, part);

    let connects: Vec<_> = connections
        .iter()
        .filter(|c| c.kind == ConnectionKind::Connect)
        .collect();
    let flows: Vec<_> = connections
        .iter()
        .filter(|c| c.kind == ConnectionKind::Flow)
        .collect();

    assert!(
        connects.len() >= 3,
        "AudioPipeline should have at least 3 connect statements, found {}",
        connects.len()
    );
    assert!(
        flows.len() >= 1,
        "AudioPipeline should have at least 1 flow statement, found {}",
        flows.len()
    );
}

#[test]
fn test_metadata_values_structured() {
    let ws = load_valid_workspace();
    let (file, part) = ws
        .all_symbols()
        .find(|(_, sym)| sym.kind == SymbolKind::PartDefinition && *sym.name == *"BtA2dpSink")
        .expect("should find BtA2dpSink");

    let annotations = extract_metadata(file, part);

    // Check @MemoryModel
    let mm = annotations
        .iter()
        .find(|a| a.name == "MemoryModel")
        .expect("should find @MemoryModel");

    let alloc = mm.fields.iter().find(|f| f.name == "allocation" || f.name.ends_with("allocation"));
    assert!(alloc.is_some(), "should find allocation field");
    match &alloc.unwrap().value {
        MetadataValue::EnumRef { enum_type, variant } => {
            assert_eq!(enum_type, "AllocationKind");
            assert_eq!(variant, "static_alloc");
        }
        other => panic!("expected EnumRef, got {:?}", other),
    }

    // Check @ISRSafe
    let isr = annotations
        .iter()
        .find(|a| a.name == "ISRSafe")
        .expect("should find @ISRSafe");
    let safe_field = isr.fields.iter().find(|f| f.name == "safe");
    assert!(safe_field.is_some(), "should find safe field");
    assert_eq!(safe_field.unwrap().value, MetadataValue::Boolean(false));
}

#[test]
fn test_symbol_kind_mapping() {
    let ws = load_valid_workspace();

    // Check firmware_library.sysml for metadata defs
    let lib_file = ws
        .files()
        .iter()
        .find(|f| f.path.to_string_lossy().contains("firmware_library"))
        .expect("should find firmware_library file");

    let mut metadata_count = 0;
    for sym in &lib_file.symbols {
        let classified = classify_symbol(lib_file, sym);
        if classified == MappedSymbolKind::MetadataDefinition {
            metadata_count += 1;
        }
    }

    assert!(
        metadata_count >= 6,
        "should classify at least 6 metadata defs, found {}",
        metadata_count
    );
}

#[test]
fn test_led_fsm_structure() {
    let ws = load_valid_workspace();
    let (file, part) = ws
        .all_symbols()
        .find(|(_, sym)| sym.kind == SymbolKind::PartDefinition && *sym.name == *"StatusLed")
        .expect("should find StatusLed");

    let machines = extract_state_machines(file, part);
    let fsm = machines
        .iter()
        .find(|m| m.name == "LedFSM")
        .expect("should find LedFSM");

    // 3 states: off, solid, blinking
    assert_eq!(fsm.states.len(), 3, "LedFSM states: {:?}", fsm.states);

    // 6 transitions
    assert_eq!(
        fsm.transitions.len(),
        6,
        "LedFSM transitions: {:?}",
        fsm.transitions
    );

    // Initial state is off
    assert_eq!(fsm.initial_state.as_deref(), Some("off"));
}

#[test]
fn test_workspace_load_from_directory() {
    let dir = fixtures_dir();
    if dir.exists() {
        let ws = SysmlWorkspace::load(&dir).expect("should load fixtures directory");
        assert!(
            ws.files().len() >= 6,
            "should load at least 6 files from fixtures, found {}",
            ws.files().len()
        );
    }
}
