use crate::Result;
use crate::format::{Format, JsonSchemaStyle, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, assemble, reasoning_prefix, schema, styled_schema,
    tag, text_excludes, tools_with_separator, triggered_with_excludes,
};

/// DeepSeek V3.2 DSML structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeepSeekV32Builder;

impl StructuralTagBuilder for DeepSeekV32Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_deepseek_v32(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.options,
        ))
    }
}

/// DeepSeek V4 DSML structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeepSeekV4Builder;

impl StructuralTagBuilder for DeepSeekV4Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_deepseek_v4(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.options,
        ))
    }
}

/// DeepSeek V4.1 spaced-DSML structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeepSeekV41Builder;

impl StructuralTagBuilder for DeepSeekV41Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_deepseek_v41(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.options,
        ))
    }
}

/// Build a DeepSeek DSML structural tag with model-specific markers and schema style.
fn build_deepseek_dsml(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
    calls: &str,
    invoke: &str,
    style: JsonSchemaStyle,
) -> StructuralTag {
    const TOOL_CALLS_PREFIX: &str = "\n\n";
    const THINK_TAG_END: &str = "</think>";
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];
    let function_calls_begin = format!("<｜DSML｜{calls}>\n");
    let function_calls_end = format!("</｜DSML｜{calls}>");
    let function_calls_trigger = format!("<｜DSML｜{calls}>");
    let tool_tag = |tool: &FunctionToolParam| {
        tag(
            format!("<｜DSML｜{invoke} name=\"{}\">\n", tool.function.name),
            styled_schema(schema(&tool.function), style, options),
            format!("</｜DSML｜{invoke}>\n"),
        )
    };

    let suffix = match choice {
        BuilderToolChoice::Auto => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                let excludes = if style == JsonSchemaStyle::DeepseekV4_1Xml {
                    vec!["<think>", "</think>", &function_calls_trigger]
                } else {
                    THINK_EXCLUDES.to_vec()
                };
                Format::any_text_excluding(text_excludes(options, &excludes))
            } else {
                let content = tools_with_separator(tags, "", true, options);
                let outer = tag(function_calls_begin, content, function_calls_end);
                triggered_with_excludes(
                    &[&function_calls_trigger],
                    vec![outer],
                    text_excludes(options, THINK_EXCLUDES),
                    options,
                )
            }
        }
        BuilderToolChoice::Forced => Format::sequence(vec![
            Format::const_string(format!("{TOOL_CALLS_PREFIX}{function_calls_begin}")),
            Format::Tag(tool_tag(&tools[0])),
            Format::const_string(function_calls_end),
        ]),
        BuilderToolChoice::Required => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            Format::sequence(vec![
                Format::const_string(format!("{TOOL_CALLS_PREFIX}{function_calls_begin}")),
                tools_with_separator(tags, "", true, options),
                Format::const_string(function_calls_end),
            ])
        }
    };
    assemble(
        reasoning_prefix(options, "<think>", THINK_TAG_END, THINK_EXCLUDES, ""),
        suffix,
    )
}

/// Build a DeepSeek-V3.2-style structural tag (DSML format).
///
/// Supports DeepSeek-V3.2.
pub(super) fn build_deepseek_v32(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
) -> StructuralTag {
    build_deepseek_dsml(
        tools,
        choice,
        options,
        "function_calls",
        "invoke",
        JsonSchemaStyle::DeepseekXml,
    )
}

/// Build a DeepSeek-V4-style structural tag (DSML format).
///
/// Supports DeepSeek-V4.
pub(super) fn build_deepseek_v4(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
) -> StructuralTag {
    build_deepseek_dsml(
        tools,
        choice,
        options,
        "tool_calls",
        "invoke",
        JsonSchemaStyle::DeepseekXml,
    )
}

/// Build a DeepSeek-V4.1-Flash structural tag with spaced DSML markers.
///
/// The generation prompt ends with `<think>` for reasoning or `</think>` for
/// chat. Reasoning effort and image inputs affect the prompt; the output
/// protocol uses the same grammar. The tokenizer's stop token handles EOS.
pub(super) fn build_deepseek_v41(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
) -> StructuralTag {
    build_deepseek_dsml(
        tools,
        choice,
        options,
        " calls",
        " invoke",
        JsonSchemaStyle::DeepseekV4_1Xml,
    )
}
