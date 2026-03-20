//! Comparison logic: spec (ExtractedModule) vs code (CodeConstructs).
//!
//! Produces a list of AuditItems describing matches, mismatches,
//! missing items, and uncovered code.

use crate::extraction::{ExtractedModule, ParameterDirection};
use crate::util::snake_case;

use super::code_parser::{CodeConstruct, ConstructKind};
use super::AuditItem;

/// Compare a spec module against parsed code constructs.
pub fn compare_module(
    module: &ExtractedModule,
    code: &[CodeConstruct],
    show_uncovered: bool,
) -> Vec<AuditItem> {
    let mut items = Vec::new();
    let mut matched_code_indices: Vec<bool> = vec![false; code.len()];

    // Compare: module struct
    let struct_name = &module.name;
    if let Some((idx, _)) = code
        .iter()
        .enumerate()
        .find(|(_, c)| c.kind == ConstructKind::Struct && c.name == *struct_name)
    {
        items.push(AuditItem::Match {
            kind: "struct".to_string(),
            name: struct_name.clone(),
        });
        matched_code_indices[idx] = true;
    } else {
        items.push(AuditItem::Missing {
            kind: "struct".to_string(),
            name: struct_name.clone(),
            detail: format!("struct {} not found in source", struct_name),
        });
    }

    // Compare: spec actions → code functions
    for action in &module.actions {
        let action_snake = snake_case(&action.name);

        // Find matching function by name (try both original and snake_case)
        let found = code.iter().enumerate().find(|(_, c)| {
            c.kind == ConstructKind::Function
                && (c.name == action.name || c.name == action_snake)
        });

        if let Some((idx, func)) = found {
            matched_code_indices[idx] = true;

            // Compare parameters
            let spec_params: Vec<String> = action
                .parameters
                .iter()
                .map(|p| {
                    let dir = match p.direction {
                        ParameterDirection::In => "",
                        ParameterDirection::Out => "-> ",
                    };
                    format!("{}{}: {}", dir, p.name, p.type_name)
                })
                .collect();

            let code_params: Vec<String> = func
                .parameters
                .iter()
                .map(|p| {
                    if p.type_name.is_empty() {
                        p.name.clone()
                    } else {
                        format!("{}: {}", p.name, p.type_name)
                    }
                })
                .collect();

            // Simple parameter count comparison (exact matching is language-dependent)
            let spec_non_self: Vec<_> = action
                .parameters
                .iter()
                .filter(|p| p.name != "self")
                .collect();
            let code_non_self: Vec<_> = func
                .parameters
                .iter()
                .filter(|p| p.name != "self")
                .collect();

            if spec_non_self.len() == code_non_self.len() {
                items.push(AuditItem::Match {
                    kind: "action".to_string(),
                    name: action.name.clone(),
                });
            } else {
                items.push(AuditItem::Mismatch {
                    kind: "action".to_string(),
                    name: action.name.clone(),
                    spec_detail: format!("({})", spec_params.join(", ")),
                    code_detail: format!("({})", code_params.join(", ")),
                });
            }
        } else {
            items.push(AuditItem::Missing {
                kind: "action".to_string(),
                name: action.name.clone(),
                detail: format!(
                    "fn {} not found in source (tried: {}, {})",
                    action.name, action.name, action_snake
                ),
            });
        }
    }

    // Compare: spec state machines → code enums
    for fsm in &module.state_machines {
        // Look for state enum
        let state_enum_name = format!("{}State", module.name);
        let alt_state_enum = format!("{}State", fsm.name.replace("FSM", ""));

        let state_enum = code.iter().enumerate().find(|(_, c)| {
            c.kind == ConstructKind::Enum
                && (c.name == state_enum_name
                    || c.name == alt_state_enum
                    || c.name.ends_with("State"))
        });

        if let Some((idx, enum_construct)) = state_enum {
            matched_code_indices[idx] = true;

            // Check if all states are represented as variants
            let missing_states: Vec<_> = fsm
                .states
                .iter()
                .filter(|state| {
                    !enum_construct.variants.iter().any(|v| {
                        v.to_lowercase() == state.to_lowercase()
                            || snake_case(v) == snake_case(state)
                    })
                })
                .collect();

            if missing_states.is_empty() {
                items.push(AuditItem::Match {
                    kind: "state_machine".to_string(),
                    name: fsm.name.clone(),
                });
            } else {
                items.push(AuditItem::Mismatch {
                    kind: "state_machine".to_string(),
                    name: fsm.name.clone(),
                    spec_detail: format!("states: [{}]", fsm.states.join(", ")),
                    code_detail: format!(
                        "enum {} missing: [{}]",
                        enum_construct.name,
                        missing_states
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                });
            }
        } else {
            items.push(AuditItem::Missing {
                kind: "state_machine".to_string(),
                name: fsm.name.clone(),
                detail: format!(
                    "no state enum found (tried: {}, {})",
                    state_enum_name, alt_state_enum
                ),
            });
        }
    }

    // Report uncovered code items
    if show_uncovered {
        for (idx, construct) in code.iter().enumerate() {
            if !matched_code_indices[idx] {
                let kind = match construct.kind {
                    ConstructKind::Function => "function",
                    ConstructKind::Struct => "struct",
                    ConstructKind::Enum => "enum",
                    ConstructKind::ImplBlock => "impl",
                };
                items.push(AuditItem::Uncovered {
                    kind: kind.to_string(),
                    name: construct.name.clone(),
                    line: construct.line,
                });
            }
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extraction::{ActionParameter, ExtractedAction, ExtractedStateMachine};
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::super::code_parser::ParsedParameter;

    fn test_module() -> ExtractedModule {
        ExtractedModule {
            name: "BtA2dpSink".to_string(),
            qualified_name: "Firmware::BtA2dpSink".to_string(),
            source_file: PathBuf::from("test.sysml"),
            layer: Some("driver".to_string()),
            metadata: HashMap::new(),
            ports: Vec::new(),
            actions: vec![
                ExtractedAction {
                    name: "Init".to_string(),
                    parameters: vec![
                        ActionParameter {
                            name: "config".to_string(),
                            type_name: "A2dpConfig".to_string(),
                            direction: ParameterDirection::In,
                        },
                        ActionParameter {
                            name: "result".to_string(),
                            type_name: "BtA2dpSink".to_string(),
                            direction: ParameterDirection::Out,
                        },
                    ],
                },
                ExtractedAction {
                    name: "Start".to_string(),
                    parameters: vec![ActionParameter {
                        name: "self".to_string(),
                        type_name: "BtA2dpSink".to_string(),
                        direction: ParameterDirection::In,
                    }],
                },
            ],
            connections: Vec::new(),
            state_machines: vec![ExtractedStateMachine {
                name: "ConnectionFSM".to_string(),
                initial_state: Some("disconnected".to_string()),
                states: vec![
                    "disconnected".to_string(),
                    "discovering".to_string(),
                    "connected".to_string(),
                    "streaming".to_string(),
                ],
                transitions: Vec::new(),
            }],
        }
    }

    fn matching_code() -> Vec<CodeConstruct> {
        vec![
            CodeConstruct {
                kind: ConstructKind::Struct,
                name: "BtA2dpSink".to_string(),
                parameters: Vec::new(),
                fields: vec!["config".to_string(), "state".to_string()],
                variants: Vec::new(),
                line: 1,
            },
            CodeConstruct {
                kind: ConstructKind::Function,
                name: "init".to_string(),
                parameters: vec![ParsedParameter {
                    name: "config".to_string(),
                    type_name: "A2dpConfig".to_string(),
                }],
                fields: Vec::new(),
                variants: Vec::new(),
                line: 5,
            },
            CodeConstruct {
                kind: ConstructKind::Function,
                name: "start".to_string(),
                parameters: vec![ParsedParameter {
                    name: "self".to_string(),
                    type_name: "&mut Self".to_string(),
                }],
                fields: Vec::new(),
                variants: Vec::new(),
                line: 10,
            },
            CodeConstruct {
                kind: ConstructKind::Enum,
                name: "ConnectionState".to_string(),
                parameters: Vec::new(),
                fields: Vec::new(),
                variants: vec![
                    "Disconnected".to_string(),
                    "Discovering".to_string(),
                    "Connected".to_string(),
                    "Streaming".to_string(),
                ],
                line: 15,
            },
        ]
    }

    #[test]
    fn test_compare_match() {
        let module = test_module();
        let code = matching_code();
        let items = compare_module(&module, &code, false);

        let matches: Vec<_> = items
            .iter()
            .filter(|i| matches!(i, AuditItem::Match { .. }))
            .collect();
        assert!(
            matches.len() >= 3,
            "should have at least 3 matches (struct, 2 actions), got: {:?}",
            items
        );
    }

    #[test]
    fn test_compare_missing() {
        let module = test_module();
        let code = vec![]; // empty code
        let items = compare_module(&module, &code, false);

        let missing: Vec<_> = items
            .iter()
            .filter(|i| matches!(i, AuditItem::Missing { .. }))
            .collect();
        // Should be missing: struct + 2 actions + 1 state machine = 4
        assert_eq!(missing.len(), 4, "items: {:?}", items);
    }

    #[test]
    fn test_compare_mismatch() {
        let module = test_module();
        let code = vec![
            CodeConstruct {
                kind: ConstructKind::Struct,
                name: "BtA2dpSink".to_string(),
                parameters: Vec::new(),
                fields: Vec::new(),
                variants: Vec::new(),
                line: 1,
            },
            // Init function with wrong param count
            CodeConstruct {
                kind: ConstructKind::Function,
                name: "init".to_string(),
                parameters: Vec::new(), // no params → mismatch
                fields: Vec::new(),
                variants: Vec::new(),
                line: 5,
            },
        ];
        let items = compare_module(&module, &code, false);

        let mismatches: Vec<_> = items
            .iter()
            .filter(|i| matches!(i, AuditItem::Mismatch { .. }))
            .collect();
        assert!(
            !mismatches.is_empty(),
            "should have mismatches, got: {:?}",
            items
        );
    }

    #[test]
    fn test_compare_uncovered() {
        let mut module = test_module();
        module.actions.clear();
        module.state_machines.clear();

        let code = vec![
            CodeConstruct {
                kind: ConstructKind::Struct,
                name: "BtA2dpSink".to_string(),
                parameters: Vec::new(),
                fields: Vec::new(),
                variants: Vec::new(),
                line: 1,
            },
            CodeConstruct {
                kind: ConstructKind::Function,
                name: "extra_function".to_string(),
                parameters: Vec::new(),
                fields: Vec::new(),
                variants: Vec::new(),
                line: 20,
            },
        ];
        let items = compare_module(&module, &code, true);

        let uncovered: Vec<_> = items
            .iter()
            .filter(|i| matches!(i, AuditItem::Uncovered { .. }))
            .collect();
        assert_eq!(
            uncovered.len(),
            1,
            "extra_function should be uncovered, got: {:?}",
            items
        );
    }

    #[test]
    fn test_compare_empty_spec_all_uncovered() {
        let module = ExtractedModule {
            name: "Empty".to_string(),
            qualified_name: "Firmware::Empty".to_string(),
            source_file: PathBuf::from("test.sysml"),
            layer: None,
            metadata: HashMap::new(),
            ports: Vec::new(),
            actions: Vec::new(),
            connections: Vec::new(),
            state_machines: Vec::new(),
        };

        let code = vec![CodeConstruct {
            kind: ConstructKind::Function,
            name: "something".to_string(),
            parameters: Vec::new(),
            fields: Vec::new(),
            variants: Vec::new(),
            line: 1,
        }];

        let items = compare_module(&module, &code, true);
        let uncovered: Vec<_> = items
            .iter()
            .filter(|i| matches!(i, AuditItem::Uncovered { .. }))
            .collect();
        assert_eq!(uncovered.len(), 1);
    }

    #[test]
    fn test_compare_empty_code_all_missing() {
        let module = test_module();
        let items = compare_module(&module, &[], false);
        let missing: Vec<_> = items
            .iter()
            .filter(|i| matches!(i, AuditItem::Missing { .. }))
            .collect();
        assert!(!missing.is_empty());
    }
}
