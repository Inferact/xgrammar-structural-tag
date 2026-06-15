use crate::format::{Format, JsonSchemaStyle, StructuralTag, TagFormat};
use crate::tool::{FunctionToolParam, SimplifiedToolChoice};

use super::{
    json_schema, schema, styled_schema, tag, tools_with_separator, triggered_with_excludes,
    with_optional_reasoning,
};

pub(super) fn build_deepseek_r1(
    tools: &[FunctionToolParam],
    choice: SimplifiedToolChoice,
    reasoning: bool,
) -> StructuralTag {
    const TOOL_CALLS_BEGIN: &str = "<｜tool▁calls▁begin｜>";
    const TOOL_CALLS_END: &str = "<｜tool▁calls▁end｜>";
    const TOOL_CALL_BEGIN: &str = "<｜tool▁call▁begin｜>";
    const TOOL_CALL_END: &str = "<｜tool▁call▁end｜>";
    const TOOL_SEP: &str = "<｜tool▁sep｜>";
    const JSON_RENDER_BEGIN: &str = "\n```json\n";
    const JSON_RENDER_END: &str = "\n```";
    const THINK_TAG_END: &str = "</think>";
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];

    let tool_tag = |tool: &FunctionToolParam| {
        tag(
            format!(
                "{TOOL_CALL_BEGIN}function{TOOL_SEP}{}{JSON_RENDER_BEGIN}",
                tool.function.name
            ),
            json_schema(schema(&tool.function)),
            format!("{JSON_RENDER_END}{TOOL_CALL_END}"),
        )
    };

    let suffix = match choice {
        SimplifiedToolChoice::Auto => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(THINK_EXCLUDES)
            } else {
                let inner = tools_with_separator(tags, "\n", true);
                let tool_calls = tag(TOOL_CALLS_BEGIN, inner, TOOL_CALLS_END);
                triggered_with_excludes(&[TOOL_CALLS_BEGIN], vec![tool_calls], THINK_EXCLUDES)
            }
        }
        SimplifiedToolChoice::Forced => {
            let function = &tools[0].function;
            Format::tag(
                format!(
                    "{TOOL_CALLS_BEGIN}{TOOL_CALL_BEGIN}function{TOOL_SEP}{}{JSON_RENDER_BEGIN}",
                    function.name
                ),
                json_schema(schema(function)),
                format!("{JSON_RENDER_END}{TOOL_CALL_END}{TOOL_CALLS_END}"),
            )
        }
        SimplifiedToolChoice::Required => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            Format::tag(
                TOOL_CALLS_BEGIN,
                tools_with_separator(tags, "\n", true),
                TOOL_CALLS_END,
            )
        }
    };
    with_optional_reasoning(suffix, reasoning, THINK_TAG_END)
}

pub(super) fn build_deepseek_v31(
    tools: &[FunctionToolParam],
    choice: SimplifiedToolChoice,
    reasoning: bool,
) -> StructuralTag {
    const TOOL_CALLS_BEGIN: &str = "<｜tool▁calls▁begin｜>";
    const TOOL_CALLS_END: &str = "<｜tool▁calls▁end｜>";
    const TOOL_CALL_BEGIN: &str = "<｜tool▁call▁begin｜>";
    const TOOL_CALL_END: &str = "<｜tool▁call▁end｜>";
    const TOOL_SEP: &str = "<｜tool▁sep｜>";
    const THINK_TAG_END: &str = "</think>";
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];

    let tool_tag = |tool: &FunctionToolParam| {
        tag(
            format!("{TOOL_CALL_BEGIN}{}{TOOL_SEP}", tool.function.name),
            json_schema(schema(&tool.function)),
            TOOL_CALL_END,
        )
    };
    let suffix = match choice {
        SimplifiedToolChoice::Auto => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(THINK_EXCLUDES)
            } else {
                let inner = tools_with_separator(tags, "", true);
                let tool_calls = tag(TOOL_CALLS_BEGIN, inner, TOOL_CALLS_END);
                triggered_with_excludes(&[TOOL_CALLS_BEGIN], vec![tool_calls], THINK_EXCLUDES)
            }
        }
        SimplifiedToolChoice::Forced => {
            let function = &tools[0].function;
            Format::tag(
                format!(
                    "{TOOL_CALLS_BEGIN}{TOOL_CALL_BEGIN}{}{TOOL_SEP}",
                    function.name
                ),
                json_schema(schema(function)),
                format!("{TOOL_CALL_END}{TOOL_CALLS_END}"),
            )
        }
        SimplifiedToolChoice::Required => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            Format::tag(
                TOOL_CALLS_BEGIN,
                tools_with_separator(tags, "", true),
                TOOL_CALLS_END,
            )
        }
    };
    with_optional_reasoning(suffix, reasoning, THINK_TAG_END)
}

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
