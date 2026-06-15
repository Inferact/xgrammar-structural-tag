# xgrammar-structural-tag

Rust builders for xgrammar-compatible `structural_tag` tool-calling constraints.

This crate generates the JSON string that a vLLM-style frontend can place into
`StructuredOutputsParams.structural_tag`. It has no runtime dependency on
xgrammar and does not compile grammars or parse model output.

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

`gemma_4` is excluded because upstream xgrammar currently keeps that builder
unregistered.

## Usage

```rust
use serde_json::json;
use xgrammar_structural_tag::{
    FunctionDefinition, FunctionToolParam, ToolChoice, ToolParam,
    get_model_structural_tag,
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

let tag = get_model_structural_tag(
    "qwen_3_5",
    &tools,
    ToolChoice::required(),
    true,
)?;

let structural_tag_json = tag.to_json_string()?;
# Ok::<(), xgrammar_structural_tag::Error>(())
```

For vLLM-style request lowering, use `maybe_get_model_structural_tag`. It
returns `Ok(None)` for empty tools or `tool_choice=none`.

## Development

```bash
cargo fmt --all --check
cargo test
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

The source templates are ported from xgrammar commit
`4d145cc13d878c751ebeed36af1c013074be76bc`, with vLLM/Rust frontend extensions
for `hermes` and `hy_v3`.
