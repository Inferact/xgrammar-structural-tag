use crate::Result;
use crate::format::{Format, JsonSchemaStyle, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, schema, structural, styled_schema, tag,
    tools_with_separator, triggered_with_excludes,
};

/// MiniMax XML tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct MinimaxBuilder;

impl StructuralTagBuilder for MinimaxBuilder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_minimax(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.reasoning,
        ))
    }
}

/// Build a MiniMax-style structural tag.
///
/// Supports MiniMax-M2.5 and MiniMax-M2.7.
pub(super) fn build_minimax(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    reasoning: bool,
) -> StructuralTag {
    const INVOKE_BEGIN_PREFIX: &str = "<invoke name=\"";
    const INVOKE_BEGIN_SUFFIX: &str = "\">\n";
    const INVOKE_END: &str = "</invoke>\n";
    const TOOL_CALL_BEGIN: &str = "<minimax:tool_call>\n";
    const TOOL_CALL_END: &str = "</minimax:tool_call>";
    const TOOL_CALL_TRIGGER: &str = "<minimax:tool_call>";
    const THINK_TAG_END: &str = "</think>";
    const THINK_SUFFIX: &str = "\n\n";
    const EMPTY_THINK_CONTENT: &str = "\n</think>\n\n";
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];

    let tool_tags = || {
        tools
            .iter()
            .map(|tool| {
                tag(
                    format!(
                        "{INVOKE_BEGIN_PREFIX}{}{INVOKE_BEGIN_SUFFIX}",
                        tool.function.name
                    ),
                    styled_schema(schema(&tool.function), JsonSchemaStyle::MinimaxXml),
                    INVOKE_END,
                )
            })
            .collect::<Vec<_>>()
    };

    let suffix = match choice {
        BuilderToolChoice::Auto => {
            let tags = tool_tags();
            if tags.is_empty() {
                Format::any_text_excluding(THINK_EXCLUDES)
            } else {
                let function_calling = tools_with_separator(tags, "", true);
                triggered_with_excludes(
                    &[TOOL_CALL_TRIGGER],
                    vec![tag(TOOL_CALL_BEGIN, function_calling, TOOL_CALL_END)],
                    THINK_EXCLUDES,
                )
            }
        }
        BuilderToolChoice::Forced => Format::sequence(vec![
            Format::const_string(format!("\n{TOOL_CALL_BEGIN}")),
            Format::Tag(tool_tags().remove(0)),
            Format::const_string(TOOL_CALL_END),
        ]),
        BuilderToolChoice::Required => Format::sequence(vec![
            Format::const_string(format!("\n{TOOL_CALL_BEGIN}")),
            tools_with_separator(tool_tags(), "", true),
            Format::const_string(TOOL_CALL_END),
        ]),
    };

    let think = if reasoning {
        Format::tag("", Format::any_text(), THINK_TAG_END)
    } else {
        Format::const_string(EMPTY_THINK_CONTENT)
    };
    structural(Format::sequence(vec![
        think,
        Format::const_string(THINK_SUFFIX),
        suffix,
    ]))
}
