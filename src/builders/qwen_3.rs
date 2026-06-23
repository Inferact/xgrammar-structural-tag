use crate::Result;
use crate::format::{Format, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, json_schema, schema, structural, tag,
    tools_with_separator, triggered_with_excludes,
};

/// Qwen 3 JSON-in-tag tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct Qwen3Builder;

impl StructuralTagBuilder for Qwen3Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_qwen_3(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.reasoning,
        ))
    }
}

/// Build a Qwen3-style structural tag.
///
/// Reference: <https://qwen.readthedocs.io/en/latest/framework/function_call.html>
///
/// Supports Qwen3 and Qwen3-Next.
pub(super) fn build_qwen_3(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
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
