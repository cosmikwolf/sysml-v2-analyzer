//! Maps `SymbolKind::Other` to more specific kinds.
//!
//! syster-base maps `metadata def` declarations to `SymbolKind::Other`
//! rather than a dedicated `MetadataDefinition` variant. This module
//! inspects the CST at the symbol's source span to correctly classify them.

use syster::hir::{HirSymbol, SymbolKind};

use crate::workspace::ParsedFile;

/// Extended symbol kind that includes firmware-specific classifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedSymbolKind {
    /// A standard syster-base symbol kind.
    Known(SymbolKind),
    /// A `metadata def` declaration (mapped from `SymbolKind::Other`).
    MetadataDefinition,
}

/// Classify a symbol, checking for `metadata def` if the kind is `Other`.
///
/// For `SymbolKind::Other`, inspects the CST text at the symbol's source span
/// to determine if it's a `metadata def` declaration.
pub fn classify_symbol(file: &ParsedFile, symbol: &HirSymbol) -> MappedSymbolKind {
    if symbol.kind == SymbolKind::MetadataDefinition {
        return MappedSymbolKind::MetadataDefinition;
    }

    if symbol.kind != SymbolKind::Other {
        return MappedSymbolKind::Known(symbol.kind);
    }

    // SymbolKind::Other — check CST for `metadata def` keyword pair
    let root = file.parse.syntax();
    let source_text = root.text().to_string();

    // Find the line where this symbol is declared and check for `metadata def`
    if let Some(region) = extract_symbol_region(&source_text, symbol) {
        let trimmed = region.trim();
        if trimmed.contains("metadata def") {
            return MappedSymbolKind::MetadataDefinition;
        }
    }

    MappedSymbolKind::Known(SymbolKind::Other)
}

/// Extract the source text region for a symbol based on its line/col span.
///
/// Note: syster-base uses 0-indexed line numbers in HIR symbols.
fn extract_symbol_region(source: &str, symbol: &HirSymbol) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    let start_line = symbol.start_line as usize;

    // syster-base HIR uses 0-indexed lines
    let line = lines.get(start_line)?;
    Some(line.to_string())
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

    #[test]
    fn test_classify_metadata_def() {
        let source = load_fixture("firmware_library.sysml");
        let ws = SysmlWorkspace::from_sources(vec![(
            PathBuf::from("firmware_library.sysml"),
            source,
        )]);

        let file = &ws.files()[0];

        // Find symbols classified as Other that should be MetadataDefinition
        let metadata_names = [
            "MemoryModel",
            "ConcurrencyModel",
            "ErrorHandling",
            "ISRSafe",
            "Ownership",
            "LayerConstraint",
        ];

        let mut found_metadata = 0;
        for sym in &file.symbols {
            let classified = classify_symbol(file, sym);
            if classified == MappedSymbolKind::MetadataDefinition {
                found_metadata += 1;
            }
            // Check that known metadata defs are correctly classified
            if metadata_names.iter().any(|name| *sym.name == **name) {
                assert!(
                    classified == MappedSymbolKind::MetadataDefinition
                        || sym.kind == SymbolKind::MetadataDefinition,
                    "Symbol '{}' ({:?}) should be MetadataDefinition, got {:?}",
                    sym.name,
                    sym.kind,
                    classified
                );
            }
        }

        assert!(
            found_metadata >= 6,
            "should classify at least 6 metadata defs, found {}",
            found_metadata
        );
    }

    #[test]
    fn test_classify_part_def() {
        let source = load_fixture("bt_a2dp_sink.sysml");
        let ws = SysmlWorkspace::from_sources(vec![(
            PathBuf::from("bt_a2dp_sink.sysml"),
            source,
        )]);

        let file = &ws.files()[0];

        let bt_sym = file
            .symbols
            .iter()
            .find(|s| *s.name == *"BtA2dpSink")
            .expect("should find BtA2dpSink");

        let classified = classify_symbol(file, bt_sym);
        assert_eq!(
            classified,
            MappedSymbolKind::Known(SymbolKind::PartDefinition)
        );
    }

    #[test]
    fn test_classify_enum_def() {
        let source = load_fixture("firmware_library.sysml");
        let ws = SysmlWorkspace::from_sources(vec![(
            PathBuf::from("firmware_library.sysml"),
            source,
        )]);

        let file = &ws.files()[0];

        let enum_sym = file
            .symbols
            .iter()
            .find(|s| *s.name == *"LayerKind")
            .expect("should find LayerKind");

        let classified = classify_symbol(file, enum_sym);
        assert_eq!(
            classified,
            MappedSymbolKind::Known(SymbolKind::EnumerationDefinition)
        );
    }
}
