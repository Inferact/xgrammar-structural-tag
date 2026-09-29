use crate::Result;
use crate::format::{Format, JsonSchemaStyle, StructuralTag, TagFormat};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, assemble, reasoning_prefix_with_opener,
    required_triggered_with_excludes, schema, styled_schema, tag, text_excludes,
    triggered_with_excludes,
};

/// MiMo compact-XML tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct MimoBuilder;

impl StructuralTagBuilder for MimoBuilder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_mimo(ctx.function_tools, ctx.tool_choice, ctx.options))
    }
}

fn mimo_tool_tag(tool: &FunctionToolParam, options: super::StructuralTagOptions) -> TagFormat {
    tag(
        format!("<tool_call><function={}>", tool.function.name),
        styled_schema(schema(&tool.function), JsonSchemaStyle::QwenXml, options),
        "</function></tool_call>",
    )
}

/// Build a MiMo tool-call structural tag.
///
/// Reference: <https://huggingface.co/XiaomiMiMo/MiMo-V2.6-Pro-RL/blob/main/chat_template.jinja>
///
/// MiMo uses the Qwen XML parameter format without the newlines Qwen3-Coder puts between the
/// wrapper tags:
///
/// ```text
/// <tool_call><function=NAME><parameter=KEY>VALUE</parameter></function></tool_call>
/// ```
///
/// Supports MiMo-V2.6-Pro-RL and MiMo-V2.6-Flash-RL. The generation prompt ends at
/// `<|im_start|>assistant\n` when thinking is enabled, so enabled reasoning generates the complete
/// `<think>...</think>` block. When thinking is disabled the template renders `<think></think>`
/// into the prompt; use disabled reasoning there, also when the serving engine manages reasoning
/// itself.
pub(super) fn build_mimo(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
) -> StructuralTag {
    const TOOL_CALL_TRIGGER: &str = "<tool_call>";
    const THINK_TAG_BEGIN: &str = "<think>";
    const THINK_TAG_END: &str = "</think>";
    const TEXT_EXCLUDES: &[&str] = &["<think>", "</think>", "</tool_call>", "<function="];
    const REASONING_EXCLUDES: &[&str] = &[
        "<tool_call>",
        "<think>",
        "</think>",
        "</tool_call>",
        "<function=",
    ];

    let tags = || {
        tools
            .iter()
            .map(|tool| mimo_tool_tag(tool, options))
            .collect::<Vec<_>>()
    };
    let suffix = match choice {
        BuilderToolChoice::Auto if tools.is_empty() => {
            Format::any_text_excluding(text_excludes(options, REASONING_EXCLUDES))
        }
        BuilderToolChoice::Auto => triggered_with_excludes(
            &[TOOL_CALL_TRIGGER],
            tags(),
            text_excludes(options, TEXT_EXCLUDES),
            options,
        ),
        BuilderToolChoice::Forced => Format::Tag(mimo_tool_tag(&tools[0], options)),
        BuilderToolChoice::Required => required_triggered_with_excludes(
            &[TOOL_CALL_TRIGGER],
            tags(),
            text_excludes(options, TEXT_EXCLUDES),
            options,
        ),
    };
    assemble(
        reasoning_prefix_with_opener(
            options,
            false,
            THINK_TAG_BEGIN,
            THINK_TAG_END,
            REASONING_EXCLUDES,
            "",
        ),
        suffix,
    )
}
