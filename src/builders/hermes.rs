use crate::format::{Format, StructuralTag, TagFormat};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, json_schema, schema, structural, tag,
    tools_with_separator,
};

/// Hermes tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct HermesBuilder;

impl StructuralTagBuilder for HermesBuilder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> StructuralTag {
        build_hermes(ctx.function_tools, ctx.tool_choice)
    }
}

/// Build a Hermes-style structural tag.
///
/// Local extension not present in upstream xgrammar. Hermes emits tool calls as
/// `<tool_call>{"name": ..., "arguments": {...}}</tool_call>`, and the format
/// has no reasoning part.
pub(super) fn build_hermes(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
) -> StructuralTag {
    fn hermes_tool_tags(tools: &[FunctionToolParam]) -> Vec<TagFormat> {
        const ARGUMENTS_FIELD_PREFIX: &str = "\", \"arguments\": ";
        let formats = [
            ("<tool_call>\n{\"name\": \"", "}\n</tool_call>"),
            ("<tool_call>{\"name\": \"", "}</tool_call>"),
        ];
        tools
            .iter()
            .flat_map(|tool| {
                formats.iter().map(move |(begin, end)| {
                    tag(
                        format!("{begin}{}{ARGUMENTS_FIELD_PREFIX}", tool.function.name),
                        json_schema(schema(&tool.function)),
                        *end,
                    )
                })
            })
            .collect()
    }

    let suffix = match choice {
        BuilderToolChoice::Auto => {
            let tags = hermes_tool_tags(tools);
            if tags.is_empty() {
                Format::any_text()
            } else {
                Format::triggered_tags(&["<tool_call>"], tags)
            }
        }
        BuilderToolChoice::Forced => {
            Format::tags_with_separator(hermes_tool_tags(tools), "", true, true)
        }
        BuilderToolChoice::Required => tools_with_separator(hermes_tool_tags(tools), "", true),
    };
    structural(suffix)
}
