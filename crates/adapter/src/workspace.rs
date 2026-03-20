//! SysML v2 workspace loading and querying.
//!
//! Loads a directory of `.sysml` files into a queryable workspace with
//! both CST (for value extraction) and HIR (for symbol queries).

use std::path::{Path, PathBuf};

use syster::base::FileId;
use syster::hir::{self, HirSymbol, SymbolKind};
use syster::parser::{self, Parse};
use syster::syntax::SyntaxFile;

/// A loaded SysML v2 workspace containing parsed files and symbols.
#[derive(Debug)]
pub struct SysmlWorkspace {
    files: Vec<ParsedFile>,
}

/// A single parsed `.sysml` file with CST, HIR symbols, and source text.
#[derive(Debug)]
pub struct ParsedFile {
    /// Path to the source file.
    pub path: PathBuf,
    /// Original source text.
    pub source: String,
    /// CST parse result (includes syntax tree and errors).
    pub parse: Parse,
    /// Syntax file for HIR extraction.
    pub syntax_file: SyntaxFile,
    /// Extracted HIR symbols.
    pub symbols: Vec<HirSymbol>,
    /// File ID used for HIR queries.
    pub file_id: FileId,
}

/// Errors that can occur during workspace loading.
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("failed to read SysML file {}: {source}", path.display())]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("parse errors in {}: {count} error(s)", path.display())]
    ParseErrors { path: PathBuf, count: usize },

    #[error("no .sysml files found in {}", path.display())]
    EmptyWorkspace { path: PathBuf },
}

impl SysmlWorkspace {
    /// Load all `.sysml` files from a directory into a workspace.
    ///
    /// Files are discovered by scanning `root` recursively for `*.sysml` files.
    /// Parse errors are collected but do not prevent loading — the workspace
    /// will contain partial results for files with errors.
    pub fn load(root: &Path) -> Result<Self, AdapterError> {
        let mut sysml_paths: Vec<PathBuf> = Vec::new();
        collect_sysml_files(root, &mut sysml_paths)?;

        if sysml_paths.is_empty() {
            return Err(AdapterError::EmptyWorkspace {
                path: root.to_path_buf(),
            });
        }

        // Sort for deterministic file_id assignment
        sysml_paths.sort();

        let mut files = Vec::with_capacity(sysml_paths.len());

        for (idx, path) in sysml_paths.into_iter().enumerate() {
            let source = std::fs::read_to_string(&path).map_err(|e| AdapterError::FileRead {
                path: path.clone(),
                source: e,
            })?;

            let parse = parser::parse_sysml(&source);
            let syntax_file = SyntaxFile::sysml(&source);
            let file_id = FileId::new(idx as u32);
            let symbols = hir::file_symbols(file_id, &syntax_file);

            files.push(ParsedFile {
                path,
                source,
                parse,
                syntax_file,
                symbols,
                file_id,
            });
        }

        Ok(SysmlWorkspace { files })
    }

    /// Load a workspace from explicit source strings (for testing).
    pub fn from_sources(sources: Vec<(PathBuf, String)>) -> Self {
        let mut files = Vec::with_capacity(sources.len());

        for (idx, (path, source)) in sources.into_iter().enumerate() {
            let parse = parser::parse_sysml(&source);
            let syntax_file = SyntaxFile::sysml(&source);
            let file_id = FileId::new(idx as u32);
            let symbols = hir::file_symbols(file_id, &syntax_file);

            files.push(ParsedFile {
                path,
                source,
                parse,
                syntax_file,
                symbols,
                file_id,
            });
        }

        SysmlWorkspace { files }
    }

    /// Get all parsed files in the workspace.
    pub fn files(&self) -> &[ParsedFile] {
        &self.files
    }

    /// Iterate over all symbols across all files.
    pub fn all_symbols(&self) -> impl Iterator<Item = (&ParsedFile, &HirSymbol)> {
        self.files
            .iter()
            .flat_map(|file| file.symbols.iter().map(move |sym| (file, sym)))
    }

    /// Find all symbols of a given kind across the workspace.
    pub fn symbols_of_kind(&self, kind: SymbolKind) -> Vec<(&ParsedFile, &HirSymbol)> {
        self.all_symbols()
            .filter(|(_, sym)| sym.kind == kind)
            .collect()
    }

    /// Find a symbol by its qualified name (or name substring).
    pub fn find_by_qualified_name(&self, name: &str) -> Option<(&ParsedFile, &HirSymbol)> {
        self.all_symbols()
            .find(|(_, sym)| *sym.qualified_name == *name || *sym.name == *name)
    }

    /// Get all files that parsed without errors.
    pub fn clean_files(&self) -> impl Iterator<Item = &ParsedFile> {
        self.files.iter().filter(|f| f.parse.errors.is_empty())
    }

    /// Check if the entire workspace parsed without errors.
    pub fn has_errors(&self) -> bool {
        self.files.iter().any(|f| !f.parse.errors.is_empty())
    }

    /// Get all parse errors across all files.
    pub fn all_errors(&self) -> Vec<(&ParsedFile, &syster::parser::SyntaxError)> {
        self.files
            .iter()
            .flat_map(|file| file.parse.errors.iter().map(move |err| (file, err)))
            .collect()
    }
}

/// Extract the full text of a definition body, including everything from
/// the declaration line through the matching closing `}`.
///
/// HIR symbols only report the *name* span (e.g. `start=7:13 end=7:23` for
/// just `BtA2dpSink`), not the full body. This function scans forward from
/// the symbol's start line to find the `{ ... }` body.
pub fn extract_definition_body(source: &str, symbol: &HirSymbol) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    // syster-base HIR uses 0-indexed line numbers
    let start_line = symbol.start_line as usize;

    if start_line >= lines.len() {
        return None;
    }

    // Find the opening '{' on or after the start line
    let mut brace_depth = 0;
    let mut found_open = false;
    let mut body_start = start_line;

    for (i, line) in lines.iter().enumerate().skip(start_line) {
        for ch in line.chars() {
            match ch {
                '{' => {
                    if !found_open {
                        found_open = true;
                        body_start = i;
                    }
                    brace_depth += 1;
                }
                '}' => {
                    brace_depth -= 1;
                    if brace_depth == 0 && found_open {
                        let region: String = lines[body_start..=i].join("\n");
                        return Some(region);
                    }
                }
                _ => {}
            }
        }
    }

    // If we never found a matching brace, return from start to end of file
    if found_open {
        let region: String = lines[body_start..].join("\n");
        Some(region)
    } else {
        None
    }
}

/// Recursively collect `.sysml` files from a directory.
fn collect_sysml_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), AdapterError> {
    let entries = std::fs::read_dir(dir).map_err(|e| AdapterError::FileRead {
        path: dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| AdapterError::FileRead {
            path: dir.to_path_buf(),
            source: e,
        })?;
        let path = entry.path();

        if path.is_dir() {
            collect_sysml_files(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "sysml") {
            out.push(path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Valid fixture file names (excludes malformed.sysml).
    const VALID_FIXTURES: &[&str] = &[
        "firmware_library.sysml",
        "interfaces.sysml",
        "bt_a2dp_sink.sysml",
        "audio_pipeline.sysml",
        "i2s_output.sysml",
        "status_led.sysml",
    ];

    fn load_valid_workspace() -> SysmlWorkspace {
        let sources: Vec<(PathBuf, String)> = VALID_FIXTURES
            .iter()
            .map(|name| (PathBuf::from(name), load_fixture(name)))
            .collect();
        SysmlWorkspace::from_sources(sources)
    }

    #[test]
    fn test_load_workspace() {
        let ws = load_valid_workspace();
        assert_eq!(ws.files().len(), VALID_FIXTURES.len());

        // All valid fixtures should parse without errors
        for file in ws.files() {
            assert!(
                file.parse.errors.is_empty(),
                "{} had parse errors: {:?}",
                file.path.display(),
                file.parse.errors
            );
        }
    }

    #[test]
    fn test_symbols_of_kind_part_def() {
        let ws = load_valid_workspace();
        let part_defs = ws.symbols_of_kind(SymbolKind::PartDefinition);

        let names: Vec<&str> = part_defs.iter().map(|(_, sym)| sym.name.as_ref()).collect();
        assert!(names.contains(&"BtA2dpSink"), "missing BtA2dpSink: {:?}", names);
        assert!(names.contains(&"AudioPipeline"), "missing AudioPipeline: {:?}", names);
        assert!(names.contains(&"I2sOutput"), "missing I2sOutput: {:?}", names);
        assert!(names.contains(&"StatusLed"), "missing StatusLed: {:?}", names);
    }

    #[test]
    fn test_symbols_of_kind_port_def() {
        let ws = load_valid_workspace();
        let port_defs = ws.symbols_of_kind(SymbolKind::PortDefinition);

        assert!(
            port_defs.len() >= 4,
            "expected at least 4 port defs, found {}",
            port_defs.len()
        );
    }

    #[test]
    fn test_symbols_of_kind_state_def() {
        let ws = load_valid_workspace();
        let state_defs = ws.symbols_of_kind(SymbolKind::StateDefinition);

        let names: Vec<&str> = state_defs.iter().map(|(_, sym)| sym.name.as_ref()).collect();
        assert!(names.contains(&"ConnectionFSM"), "missing ConnectionFSM: {:?}", names);
        assert!(names.contains(&"LedFSM"), "missing LedFSM: {:?}", names);
    }

    #[test]
    fn test_find_by_qualified_name() {
        let ws = load_valid_workspace();

        let found = ws.find_by_qualified_name("BtA2dpSink");
        assert!(found.is_some(), "should find BtA2dpSink by name");

        let (_, sym) = found.unwrap();
        assert_eq!(sym.kind, SymbolKind::PartDefinition);
    }

    #[test]
    fn test_parse_error_recovery() {
        let source = load_fixture("malformed.sysml");
        let ws = SysmlWorkspace::from_sources(vec![(PathBuf::from("malformed.sysml"), source)]);

        let file = &ws.files()[0];
        assert!(
            !file.parse.errors.is_empty(),
            "malformed.sysml should have parse errors"
        );

        // Should still produce a syntax tree
        let root = file.parse.syntax();
        assert!(root.text().len() > 0.into(), "should have a syntax tree");
    }

    #[test]
    fn test_cst_round_trip() {
        let source = load_fixture("firmware_library.sysml");
        let ws = SysmlWorkspace::from_sources(vec![(
            PathBuf::from("firmware_library.sysml"),
            source.clone(),
        )]);

        let file = &ws.files()[0];
        let reconstructed = file.parse.syntax().text().to_string();
        assert_eq!(source, reconstructed, "CST should preserve source text (lossless)");
    }

    #[test]
    fn test_load_from_directory() {
        let dir = fixtures_dir();
        if dir.exists() {
            let ws = SysmlWorkspace::load(&dir).expect("should load fixtures directory");
            // 7 .sysml files in fixtures (including malformed and large_model)
            assert!(
                ws.files().len() >= 6,
                "should load at least 6 files, found {}",
                ws.files().len()
            );
        }
    }
}
