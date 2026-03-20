# Architecture Decision Record

Decisions that are expensive to reverse. Each entry captures what we decided, why, what alternatives we rejected, and what would trigger reconsideration.

---

## D1: Use syster-base as a dependency, not build a parser from scratch

**Date:** 2026-03-18
**Status:** Accepted

**Context:** The original plan called for building a custom SysML v2 parser (~33K lines, ~6 months). A Rust-native parser (syster-base) was discovered in the ecosystem.

**Decision:** Use syster-base 0.4.0-alpha as a pinned dependency. Build a ~500-line adapter on top.

**Why:** Hands-on evaluation scored 2.65/3.0 (above the 2.50 "use as dependency" threshold). All firmware SysML v2 constructs parse correctly. HIR extracts typed symbols with qualified names. Sub-millisecond parse times. The only gaps (metadata field values, symbol kind mapping) are covered by thin CST workarounds.

**Rejected alternatives:**
- **Build from scratch** — 33K lines, 6 months, grammar maintenance burden
- **Fork syster-base** — unnecessary; the gaps are small enough for an adapter layer
- **Wrap heavily** — evaluation showed the HIR is rich enough that heavy wrapping isn't needed

**Risks:** Alpha API instability. Mitigated by exact version pin (`=0.4.0-alpha`). The adapter's API surface against syster-base is small (3 key functions: `parse_sysml`, `SyntaxFile::sysml`, `file_symbols`), so version bumps are manageable.

**Reconsider if:** syster-base is abandoned, or a breaking change invalidates our adapter approach, or we need KerML features not in the HIR.

**Evidence:** Full evaluation in [`docs/sysml-toolchain/archive/09-evaluation-syster-base.md`](../../docs/sysml-toolchain/archive/09-evaluation-syster-base.md).

---

## D2: Single engine crate instead of separate validate/extract/gencontract/codegen crates

**Date:** 2026-03-20
**Status:** Accepted

**Context:** The original plan had 5 separate pipeline crates (validate, extract, gencontract, codegen, cli). After deciding to use syster-base + adapter, the custom code shrank from ~40K lines to ~3-4K lines.

**Decision:** Collapse validate, extract, gencontract, and codegen into a single `engine` crate with modules.

**Why:**
- The stages share types (diagnostics, domain config, workspace references) — separate crates means re-exporting these types across crate boundaries
- ~3-4K lines of code doesn't justify 4 crates with 4 Cargo.toml files, 4 lib.rs files, and cross-crate dependency management
- The stages aren't independently useful — nobody uses the extraction engine without the validation engine
- Simpler for a small team to maintain

**Rejected alternatives:**
- **Keep 4 separate crates** — premature modularity for the code volume involved. Adds boilerplate without benefit at this scale.
- **Two crates (validate+extract, gencontract+codegen)** — arbitrary split with no clean dependency boundary

**Reconsider if:** The engine grows past ~10K lines, or we need to publish individual stages as independent libraries, or compile times become a problem.

---

## D3: Domain-agnostic engine with domain plugins as directories

**Date:** 2026-03-20
**Status:** Accepted

**Context:** The original design baked firmware knowledge directly into the validation and codegen crates. Discussion revealed that most "firmware-specific" validation rules are actually generic patterns parameterized by config: layer dependency checking is just "element categories + allowed dependency graph", required metadata is just "element X must have annotation Y", FSM well-formedness is pure graph theory.

**Decision:** The engine crate is domain-agnostic. Domain-specific knowledge lives in `domains/<name>/` as config files (`domain.toml`), SysML metadata libraries (`.sysml`), and codegen templates (`.j2`). Adding a domain requires no Rust code.

**Why:**
- Clean separation of concerns — engine tests don't depend on firmware concepts
- Adding a domain is a directory, not a crate — lowers the barrier to supporting new domains
- Validation rules like "layer X can't depend on layer Y" are the same algorithm regardless of what the layers are called
- The adapter was already domain-agnostic — extending this principle upward is natural

**Rejected alternatives:**
- **Firmware-specific engine** — works for one domain but means forking the entire engine for a second domain
- **Trait-based plugin system** — over-engineered for current needs. A `DomainPlugin` trait can be added later when a domain needs custom Rust validation logic that config can't express.

**Reconsider if:** A domain needs validation logic that fundamentally can't be expressed as "check this graph property" or "check this annotation exists" — e.g., deep semantic analysis of SysML constraint expressions. At that point, add a `DomainPlugin` trait.

---

## D4: MiniJinja for template engine

**Date:** 2026-03-20
**Status:** Accepted

**Context:** Code generation needs a runtime template engine (templates live on disk in domain directories, not compiled into the binary). Evaluated MiniJinja, Tera, Handlebars, and Askama.

**Decision:** MiniJinja.

**Why (vs. Tera, the closest alternative):**
- **`trim_blocks` + `lstrip_blocks`** — MiniJinja supports these globally; Tera does not. Without them, every `{% if %}` and `{% for %}` tag in a codegen template needs manual `{%- -%}` whitespace trimming. This is the single biggest differentiator for code generation — templates are dramatically cleaner.
- **Error messages** — MiniJinja reports template name, line number, byte range, and surrounding source context. Tera doesn't reliably surface line numbers. Critical for iterating on templates.
- **Richer loop variables** — `loop.changed()`, `loop.previtem`/`loop.nextitem`, `break`/`continue`, recursive loops. Code generation constantly hits edge cases these solve.
- **Block-set capture** — `{% set content %}...{% endset %}` builds a string in a block for later use. Tera doesn't have this.
- **Cleaner filter API** — register a plain `fn(&str) -> String`. Tera requires `fn(&Value, &HashMap<String, Value>) -> Result<Value>`.
- **Lighter dependencies** — 2 mandatory deps vs. 8 (Tera pulls in `pest_derive` proc macro, `regex`).
- **Author pedigree** — created by Armin Ronacher, who designed the original Jinja2 language.

**Rejected alternatives:**
- **Tera** — mature and widely used, but lacks `trim_blocks`/`lstrip_blocks`, weaker error messages, heavier deps. Better suited for web templating than code generation.
- **Handlebars** — logic-less templates are too restrictive for code generation (need conditionals, loops with context, filters).
- **Askama** — compile-time templates. Can't load from domain directories at runtime.

**Reconsider if:** MiniJinja is abandoned, or Tera v2 (currently alpha) ships with `trim_blocks` support and better error messages.

---

## D5: Workspace config (`sysml.toml`) + domain config (`domain.toml`) as separate files

**Date:** 2026-03-20
**Status:** Accepted

**Context:** Need a way for projects to select a domain and override its defaults. Also need domain definitions to be shareable across projects.

**Decision:** Two config files with different scopes:
- `domain.toml` lives in `domains/<name>/` — shared domain definition (layer hierarchy, required metadata, type maps, default rule severities)
- `sysml.toml` lives in the user's project root — selects the domain and provides per-project overrides

The engine merges them: domain defaults ← project overrides.

**Why:**
- **Separation of concerns** — domain authors define what's valid; project authors decide which rules to enforce for their specific project
- **Shareability** — `domain.toml` can be reused across many projects without modification
- **Familiar pattern** — mirrors `Cargo.toml` (project) vs. crate defaults, or ESLint config extending a shared config
- **CLI discovery** — `sysml.toml` is found by walking up from cwd (like `Cargo.toml`), providing a natural workspace root marker

**Rejected alternatives:**
- **Single config file** — would mean copying the full domain definition into every project, or embedding domain selection in the same file as domain rules (confusing)
- **CLI flags only** — no persistent project configuration; every invocation needs `--domain firmware --rule META010=off ...`
- **Environment variables** — non-discoverable, hard to version control

**Reconsider if:** the two-file approach causes confusion in practice. Could merge into a single `sysml.toml` with an `extends = "firmware"` field, but this loses the clean separation.

---

## D6: `.j2` template extension with double-extension naming

**Date:** 2026-03-20
**Status:** Accepted

**Context:** Need a file naming convention for MiniJinja codegen templates that makes the output type obvious, works with editor tooling, and avoids MiniJinja's auto-escape triggers.

**Decision:** `.j2` extension with double-extension naming, organized by target language:
```
templates/rust/module.rs.j2
templates/c/module.c.j2
templates/c/module.h.j2
```

**Why:**
- **`.j2` is the dominant Jinja ecosystem convention** — used by Ansible, Cookiecutter, and MiniJinja's own CLI tool. Editors recognize it for Jinja syntax highlighting.
- **Double extension** (`module.rs.j2`) — makes the output type immediately obvious. When you see `state_machine.rs.j2` next to `test.rs.j2`, the purpose is clear.
- **Doesn't trigger auto-escape** — MiniJinja auto-escapes `.html`/`.htm`/`.xml` extensions. `.j2` produces raw output, which is what code generation needs.
- **Not the bare target extension** — naming templates `.rs` or `.c` confuses editors into applying Rust/C linting to Jinja syntax, and makes templates indistinguishable from generated output.

**Rejected alternatives:**
- **`.jinja`** — valid but less common than `.j2`, longer to type
- **`.tera`** — engine-specific, and we might switch engines someday
- **Bare target extension (`.rs`, `.c`)** — confuses editors, loses the "this is a template" signal
- **No extension convention** — MiniJinja doesn't care, but humans and editors do

**Reconsider if:** a better convention emerges in the Jinja ecosystem, or editor tooling improves for a different extension.

---

## D7: Domain-agnostic validation rule IDs

**Date:** 2026-03-20
**Status:** Accepted

**Context:** The original plan used `FW001`–`FW053` rule IDs with a firmware-specific prefix. But the validation engine is now domain-agnostic.

**Decision:** Rule IDs use category prefixes defined by the engine, not the domain:
- `LAYER001`–`LAYER004` — layer dependency rules
- `META010`–`META015` — required metadata rules
- `FSM020`–`FSM026` — state machine well-formedness
- `PORT030`–`PORT034` — port compatibility
- `WS050`–`WS053` — workspace-level rules

Domains configure severity per rule ID in `domain.toml`. Projects override severity in `sysml.toml`.

**Why:** The rules are generic algorithms (graph analysis, set membership, reachability). The rule ID should describe the *check*, not the *domain*. A `LAYER001` violation means the same thing whether the layers are called "driver/hal" or "perception/planning".

**Rejected alternatives:**
- **Domain-prefixed IDs (`FW001`)** — implies the rule is firmware-specific when it's actually a generic graph check
- **Configurable ID prefixes** — unnecessary complexity; the rule semantics don't change across domains

**Reconsider if:** domains need custom validation rules with their own IDs. At that point, add a `[domain.custom_rules]` section in `domain.toml` with domain-prefixed IDs.
