use crate::Result;
use crate::format::{Format, JsonSchemaStyle, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    ReasoningMode, StructuralTagBuilder, StructuralTagContext, assemble, reasoning_prefix, schema,
    styled_schema, tag, text_excludes, tools_with_separator, triggered_with_excludes,
};

/// Cohere XML tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct CohereBuilder;

impl StructuralTagBuilder for CohereBuilder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_cohere(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.options,
        ))
    }
}

/// Build a structural tag for Cohere Command models using XML tool calls.
pub(super) fn build_cohere(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
) -> StructuralTag {
    const BEGIN: &str = "<cofl:tool_calls>";
    const END: &str = "</cofl:tool_calls>";
    const THINK_BEGIN: &str = "<|START_THINKING|>";
    const THINK_END: &str = "<|END_THINKING|>";
    const THINK_EXCLUDES: &[&str] = &[THINK_BEGIN, THINK_END];
    let tool_tag = |tool: &FunctionToolParam| {
        tag(
            "<cofl:tool_call id=\"",
            Format::sequence(vec![
                Format::regex(r"\d+"),
                Format::const_string(format!("\" name=\"{}\">", tool.function.name)),
                styled_schema(schema(&tool.function), JsonSchemaStyle::CohereXml, options),
            ]),
            "</cofl:tool_call>",
        )
    };
    let suffix = match choice {
        BuilderToolChoice::Auto => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(text_excludes(options, THINK_EXCLUDES))
            } else {
                triggered_with_excludes(
                    &[BEGIN],
                    vec![tag(
                        BEGIN,
                        tools_with_separator(tags, "", true, options),
                        END,
                    )],
                    text_excludes(options, THINK_EXCLUDES),
                    options,
                )
            }
        }
        BuilderToolChoice::Forced => Format::sequence(vec![
            Format::const_string(BEGIN),
            Format::Tag(tool_tag(&tools[0])),
            Format::const_string(END),
        ]),
        BuilderToolChoice::Required => Format::sequence(vec![
            Format::const_string(BEGIN),
            tools_with_separator(tools.iter().map(tool_tag).collect(), "", true, options),
            Format::const_string(END),
        ]),
    };
    let excludes = if options.reasoning == ReasoningMode::Enabled {
        &[][..]
    } else {
        THINK_EXCLUDES
    };
    assemble(
        reasoning_prefix(options, THINK_BEGIN, THINK_END, excludes, ""),
        suffix,
    )
}
