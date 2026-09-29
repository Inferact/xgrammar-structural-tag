use crate::Result;
use crate::format::{Format, JsonSchemaStyle, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    ReasoningMode, StructuralTagBuilder, StructuralTagContext, reasoning_prefix, schema,
    structural, styled_schema_excluding, tag, text_excludes, tools_with_separator,
};

/// Kimi K3 channel-format structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct KimiK3Builder;

impl StructuralTagBuilder for KimiK3Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_kimi_k3(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.options,
        ))
    }
}

/// Build a Kimi-K3 structural tag.
///
/// The generation prompt opens the think block in enabled mode and the response
/// block in disabled mode. The constrained output continues inside that block.
/// Later blocks and the message close are generated explicitly:
///
/// ```text
/// <|open|>think<|sep|>...<|close|>think<|sep|>
/// <|open|>response<|sep|>...<|close|>response<|sep|>
/// <|open|>tools<|sep|>
/// <|open|>call tool="NAME" index="1"<|sep|>
/// <|open|>argument key="KEY" type="TYPE"<|sep|>VALUE<|close|>argument<|sep|>
/// <|close|>call<|sep|><|close|>tools<|sep|><|close|>message<|sep|>
/// ```
///
/// Line breaks above are illustrative. Arguments use the Kimi K3 XML schema
/// style: strings are raw text and other values use JSON-style encoding. Unless special-token
/// exclusion is disabled, argument strings and property names may not contain `<|open|>`,
/// `<|close|>`, or `<|sep|>`; the argument and call wrappers stay outside that scope.
pub(super) fn build_kimi_k3(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
) -> StructuralTag {
    const TOOLS_BEGIN: &str = "<|open|>tools<|sep|>";
    const TOOLS_END: &str = "<|close|>tools<|sep|>";
    const SPECIAL_EXCLUDES: &[&str] = &["<|open|>", "<|close|>"];
    const ARGUMENT_EXCLUDES: &[&str] = &["<|open|>", "<|close|>", "<|sep|>"];
    let tool_tag = |tool: &FunctionToolParam| {
        tag(
            format!("<|open|>call tool=\"{}\" index=\"", tool.function.name),
            Format::sequence(vec![
                Format::regex(r"\d+"),
                Format::const_string("\"<|sep|>"),
                styled_schema_excluding(
                    schema(&tool.function),
                    JsonSchemaStyle::KimiK3Xml,
                    text_excludes(options, ARGUMENT_EXCLUDES),
                    options,
                ),
            ]),
            "<|close|>call<|sep|>",
        )
    };
    let tools_part = match choice {
        BuilderToolChoice::Auto if tools.is_empty() => None,
        BuilderToolChoice::Auto => Some(Format::optional(Format::tag(
            TOOLS_BEGIN,
            tools_with_separator(tools.iter().map(tool_tag).collect(), "", true, options),
            TOOLS_END,
        ))),
        BuilderToolChoice::Forced => Some(Format::sequence(vec![
            Format::const_string(TOOLS_BEGIN),
            Format::Tag(tool_tag(&tools[0])),
            Format::const_string(TOOLS_END),
        ])),
        BuilderToolChoice::Required => Some(Format::sequence(vec![
            Format::const_string(TOOLS_BEGIN),
            tools_with_separator(tools.iter().map(tool_tag).collect(), "", true, options),
            Format::const_string(TOOLS_END),
        ])),
    };
    let mut elements = Vec::new();
    if let Some(prefix) = reasoning_prefix(
        options,
        "<|open|>think<|sep|>",
        "<|close|>think<|sep|>",
        SPECIAL_EXCLUDES,
        "",
    ) {
        elements.push(prefix);
    }
    elements.push(Format::tag(
        if options.reasoning == ReasoningMode::Disabled {
            ""
        } else {
            "<|open|>response<|sep|>"
        },
        Format::any_text_excluding(text_excludes(options, SPECIAL_EXCLUDES)),
        "<|close|>response<|sep|>",
    ));
    elements.extend(tools_part);
    elements.push(Format::const_string("<|close|>message<|sep|>"));
    structural(Format::sequence(elements))
}
