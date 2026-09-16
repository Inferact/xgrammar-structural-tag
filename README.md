# xgrammar-structural-tag

Rust builders for xgrammar-compatible `structural_tag` tool-calling constraints.

This crate generates the JSON string that a serving frontend can pass to an
xgrammar structural-tag backend. It has no runtime dependency on xgrammar and
does not compile grammars or parse model output.

## Supported models

- `llama`
- `kimi`
- `kimi_k3`
- `deepseek_r1`
- `deepseek_v3_1`
- `qwen_3_5`
- `qwen_3_coder`
- `qwen_3`
- `harmony`
- `deepseek_v3_2`
- `minimax`
- `minimax_m3`
- `glm_4_7`
- `deepseek_v4`
- `deepseek_v4_1`
- `cohere`
- `exaone`
- `hermes`
- `hy_v3`

## Usage

```rust
use serde_json::json;
use xgrammar_structural_tag::{
    FunctionDefinition, FunctionToolParam, Model, ToolChoice, ToolParam,
    build_structural_tag,
    builders::StructuralTagOptions,
};

fn main() -> xgrammar_structural_tag::Result<()> {
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
        StructuralTagOptions::default(),
    )?;

    let structural_tag_json = tag.to_json_string()?;
    assert!(structural_tag_json.contains("structural_tag"));
    Ok(())
}
```

`Model` is the built-in catalog. Callers can also pass a concrete
builder such as `builders::Qwen35Builder`, `Model::Qwen35.builder()`, or their
own `builders::StructuralTagBuilder` implementation.

For serving request lowering, use `build_optional_structural_tag`. It
returns `Ok(None)` for empty tools or `tool_choice=none`.

See `examples/custom_builder.rs` for a custom `StructuralTagBuilder`
implementation.

## Reasoning, call limits, and region budgets

`StructuralTagOptions` supports `ReasoningMode::{Enabled, Disabled, Auto}`.
Enabled mode continues a reasoning block opened by the prompt; adaptive mode
allows a complete optional reasoning block before the visible output. Harmony
uses the mode to select whether its analysis channel is available. Options
default to enabled reasoning for every builder, including MiniMax M3; set
`Auto` explicitly for its adaptive prompt.

```rust
use xgrammar_structural_tag::builders::{ReasoningMode, StructuralTagOptions};

let options = StructuralTagOptions::default()
    .with_reasoning(ReasoningMode::Auto)
    .with_parallel_tool_calls(false);
```

With `parallel_tool_calls(false)`, a response contains at most one call, and
the tool-call envelope closes the constrained output. Harmony can still emit
analysis messages before that call. `with_reasoning(true/false)` remains
available as a convenience for enabled/disabled mode.

Advanced callers can bound a free region directly in the format AST:

```rust
use xgrammar_structural_tag::format::{AnyTextFormat, Format, StructuralTag};

let grammar = StructuralTag::new(Format::tag(
    "",
    Format::AnyText(AnyTextFormat {
        max_tokens: Some(128),
        ..Default::default()
    }),
    "</think>",
));
```

`AnyTextFormat` supports token and Unicode-codepoint budgets; `AnyTokensFormat`
supports token budgets. `None` is unbounded and serialized as `null`; `Some(0)` forces an immediate
region end. Token limits take effect during mask-driven generation; direct
string acceptance enforces character limits. Limits must fit `0..=i32::MAX`.

MiniMax M3 requires constrained argument schemas: XGrammar rejects the
unconstrained schemas produced by missing parameters or `strict=false`.
The Kimi K3 builder follows upstream's response-then-tools channel template;
serving frontends still own prompt-state alignment and grammar activation.

### Updating from 0.2

The 0.3 API changes `StructuralTagOptions::reasoning` from `bool` to
`ReasoningMode` and adds budget fields to `AnyTextFormat` and `AnyTokensFormat`.
Existing `with_reasoning(bool)` calls continue to work; use `..Default::default()`
with direct free-region struct literals. Generated JSON follows XGrammar 0.2.7,
including reasoning-region exclusions and explicit embedded-tag discriminators.

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

Refresh upstream fixtures with the Python xgrammar version pinned in the
script's uv metadata:

```bash
uv run scripts/generate_python_golden.py
```

Check upstream parity and compile/replay the Rust fixtures with XGrammar:

```bash
uv run scripts/generate_python_golden.py --check
uv run scripts/test_python_compat.py
```

The crate version build metadata records the xgrammar source version used for
the structural tag templates. The crate also includes extra model templates for
`hermes` and `hy_v3`.

## Credits

This crate ports the `structural_tag` builders and schema shapes from
[XGrammar](https://github.com/mlc-ai/xgrammar) (Apache-2.0), with a more typed
Rust API and additional local model templates. See [`NOTICE`](NOTICE).
