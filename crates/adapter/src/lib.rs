//! Domain-agnostic SysML v2 query library built on syster-base.
//!
//! Provides structured APIs for extracting data from SysML v2 models that
//! syster-base's HIR does not directly expose. Works for any SysML v2 domain
//! — firmware-specific meaning is assigned by downstream consumers.
//!
//! - **Workspace loading** — multi-file `.sysml` parsing with both CST and HIR
//! - **Metadata extraction** — CST traversal for `@Annotation { field = value; }` bodies
//! - **Connection resolution** — topology from `connect`/`flow` statements
//! - **State machine extraction** — states, transitions, guards, actions
//! - **Symbol kind mapping** — `SymbolKind::Other` → `MetadataDefinition`

pub mod connection_resolver;
pub mod metadata_extractor;
pub mod state_machine_extractor;
pub mod symbol_kind_mapper;
pub mod workspace;

pub use connection_resolver::{Connection, ConnectionKind};
pub use metadata_extractor::{MetadataAnnotation, MetadataField, MetadataValue};
pub use state_machine_extractor::{State, StateMachine, Transition};
pub use symbol_kind_mapper::{MappedSymbolKind, classify_symbol};
pub use workspace::{AdapterError, ParsedFile, SysmlWorkspace};

// Re-export key syster-base types for downstream convenience
pub use syster::hir::{HirSymbol, SymbolKind};
