use crate::format::{Format, JsonSchemaStyle, StructuralTag, TagFormat};
use crate::tool::{FunctionToolParam, SimplifiedToolChoice};

use super::{
    schema, styled_schema, tag, tools_with_separator, triggered_with_excludes,
    with_optional_reasoning,
};

fn dsml_tool_tag(
    tool: &FunctionToolParam,
    invoke_begin_prefix: &str,
    invoke_begin_suffix: &str,
    invoke_end: &str,
) -> TagFormat {
    tag(
        format!(
            "{invoke_begin_prefix}{}{invoke_begin_suffix}",
            tool.function.name
        ),
        styled_schema(schema(&tool.function), JsonSchemaStyle::DeepseekXml),
        invoke_end,
    )
}

fn build_deepseek_dsml(
    tools: &[FunctionToolParam],
    choice: SimplifiedToolChoice,
    reasoning: bool,
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
        SimplifiedToolChoice::Auto => {
            let tags = tools
                .iter()
                .map(|tool| {
                    dsml_tool_tag(tool, INVOKE_BEGIN_PREFIX, INVOKE_BEGIN_SUFFIX, INVOKE_END)
                })
                .collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(THINK_EXCLUDES)
            } else {
                let content = tools_with_separator(tags, INVOKE_SEPARATOR, true);
                let outer = tag(function_calls_begin, content, function_calls_end);
                triggered_with_excludes(&[function_calls_trigger], vec![outer], THINK_EXCLUDES)
            }
        }
        SimplifiedToolChoice::Forced => Format::sequence(vec![
            Format::const_string(format!("{TOOL_CALLS_PREFIX}{function_calls_begin}")),
            Format::Tag(dsml_tool_tag(
                &tools[0],
                INVOKE_BEGIN_PREFIX,
                INVOKE_BEGIN_SUFFIX,
                INVOKE_END,
            )),
            Format::const_string(function_calls_end),
        ]),
        SimplifiedToolChoice::Required => {
            let tags = tools
                .iter()
                .map(|tool| {
                    dsml_tool_tag(tool, INVOKE_BEGIN_PREFIX, INVOKE_BEGIN_SUFFIX, INVOKE_END)
                })
                .collect::<Vec<_>>();
            Format::sequence(vec![
                Format::const_string(format!("{TOOL_CALLS_PREFIX}{function_calls_begin}")),
                tools_with_separator(tags, INVOKE_SEPARATOR, true),
                Format::const_string(function_calls_end),
            ])
        }
    };
    with_optional_reasoning(suffix, reasoning, THINK_TAG_END)
}

pub(super) fn build_deepseek_v32(
    tools: &[FunctionToolParam],
    choice: SimplifiedToolChoice,
    reasoning: bool,
) -> StructuralTag {
    build_deepseek_dsml(
        tools,
        choice,
        reasoning,
        "<｜DSML｜function_calls>\n",
        "</｜DSML｜function_calls>",
        "<｜DSML｜function_calls>",
    )
}

pub(super) fn build_deepseek_v4(
    tools: &[FunctionToolParam],
    choice: SimplifiedToolChoice,
    reasoning: bool,
) -> StructuralTag {
    build_deepseek_dsml(
        tools,
        choice,
        reasoning,
        "<｜DSML｜tool_calls>\n",
        "</｜DSML｜tool_calls>",
        "<｜DSML｜tool_calls>",
    )
}
