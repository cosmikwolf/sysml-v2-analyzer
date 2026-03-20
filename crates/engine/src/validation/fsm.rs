//! State machine well-formedness validation.
//!
//! - FSM020: No initial state
//! - FSM021: Unreachable state
//! - FSM022: Non-deterministic transitions (same event from same state, no distinct guards)
//! - FSM024: Transition targets a state not defined in this FSM
//! - FSM025: Terminal state (no outgoing transitions)

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use sysml_v2_adapter::state_machine_extractor::extract_state_machines;
use sysml_v2_adapter::{SysmlWorkspace, SymbolKind};

use crate::diagnostic::{Diagnostic, Severity};
use crate::domain::DomainConfig;

use super::{effective_severity, to_display_line};

/// Check all state machines in the workspace for well-formedness.
///
/// Returns diagnostics and the number of state machines checked.
pub(crate) fn check_fsm_wellformedness(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
) -> (Vec<Diagnostic>, usize) {
    let mut diagnostics = Vec::new();
    let mut fsm_count = 0;

    for (file, sym) in workspace.all_symbols() {
        if sym.kind != SymbolKind::PartDefinition {
            continue;
        }

        let state_machines = extract_state_machines(file, sym);
        for fsm in &state_machines {
            fsm_count += 1;
            let file_path = &file.path;
            let line = to_display_line(sym.start_line);

            check_initial_state(fsm, file_path, line, config, &mut diagnostics);
            check_unreachable_states(fsm, file_path, line, config, &mut diagnostics);
            check_nondeterminism(fsm, file_path, line, config, &mut diagnostics);
            check_invalid_targets(fsm, file_path, line, config, &mut diagnostics);
            check_terminal_states(fsm, file_path, line, config, &mut diagnostics);
        }
    }

    (diagnostics, fsm_count)
}

/// FSM020: No initial state.
fn check_initial_state(
    fsm: &sysml_v2_adapter::StateMachine,
    file: &Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("FSM020", Severity::Error, config) else {
        return;
    };

    if fsm.initial_state.is_none() {
        diagnostics.push(Diagnostic {
            file: file.to_path_buf(),
            line,
            col: 1,
            severity,
            rule_id: "FSM020".to_string(),
            message: format!(
                "state machine '{}' has no initial state (missing 'entry; then X;')",
                fsm.name,
            ),
            help: None,
        });
    }
}

/// FSM021: Unreachable state.
fn check_unreachable_states(
    fsm: &sysml_v2_adapter::StateMachine,
    file: &Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("FSM021", Severity::Error, config) else {
        return;
    };

    // Skip if no initial state (FSM020 already handles that case)
    let Some(initial) = &fsm.initial_state else {
        return;
    };

    // BFS from initial state
    let mut reachable: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    reachable.insert(initial);
    queue.push_back(initial);

    while let Some(state) = queue.pop_front() {
        for t in &fsm.transitions {
            if t.from_state == state && !reachable.contains(t.to_state.as_str()) {
                reachable.insert(&t.to_state);
                queue.push_back(&t.to_state);
            }
        }
    }

    for state in &fsm.states {
        if !reachable.contains(state.name.as_str()) {
            diagnostics.push(Diagnostic {
                file: file.to_path_buf(),
                line,
                col: 1,
                severity,
                rule_id: "FSM021".to_string(),
                message: format!(
                    "state '{}' in '{}' is unreachable — no incoming transitions and not the initial state",
                    state.name, fsm.name,
                ),
                help: None,
            });
        }
    }
}

/// FSM022: Non-deterministic transitions.
fn check_nondeterminism(
    fsm: &sysml_v2_adapter::StateMachine,
    file: &Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("FSM022", Severity::Warning, config) else {
        return;
    };

    // Group transitions by (from_state, event)
    let mut groups: HashMap<(&str, &str), Vec<&sysml_v2_adapter::Transition>> = HashMap::new();

    for t in &fsm.transitions {
        if let Some(event) = &t.event {
            groups
                .entry((&t.from_state, event))
                .or_default()
                .push(t);
        }
    }

    for ((from_state, event), transitions) in &groups {
        if transitions.len() <= 1 {
            continue;
        }

        // Check if all transitions have distinct guards
        let guards: Vec<Option<&str>> = transitions
            .iter()
            .map(|t| t.guard.as_deref())
            .collect();

        let distinct_guards: HashSet<Option<&str>> = guards.iter().copied().collect();

        // If fewer distinct guards than transitions, or any guard is None → non-deterministic
        if distinct_guards.len() < transitions.len() || guards.iter().any(|g| g.is_none()) {
            diagnostics.push(Diagnostic {
                file: file.to_path_buf(),
                line,
                col: 1,
                severity,
                rule_id: "FSM022".to_string(),
                message: format!(
                    "non-deterministic transitions in '{}': state '{}' has {} transitions on event '{}' without distinct guards",
                    fsm.name, from_state, transitions.len(), event,
                ),
                help: None,
            });
        }
    }
}

/// FSM024: Transition targets a state not defined in this FSM.
fn check_invalid_targets(
    fsm: &sysml_v2_adapter::StateMachine,
    file: &Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("FSM024", Severity::Error, config) else {
        return;
    };

    let state_names: HashSet<&str> = fsm.states.iter().map(|s| s.name.as_str()).collect();

    for t in &fsm.transitions {
        if !state_names.contains(t.to_state.as_str()) {
            diagnostics.push(Diagnostic {
                file: file.to_path_buf(),
                line,
                col: 1,
                severity,
                rule_id: "FSM024".to_string(),
                message: format!(
                    "transition '{}' in '{}' targets state '{}' which is not defined in this state machine",
                    t.name, fsm.name, t.to_state,
                ),
                help: Some(format!(
                    "defined states: {}",
                    state_names
                        .iter()
                        .copied()
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            });
        }
        if !state_names.contains(t.from_state.as_str()) {
            diagnostics.push(Diagnostic {
                file: file.to_path_buf(),
                line,
                col: 1,
                severity,
                rule_id: "FSM024".to_string(),
                message: format!(
                    "transition '{}' in '{}' originates from state '{}' which is not defined in this state machine",
                    t.name, fsm.name, t.from_state,
                ),
                help: None,
            });
        }
    }
}

/// FSM025: Terminal state (no outgoing transitions).
fn check_terminal_states(
    fsm: &sysml_v2_adapter::StateMachine,
    file: &Path,
    line: usize,
    config: &DomainConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(severity) = effective_severity("FSM025", Severity::Warning, config) else {
        return;
    };

    let states_with_outgoing: HashSet<&str> = fsm
        .transitions
        .iter()
        .map(|t| t.from_state.as_str())
        .collect();

    for state in &fsm.states {
        if !states_with_outgoing.contains(state.name.as_str()) {
            diagnostics.push(Diagnostic {
                file: file.to_path_buf(),
                line,
                col: 1,
                severity,
                rule_id: "FSM025".to_string(),
                message: format!(
                    "state '{}' in '{}' has no outgoing transitions (terminal state)",
                    state.name, fsm.name,
                ),
                help: None,
            });
        }
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

    #[test]
    fn test_fsm_valid() {
        // ConnectionFSM in bt_a2dp_sink is well-formed
        let lib = load_fixture("firmware_library.sysml");
        let src = load_fixture("bt_a2dp_sink.sysml");
        let ws = SysmlWorkspace::from_sources(vec![
            (PathBuf::from("firmware_library.sysml"), lib),
            (PathBuf::from("bt_a2dp_sink.sysml"), src),
        ]);
        let config = minimal_config();
        let (diags, fsm_count) = check_fsm_wellformedness(&ws, &config);
        assert!(fsm_count > 0, "should find at least one FSM");
        let fsm_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.rule_id.starts_with("FSM"))
            .collect();
        assert!(
            fsm_diags.is_empty(),
            "ConnectionFSM should be well-formed, got: {:?}",
            fsm_diags
        );
    }

    #[test]
    fn test_fsm_no_initial() {
        let source = r#"
package Test {
    part def Widget {
        state def BadFSM {
            state idle;
            state running;
            transition start first idle accept GoEvent then running;
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_fsm_wellformedness(&ws, &config);
        let fsm020: Vec<_> = diags.iter().filter(|d| d.rule_id == "FSM020").collect();
        assert!(!fsm020.is_empty(), "should detect missing initial state");
    }

    #[test]
    fn test_fsm_unreachable() {
        let source = r#"
package Test {
    part def Widget {
        state def IslandFSM {
            entry; then idle;
            state idle;
            state running;
            state island;
            transition start first idle accept GoEvent then running;
            transition stop first running accept StopEvent then idle;
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_fsm_wellformedness(&ws, &config);
        let fsm021: Vec<_> = diags.iter().filter(|d| d.rule_id == "FSM021").collect();
        assert!(
            fsm021.iter().any(|d| d.message.contains("island")),
            "should detect unreachable 'island' state: {:?}",
            fsm021
        );
    }

    #[test]
    fn test_fsm_nondeterministic() {
        let source = r#"
package Test {
    part def Widget {
        state def AmbigFSM {
            entry; then idle;
            state idle;
            state a;
            state b;
            transition go_a first idle accept TickEvent then a;
            transition go_b first idle accept TickEvent then b;
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_fsm_wellformedness(&ws, &config);
        let fsm022: Vec<_> = diags.iter().filter(|d| d.rule_id == "FSM022").collect();
        assert!(
            !fsm022.is_empty(),
            "should detect non-deterministic transitions on TickEvent: {:?}",
            fsm022
        );
    }

    #[test]
    fn test_fsm_bad_target() {
        let source = r#"
package Test {
    part def Widget {
        state def BadTargetFSM {
            entry; then idle;
            state idle;
            transition go first idle accept GoEvent then nonexistent;
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_fsm_wellformedness(&ws, &config);
        let fsm024: Vec<_> = diags.iter().filter(|d| d.rule_id == "FSM024").collect();
        assert!(
            fsm024.iter().any(|d| d.message.contains("nonexistent")),
            "should detect invalid target state: {:?}",
            fsm024
        );
    }

    #[test]
    fn test_fsm_terminal() {
        let source = r#"
package Test {
    part def Widget {
        state def SinkFSM {
            entry; then idle;
            state idle;
            state done;
            transition finish first idle accept DoneEvent then done;
        }
    }
}
"#;
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.into())]);
        let config = minimal_config();
        let (diags, _) = check_fsm_wellformedness(&ws, &config);
        let fsm025: Vec<_> = diags.iter().filter(|d| d.rule_id == "FSM025").collect();
        assert!(
            fsm025.iter().any(|d| d.message.contains("done")),
            "should detect terminal state 'done': {:?}",
            fsm025
        );
    }
}
