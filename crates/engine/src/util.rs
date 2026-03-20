//! Shared utilities for the engine crate.

use std::collections::HashSet;

use sysml_v2_adapter::workspace::extract_definition_body;
use sysml_v2_adapter::{HirSymbol, ParsedFile};

/// Convert a PascalCase or camelCase identifier to snake_case.
pub fn snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                // Don't insert underscore between consecutive uppercase letters
                // unless followed by a lowercase letter (e.g., "FSMState" → "fsm_state")
                let prev_upper = s.as_bytes().get(i.wrapping_sub(1)).is_some_and(|b| b.is_ascii_uppercase());
                let next_lower = s.as_bytes().get(i + 1).is_some_and(|b| b.is_ascii_lowercase());
                if !prev_upper || next_lower {
                    result.push('_');
                }
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Map a language name to its file extension.
pub fn language_extension(language: &str) -> &str {
    match language {
        "rust" => "rs",
        "c" => "c",
        "cpp" | "c++" => "cpp",
        _ => language,
    }
}

/// Extract the layer value for a part definition.
///
/// Domain-agnostic: scans the part's body text for `::layer_name` where
/// `layer_name` matches one of the configured layer names.
pub(crate) fn extract_layer_for_part(
    file: &ParsedFile,
    part_symbol: &HirSymbol,
    known_layers: &HashSet<String>,
) -> Option<String> {
    let body = extract_definition_body(&file.source, part_symbol)?;

    // Look for the first `::layer_name` where layer_name is a known layer.
    // We only scan the top-level body (not nested state def blocks), so we
    // search for the pattern before any nested `state def` or `part def`.
    let search_region = if let Some(pos) = body.find("state def") {
        &body[..pos]
    } else {
        &body
    };

    for layer_name in known_layers {
        let pattern = format!("::{}", layer_name);
        if search_region.contains(&pattern) {
            return Some(layer_name.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case_pascal() {
        assert_eq!(snake_case("BtA2dpSink"), "bt_a2dp_sink");
    }

    #[test]
    fn test_snake_case_simple() {
        assert_eq!(snake_case("AudioPipeline"), "audio_pipeline");
    }

    #[test]
    fn test_snake_case_acronym() {
        assert_eq!(snake_case("FSMState"), "fsm_state");
    }

    #[test]
    fn test_snake_case_already_snake() {
        assert_eq!(snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn test_snake_case_single_word() {
        assert_eq!(snake_case("Status"), "status");
    }

    #[test]
    fn test_snake_case_empty() {
        assert_eq!(snake_case(""), "");
    }

    #[test]
    fn test_language_extension() {
        assert_eq!(language_extension("rust"), "rs");
        assert_eq!(language_extension("c"), "c");
        assert_eq!(language_extension("cpp"), "cpp");
    }
}
