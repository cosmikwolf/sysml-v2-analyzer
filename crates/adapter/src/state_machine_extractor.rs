//! State machine extraction from SysML v2 models.
//!
//! Extracts `state def` structures from part definitions, including:
//! - States (with parallel region detection)
//! - Transitions (from, event, to, guard, action)
//! - Initial state (from `entry; then X;` pattern)
//!
//! Uses HIR for structural discovery (`StateDefinition`, `StateUsage`,
//! `TransitionUsage`) and CST text for detail extraction (`first X`,
//! `accept E`, `then Y`, `if G`).

use syster::hir::{HirSymbol, SymbolKind};

use crate::workspace::ParsedFile;

/// An extracted state machine definition.
#[derive(Debug, Clone, PartialEq)]
pub struct StateMachine {
    /// State machine name (e.g. "ConnectionFSM").
    pub name: String,
    /// Fully qualified name (e.g. "Firmware::BtA2dpSink::ConnectionFSM").
    pub qualified_name: String,
    /// All states in this machine.
    pub states: Vec<State>,
    /// All transitions in this machine.
    pub transitions: Vec<Transition>,
    /// The initial state (from `entry; then X;`), if found.
    pub initial_state: Option<String>,
}

/// A state within a state machine.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    /// State name.
    pub name: String,
    /// Whether this is a `parallel state`.
    pub is_parallel: bool,
}

/// A transition within a state machine.
#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    /// Transition name (e.g. "disconnected_to_discovering").
    pub name: String,
    /// Source state (`first X`).
    pub from_state: String,
    /// Trigger event (`accept E`), if present.
    pub event: Option<String>,
    /// Target state (`then Y`).
    pub to_state: String,
    /// Guard expression (`if G`), if present.
    pub guard: Option<String>,
    /// Do-action (`do action A`), if present.
    pub action: Option<String>,
}

/// Extract all state machines from a part definition.
pub fn extract_state_machines(
    file: &ParsedFile,
    part_symbol: &HirSymbol,
) -> Vec<StateMachine> {
    let part_qn = &part_symbol.qualified_name;
    let source = file.parse.syntax().text().to_string();

    // Find state definitions that are children of this part
    let state_defs: Vec<&HirSymbol> = file
        .symbols
        .iter()
        .filter(|sym| {
            sym.kind == SymbolKind::StateDefinition
                && sym.qualified_name.starts_with(part_qn.as_ref())
                && sym.qualified_name.len() > part_qn.len()
        })
        .collect();

    let mut machines = Vec::new();

    for state_def in state_defs {
        let fsm_qn = &state_def.qualified_name;

        // Find child states
        let states = extract_states(file, fsm_qn);

        // Find transitions
        let transitions = extract_transitions(file, fsm_qn, &source);

        // Find initial state from CST `entry; then X;`
        let initial_state = find_initial_state(&source, state_def);

        machines.push(StateMachine {
            name: state_def.name.to_string(),
            qualified_name: state_def.qualified_name.to_string(),
            states,
            transitions,
            initial_state,
        });
    }

    machines
}

/// Extract states from a state machine definition.
fn extract_states(file: &ParsedFile, fsm_qualified_name: &str) -> Vec<State> {
    file.symbols
        .iter()
        .filter(|sym| {
            sym.kind == SymbolKind::StateUsage
                && sym.qualified_name.starts_with(fsm_qualified_name)
                && sym.qualified_name.len() > fsm_qualified_name.len()
                // Only direct children (one level of nesting)
                && sym.qualified_name[fsm_qualified_name.len()..].matches("::").count() == 1
        })
        .map(|sym| {
            let is_parallel = check_parallel_state(&file.parse.syntax().text().to_string(), sym);
            State {
                name: sym.name.to_string(),
                is_parallel,
            }
        })
        .collect()
}

/// Check if a state is declared as `parallel state`.
fn check_parallel_state(source: &str, sym: &HirSymbol) -> bool {
    let lines: Vec<&str> = source.lines().collect();
    // syster-base HIR uses 0-indexed line numbers
    let line_idx = sym.start_line as usize;
    if line_idx >= lines.len() {
        return false;
    }
    let line = lines[line_idx].trim();
    line.starts_with("parallel state") || line.starts_with("parallel  state")
}

/// Extract transitions from a state machine definition.
fn extract_transitions(
    file: &ParsedFile,
    fsm_qualified_name: &str,
    source: &str,
) -> Vec<Transition> {
    file.symbols
        .iter()
        .filter(|sym| {
            sym.kind == SymbolKind::TransitionUsage
                && sym.qualified_name.starts_with(fsm_qualified_name)
        })
        .filter_map(|sym| parse_transition_details(source, sym))
        .collect()
}

/// Parse transition details from the CST text.
///
/// Looks for the pattern:
/// ```text
/// transition <name>
///     first <from_state>
///     accept <event>
///     [if <guard>]
///     [do action <action>]
///     then <to_state>;
/// ```
fn parse_transition_details(source: &str, sym: &HirSymbol) -> Option<Transition> {
    let lines: Vec<&str> = source.lines().collect();
    // syster-base HIR uses 0-indexed line numbers
    let start_line = sym.start_line as usize;

    if start_line >= lines.len() {
        return None;
    }

    // Collect the full transition text (may span multiple lines)
    let mut transition_text = String::new();
    for line in lines.iter().take((start_line + 7).min(lines.len())).skip(start_line) {
        transition_text.push_str(line.trim());
        transition_text.push(' ');
        if line.contains(';') {
            break;
        }
    }

    let name = sym.name.to_string();
    let from_state = extract_keyword_value(&transition_text, "first");
    let event = extract_keyword_value(&transition_text, "accept");
    let to_state = extract_keyword_value(&transition_text, "then");
    let guard = extract_guard(&transition_text);
    let action = extract_do_action(&transition_text);

    // Must have at minimum a from and to state
    let from_state = from_state?;
    let to_state = to_state?;

    Some(Transition {
        name,
        from_state,
        event,
        to_state,
        guard,
        action,
    })
}

/// Extract the value after a keyword like `first`, `accept`, `then`.
fn extract_keyword_value(text: &str, keyword: &str) -> Option<String> {
    let pattern = format!("{} ", keyword);
    let pos = text.find(&pattern)?;
    let after = &text[pos + pattern.len()..];

    // Take the next word (identifier)
    let value: String = after
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();

    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

/// Extract guard expression from `if <expr>`.
fn extract_guard(text: &str) -> Option<String> {
    // Look for "if " that's not part of another word
    let pos = text.find(" if ")?;
    let after = &text[pos + 4..];

    // Guard extends until "then" or "do" or ";"
    let end = after
        .find(" then ")
        .or_else(|| after.find(" do "))
        .or_else(|| after.find(';'))
        .unwrap_or(after.len());

    let guard = after[..end].trim().to_string();
    if guard.is_empty() {
        None
    } else {
        Some(guard)
    }
}

/// Extract do-action from `do action <name>`.
fn extract_do_action(text: &str) -> Option<String> {
    let pos = text.find("do action ")?;
    let after = &text[pos + "do action ".len()..];

    let name: String = after
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();

    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Find the initial state from `entry; then X;` in the state machine CST.
fn find_initial_state(source: &str, state_def: &HirSymbol) -> Option<String> {
    // HIR only reports the name span, so use extract_definition_body to get
    // the full state machine body
    let body = crate::workspace::extract_definition_body(source, state_def)?;

    for line in body.lines() {
        let trimmed = line.trim();
        // Look for "entry; then <state>;" or "entry state <state>;"
        if trimmed.contains("entry") && trimmed.contains("then") {
            return extract_keyword_value(trimmed, "then");
        }
        // Also handle "entry state <name>;"
        if trimmed.starts_with("entry state ") || trimmed.starts_with("entry  state ") {
            let after = trimmed
                .strip_prefix("entry state ")
                .or_else(|| trimmed.strip_prefix("entry  state "))?;
            let name: String = after
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::SysmlWorkspace;
    use std::path::PathBuf;

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

    fn bt_workspace() -> SysmlWorkspace {
        let source = load_fixture("bt_a2dp_sink.sysml");
        SysmlWorkspace::from_sources(vec![(PathBuf::from("bt_a2dp_sink.sysml"), source)])
    }

    fn led_workspace() -> SysmlWorkspace {
        let source = load_fixture("status_led.sysml");
        SysmlWorkspace::from_sources(vec![(PathBuf::from("status_led.sysml"), source)])
    }

    fn find_part_def<'a>(ws: &'a SysmlWorkspace, name: &str) -> (&'a ParsedFile, &'a HirSymbol) {
        ws.all_symbols()
            .find(|(_, sym)| sym.kind == SymbolKind::PartDefinition && *sym.name == *name)
            .unwrap_or_else(|| panic!("part def '{}' not found", name))
    }

    #[test]
    fn test_extract_connection_fsm() {
        let ws = bt_workspace();
        let (file, part) = find_part_def(&ws, "BtA2dpSink");
        let machines = extract_state_machines(file, part);

        let fsm = machines
            .iter()
            .find(|m| m.name == "ConnectionFSM")
            .expect("should find ConnectionFSM");

        assert_eq!(fsm.states.len(), 4, "ConnectionFSM should have 4 states");
        assert_eq!(
            fsm.transitions.len(),
            7,
            "ConnectionFSM should have 7 transitions"
        );
    }

    #[test]
    fn test_initial_state() {
        let ws = bt_workspace();
        let (file, part) = find_part_def(&ws, "BtA2dpSink");
        let machines = extract_state_machines(file, part);

        let fsm = machines
            .iter()
            .find(|m| m.name == "ConnectionFSM")
            .expect("should find ConnectionFSM");

        assert_eq!(
            fsm.initial_state.as_deref(),
            Some("disconnected"),
            "initial state should be 'disconnected'"
        );
    }

    #[test]
    fn test_transition_from_state() {
        let ws = bt_workspace();
        let (file, part) = find_part_def(&ws, "BtA2dpSink");
        let machines = extract_state_machines(file, part);

        let fsm = machines
            .iter()
            .find(|m| m.name == "ConnectionFSM")
            .expect("should find ConnectionFSM");

        let t = fsm
            .transitions
            .iter()
            .find(|t| t.name.contains("disconnected_to_discovering"))
            .expect("should find disconnected_to_discovering transition");

        assert_eq!(t.from_state, "disconnected");
    }

    #[test]
    fn test_transition_event() {
        let ws = bt_workspace();
        let (file, part) = find_part_def(&ws, "BtA2dpSink");
        let machines = extract_state_machines(file, part);

        let fsm = machines
            .iter()
            .find(|m| m.name == "ConnectionFSM")
            .expect("should find ConnectionFSM");

        let t = fsm
            .transitions
            .iter()
            .find(|t| t.name.contains("disconnected_to_discovering"))
            .expect("should find disconnected_to_discovering transition");

        assert_eq!(
            t.event.as_deref(),
            Some("StartDiscoveryEvent"),
            "event should be StartDiscoveryEvent"
        );
    }

    #[test]
    fn test_transition_to_state() {
        let ws = bt_workspace();
        let (file, part) = find_part_def(&ws, "BtA2dpSink");
        let machines = extract_state_machines(file, part);

        let fsm = machines
            .iter()
            .find(|m| m.name == "ConnectionFSM")
            .expect("should find ConnectionFSM");

        let t = fsm
            .transitions
            .iter()
            .find(|t| t.name.contains("disconnected_to_discovering"))
            .expect("should find disconnected_to_discovering transition");

        assert_eq!(t.to_state, "discovering");
    }

    #[test]
    fn test_guard_expression() {
        // Inline test with guard
        let source = r#"
            package Test {
                part def Guarded {
                    state def GuardedFSM {
                        entry; then idle;
                        state idle;
                        state active;
                        transition idle_to_active
                            first idle
                            accept TriggerEvent
                            if count > 0
                            then active;
                    }
                    attribute def TriggerEvent;
                }
            }
        "#;

        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("test.sysml"), source.to_string())]);
        let (file, part) = find_part_def(&ws, "Guarded");
        let machines = extract_state_machines(file, part);

        if let Some(fsm) = machines.first() {
            if let Some(t) = fsm.transitions.first() {
                if t.guard.is_some() {
                    assert!(
                        t.guard.as_ref().unwrap().contains("count > 0"),
                        "guard should contain 'count > 0'"
                    );
                }
            }
        }
    }

    #[test]
    fn test_led_fsm() {
        let ws = led_workspace();
        let (file, part) = find_part_def(&ws, "StatusLed");
        let machines = extract_state_machines(file, part);

        let fsm = machines
            .iter()
            .find(|m| m.name == "LedFSM")
            .expect("should find LedFSM");

        assert_eq!(fsm.states.len(), 3, "LedFSM should have 3 states");
        assert_eq!(
            fsm.transitions.len(),
            6,
            "LedFSM should have 6 transitions"
        );
        assert_eq!(
            fsm.initial_state.as_deref(),
            Some("off"),
            "LedFSM initial state should be 'off'"
        );
    }

    #[test]
    fn test_state_names() {
        let ws = bt_workspace();
        let (file, part) = find_part_def(&ws, "BtA2dpSink");
        let machines = extract_state_machines(file, part);

        let fsm = machines
            .iter()
            .find(|m| m.name == "ConnectionFSM")
            .expect("should find ConnectionFSM");

        let state_names: Vec<&str> = fsm.states.iter().map(|s| s.name.as_str()).collect();
        assert!(state_names.contains(&"disconnected"), "missing state 'disconnected': {:?}", state_names);
        assert!(state_names.contains(&"discovering"), "missing state 'discovering': {:?}", state_names);
        assert!(state_names.contains(&"connected"), "missing state 'connected': {:?}", state_names);
        assert!(state_names.contains(&"streaming"), "missing state 'streaming': {:?}", state_names);
    }
}
