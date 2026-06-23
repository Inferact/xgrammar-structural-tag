use crate::format::{Format, JsonSchemaStyle, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, schema, styled_schema, tag, tools_with_separator,
    triggered_with_excludes, with_optional_reasoning,
};

/// GLM XML tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct Glm47Builder;

impl StructuralTagBuilder for Glm47Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> StructuralTag {
        build_glm_47(ctx.function_tools, ctx.tool_choice, ctx.reasoning)
    }
}

/// Build a GLM-4.7 / GLM-5 structural tag.
///
/// The GLM tool-call format uses XML-like tags:
/// `<tool_call>name<arg_key>key</arg_key><arg_value>value</arg_value></tool_call>`.
///
/// Supports GLM-5 and GLM-4.7.
pub(super) fn build_glm_47(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
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
        BuilderToolChoice::Auto => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(THINK_EXCLUDES)
            } else {
                triggered_with_excludes(&[TOOL_CALL_TRIGGER], tags, THINK_EXCLUDES)
            }
        }
        BuilderToolChoice::Forced => Format::Tag(tool_tag(&tools[0])),
        BuilderToolChoice::Required => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            tools_with_separator(tags, "", true)
        }
    };
    with_optional_reasoning(suffix, reasoning, THINK_TAG_END)
}
