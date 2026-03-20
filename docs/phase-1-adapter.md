# Phase 1: Adapter Crate — COMPLETE

## Summary

Built the domain-agnostic SysML v2 query library on top of syster-base. This is the foundation that all subsequent phases build on.

## What was built

| Module | Lines (approx) | Tests |
|---|---|---|
| `workspace.rs` | 170 | 8 unit |
| `metadata_extractor.rs` | 210 | 10 unit |
| `connection_resolver.rs` | 190 | 7 unit |
| `state_machine_extractor.rs` | 230 | 8 unit |
| `symbol_kind_mapper.rs` | 80 | 3 unit |
| `integration.rs` | 120 | 9 integration |
| **Total** | **~1000** | **45** |

## Key decisions made during implementation

1. **syster-base uses 0-indexed line numbers** — all source line access uses `lines[sym.start_line]` not `lines[sym.start_line - 1]`
2. **HIR spans are name-only** — `extract_definition_body()` utility scans for matching `{}` braces from the name span
3. **Metadata extraction via CST text parsing** — the HIR identifies which annotations exist but doesn't expose field values; we parse `{ field = value; }` bodies from the CST text
4. **Dual-strategy extraction** — connection resolver tries HIR symbols first, falls back to CST text parsing

## Test fixtures

8 SysML v2 files in `tests/fixtures/` modeling a Bluetooth audio sink system:

| Fixture | Purpose |
|---|---|
| `firmware_library.sysml` | Metadata defs, enums |
| `interfaces.sysml` | Port types, data structures |
| `bt_a2dp_sink.sysml` | Full part with ConnectionFSM (4 states, 7 transitions) |
| `audio_pipeline.sysml` | Composition with 3 connects + 1 flow |
| `i2s_output.sysml` | Simple driver |
| `status_led.sysml` | LedFSM (3 states, 6 transitions) |
| `large_model.sysml` | 50-module stress test (1075 lines) |
| `malformed.sysml` | Intentional errors for recovery testing |

## Verification

```
cargo test -p sysml-v2-adapter   # 45/45 pass
cargo clippy --workspace         # 0 warnings
cargo build                      # compiles clean
```
