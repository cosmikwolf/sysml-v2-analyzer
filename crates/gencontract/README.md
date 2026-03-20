# sysml-v2-gencontract

Generation contract schema and cross-validation.

**Status: Scaffold** — Crate structure exists, implementation pending.

## Domain scope

This crate is **firmware-specific**. The generation contract schema encodes firmware constraints: static vs. heap memory allocation, ISR-safe code patterns, concurrency protection mechanisms (mutexes, critical sections, IRQ disable), and embedded platform targets (ESP32, bare-metal Rust, ESP-IDF C). The concept of a generation contract is general (see Smithy, Terraform, OpenAPI Generator), but this schema is tailored to embedded firmware.

## Purpose

Defines the firmware-specific contract between extracted SysML v2 models and the code generator. The generation contract specifies *how* to generate embedded code — target language, error handling strategy, memory allocation enforcement, concurrency protection, naming conventions, file layout — while the extracted model specifies *what* to generate.

```
Architecture Model (WHAT)  →  Generation Contract (HOW)  →  Generated Code
     (stable)                   (firmware-specific)          (derived)
```

## Planned components

| Component | Purpose |
|---|---|
| `schema.rs` | `GenContract`, `PlatformConfig`, `LanguageConfig` types with serde derives |
| `validation.rs` | GC001–GC010 cross-validation rules (contract vs. model consistency) |
| `resolution.rs` | Merge default config with per-module overrides |
| `type_map.rs` | SysML v2 type → target language type mapping |

## Dependencies

- `sysml-v2-extract` — Extracted model types
- `serde` / `serde_json` / `serde_yaml` — Serialization

## Design spec

See [`docs/sysml-toolchain/06-sysml-v2-gencontract.md`](../../../../docs/sysml-toolchain/06-sysml-v2-gencontract.md) for the full design.
