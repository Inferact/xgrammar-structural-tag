use crate::Result;
use crate::format::{Format, JsonSchemaStyle, StructuralTag, TagFormat};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, schema, styled_schema, tag, text_excludes,
    tools_with_separator, triggered_with_excludes, with_optional_reasoning,
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

fn dsml_tool_tag(
    tool: &FunctionToolParam,
    invoke_begin_prefix: &str,
    invoke_begin_suffix: &str,
    invoke_end: &str,
    options: super::StructuralTagOptions,
) -> TagFormat {
    tag(
        format!(
            "{invoke_begin_prefix}{}{invoke_begin_suffix}",
            tool.function.name
        ),
        styled_schema(
            schema(&tool.function),
            JsonSchemaStyle::DeepseekXml,
            options,
        ),
        invoke_end,
    )
}

/// Build a DeepSeek DSML structural tag, shared by V3.2 and V4.
///
/// Both wrap each tool call in `<｜DSML｜invoke name="…">…</｜DSML｜invoke>` with
/// DeepSeek XML arguments, differing only in the outer function-calls markers.
fn build_deepseek_dsml(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
    function_calls_begin: &str,
    function_calls_end: &str,
    function_calls_trigger: &str,
) -> StructuralTag {
    const INVOKE_BEGIN_PREFIX: &str = "<｜DSML｜invoke name=\"";
    const INVOKE_BEGIN_SUFFIX: &str = "\">\n";
    const INVOKE_END: &str = "</｜DSML｜invoke>\n";
    const INVOKE_SEPARATOR: &str = "";
    const TOOL_CALLS_PREFIX: &str = "\n\n";
    const THINK_TAG_END: &str = "</think>";
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];

    let suffix = match choice {
        BuilderToolChoice::Auto => {
            let tags = tools
                .iter()
                .map(|tool| {
                    dsml_tool_tag(
                        tool,
                        INVOKE_BEGIN_PREFIX,
                        INVOKE_BEGIN_SUFFIX,
                        INVOKE_END,
                        options,
                    )
                })
                .collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(text_excludes(options, THINK_EXCLUDES))
            } else {
                let content = tools_with_separator(tags, INVOKE_SEPARATOR, true);
                let outer = tag(function_calls_begin, content, function_calls_end);
                triggered_with_excludes(
                    &[function_calls_trigger],
                    vec![outer],
                    text_excludes(options, THINK_EXCLUDES),
                )
            }
        }
        BuilderToolChoice::Forced => Format::sequence(vec![
            Format::const_string(format!("{TOOL_CALLS_PREFIX}{function_calls_begin}")),
            Format::Tag(dsml_tool_tag(
                &tools[0],
                INVOKE_BEGIN_PREFIX,
                INVOKE_BEGIN_SUFFIX,
                INVOKE_END,
                options,
            )),
            Format::const_string(function_calls_end),
        ]),
        BuilderToolChoice::Required => {
            let tags = tools
                .iter()
                .map(|tool| {
                    dsml_tool_tag(
                        tool,
                        INVOKE_BEGIN_PREFIX,
                        INVOKE_BEGIN_SUFFIX,
                        INVOKE_END,
                        options,
                    )
                })
                .collect::<Vec<_>>();
            Format::sequence(vec![
                Format::const_string(format!("{TOOL_CALLS_PREFIX}{function_calls_begin}")),
                tools_with_separator(tags, INVOKE_SEPARATOR, true),
                Format::const_string(function_calls_end),
            ])
        }
    };
    with_optional_reasoning(suffix, options.reasoning, THINK_TAG_END)
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
        "<｜DSML｜function_calls>\n",
        "</｜DSML｜function_calls>",
        "<｜DSML｜function_calls>",
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
        "<｜DSML｜tool_calls>\n",
        "</｜DSML｜tool_calls>",
        "<｜DSML｜tool_calls>",
    )
}
