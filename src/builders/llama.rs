use crate::Result;
use crate::format::{Format, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, json_schema, required_triggered_with_excludes,
    schema, structural, tag, text_excludes, triggered_with_excludes,
};

/// Llama JSON function-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct LlamaBuilder;

impl StructuralTagBuilder for LlamaBuilder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_llama(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.options,
        ))
    }
}

/// Build a Llama-style structural tag.
///
/// Reference: <https://www.llama.com/docs/model-cards-and-prompt-formats/llama3_1/>
///
/// Supports Meta-Llama-3, Llama-3.1, and Llama-3.2. This format has no
/// reasoning part, so `reasoning` is ignored.
pub(super) fn build_llama(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
) -> StructuralTag {
    const TOOL_NAME_PREFIX: &str = "{\"name\": \"";
    const PARAMETERS_FIELD_PREFIX: &str = "\", \"parameters\": ";
    const TOOL_OBJECT_BEGIN_PREFIX: &str = "{\"name\": \"";
    const TOOL_OBJECT_PARAMETERS_PREFIX: &str = "\", \"parameters\": ";
    const TOOLS_TRIGGER: &str = "{\"name\": ";
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];

    let suffix = match choice {
        BuilderToolChoice::Auto => {
            let tags = tools
                .iter()
                .map(|tool| {
                    tag(
                        format!(
                            "{TOOL_OBJECT_BEGIN_PREFIX}{}{TOOL_OBJECT_PARAMETERS_PREFIX}",
                            tool.function.name
                        ),
                        json_schema(schema(&tool.function), options),
                        "}",
                    )
                })
                .collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(text_excludes(options, THINK_EXCLUDES))
            } else {
                triggered_with_excludes(
                    &[TOOLS_TRIGGER],
                    tags,
                    text_excludes(options, THINK_EXCLUDES),
                )
            }
        }
        BuilderToolChoice::Forced => {
            let function = &tools[0].function;
            Format::tag(
                format!(
                    "{TOOL_NAME_PREFIX}{}{PARAMETERS_FIELD_PREFIX}",
                    function.name
                ),
                json_schema(schema(function), options),
                "}",
            )
        }
        BuilderToolChoice::Required => {
            let tags = tools
                .iter()
                .map(|tool| {
                    tag(
                        format!(
                            "{TOOL_OBJECT_BEGIN_PREFIX}{}{TOOL_OBJECT_PARAMETERS_PREFIX}",
                            tool.function.name
                        ),
                        json_schema(schema(&tool.function), options),
                        "}",
                    )
                })
                .collect();
            required_triggered_with_excludes(
                &[TOOLS_TRIGGER],
                tags,
                text_excludes(options, THINK_EXCLUDES),
            )
        }
    };
    structural(suffix)
}
