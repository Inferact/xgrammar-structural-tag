use crate::Result;
use crate::format::{Format, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    ReasoningMode, StructuralTagBuilder, StructuralTagContext, assemble, json_schema,
    reasoning_prefix, required_triggered_with_excludes, schema, tag, text_excludes,
    triggered_with_excludes,
};

/// EXAONE 4.0 tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExaoneBuilder;

impl StructuralTagBuilder for ExaoneBuilder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_exaone(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.options,
        ))
    }
}

/// Build an EXAONE 4.0 JSON-in-tag structural tag.
///
/// Reference: <https://huggingface.co/LGAI-EXAONE/EXAONE-4.0-32B>
///
/// Supports EXAONE-4.0-32B and EXAONE-4.0-1.2B.
pub(super) fn build_exaone(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
) -> StructuralTag {
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];
    let tool_tag = |tool: &FunctionToolParam| {
        tag(
            format!(
                "<tool_call>{{\"name\": \"{}\", \"arguments\": ",
                tool.function.name
            ),
            json_schema(schema(&tool.function), options),
            "}</tool_call>",
        )
    };
    let suffix = match choice {
        BuilderToolChoice::Auto => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(text_excludes(options, THINK_EXCLUDES))
            } else {
                triggered_with_excludes(
                    &["<tool_call>"],
                    tags,
                    text_excludes(options, THINK_EXCLUDES),
                    options,
                )
            }
        }
        BuilderToolChoice::Forced => Format::Tag(tool_tag(&tools[0])),
        BuilderToolChoice::Required => required_triggered_with_excludes(
            &["<tool_call>"],
            tools.iter().map(tool_tag).collect(),
            text_excludes(options, THINK_EXCLUDES),
            options,
        ),
    };
    let excludes = if options.reasoning == ReasoningMode::Enabled {
        &[][..]
    } else {
        THINK_EXCLUDES
    };
    assemble(
        reasoning_prefix(options, "<think>", "</think>", excludes, "\n\n"),
        suffix,
    )
}
