//! Shared utilities for the engine crate.

use std::collections::HashSet;

use sysml_v2_adapter::workspace::extract_definition_body;
use sysml_v2_adapter::{HirSymbol, ParsedFile};

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
