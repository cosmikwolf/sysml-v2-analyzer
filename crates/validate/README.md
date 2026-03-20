# sysml-v2-validate

Firmware-specific SysML v2 model validation rules.

**Status: Scaffold** — Crate structure exists, implementation pending.

## Purpose

Validates parsed SysML v2 models against firmware domain rules that go beyond general SysML v2 syntax/type checking. This is the equivalent of `/fw-validate-spec` and `/fw-audit-deps`, operating on SysML v2 models instead of YAML specs.

## Planned rules

| Category | Rule IDs | Examples |
|---|---|---|
| Layer dependencies | FW001–FW004 | Driver can't connect to application layer; cycle detection |
| Metadata completeness | FW010–FW015 | Missing `@MemoryModel`; heap allocation forbidden |
| State machine well-formedness | FW020–FW026 | No initial state; unreachable states; non-deterministic transitions |
| Port compatibility | FW030–FW034 | Type mismatch; two outputs connected; unused ports |
| Constraint satisfaction | FW040–FW042 | Undefined attribute refs; contradictory constraints |
| Workspace rules | FW050–FW053 | Duplicate names; unused parts; import shadowing |

## Dependencies

- `sysml-v2-adapter` — Workspace loading, metadata extraction, connection resolution, FSM extraction
- `petgraph` — Graph algorithms for cycle detection
- `toml` — Configuration parsing (`sysml.toml`)

## Configuration

Rules are configurable via `sysml.toml`:

```toml
[validation.rules]
FW002 = "error"     # upgrade "no layer" from warning to error
FW004 = "off"       # disable same-layer dependency info

[validation.required_metadata]
parts = ["MemoryModel", "ConcurrencyModel", "ErrorHandling"]
```

## Design spec

See [`docs/sysml-toolchain/04-sysml-v2-validate.md`](../../../../docs/sysml-toolchain/04-sysml-v2-validate.md) for the full design.
