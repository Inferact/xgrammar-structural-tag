use crate::format::{Format, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, json_schema, schema, tag, tools_with_separator,
    triggered_with_excludes, with_optional_reasoning,
};

/// DeepSeek V3.1 tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeepSeekV31Builder;

impl StructuralTagBuilder for DeepSeekV31Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> StructuralTag {
        build_deepseek_v31(ctx.function_tools, ctx.tool_choice, ctx.reasoning)
    }
}

/// Build a DeepSeek-V3.1-style structural tag.
///
/// Reference: <https://huggingface.co/deepseek-ai/DeepSeek-V3.1/blob/main/tokenizer_config.json>
///
/// Supports DeepSeek-V3.1 and DeepSeek-V3.2-Exp. (DeepSeek-V3.2 itself uses the
/// DSML format built by `build_deepseek_v32`.)
pub(super) fn build_deepseek_v31(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
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
        BuilderToolChoice::Auto => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(THINK_EXCLUDES)
            } else {
                let inner = tools_with_separator(tags, "", true);
                let tool_calls = tag(TOOL_CALLS_BEGIN, inner, TOOL_CALLS_END);
                triggered_with_excludes(&[TOOL_CALLS_BEGIN], vec![tool_calls], THINK_EXCLUDES)
            }
        }
        BuilderToolChoice::Forced => {
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
        BuilderToolChoice::Required => {
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
