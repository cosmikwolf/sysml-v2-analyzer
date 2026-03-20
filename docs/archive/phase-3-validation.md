# Phase 3: Validation Engine

## Goal

Implement the domain-agnostic validation engine that checks SysML v2 models against rules defined in `domain.toml`. The engine doesn't know about firmware — it applies generic checks parameterized by config.

## Validation categories

### Layer dependency checking

Uses `domain.toml` `[layers]` config to validate connection directions.

| Rule pattern | Check | Config source |
|---|---|---|
| `LAYER001` | Connection from layer X to layer Y where Y not in `allowed_deps[X]` | `layers.allowed_deps` |
| `LAYER002` | Part has no layer attribute — can't verify deps | `layers.order` (attribute name from metadata) |
| `LAYER003` | Circular dependency between parts | `layers.allowed_deps` (graph cycle detection) |
| `LAYER004` | Same-layer dependency (allowed but flagged) | `layers.allowed_deps` |

Implementation: build a `petgraph::DiGraph` of part dependencies from adapter's `resolve_connections()`. Check edges against `allowed_deps`. Run cycle detection.

### Required metadata checking

Uses `domain.toml` `[required_metadata]` config.

| Rule pattern | Check |
|---|---|
| `META010` | Part definition missing a required annotation (one diagnostic per missing annotation) |

Implementation: for each `PartDefinition` symbol, call `extract_metadata()`, check that all names in `required_metadata.parts` are present.

### FSM well-formedness (generic — no config needed)

These are universal state machine invariants, not domain-specific.

| Rule pattern | Check |
|---|---|
| `FSM020` | No initial state (`entry; then X;` missing) |
| `FSM021` | Unreachable state (no incoming transitions and not initial) |
| `FSM022` | Non-deterministic: same event triggers two transitions from same state without distinct guards |
| `FSM024` | Transition targets a state not defined in this FSM |
| `FSM025` | Terminal state (no outgoing transitions) — warning |

Implementation: build a directed graph from adapter's `extract_state_machines()`. Check reachability from initial state. Check for duplicate (from_state, event) pairs. Verify all transition targets exist in the state list.

### Port compatibility (generic — no config needed)

| Rule pattern | Check | Status |
|---|---|---|
| `PORT030` | Connected ports have incompatible types | **Deferred** — adapter lacks structured port type info |
| `PORT033` | Port defined but not connected | Implemented |

Implementation: cross-reference port usage symbols with connections from `resolve_connections()`. PORT030 requires type resolution that the adapter does not currently expose; it will be implemented when the adapter gains structured port type extraction.

### Workspace rules (generic — no config needed)

| Rule pattern | Check |
|---|---|
| `WS050` | Duplicate qualified names across workspace |
| `WS051` | Part definition never instantiated (unused) |

Implementation: iterate all symbols, check for name collisions. Cross-reference PartDefinition with PartUsage supertypes.

## Public API

```rust
pub fn validate(
    workspace: &SysmlWorkspace,
    config: &DomainConfig,
) -> ValidationResult;

pub struct ValidationResult {
    pub diagnostics: Vec<Diagnostic>,
    pub parts_checked: usize,
    pub state_machines_checked: usize,
    pub connections_checked: usize,
}
```

## Rule ID convention

Rule IDs use a category prefix + number. The prefix is defined by the engine (not configurable):

| Prefix | Category |
|---|---|
| `LAYER` | Layer dependency rules |
| `META` | Required metadata rules |
| `FSM` | State machine well-formedness |
| `PORT` | Port compatibility |
| `WS` | Workspace-level rules |

## Tests

One test per rule minimum:

- `test_layer_valid_connection` — allowed dependency passes
- `test_layer_violation` — forbidden dependency produces LAYER001
- `test_layer_missing_attribute` — no layer → LAYER002
- `test_layer_cycle` — A→B→A → LAYER003
- `test_meta_all_present` — all required annotations → no diagnostic
- `test_meta_missing` — missing annotation → META010
- `test_fsm_no_initial` — missing entry → FSM020
- `test_fsm_unreachable` — disconnected state → FSM021
- `test_fsm_nondeterministic` — same event, two transitions → FSM022
- `test_fsm_bad_target` — transition to nonexistent state → FSM024
- `test_port_unused` — unconnected port → PORT033
- `test_ws_duplicate` — same qualified name twice → WS050
- `test_severity_override` — domain says warning, project says error → error
- `test_rule_disabled` — severity "off" → no diagnostic

## Verification

```
cargo test -p sysml-v2-engine    # all validation tests pass
cargo clippy --workspace         # clean
```
