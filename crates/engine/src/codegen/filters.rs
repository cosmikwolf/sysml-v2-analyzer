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
            result.push('_');
            prev_was_upper = false;
            prev_was_separator = true;
            continue;
        }

        if ch.is_uppercase() {
            // Insert underscore before uppercase if:
            // - not at start
            // - previous char was lowercase, OR
            // - previous was uppercase but next is lowercase (e.g. "FSM" → "fs_m" is wrong,
            //   but "FSMState" → "fsm_state" needs underscore before "State")
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

    #[test]
    fn test_map_type_unknown() {
        let type_map = HashMap::new();
        assert_eq!(map_type("CustomType", &type_map), "CustomType");
    }
}
