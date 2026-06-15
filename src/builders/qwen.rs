use crate::format::{Format, JsonSchemaStyle, StructuralTag, TagFormat};
use crate::tool::{FunctionToolParam, SimplifiedToolChoice};

use super::{
    json_schema, schema, structural, styled_schema, tag, tools_with_separator,
    triggered_with_excludes,
};

fn qwen_35_tool_tag(tool: &FunctionToolParam) -> TagFormat {
    const TOOL_CALL_BEGIN_PREFIX: &str = "<tool_call>\n<function=";
    const TOOL_CALL_BEGIN_SUFFIX: &str = ">\n";
    const TOOL_CALL_END: &str = "\n</function>\n</tool_call>";
    tag(
        format!(
            "{TOOL_CALL_BEGIN_PREFIX}{}{TOOL_CALL_BEGIN_SUFFIX}",
            tool.function.name
        ),
        styled_schema(schema(&tool.function), JsonSchemaStyle::QwenXml),
        TOOL_CALL_END,
    )
}

pub(super) fn build_qwen_35(
    tools: &[FunctionToolParam],
    choice: SimplifiedToolChoice,
    reasoning: bool,
) -> StructuralTag {
    const TOOL_CALL_TRIGGER: &str = "<tool_call>\n<function=";
    const THINK_TAG_END: &str = "</think>";
    const THINK_SUFFIX: &str = "\n\n";
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];

    let suffix = match choice {
        SimplifiedToolChoice::Auto => {
            let tags = tools.iter().map(qwen_35_tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(THINK_EXCLUDES)
            } else {
                triggered_with_excludes(&[TOOL_CALL_TRIGGER], tags, THINK_EXCLUDES)
            }
        }
        SimplifiedToolChoice::Forced => Format::Tag(qwen_35_tool_tag(&tools[0])),
        SimplifiedToolChoice::Required => {
            let tags = tools.iter().map(qwen_35_tool_tag).collect::<Vec<_>>();
            tools_with_separator(tags, "\n", true)
        }
    };
    if !reasoning {
        return structural(suffix);
    }
    structural(Format::sequence(vec![
        Format::sequence(vec![
            Format::tag("", Format::any_text(), THINK_TAG_END),
            Format::const_string(THINK_SUFFIX),
        ]),
        suffix,
    ]))
}

pub(super) fn build_qwen_3(
    tools: &[FunctionToolParam],
    choice: SimplifiedToolChoice,
    reasoning: bool,
) -> StructuralTag {
    const TOOL_CALL_BEGIN_PREFIX: &str = "<tool_call>\n{\"name\": \"";
    const ARGUMENTS_FIELD_PREFIX: &str = "\", \"arguments\": ";
    const TOOL_CALL_END: &str = "}\n</tool_call>";
    const TOOL_CALL_TRIGGER: &str = "<tool_call>";
    const THINK_TAG_END: &str = "</think>";
    const THINK_SUFFIX: &str = "\n\n";
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];

    let tool_tag = |tool: &FunctionToolParam| {
        tag(
            format!(
                "{TOOL_CALL_BEGIN_PREFIX}{}{ARGUMENTS_FIELD_PREFIX}",
                tool.function.name
            ),
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
                triggered_with_excludes(&[TOOL_CALL_TRIGGER], tags, THINK_EXCLUDES)
            }
        }
        SimplifiedToolChoice::Forced => Format::Tag(tool_tag(&tools[0])),
        SimplifiedToolChoice::Required => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            tools_with_separator(tags, "\n", true)
        }
    };
    if !reasoning {
        return structural(suffix);
    }
    structural(Format::sequence(vec![
        Format::sequence(vec![
            Format::tag("", Format::any_text(), THINK_TAG_END),
            Format::const_string(THINK_SUFFIX),
        ]),
        suffix,
    ]))
}
