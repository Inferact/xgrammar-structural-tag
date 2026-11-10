use crate::Result;
use crate::format::{Format, JsonSchemaStyle, StructuralTag, TagFormat};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, assemble, reasoning_prefix_with_opener,
    required_triggered_with_excludes, schema, styled_schema_excluding, tag, text_excludes,
    triggered_with_excludes,
};

/// Gemma 4 channel-marker tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct Gemma4Builder;

impl StructuralTagBuilder for Gemma4Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_gemma_4(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.options,
        ))
    }
}

const TOOL_CALL_BEGIN_PREFIX: &str = "<|tool_call>call:";
const TOOL_CALL_END: &str = "<tool_call|>";
const TOOL_CALL_TRIGGER: &str = "<|tool_call>";
const THINK_TAG_BEGIN: &str = "<|channel>thought\n";
const THINK_TAG_END: &str = "<channel|>";
// <|tool_response> is deliberately not excluded: the template emits it as the halt signal after a
// tool call, so engines handle it as a stop sequence rather than the grammar blocking it. The
// triggered free text must not exclude <|tool_call> either: it is the trigger, and excluding it
// would remove the dispatch into the tool-call tags.
const EXCLUDES: &[&str] = &["<|channel>", "<channel|>"];
// <|tool_call> is excluded from the thought channel, and from free text when no tools are
// available, so a tool call cannot start where its arguments would be unconstrained.
const REASONING_EXCLUDES: &[&str] = &["<|channel>", "<channel|>", TOOL_CALL_TRIGGER];
// Argument strings and keys exclude every control marker: a <tool_call|> inside a string value
// would otherwise end the call at the engine's parser.
const ARGUMENT_EXCLUDES: &[&str] = &["<|channel>", "<channel|>", TOOL_CALL_TRIGGER, TOOL_CALL_END];

fn gemma_4_tool_tag(tool: &FunctionToolParam, options: super::StructuralTagOptions) -> TagFormat {
    tag(
        format!("{TOOL_CALL_BEGIN_PREFIX}{}", tool.function.name),
        styled_schema_excluding(
            schema(&tool.function),
            JsonSchemaStyle::Gemma,
            text_excludes(options, ARGUMENT_EXCLUDES),
            options,
        ),
        TOOL_CALL_END,
    )
}

/// Build a Gemma 4 tool-call structural tag.
///
/// Reference: <https://ai.google.dev/gemma/docs/core/prompt-formatting-gemma4>
///
/// Gemma 4 uses channel markers for reasoning and tool calls instead of `<think>`/`</think>`:
///
/// ```text
/// <|channel>thought\n...thinking...<channel|>
/// <|tool_call>call:NAME{key:<|"|>string value<|"|>,key2:123}<tool_call|>
/// ```
///
/// Arguments use [`JsonSchemaStyle::Gemma`]: keys are unquoted and strings are delimited by the
/// `<|"|>` token. With `exclude_special_tokens`, argument strings and property names exclude
/// `<|tool_call>`, `<tool_call|>`, `<|channel>` and `<channel|>`. The generation prompt does not
/// open the thought channel, so enabled reasoning generates the complete channel block.
///
/// Supports the Gemma 4 family (gemma-4-12b-it, gemma-4-26b-a4b-it, gemma-4-31b-it,
/// gemma-4-e2b-it).
pub(super) fn build_gemma_4(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
) -> StructuralTag {
    let tags = || {
        tools
            .iter()
            .map(|tool| gemma_4_tool_tag(tool, options))
            .collect::<Vec<_>>()
    };
    let suffix = match choice {
        BuilderToolChoice::Auto if tools.is_empty() => {
            Format::any_text_excluding(text_excludes(options, REASONING_EXCLUDES))
        }
        BuilderToolChoice::Auto => triggered_with_excludes(
            &[TOOL_CALL_TRIGGER],
            tags(),
            text_excludes(options, EXCLUDES),
            options,
        ),
        BuilderToolChoice::Forced => Format::Tag(gemma_4_tool_tag(&tools[0], options)),
        BuilderToolChoice::Required => required_triggered_with_excludes(
            &[TOOL_CALL_TRIGGER],
            tags(),
            text_excludes(options, EXCLUDES),
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
