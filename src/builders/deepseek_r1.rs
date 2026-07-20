use crate::Result;
use crate::format::{Format, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, json_schema, schema, tag, text_excludes,
    tools_with_separator, triggered_with_excludes, with_optional_reasoning,
};

/// DeepSeek R1 tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeepSeekR1Builder;

impl StructuralTagBuilder for DeepSeekR1Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_deepseek_r1(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.options,
        ))
    }
}

/// Build a DeepSeek-R1-style structural tag.
///
/// Reference: <https://huggingface.co/deepseek-ai/DeepSeek-R1/blob/main/tokenizer_config.json>
///
/// Supports DeepSeek-R1 and DeepSeek-R1-0528.
pub(super) fn build_deepseek_r1(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
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
            json_schema(schema(&tool.function), options),
            format!("{JSON_RENDER_END}{TOOL_CALL_END}"),
        )
    };

    let suffix = match choice {
        BuilderToolChoice::Auto => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(text_excludes(options, THINK_EXCLUDES))
            } else {
                let inner = tools_with_separator(tags, "\n", true);
                let tool_calls = tag(TOOL_CALLS_BEGIN, inner, TOOL_CALLS_END);
                triggered_with_excludes(
                    &[TOOL_CALLS_BEGIN],
                    vec![tool_calls],
                    text_excludes(options, THINK_EXCLUDES),
                )
            }
        }
        BuilderToolChoice::Forced => {
            let function = &tools[0].function;
            Format::tag(
                format!(
                    "{TOOL_CALLS_BEGIN}{TOOL_CALL_BEGIN}function{TOOL_SEP}{}{JSON_RENDER_BEGIN}",
                    function.name
                ),
                json_schema(schema(function), options),
                format!("{JSON_RENDER_END}{TOOL_CALL_END}{TOOL_CALLS_END}"),
            )
        }
        BuilderToolChoice::Required => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            Format::tag(
                TOOL_CALLS_BEGIN,
                tools_with_separator(tags, "\n", true),
                TOOL_CALLS_END,
            )
        }
    };
    with_optional_reasoning(suffix, options.reasoning, THINK_TAG_END)
}
