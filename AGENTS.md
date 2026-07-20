# xgrammar-structural-tag Agent Notes

This repository contains a standalone Rust crate that builds xgrammar-compatible
`structural_tag` JSON for tool-calling constraints. It is a DTO/template
builder crate: it serializes structural-tag payloads. Runtime grammar
compilation, token masking, xgrammar calls, and model-output parsing live in the
serving stack.

## Crate Positioning

- Keep the runtime dependency surface small. The normal dependency set is
  `serde`, `serde_json`, `strum`, and `thiserror`.
- Public entry points are typed Rust builders:
  `build_structural_tag` and `build_optional_structural_tag`.
- Built-in model selection uses the non-exhaustive `Model` enum. Keep
  registration static through `Model` and builder dispatch; use a designed
  trait/object API if extension hooks become necessary later.
- The `format` module is intentionally public. It exposes the structural-tag
  AST DTOs for advanced callers.
- Tool-choice normalization is crate-internal. Keep `NormalizedToolChoice`,
  `SimplifiedToolChoice`, and `normalize_tool_choice` private to the crate
  unless a dedicated extension API is designed.

## Relationship To Upstream xgrammar

The crate follows xgrammar's Python structural-tag builders and schema shapes,
with additional local model templates such as `hermes` and `hy_v3`. The Rust API
may be more typed than the Python API while preserving the serialized JSON wire
shape.

When recording upstream provenance, use Cargo version build metadata such as
`0.1.0+xgrammar.0.2.2.4d145cc`. Keep upstream commit hashes and release versions
out of source comments and rustdoc, where they become stale quickly.

## Documentation Style

Rustdoc tracks the upstream Python docstrings but stays Rust-idiomatic: prose
sentences, not Sphinx-style `Parameters`/`Returns` blocks, and links via
intra-doc `[`Item`]` references. Port the substantive content (semantics,
wire-shape notes, accepted-output illustrations) rather than the structure, and
do not re-document Rust-specific redesigns (`TokenValue`, `EndBoundary`, the
`AllowedTools`/`FlatAllowedTools` split, the typed `Model` enum) as if they were
the Python `Union` shapes.

- Per-model reference URLs and supported-model lists live on the builder
  functions in `src/builders/`, mirroring where upstream keeps them on
  `get_<model>_structural_tag`. Keep `Model` variant docs to a concise line.
- `hermes` and `hy_v3` have no upstream docstring; label them as local
  extensions.
- Migrate worthwhile examples as compiling doctests when they use the public
  API; keep accepted-output illustrations as ```text``` blocks so they are not
  run.
- Skip deprecated upstream surface absent here (`force_reasoning`,
  `QwenXMLParameterFormat`, `StructuralTagItem`, the string registry).

## Syncing With Upstream

1. Identify the upstream xgrammar release/tag/commit to sync against.
2. Inspect upstream Python files that define the public contract:
   - `python/xgrammar/builtin_structural_tag.py`
   - `python/xgrammar/structural_tag.py`
   - `python/xgrammar/openai_tool_call_schema.py`
3. Update Rust DTOs in `src/format.rs` and `src/tool.rs` when upstream schema
   shapes change. Preserve JSON compatibility and prefer typed Rust fields for
   fixed string markers.
4. Update `src/model.rs`, `src/builders.rs`, and the relevant files under
   `src/builders/` when upstream adds, removes, or changes active model keys.
   Keep one model family/style per builder file where practical.
5. Align rustdoc with the upstream docstrings touched by the sync, following
   the Documentation Style above.
6. Update `scripts/generate_python_golden.py` when the upstream model list,
   golden case matrix, or pinned Python xgrammar version changes. The script's
   uv metadata supplies xgrammar and covers upstream xgrammar models only.
7. Regenerate Rust fixtures with:

   ```bash
   cargo run --example write_golden -- --write
   ```

8. Compare upstream fixtures through the script's pinned uv environment:

   ```bash
   uv run scripts/generate_python_golden.py
   ```

   Reconcile intentional local extensions separately.
9. Update `Cargo.toml` package version build metadata to record the upstream
   source version used for the templates.
10. Run the full local validation set:

   ```bash
   cargo fmt --all --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
   ```

Keep generated golden fixture diffs in the same commit as the builder or DTO
changes that caused them.
