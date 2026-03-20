//! Metadata annotation extraction via CST traversal.
//!
//! The HIR identifies metadata usages as `AttributeUsage` symbols with
//! supertypes referencing the metadata def name (e.g. `["MemoryModel"]`).
//! However, the field *values* inside `@M { field = value; }` are not
//! exposed in the HIR — they require CST traversal.
//!
//! This module:
//! 1. Uses HIR to identify which metadata annotations exist on a part
//! 2. Locates the corresponding CST nodes by source span
//! 3. Parses `{ field = value; }` bodies into structured `MetadataValue`s

use syster::hir::HirSymbol;

use crate::workspace::ParsedFile;

/// A metadata annotation extracted from a part definition.
#[derive(Debug, Clone, PartialEq)]
pub struct MetadataAnnotation {
    /// Name of the metadata def (e.g. "MemoryModel").
    pub name: String,
    /// Extracted fields with values.
    pub fields: Vec<MetadataField>,
}

/// A single field within a metadata annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct MetadataField {
    /// Field name (e.g. "allocation").
    pub name: String,
    /// Parsed field value.
    pub value: MetadataValue,
}

/// A parsed metadata field value.
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    /// An enum reference like `AllocationKind::static_alloc`.
    EnumRef {
        enum_type: String,
        variant: String,
    },
    /// A boolean value.
    Boolean(bool),
    /// An integer value.
    Integer(i64),
    /// A string value.
    String(String),
    /// A tuple of values like `("x", "y")`.
    Tuple(Vec<MetadataValue>),
}

/// Extract all metadata annotations from a part definition.
///
/// Finds `@MetadataName { field = value; ... }` annotations in the CST
/// that appear within the body of the given part symbol.
pub fn extract_metadata(file: &ParsedFile, part_symbol: &HirSymbol) -> Vec<MetadataAnnotation> {
    let root = file.parse.syntax();
    let source = root.text().to_string();

    // Use the body extraction utility that finds the matching { ... } block
    let part_text = crate::workspace::extract_definition_body(&source, part_symbol);
    let Some(part_text) = part_text else {
        return Vec::new();
    };

    parse_annotations_from_text(&part_text)
}

/// Extract all metadata annotations from the entire file source.
///
/// Useful when you want annotations at the file/package level, not scoped
/// to a specific part.
pub fn extract_all_metadata(file: &ParsedFile) -> Vec<MetadataAnnotation> {
    let root = file.parse.syntax();
    let source = root.text().to_string();
    parse_annotations_from_text(&source)
}

/// Parse `@Name { field = value; ... }` annotations from a text region.
fn parse_annotations_from_text(text: &str) -> Vec<MetadataAnnotation> {
    let mut annotations = Vec::new();
    let mut chars = text.char_indices().peekable();

    while let Some((i, ch)) = chars.next() {
        if ch == '@' {
            // Found an annotation — parse the name
            let name_start = i + 1;
            let mut name_end = name_start;

            while let Some(&(j, c)) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    name_end = j + c.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }

            let name = text[name_start..name_end].to_string();
            if name.is_empty() {
                continue;
            }

            // Skip whitespace to find '{'
            while let Some(&(_, c)) = chars.peek() {
                if c.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }

            // Parse body if present
            let fields = if chars.peek().map(|&(_, c)| c) == Some('{') {
                chars.next(); // consume '{'
                parse_annotation_body(&text[name_end..], &mut chars)
            } else {
                Vec::new()
            };

            annotations.push(MetadataAnnotation { name, fields });
        }
    }

    annotations
}

/// Parse the body of `{ field = value; field2 = value2; }`.
fn parse_annotation_body(
    _context: &str,
    chars: &mut std::iter::Peekable<std::str::CharIndices>,
) -> Vec<MetadataField> {
    let mut fields = Vec::new();
    let mut current_content = String::new();

    // Collect everything until matching '}'
    let mut depth = 1;
    while let Some(&(_, c)) = chars.peek() {
        chars.next();
        match c {
            '{' => {
                depth += 1;
                current_content.push(c);
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                current_content.push(c);
            }
            _ => {
                current_content.push(c);
            }
        }
    }

    // Parse individual field assignments from the collected content
    for assignment in current_content.split(';') {
        let assignment = assignment.trim();
        if assignment.is_empty() {
            continue;
        }

        if let Some(eq_pos) = assignment.find('=') {
            let field_name = assignment[..eq_pos].trim().to_string();
            let value_str = assignment[eq_pos + 1..].trim();

            if !field_name.is_empty() && !value_str.is_empty() {
                let value = parse_value(value_str);
                fields.push(MetadataField {
                    name: field_name,
                    value,
                });
            }
        }
    }

    fields
}

/// Parse a value string into a `MetadataValue`.
fn parse_value(s: &str) -> MetadataValue {
    let s = s.trim();

    // Boolean
    if s == "true" {
        return MetadataValue::Boolean(true);
    }
    if s == "false" {
        return MetadataValue::Boolean(false);
    }

    // Integer
    if let Ok(n) = s.parse::<i64>() {
        return MetadataValue::Integer(n);
    }

    // Tuple: ( ... )
    if s.starts_with('(') && s.ends_with(')') {
        let inner = &s[1..s.len() - 1];
        let values = parse_tuple_elements(inner);
        return MetadataValue::Tuple(values);
    }

    // String literal: "..."
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        return MetadataValue::String(s[1..s.len() - 1].to_string());
    }

    // Enum reference: EnumType::variant
    if let Some(sep) = s.find("::") {
        let enum_type = s[..sep].to_string();
        let variant = s[sep + 2..].to_string();
        if !enum_type.is_empty() && !variant.is_empty() {
            return MetadataValue::EnumRef {
                enum_type,
                variant,
            };
        }
    }

    // Fallback: treat as string
    MetadataValue::String(s.to_string())
}

/// Parse comma-separated tuple elements.
fn parse_tuple_elements(inner: &str) -> Vec<MetadataValue> {
    if inner.trim().is_empty() {
        return Vec::new();
    }

    inner
        .split(',')
        .map(|elem| parse_value(elem.trim()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::SysmlWorkspace;
    use syster::hir::SymbolKind;
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

    fn find_part_def<'a>(ws: &'a SysmlWorkspace, name: &str) -> (&'a ParsedFile, &'a HirSymbol) {
        ws.all_symbols()
            .find(|(_, sym)| sym.kind == SymbolKind::PartDefinition && *sym.name == *name)
            .unwrap_or_else(|| panic!("part def '{}' not found", name))
    }

    #[test]
    fn test_extract_memory_model() {
        let ws = bt_workspace();
        let (file, part) = find_part_def(&ws, "BtA2dpSink");
        let annotations = extract_metadata(file, part);

        let mm = annotations
            .iter()
            .find(|a| a.name == "MemoryModel")
            .expect("should find @MemoryModel");

        // allocation = AllocationKind::static_alloc
        let alloc = mm
            .fields
            .iter()
            .find(|f| f.name == "allocation" || f.name.ends_with("allocation"))
            .expect("should find allocation field");
        match &alloc.value {
            MetadataValue::EnumRef { enum_type, variant } => {
                assert_eq!(enum_type, "AllocationKind");
                assert_eq!(variant, "static_alloc");
            }
            other => panic!("expected EnumRef, got {:?}", other),
        }

        // maxInstances = 1
        let max = mm
            .fields
            .iter()
            .find(|f| f.name == "maxInstances")
            .expect("should find maxInstances field");
        assert_eq!(max.value, MetadataValue::Integer(1));
    }

    #[test]
    fn test_extract_concurrency_model() {
        let ws = bt_workspace();
        let (file, part) = find_part_def(&ws, "BtA2dpSink");
        let annotations = extract_metadata(file, part);

        let cm = annotations
            .iter()
            .find(|a| a.name == "ConcurrencyModel")
            .expect("should find @ConcurrencyModel");

        // threadSafe = true
        let ts = cm
            .fields
            .iter()
            .find(|f| f.name == "threadSafe")
            .expect("should find threadSafe field");
        assert_eq!(ts.value, MetadataValue::Boolean(true));

        // protection = ProtectionKind::mutex
        let prot = cm
            .fields
            .iter()
            .find(|f| f.name == "protection")
            .expect("should find protection field");
        match &prot.value {
            MetadataValue::EnumRef { variant, .. } => {
                assert_eq!(variant, "mutex");
            }
            other => panic!("expected EnumRef for protection, got {:?}", other),
        }
    }

    #[test]
    fn test_extract_error_handling() {
        let ws = bt_workspace();
        let (file, part) = find_part_def(&ws, "BtA2dpSink");
        let annotations = extract_metadata(file, part);

        let eh = annotations
            .iter()
            .find(|a| a.name == "ErrorHandling")
            .expect("should find @ErrorHandling");

        let strategy = eh
            .fields
            .iter()
            .find(|f| f.name == "strategy")
            .expect("should find strategy field");
        match &strategy.value {
            MetadataValue::EnumRef { variant, .. } => {
                assert_eq!(variant, "result");
            }
            other => panic!("expected EnumRef, got {:?}", other),
        }
    }

    #[test]
    fn test_extract_isr_safe() {
        let ws = bt_workspace();
        let (file, part) = find_part_def(&ws, "BtA2dpSink");
        let annotations = extract_metadata(file, part);

        let isr = annotations
            .iter()
            .find(|a| a.name == "ISRSafe")
            .expect("should find @ISRSafe");

        let safe = isr
            .fields
            .iter()
            .find(|f| f.name == "safe")
            .expect("should find safe field");
        assert_eq!(safe.value, MetadataValue::Boolean(false));
    }

    #[test]
    fn test_extract_ownership() {
        let ws = bt_workspace();
        let (file, part) = find_part_def(&ws, "BtA2dpSink");
        let annotations = extract_metadata(file, part);

        let own = annotations
            .iter()
            .find(|a| a.name == "Ownership")
            .expect("should find @Ownership");

        let owns = own
            .fields
            .iter()
            .find(|f| f.name == "owns")
            .expect("should find owns field");
        match &owns.value {
            MetadataValue::Tuple(values) => {
                assert!(!values.is_empty(), "owns tuple should not be empty");
            }
            other => panic!("expected Tuple for owns, got {:?}", other),
        }
    }

    #[test]
    fn test_missing_metadata() {
        let source = load_fixture("i2s_output.sysml");
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("i2s_output.sysml"), source)]);
        let (file, part) = find_part_def(&ws, "I2sOutput");
        let annotations = extract_metadata(file, part);

        // I2sOutput should NOT have @LayerConstraint
        let lc = annotations.iter().find(|a| a.name == "LayerConstraint");
        assert!(lc.is_none(), "I2sOutput should not have @LayerConstraint");

        // But should have @ISRSafe
        let isr = annotations.iter().find(|a| a.name == "ISRSafe");
        assert!(isr.is_some(), "I2sOutput should have @ISRSafe");
    }

    #[test]
    fn test_enum_ref_value() {
        let value = parse_value("AllocationKind::static_alloc");
        assert_eq!(
            value,
            MetadataValue::EnumRef {
                enum_type: "AllocationKind".to_string(),
                variant: "static_alloc".to_string()
            }
        );
    }

    #[test]
    fn test_boolean_value() {
        assert_eq!(parse_value("true"), MetadataValue::Boolean(true));
        assert_eq!(parse_value("false"), MetadataValue::Boolean(false));
    }

    #[test]
    fn test_integer_value() {
        assert_eq!(parse_value("1"), MetadataValue::Integer(1));
        assert_eq!(parse_value("42"), MetadataValue::Integer(42));
    }

    #[test]
    fn test_tuple_value() {
        let value = parse_value(r#"("Bluetooth controller",)"#);
        match value {
            MetadataValue::Tuple(values) => {
                assert!(!values.is_empty(), "tuple should have elements");
            }
            other => panic!("expected Tuple, got {:?}", other),
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn parse_value_never_panics(s in ".*") {
            // parse_value should handle any input without panicking
            let _ = parse_value(&s);
        }

        #[test]
        fn parse_true_false_is_boolean(b in prop::bool::ANY) {
            let s = if b { "true" } else { "false" };
            prop_assert_eq!(parse_value(s), MetadataValue::Boolean(b));
        }

        #[test]
        fn parse_integer_roundtrips(n in -1000000i64..1000000) {
            let s = n.to_string();
            prop_assert_eq!(
                parse_value(&s),
                MetadataValue::Integer(n),
                "parse_value('{}') should be Integer({})",
                s, n,
            );
        }

        #[test]
        fn parse_quoted_string_unquotes(s in "[a-zA-Z0-9 ]{0,30}") {
            let quoted = format!("\"{}\"", s);
            match parse_value(&quoted) {
                MetadataValue::String(inner) => {
                    prop_assert_eq!(inner, s, "quoted string not unquoted properly");
                }
                other => prop_assert!(false, "expected String, got {:?}", other),
            }
        }

        #[test]
        fn parse_enum_ref_format(
            t in "[A-Z][a-zA-Z]{1,10}",
            v in "[a-z][a-zA-Z]{1,10}"
        ) {
            let s = format!("{}::{}", t, v);
            match parse_value(&s) {
                MetadataValue::EnumRef { enum_type, variant } => {
                    prop_assert_eq!(enum_type, t);
                    prop_assert_eq!(variant, v);
                }
                other => prop_assert!(false, "expected EnumRef for '{}', got {:?}", s, other),
            }
        }

        #[test]
        fn parse_empty_tuple(s in prop::string::string_regex(r"\(\s*\)").unwrap()) {
            match parse_value(&s) {
                MetadataValue::Tuple(vals) => {
                    prop_assert!(vals.is_empty(), "empty tuple should have no elements");
                }
                other => prop_assert!(false, "expected Tuple for '{}', got {:?}", s, other),
            }
        }

        #[test]
        fn parse_whitespace_trimmed(s in " {0,5}(true|false|42) {0,5}") {
            // parse_value trims whitespace, so " true " should equal "true"
            let result = parse_value(&s);
            let trimmed_result = parse_value(s.trim());
            prop_assert_eq!(
                result, trimmed_result,
                "whitespace should not affect result: '{}'",
                s,
            );
        }
    }
}
