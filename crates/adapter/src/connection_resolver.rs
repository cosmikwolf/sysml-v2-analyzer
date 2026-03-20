//! Connection topology resolution.
//!
//! Extracts `connect` and `flow` statements from a part definition,
//! resolving source/target port references.
//!
//! Uses HIR `ConnectionUsage` / `FlowConnectionUsage` symbols for detection,
//! then CST text parsing for source/target extraction.

use syster::hir::{HirSymbol, SymbolKind};

use crate::workspace::ParsedFile;

/// A resolved connection between ports.
#[derive(Debug, Clone, PartialEq)]
pub struct Connection {
    /// Connection name (may be auto-generated from the HIR symbol name).
    pub name: String,
    /// Whether this is a `connect` or `flow` statement.
    pub kind: ConnectionKind,
    /// Source port reference (e.g. "bt.audioOut").
    pub source: String,
    /// Target port reference (e.g. "audioIn").
    pub target: String,
    /// For flow connections, the data type (e.g. "Integer").
    pub flow_type: Option<String>,
}

/// The kind of connection statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionKind {
    /// A `connect A to B;` statement.
    Connect,
    /// A `flow of T from A to B;` statement.
    Flow,
}

/// Resolve all connections within a part definition.
///
/// Finds both `connect` and `flow` statements using a combination of
/// HIR symbols and CST text parsing.
pub fn resolve_connections(file: &ParsedFile, part_symbol: &HirSymbol) -> Vec<Connection> {
    let mut connections = Vec::new();

    // Strategy 1: Find ConnectionUsage and FlowConnectionUsage symbols via HIR
    let part_qn = &part_symbol.qualified_name;
    for sym in &file.symbols {
        let is_child = sym.qualified_name.starts_with(part_qn.as_ref())
            && sym.qualified_name.len() > part_qn.len();

        if !is_child {
            continue;
        }

        match sym.kind {
            SymbolKind::ConnectionUsage => {
                if let Some(conn) = parse_connection_from_symbol(file, sym) {
                    connections.push(conn);
                }
            }
            SymbolKind::FlowConnectionUsage => {
                if let Some(conn) = parse_flow_from_symbol(file, sym) {
                    connections.push(conn);
                }
            }
            _ => {}
        }
    }

    // Strategy 2: If HIR didn't find connections, fall back to CST text parsing
    if connections.is_empty() {
        let source = file.parse.syntax().text().to_string();
        if let Some(part_text) = crate::workspace::extract_definition_body(&source, part_symbol) {
            connections = parse_connections_from_text(&part_text);
        }
    }

    connections
}

/// Parse a Connection from a ConnectionUsage HIR symbol.
fn parse_connection_from_symbol(file: &ParsedFile, sym: &HirSymbol) -> Option<Connection> {
    // The symbol name often encodes source/target like "<to:bt.audioOut#6@L27>"
    // We also need to check the CST text for the actual connect statement
    let source_text = file.parse.syntax().text().to_string();
    let lines: Vec<&str> = source_text.lines().collect();

    // syster-base HIR uses 0-indexed line numbers
    let start_line = sym.start_line as usize;
    if start_line >= lines.len() {
        return None;
    }

    // Gather the connect statement text (may span multiple lines)
    let mut stmt = String::new();
    for line in lines.iter().take((start_line + 3).min(lines.len())).skip(start_line) {
        stmt.push_str(line);
        stmt.push(' ');
        if line.contains(';') {
            break;
        }
    }

    parse_connect_statement(&stmt)
}

/// Parse a Connection from a FlowConnectionUsage HIR symbol.
fn parse_flow_from_symbol(file: &ParsedFile, sym: &HirSymbol) -> Option<Connection> {
    let source_text = file.parse.syntax().text().to_string();
    let lines: Vec<&str> = source_text.lines().collect();

    // syster-base HIR uses 0-indexed line numbers
    let start_line = sym.start_line as usize;
    if start_line >= lines.len() {
        return None;
    }

    // Gather the flow statement text (may span multiple lines)
    let mut stmt = String::new();
    for line in lines.iter().take((start_line + 5).min(lines.len())).skip(start_line) {
        stmt.push_str(line);
        stmt.push(' ');
        if line.contains(';') {
            break;
        }
    }

    parse_flow_statement(&stmt)
}

/// Parse a `connect A to B;` statement from text.
fn parse_connect_statement(text: &str) -> Option<Connection> {
    let trimmed = text.trim();

    // Look for "connect <source> to <target>"
    let connect_pos = trimmed.find("connect ")?;
    let after_connect = &trimmed[connect_pos + "connect ".len()..];

    let to_pos = after_connect.find(" to ")?;
    let source = after_connect[..to_pos].trim().to_string();
    let after_to = &after_connect[to_pos + " to ".len()..];

    // Target ends at ';' or end of string
    let target = after_to
        .split(';')
        .next()?
        .trim()
        .to_string();

    if source.is_empty() || target.is_empty() {
        return None;
    }

    Some(Connection {
        name: format!("connect_{}_to_{}", source.replace('.', "_"), target.replace('.', "_")),
        kind: ConnectionKind::Connect,
        source,
        target,
        flow_type: None,
    })
}

/// Parse a `flow of T from A to B;` statement from text.
fn parse_flow_statement(text: &str) -> Option<Connection> {
    let trimmed = text.trim();

    // Look for "flow of <type> from <source> to <target>"
    let flow_pos = trimmed.find("flow of ")?;
    let after_flow = &trimmed[flow_pos + "flow of ".len()..];

    let from_pos = after_flow.find(" from ")?;
    let flow_type = after_flow[..from_pos].trim().to_string();
    let after_from = &after_flow[from_pos + " from ".len()..];

    let to_pos = after_from.find(" to ")?;
    let source = after_from[..to_pos].trim().to_string();
    let after_to = &after_from[to_pos + " to ".len()..];

    let target = after_to
        .split(';')
        .next()?
        .trim()
        .to_string();

    Some(Connection {
        name: format!("flow_{}_to_{}", source.replace('.', "_"), target.replace('.', "_")),
        kind: ConnectionKind::Flow,
        source,
        target,
        flow_type: Some(flow_type),
    })
}

/// Parse connections from the raw text of a part definition body.
fn parse_connections_from_text(text: &str) -> Vec<Connection> {
    let mut connections = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("connect ") {
            if let Some(conn) = parse_connect_statement(trimmed) {
                connections.push(conn);
            }
        } else if trimmed.starts_with("flow of ") || trimmed.starts_with("flow ") {
            if let Some(conn) = parse_flow_statement(trimmed) {
                connections.push(conn);
            }
        }
    }

    connections
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

    fn pipeline_workspace() -> SysmlWorkspace {
        let source = load_fixture("audio_pipeline.sysml");
        SysmlWorkspace::from_sources(vec![(PathBuf::from("audio_pipeline.sysml"), source)])
    }

    fn find_part_def<'a>(ws: &'a SysmlWorkspace, name: &str) -> (&'a ParsedFile, &'a HirSymbol) {
        ws.all_symbols()
            .find(|(_, sym)| sym.kind == SymbolKind::PartDefinition && *sym.name == *name)
            .unwrap_or_else(|| panic!("part def '{}' not found", name))
    }

    #[test]
    fn test_resolve_connect_statements() {
        let ws = pipeline_workspace();
        let (file, part) = find_part_def(&ws, "AudioPipeline");
        let connections = resolve_connections(file, part);

        let connects: Vec<_> = connections
            .iter()
            .filter(|c| c.kind == ConnectionKind::Connect)
            .collect();

        assert!(
            connects.len() >= 3,
            "AudioPipeline should have at least 3 connect statements, found {}",
            connects.len()
        );
    }

    #[test]
    fn test_resolve_flow_statement() {
        let ws = pipeline_workspace();
        let (file, part) = find_part_def(&ws, "AudioPipeline");
        let connections = resolve_connections(file, part);

        let flows: Vec<_> = connections
            .iter()
            .filter(|c| c.kind == ConnectionKind::Flow)
            .collect();

        assert!(
            !flows.is_empty(),
            "AudioPipeline should have at least 1 flow statement"
        );
    }

    #[test]
    fn test_connection_source_target() {
        let ws = pipeline_workspace();
        let (file, part) = find_part_def(&ws, "AudioPipeline");
        let connections = resolve_connections(file, part);

        // Should find connect bt.audioOut to audioIn
        let bt_conn = connections.iter().find(|c| {
            c.source.contains("bt.audioOut") || c.target.contains("audioIn")
        });
        assert!(bt_conn.is_some(), "should find bt.audioOut connection");
    }

    #[test]
    fn test_flow_type() {
        let ws = pipeline_workspace();
        let (file, part) = find_part_def(&ws, "AudioPipeline");
        let connections = resolve_connections(file, part);

        let flow = connections
            .iter()
            .find(|c| c.kind == ConnectionKind::Flow);
        assert!(flow.is_some(), "should find a flow connection");

        let flow = flow.unwrap();
        assert_eq!(
            flow.flow_type.as_deref(),
            Some("Integer"),
            "flow type should be Integer"
        );
    }

    #[test]
    fn test_no_connections() {
        let source = load_fixture("i2s_output.sysml");
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("i2s_output.sysml"), source)]);
        let (file, part) = find_part_def(&ws, "I2sOutput");
        let connections = resolve_connections(file, part);

        assert!(
            connections.is_empty(),
            "I2sOutput should have no connections, found {}",
            connections.len()
        );
    }

    #[test]
    fn test_parse_connect_statement() {
        let conn = parse_connect_statement("connect bt.audioOut to audioIn;");
        assert!(conn.is_some());
        let conn = conn.unwrap();
        assert_eq!(conn.source, "bt.audioOut");
        assert_eq!(conn.target, "audioIn");
        assert_eq!(conn.kind, ConnectionKind::Connect);
    }

    #[test]
    fn test_parse_flow_statement() {
        let conn = parse_flow_statement("flow of Integer from bt.audioOut.data to i2s.i2sIn.samples;");
        assert!(conn.is_some());
        let conn = conn.unwrap();
        assert_eq!(conn.source, "bt.audioOut.data");
        assert_eq!(conn.target, "i2s.i2sIn.samples");
        assert_eq!(conn.flow_type.as_deref(), Some("Integer"));
        assert_eq!(conn.kind, ConnectionKind::Flow);
    }
}
