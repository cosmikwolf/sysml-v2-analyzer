//! SysML v2 analysis engine.
//!
//! Domain-agnostic engine that validates, extracts, and audits source code
//! against SysML v2 models using domain plugin configurations.

pub mod audit;
pub mod diagnostic;
pub mod domain;
pub mod extraction;
pub mod validation;

pub mod util;
