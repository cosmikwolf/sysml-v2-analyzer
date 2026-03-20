//! Standard template filters for code generation.
//!
//! Registered on every MiniJinja environment.

use std::collections::HashMap;

/// Convert a name to `snake_case`.
///
/// `"AudioPipeline"` → `"audio_pipeline"`
/// `"BtA2dpSink"` → `"bt_a2dp_sink"`
pub fn snake_case(value: &str) -> String {
    let mut result = String::with_capacity(value.len() + 4);
    let mut prev_was_upper = false;
    let mut prev_was_separator = true; // treat start as separator

    for (i, ch) in value.chars().enumerate() {
        if ch == '_' || ch == '-' || ch == ' ' {
            // Collapse consecutive separators, skip leading separators
            if !result.is_empty() && !prev_was_separator {
                result.push('_');
            }
            prev_was_upper = false;
            prev_was_separator = true;
            continue;
        }

        if ch.is_uppercase() {
            if !prev_was_separator && i > 0 {
                let next_is_lower = value.chars().nth(i + 1).is_some_and(|c| c.is_lowercase());
                if !prev_was_upper || next_is_lower {
                    result.push('_');
                }
            }
            result.push(ch.to_lowercase().next().unwrap());
            prev_was_upper = true;
        } else {
            result.push(ch);
            prev_was_upper = false;
        }
        prev_was_separator = false;
    }

    // Remove trailing underscore
    if result.ends_with('_') {
        result.pop();
    }

    result
}

/// Convert a name to `PascalCase`.
///
/// `"audio_pipeline"` → `"AudioPipeline"`
/// `"bt_a2dp_sink"` → `"BtA2dpSink"`
pub fn pascal_case(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut capitalize_next = true;

    for ch in value.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            capitalize_next = true;
            continue;
        }

        if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}

/// Convert a name to `SCREAMING_SNAKE_CASE`.
///
/// `"AudioPipeline"` → `"AUDIO_PIPELINE"`
pub fn screaming_snake(value: &str) -> String {
    snake_case(value).to_uppercase()
}

/// Map a SysML type name to a target language type.
///
/// Uses the provided type map. Falls through to the original name if not mapped.
pub fn map_type(value: &str, type_map: &HashMap<String, String>) -> String {
    type_map
        .get(value)
        .cloned()
        .unwrap_or_else(|| value.to_string())
}

/// Register all standard filters on a MiniJinja environment.
pub fn register_filters(env: &mut minijinja::Environment<'_>) {
    env.add_filter("snake_case", |value: String| -> String {
        snake_case(&value)
    });
    env.add_filter("pascal_case", |value: String| -> String {
        pascal_case(&value)
    });
    env.add_filter("screaming_snake", |value: String| -> String {
        screaming_snake(&value)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case() {
        assert_eq!(snake_case("AudioPipeline"), "audio_pipeline");
        assert_eq!(snake_case("BtA2dpSink"), "bt_a2dp_sink");
        assert_eq!(snake_case("I2sOutput"), "i2s_output");
        assert_eq!(snake_case("StatusLed"), "status_led");
        assert_eq!(snake_case("already_snake"), "already_snake");
        assert_eq!(snake_case("FSMState"), "fsm_state");
        assert_eq!(snake_case("A"), "a");
        assert_eq!(snake_case(""), "");
    }

    #[test]
    fn test_pascal_case() {
        assert_eq!(pascal_case("audio_pipeline"), "AudioPipeline");
        assert_eq!(pascal_case("bt_a2dp_sink"), "BtA2dpSink");
        assert_eq!(pascal_case("status_led"), "StatusLed");
        assert_eq!(pascal_case("AlreadyPascal"), "AlreadyPascal");
        assert_eq!(pascal_case("a"), "A");
        assert_eq!(pascal_case(""), "");
    }

    #[test]
    fn test_screaming_snake() {
        assert_eq!(screaming_snake("AudioPipeline"), "AUDIO_PIPELINE");
        assert_eq!(screaming_snake("BtA2dpSink"), "BT_A2DP_SINK");
        assert_eq!(screaming_snake("StatusLed"), "STATUS_LED");
    }

    #[test]
    fn test_map_type_known() {
        let mut type_map = HashMap::new();
        type_map.insert("Integer".to_string(), "i32".to_string());
        type_map.insert("Boolean".to_string(), "bool".to_string());

        assert_eq!(map_type("Integer", &type_map), "i32");
        assert_eq!(map_type("Boolean", &type_map), "bool");
    }

    // Note: test_map_type_unknown removed — covered by proptest map_type_passthrough_when_not_in_map.
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Strategy for generating realistic identifier strings.
    fn identifier() -> impl Strategy<Value = String> {
        prop::string::string_regex("[A-Za-z][A-Za-z0-9_]{0,30}")
            .unwrap()
    }

    proptest! {
        // ── snake_case properties ──

        #[test]
        fn snake_case_has_no_uppercase(s in identifier()) {
            let result = snake_case(&s);
            prop_assert!(
                !result.chars().any(|c| c.is_uppercase()),
                "snake_case('{}') = '{}' contains uppercase",
                s, result,
            );
        }

        #[test]
        fn snake_case_is_idempotent(s in identifier()) {
            let once = snake_case(&s);
            let twice = snake_case(&once);
            prop_assert_eq!(
                &once, &twice,
                "snake_case is not idempotent: snake_case('{}') = '{}', snake_case('{}') = '{}'",
                s, once, once, twice,
            );
        }

        #[test]
        fn snake_case_no_double_underscores(s in identifier()) {
            let result = snake_case(&s);
            prop_assert!(
                !result.contains("__"),
                "snake_case('{}') = '{}' contains double underscores",
                s, result,
            );
        }

        #[test]
        fn snake_case_no_leading_underscore(s in identifier()) {
            let result = snake_case(&s);
            prop_assert!(
                !result.starts_with('_'),
                "snake_case('{}') = '{}' starts with underscore",
                s, result,
            );
        }

        #[test]
        fn snake_case_preserves_emptiness(s in ".*") {
            let result = snake_case(&s);
            if s.is_empty() {
                prop_assert!(result.is_empty());
            }
        }

        // ── pascal_case properties ──

        #[test]
        fn pascal_case_is_idempotent(s in identifier()) {
            let once = pascal_case(&s);
            let twice = pascal_case(&once);
            prop_assert_eq!(
                &once, &twice,
                "pascal_case not idempotent: pascal_case('{}') = '{}', pascal_case('{}') = '{}'",
                s, once, once, twice,
            );
        }

        #[test]
        fn pascal_case_starts_uppercase(s in identifier()) {
            let result = pascal_case(&s);
            if !result.is_empty() {
                prop_assert!(
                    result.chars().next().unwrap().is_uppercase(),
                    "pascal_case('{}') = '{}' does not start with uppercase",
                    s, result,
                );
            }
        }

        #[test]
        fn pascal_case_no_underscores(s in identifier()) {
            let result = pascal_case(&s);
            prop_assert!(
                !result.contains('_'),
                "pascal_case('{}') = '{}' contains underscores",
                s, result,
            );
        }

        // ── screaming_snake properties ──

        #[test]
        fn screaming_snake_all_uppercase(s in identifier()) {
            let result = screaming_snake(&s);
            prop_assert!(
                !result.chars().any(|c| c.is_lowercase()),
                "screaming_snake('{}') = '{}' contains lowercase",
                s, result,
            );
        }

        #[test]
        fn screaming_snake_equals_snake_uppercased(s in identifier()) {
            let result = screaming_snake(&s);
            let expected = snake_case(&s).to_uppercase();
            prop_assert_eq!(
                result, expected,
                "screaming_snake('{}') should equal snake_case uppercased",
                s,
            );
        }

        // ── Cross-function properties ──

        #[test]
        fn snake_then_pascal_preserves_alpha_content(s in identifier()) {
            // The alphabetic characters should be preserved (case may change)
            let snake = snake_case(&s);
            let pascal = pascal_case(&snake);
            let snake_alpha: String = snake.chars().filter(|c| c.is_alphanumeric()).collect();
            let pascal_alpha: String = pascal.chars().filter(|c| c.is_alphanumeric()).collect();
            prop_assert_eq!(
                snake_alpha.to_lowercase(),
                pascal_alpha.to_lowercase(),
                "round-trip snake→pascal lost characters for '{}'",
                s,
            );
        }

        // ── map_type properties ──

        #[test]
        fn map_type_passthrough_when_not_in_map(s in identifier()) {
            let empty_map = HashMap::new();
            prop_assert_eq!(map_type(&s, &empty_map), s);
        }
    }
}
