# xgrammar-structural-tag

Rust builders for xgrammar-compatible `structural_tag` tool-calling constraints.

This crate generates the JSON string that a serving frontend can pass to an
xgrammar structural-tag backend. It has no runtime dependency on xgrammar and
does not compile grammars or parse model output.

## Supported models

- `llama`
- `kimi`
- `deepseek_r1`
- `deepseek_v3_1`
- `qwen_3_5`
- `qwen_3_coder`
- `qwen_3`
- `harmony`
- `deepseek_v3_2`
- `minimax`
- `glm_4_7`
- `deepseek_v4`
- `hermes`
- `hy_v3`

## Usage

```rust
use serde_json::json;
use xgrammar_structural_tag::{
    FunctionDefinition, FunctionToolParam, Model, ToolChoice, ToolParam,
    build_structural_tag,
};

let tools = vec![ToolParam::Function(FunctionToolParam::new(
    FunctionDefinition::new("get_weather").with_parameters(json!({
        "type": "object",
        "properties": {
            "city": { "type": "string" }
        },
        "required": ["city"]
    })),
))];

let tag = build_structural_tag(
    Model::Qwen35,
    &tools,
    ToolChoice::required(),
    true,
)?;

let structural_tag_json = tag.to_json_string()?;
# Ok::<(), xgrammar_structural_tag::Error>(())
```

`Model` is the built-in catalog. Callers can also pass a concrete
builder such as `builders::Qwen35Builder`, `Model::Qwen35.builder()`, or their
own `builders::StructuralTagBuilder` implementation.

For serving request lowering, use `build_optional_structural_tag`. It
returns `Ok(None)` for empty tools or `tool_choice=none`.

See `examples/custom_builder.rs` for a custom `StructuralTagBuilder`
implementation.

## Development

```bash
cargo fmt --all --check
cargo test
cargo run --example custom_builder
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

Refresh Rust-generated golden fixtures:

```bash
cargo run --example write_golden -- --write
```

Refresh fixtures from Python xgrammar when an installed xgrammar build is
available:

```bash
python3 scripts/generate_python_golden.py
```

The crate version build metadata records the xgrammar source version used for
the structural tag templates. The crate also includes extra model templates for
`hermes` and `hy_v3`.

## Credits

This crate ports the `structural_tag` builders and schema shapes from
[XGrammar](https://github.com/mlc-ai/xgrammar) (Apache-2.0), with a more typed
Rust API and additional local model templates. See [`NOTICE`](NOTICE).
