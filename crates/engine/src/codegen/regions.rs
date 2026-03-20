//! Protected user code regions.
//!
//! Generated files use `// BEGIN USER CODE <id>` / `// END USER CODE <id>` markers.
//! On regeneration, content inside markers is preserved from the existing file.
//! Content outside markers is regenerated from templates.

use std::collections::HashMap;

const BEGIN_MARKER: &str = "// BEGIN USER CODE ";
const END_MARKER: &str = "// END USER CODE ";

/// Extract user code regions from an existing generated file.
///
/// Returns `region_id → content` where content is the lines between
/// markers (excluding the marker lines themselves).
pub(crate) fn extract_user_regions(file_content: &str) -> HashMap<String, String> {
    let mut regions = HashMap::new();
    let mut current_id: Option<String> = None;
    let mut current_lines: Vec<&str> = Vec::new();

    for line in file_content.lines() {
        let trimmed = line.trim();

        if let Some(id) = trimmed.strip_prefix(BEGIN_MARKER) {
            current_id = Some(id.trim().to_string());
            current_lines.clear();
        } else if let Some(id) = trimmed.strip_prefix(END_MARKER) {
            if let Some(ref begin_id) = current_id {
                if begin_id == id.trim() {
                    regions.insert(begin_id.clone(), current_lines.join("\n"));
                }
            }
            current_id = None;
            current_lines.clear();
        } else if current_id.is_some() {
            current_lines.push(line);
        }
    }

    regions
}

/// Merge preserved user code into freshly rendered output.
///
/// For each `BEGIN USER CODE <id>` / `END USER CODE <id>` pair in `rendered`,
/// if `regions` has a matching key, replace the default content between the
/// markers with the preserved content. If no match, keep the template default.
pub(crate) fn merge_user_regions(
    rendered: &str,
    regions: &HashMap<String, String>,
) -> String {
    let mut result = Vec::new();
    let mut skip_until_end: Option<String> = None;

    for line in rendered.lines() {
        let trimmed = line.trim();

        if let Some(id) = trimmed.strip_prefix(BEGIN_MARKER) {
            let id = id.trim().to_string();
            result.push(line.to_string());

            if let Some(preserved) = regions.get(&id) {
                // Emit preserved user code
                result.push(preserved.to_string());
                // Skip template default lines until END marker
                skip_until_end = Some(id);
            }
        } else if let Some(id) = trimmed.strip_prefix(END_MARKER) {
            let id = id.trim();
            if skip_until_end.as_deref() == Some(id) {
                skip_until_end = None;
            }
            result.push(line.to_string());
        } else if skip_until_end.is_none() {
            result.push(line.to_string());
        }
        // else: skip template default line (replaced by preserved content)
    }

    let mut output = result.join("\n");
    // Preserve trailing newline if original had one
    if rendered.ends_with('\n') && !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

#[allow(dead_code)] // Available for CLI status reporting
/// Check if a file contains any user code regions with non-default content.
///
/// Returns true if any region has content that isn't just whitespace or `todo!()`.
pub(crate) fn has_user_modifications(file_content: &str) -> bool {
    let regions = extract_user_regions(file_content);
    regions.values().any(|content| {
        let trimmed = content.trim();
        !trimmed.is_empty()
            && !trimmed.starts_with("todo!(")
            && trimmed != "todo!()"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_empty_file() {
        let regions = extract_user_regions("");
        assert!(regions.is_empty());
    }

    #[test]
    fn test_extract_no_regions() {
        let content = "fn main() {}\n";
        let regions = extract_user_regions(content);
        assert!(regions.is_empty());
    }

    #[test]
    fn test_extract_single_region() {
        let content = "\
pub fn init(&self) {
    // BEGIN USER CODE init
    self.hardware.setup();
    self.state = State::Ready;
    // END USER CODE init
}";
        let regions = extract_user_regions(content);
        assert_eq!(regions.len(), 1);
        let init = regions.get("init").unwrap();
        assert!(init.contains("self.hardware.setup()"));
        assert!(init.contains("self.state = State::Ready;"));
    }

    #[test]
    fn test_extract_multiple_regions() {
        let content = "\
    // BEGIN USER CODE fields
    count: u32,
    name: String,
    // END USER CODE fields

    // BEGIN USER CODE init
    self.count = 0;
    // END USER CODE init";
        let regions = extract_user_regions(content);
        assert_eq!(regions.len(), 2);
        assert!(regions.get("fields").unwrap().contains("count: u32"));
        assert!(regions.get("init").unwrap().contains("self.count = 0"));
    }

    #[test]
    fn test_extract_nested_content() {
        let content = "\
    // BEGIN USER CODE complex
    if x > 0 {
        for i in 0..x {
            println!(\"{}\", i);
        }
    }
    // END USER CODE complex";
        let regions = extract_user_regions(content);
        let complex = regions.get("complex").unwrap();
        assert!(complex.contains("if x > 0 {"));
        assert!(complex.contains("for i in 0..x {"));
    }

    #[test]
    fn test_merge_preserves_user_code() {
        let rendered = "\
pub fn init(&self) {
    // BEGIN USER CODE init
    todo!(\"implement init\")
    // END USER CODE init
}";
        let mut regions = HashMap::new();
        regions.insert(
            "init".to_string(),
            "    self.hardware.setup();\n    Ok(())".to_string(),
        );

        let merged = merge_user_regions(rendered, &regions);
        assert!(merged.contains("self.hardware.setup()"));
        assert!(merged.contains("Ok(())"));
        assert!(!merged.contains("todo!"));
        // Markers still present
        assert!(merged.contains("// BEGIN USER CODE init"));
        assert!(merged.contains("// END USER CODE init"));
    }

    #[test]
    fn test_merge_new_region() {
        let rendered = "\
    // BEGIN USER CODE new_action
    todo!(\"implement new_action\")
    // END USER CODE new_action";
        let regions = HashMap::new(); // no existing regions

        let merged = merge_user_regions(rendered, &regions);
        assert!(merged.contains("todo!"), "new region should keep template default");
    }

    #[test]
    fn test_merge_removed_region() {
        let rendered = "pub fn only_this() {}\n";
        let mut regions = HashMap::new();
        regions.insert("old_action".to_string(), "old code here".to_string());

        let merged = merge_user_regions(rendered, &regions);
        assert!(!merged.contains("old code here"), "removed region should be dropped");
    }

    #[test]
    fn test_has_user_modifications_todo() {
        let content = "\
    // BEGIN USER CODE init
    todo!(\"implement init\")
    // END USER CODE init";
        assert!(!has_user_modifications(content));
    }

    #[test]
    fn test_has_user_modifications_real() {
        let content = "\
    // BEGIN USER CODE init
    self.setup();
    // END USER CODE init";
        assert!(has_user_modifications(content));
    }

    #[test]
    fn test_has_user_modifications_empty() {
        let content = "\
    // BEGIN USER CODE init
    // END USER CODE init";
        assert!(!has_user_modifications(content));
    }

    #[test]
    fn test_round_trip() {
        let original = "\
pub struct Foo {
    // BEGIN USER CODE foo_fields
    count: u32,
    name: String,
    // END USER CODE foo_fields
}

impl Foo {
    pub fn bar(&self) {
        // BEGIN USER CODE bar
        println!(\"hello\");
        // END USER CODE bar
    }
}";
        let regions = extract_user_regions(original);
        let merged = merge_user_regions(original, &regions);
        assert_eq!(original, merged, "round-trip should preserve content exactly");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn extract_never_panics(s in ".*") {
            let _ = extract_user_regions(&s);
        }

        #[test]
        fn merge_never_panics(s in ".*") {
            let regions = HashMap::new();
            let _ = merge_user_regions(&s, &regions);
        }

        #[test]
        fn extract_then_merge_is_idempotent(
            id in "[a-z_]{1,20}",
            body in "[a-zA-Z0-9 _;(){}\n]{0,100}"
        ) {
            let content = format!(
                "before\n    // BEGIN USER CODE {id}\n{body}\n    // END USER CODE {id}\nafter"
            );
            let regions = extract_user_regions(&content);
            let merged = merge_user_regions(&content, &regions);
            prop_assert_eq!(content, merged, "extract→merge should be identity");
        }
    }
}
