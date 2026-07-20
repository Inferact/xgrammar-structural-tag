use crate::Result;
use crate::format::{Format, JsonSchemaStyle, StructuralTag, TagFormat};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, required_triggered_with_excludes, schema,
    structural, styled_schema, tag, text_excludes, triggered_with_excludes,
};

/// Qwen 3.5 XML tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct Qwen35Builder;

impl StructuralTagBuilder for Qwen35Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_qwen_35(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.options,
        ))
    }
}

fn qwen_35_tool_tag(tool: &FunctionToolParam, options: super::StructuralTagOptions) -> TagFormat {
    const TOOL_CALL_BEGIN_PREFIX: &str = "<tool_call>\n<function=";
    const TOOL_CALL_BEGIN_SUFFIX: &str = ">\n";
    const TOOL_CALL_END: &str = "\n</function>\n</tool_call>";
    tag(
        format!(
            "{TOOL_CALL_BEGIN_PREFIX}{}{TOOL_CALL_BEGIN_SUFFIX}",
            tool.function.name
        ),
        styled_schema(schema(&tool.function), JsonSchemaStyle::QwenXml, options),
        TOOL_CALL_END,
    )
}

/// Build a Qwen XML tool-call structural tag.
///
/// Reference: <https://huggingface.co/Qwen/Qwen3-Coder-480B-A35B-Instruct-FP8/blob/main/chat_template.jinja>
///
/// Backs the `qwen_3_5` and `qwen_3_coder` model keys. Supports Qwen3.5,
/// Qwen3.6, Qwen3-Coder, and Qwen3-Coder-Next. When `reasoning` is `true`, a
/// `</think>` reasoning prefix precedes the tool/text part.
pub(super) fn build_qwen_35(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
) -> StructuralTag {
    const TOOL_CALL_TRIGGER: &str = "<tool_call>\n<function=";
    const THINK_TAG_END: &str = "</think>";
    const THINK_SUFFIX: &str = "\n\n";
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];

    let suffix = match choice {
        BuilderToolChoice::Auto => {
            let tags = tools
                .iter()
                .map(|tool| qwen_35_tool_tag(tool, options))
                .collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(text_excludes(options, THINK_EXCLUDES))
            } else {
                triggered_with_excludes(
                    &[TOOL_CALL_TRIGGER],
                    tags,
                    text_excludes(options, THINK_EXCLUDES),
                )
            }
        }
        BuilderToolChoice::Forced => Format::Tag(qwen_35_tool_tag(&tools[0], options)),
        BuilderToolChoice::Required => {
            let tags = tools
                .iter()
                .map(|tool| qwen_35_tool_tag(tool, options))
                .collect::<Vec<_>>();
            required_triggered_with_excludes(
                &[TOOL_CALL_TRIGGER],
                tags,
                text_excludes(options, THINK_EXCLUDES),
            )
        }
    };
    if !options.reasoning {
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
