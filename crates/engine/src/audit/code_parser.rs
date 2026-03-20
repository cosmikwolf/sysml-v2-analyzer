//! Tree-sitter based source code parser for audit comparison.
//!
//! Parses source files using tree-sitter grammars and query patterns
//! to extract structural constructs (functions, structs, enums, impl blocks).

use std::path::Path;

use tree_sitter::StreamingIterator;

use super::AuditError;

/// A structural construct extracted from source code.
#[derive(Debug, Clone, PartialEq)]
pub struct CodeConstruct {
    pub kind: ConstructKind,
    pub name: String,
    pub parameters: Vec<ParsedParameter>,
    pub fields: Vec<String>,
    pub variants: Vec<String>,
    pub line: usize,
}

/// The kind of a source code construct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructKind {
    Function,
    Struct,
    Enum,
    ImplBlock,
}

/// A parsed function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedParameter {
    pub name: String,
    pub type_name: String,
}

/// Parse source code using tree-sitter and query patterns.
///
/// Loads the grammar based on `language` (compiled-in via tree-sitter-rust/tree-sitter-c).
/// Loads query patterns from `queries_dir/<language>/audit.scm`.
/// Returns list of constructs found in the source.
pub fn parse_source(
    source: &str,
    language: &str,
    queries_dir: &Path,
) -> Result<Vec<CodeConstruct>, AuditError> {
    let ts_language = match language {
        "rust" => tree_sitter_rust::LANGUAGE,
        "c" => tree_sitter_c::LANGUAGE,
        other => {
            return Err(AuditError::UnsupportedLanguage(other.to_string()));
        }
    };

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&ts_language.into())
        .map_err(|e| AuditError::TreeSitter(format!("failed to set language: {e}")))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| AuditError::TreeSitter("parse returned None".to_string()))?;

    let query_path = queries_dir.join(language).join("audit.scm");
    let query_source = std::fs::read_to_string(&query_path).map_err(|e| {
        AuditError::Io(std::io::Error::new(
            e.kind(),
            format!("failed to read query file {}: {}", query_path.display(), e),
        ))
    })?;

    let query = tree_sitter::Query::new(&ts_language.into(), &query_source).map_err(|e| {
        AuditError::TreeSitter(format!(
            "failed to compile query {}: {}",
            query_path.display(),
            e
        ))
    })?;

    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut constructs = Vec::new();

    while let Some(m) = matches.next() {
        if let Some(construct) = match_to_construct(&query, m, source, language) {
            constructs.push(construct);
        }
    }

    Ok(constructs)
}

/// Convert a tree-sitter query match to a CodeConstruct.
fn match_to_construct(
    query: &tree_sitter::Query,
    m: &tree_sitter::QueryMatch,
    source: &str,
    language: &str,
) -> Option<CodeConstruct> {
    // Determine construct kind from capture names
    let capture_names: Vec<&str> = m
        .captures
        .iter()
        .map(|c| *query.capture_names().get(c.index as usize).unwrap())
        .collect();

    if capture_names.contains(&"fn.def") {
        return parse_function_match(query, m, source, language);
    }
    if capture_names.contains(&"struct.def") {
        return parse_struct_match(query, m, source);
    }
    if capture_names.contains(&"enum.def") {
        return parse_enum_match(query, m, source);
    }
    if capture_names.contains(&"impl.def") {
        return parse_impl_match(query, m, source);
    }

    None
}

fn get_capture_text<'a>(
    query: &tree_sitter::Query,
    m: &tree_sitter::QueryMatch,
    source: &'a str,
    capture_name: &str,
) -> Option<&'a str> {
    let idx = query
        .capture_names()
        .iter()
        .position(|n| *n == capture_name)?;
    let capture = m.captures.iter().find(|c| c.index as usize == idx)?;
    Some(&source[capture.node.byte_range()])
}

fn get_capture_node<'a>(
    query: &tree_sitter::Query,
    m: &'a tree_sitter::QueryMatch<'a, 'a>,
    capture_name: &str,
) -> Option<tree_sitter::Node<'a>> {
    let idx = query
        .capture_names()
        .iter()
        .position(|n| *n == capture_name)?;
    let capture = m.captures.iter().find(|c| c.index as usize == idx)?;
    Some(capture.node)
}

fn parse_function_match(
    query: &tree_sitter::Query,
    m: &tree_sitter::QueryMatch,
    source: &str,
    language: &str,
) -> Option<CodeConstruct> {
    let name = get_capture_text(query, m, source, "fn.name")?.to_string();
    let params_node = get_capture_node(query, m, "fn.params")?;
    let line = params_node.start_position().row + 1;

    let parameters = parse_parameters(params_node, source, language);

    Some(CodeConstruct {
        kind: ConstructKind::Function,
        name,
        parameters,
        fields: Vec::new(),
        variants: Vec::new(),
        line,
    })
}

fn parse_struct_match(
    query: &tree_sitter::Query,
    m: &tree_sitter::QueryMatch,
    source: &str,
) -> Option<CodeConstruct> {
    let name = get_capture_text(query, m, source, "struct.name")?.to_string();
    let def_node = get_capture_node(query, m, "struct.def")?;
    let line = def_node.start_position().row + 1;

    let fields = if let Some(fields_node) = get_capture_node(query, m, "struct.fields") {
        extract_field_names(fields_node, source)
    } else {
        Vec::new()
    };

    Some(CodeConstruct {
        kind: ConstructKind::Struct,
        name,
        parameters: Vec::new(),
        fields,
        variants: Vec::new(),
        line,
    })
}

fn parse_enum_match(
    query: &tree_sitter::Query,
    m: &tree_sitter::QueryMatch,
    source: &str,
) -> Option<CodeConstruct> {
    let name = get_capture_text(query, m, source, "enum.name")?.to_string();
    let def_node = get_capture_node(query, m, "enum.def")?;
    let line = def_node.start_position().row + 1;

    let variants = if let Some(variants_node) = get_capture_node(query, m, "enum.variants") {
        extract_variant_names(variants_node, source)
    } else {
        Vec::new()
    };

    Some(CodeConstruct {
        kind: ConstructKind::Enum,
        name,
        parameters: Vec::new(),
        fields: Vec::new(),
        variants,
        line,
    })
}

fn parse_impl_match(
    query: &tree_sitter::Query,
    m: &tree_sitter::QueryMatch,
    source: &str,
) -> Option<CodeConstruct> {
    let type_text = get_capture_text(query, m, source, "impl.type")?.to_string();
    let def_node = get_capture_node(query, m, "impl.def")?;
    let line = def_node.start_position().row + 1;

    Some(CodeConstruct {
        kind: ConstructKind::ImplBlock,
        name: type_text,
        parameters: Vec::new(),
        fields: Vec::new(),
        variants: Vec::new(),
        line,
    })
}

/// Parse function parameters from the parameters node.
fn parse_parameters(
    params_node: tree_sitter::Node,
    source: &str,
    language: &str,
) -> Vec<ParsedParameter> {
    let mut params = Vec::new();
    let mut cursor = params_node.walk();

    for child in params_node.children(&mut cursor) {
        match language {
            "rust" => {
                if child.kind() == "parameter" {
                    if let Some(param) = parse_rust_parameter(child, source) {
                        params.push(param);
                    }
                } else if child.kind() == "self_parameter" {
                    let text = &source[child.byte_range()];
                    params.push(ParsedParameter {
                        name: "self".to_string(),
                        type_name: text.to_string(),
                    });
                }
            }
            "c" => {
                if child.kind() == "parameter_declaration" {
                    if let Some(param) = parse_c_parameter(child, source) {
                        params.push(param);
                    }
                }
            }
            _ => {}
        }
    }

    params
}

fn parse_rust_parameter(node: tree_sitter::Node, source: &str) -> Option<ParsedParameter> {
    let mut name = String::new();
    let mut type_name = String::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" if name.is_empty() => {
                name = source[child.byte_range()].to_string();
            }
            "reference_type" | "type_identifier" | "generic_type" | "scoped_type_identifier"
            | "primitive_type" | "array_type" | "tuple_type" => {
                type_name = source[child.byte_range()].to_string();
            }
            _ => {}
        }
    }

    if !name.is_empty() {
        Some(ParsedParameter { name, type_name })
    } else {
        None
    }
}

fn parse_c_parameter(node: tree_sitter::Node, source: &str) -> Option<ParsedParameter> {
    let mut name = String::new();
    let mut type_name = String::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" | "pointer_declarator" => {
                name = source[child.byte_range()].to_string();
            }
            "type_identifier" | "primitive_type" | "sized_type_specifier" => {
                type_name = source[child.byte_range()].to_string();
            }
            _ => {}
        }
    }

    if !name.is_empty() {
        Some(ParsedParameter { name, type_name })
    } else {
        None
    }
}

/// Extract field names from a struct field_declaration_list node.
fn extract_field_names(node: tree_sitter::Node, source: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "field_declaration" {
            let mut inner_cursor = child.walk();
            for field_child in child.children(&mut inner_cursor) {
                if field_child.kind() == "field_identifier" {
                    fields.push(source[field_child.byte_range()].to_string());
                }
            }
        }
    }

    fields
}

/// Extract variant names from an enum_variant_list node.
fn extract_variant_names(node: tree_sitter::Node, source: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "enum_variant" {
            let mut inner_cursor = child.walk();
            for variant_child in child.children(&mut inner_cursor) {
                if variant_child.kind() == "identifier" {
                    variants.push(source[variant_child.byte_range()].to_string());
                }
            }
        }
    }

    variants
}

#[cfg(test)]
mod tests {
    use super::*;

    fn languages_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("languages")
    }

    #[test]
    fn test_parse_rust_function() {
        let source = r#"
fn init(config: A2dpConfig) -> BtA2dpSink {
    todo!()
}

fn start(sink: &mut BtA2dpSink) {
    todo!()
}
"#;
        let constructs = parse_source(source, "rust", &languages_dir()).unwrap();

        let fns: Vec<_> = constructs
            .iter()
            .filter(|c| c.kind == ConstructKind::Function)
            .collect();
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "init");
        assert_eq!(fns[1].name, "start");
        assert!(!fns[0].parameters.is_empty());
        assert_eq!(fns[0].parameters[0].name, "config");
    }

    #[test]
    fn test_parse_rust_struct() {
        let source = r#"
pub struct BtA2dpSink {
    config: A2dpConfig,
    state: ConnectionState,
}
"#;
        let constructs = parse_source(source, "rust", &languages_dir()).unwrap();

        let structs: Vec<_> = constructs
            .iter()
            .filter(|c| c.kind == ConstructKind::Struct)
            .collect();
        assert_eq!(structs.len(), 1);
        assert_eq!(structs[0].name, "BtA2dpSink");
        assert_eq!(structs[0].fields, vec!["config", "state"]);
    }

    #[test]
    fn test_parse_rust_enum() {
        let source = r#"
pub enum ConnectionState {
    Disconnected,
    Discovering,
    Connected,
    Streaming,
}
"#;
        let constructs = parse_source(source, "rust", &languages_dir()).unwrap();

        let enums: Vec<_> = constructs
            .iter()
            .filter(|c| c.kind == ConstructKind::Enum)
            .collect();
        assert_eq!(enums.len(), 1);
        assert_eq!(enums[0].name, "ConnectionState");
        assert_eq!(
            enums[0].variants,
            vec!["Disconnected", "Discovering", "Connected", "Streaming"]
        );
    }

    #[test]
    fn test_parse_rust_impl() {
        let source = r#"
struct Foo;

impl Foo {
    fn new() -> Self { Foo }
    fn bar(&self) {}
}
"#;
        let constructs = parse_source(source, "rust", &languages_dir()).unwrap();

        let impls: Vec<_> = constructs
            .iter()
            .filter(|c| c.kind == ConstructKind::ImplBlock)
            .collect();
        assert_eq!(impls.len(), 1);
        assert_eq!(impls[0].name, "Foo");
    }

    #[test]
    fn test_parse_source_never_panics_on_garbage() {
        // Tree-sitter should not panic on arbitrary input
        let garbage = "}{[]()#$@!~ fn struct enum let if while for 12345";
        let result = parse_source(garbage, "rust", &languages_dir());
        // Should succeed (possibly with empty results), not panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_unsupported_language() {
        let result = parse_source("void main() {}", "fortran", &languages_dir());
        assert!(result.is_err());
        match result.unwrap_err() {
            AuditError::UnsupportedLanguage(lang) => assert_eq!(lang, "fortran"),
            other => panic!("expected UnsupportedLanguage, got: {:?}", other),
        }
    }
}
