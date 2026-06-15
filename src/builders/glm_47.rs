use crate::format::{Format, JsonSchemaStyle, StructuralTag};
use crate::tool::{FunctionToolParam, SimplifiedToolChoice};

use super::{
    schema, styled_schema, tag, tools_with_separator, triggered_with_excludes,
    with_optional_reasoning,
};

pub(super) fn build_glm_47(
    tools: &[FunctionToolParam],
    choice: SimplifiedToolChoice,
    reasoning: bool,
) -> StructuralTag {
    const TOOL_CALL_BEGIN_PREFIX: &str = "<tool_call>";
    const TOOL_CALL_END: &str = "</tool_call>";
    const TOOL_CALL_TRIGGER: &str = "<tool_call>";
    const THINK_TAG_END: &str = "</think>";
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];

    let tool_tag = |tool: &FunctionToolParam| {
        tag(
            format!("{TOOL_CALL_BEGIN_PREFIX}{}", tool.function.name),
            styled_schema(schema(&tool.function), JsonSchemaStyle::GlmXml),
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
            tools_with_separator(tags, "", true)
        }
    };
    with_optional_reasoning(suffix, reasoning, THINK_TAG_END)
}
