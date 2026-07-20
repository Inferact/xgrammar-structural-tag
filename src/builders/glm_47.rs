use crate::Result;
use crate::format::{Format, JsonSchemaStyle, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, required_triggered_with_excludes, schema,
    structural, styled_schema, tag, text_excludes, triggered_with_excludes,
};

/// GLM XML tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct Glm47Builder;

impl StructuralTagBuilder for Glm47Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_glm_47(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.options,
        ))
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
    options: super::StructuralTagOptions,
) -> StructuralTag {
    const TOOL_CALL_BEGIN_PREFIX: &str = "<tool_call>";
    const TOOL_CALL_END: &str = "</tool_call>";
    const TOOL_CALL_TRIGGER: &str = "<tool_call>";
    const THINK_TAG_END: &str = "</think>";
    const REASONING_EXCLUDES: &[&str] = &[
        "<think>",
        "</think>",
        TOOL_CALL_BEGIN_PREFIX,
        TOOL_CALL_END,
        "<arg_key>",
        "</arg_key>",
        "<arg_value>",
        "</arg_value>",
    ];
    const TEXT_EXCLUDES: &[&str] = &[
        "<think>",
        "</think>",
        TOOL_CALL_END,
        "<arg_key>",
        "</arg_key>",
        "<arg_value>",
        "</arg_value>",
    ];
    let tool_tag = |tool: &FunctionToolParam| {
        tag(
            format!("{TOOL_CALL_BEGIN_PREFIX}{}", tool.function.name),
            styled_schema(schema(&tool.function), JsonSchemaStyle::GlmXml, options),
            TOOL_CALL_END,
        )
    };
    let suffix = match choice {
        BuilderToolChoice::Auto => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(text_excludes(options, REASONING_EXCLUDES))
            } else {
                triggered_with_excludes(
                    &[TOOL_CALL_TRIGGER],
                    tags,
                    text_excludes(options, TEXT_EXCLUDES),
                )
            }
        }
        BuilderToolChoice::Forced => Format::Tag(tool_tag(&tools[0])),
        BuilderToolChoice::Required => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            required_triggered_with_excludes(
                &[TOOL_CALL_TRIGGER],
                tags,
                text_excludes(options, TEXT_EXCLUDES),
            )
        }
    };
    if !options.reasoning {
        return structural(suffix);
    }
    structural(Format::sequence(vec![
        Format::tag(
            "",
            Format::any_text_excluding(text_excludes(options, REASONING_EXCLUDES)),
            THINK_TAG_END,
        ),
        suffix,
    ]))
}
