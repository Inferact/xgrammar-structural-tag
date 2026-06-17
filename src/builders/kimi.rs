use crate::format::{Format, StructuralTag, TagFormat};
use crate::tool::{FunctionToolParam, SimplifiedToolChoice};

use super::{json_schema, schema, structural, tag, tools_with_separator, triggered_with_excludes};

fn kimi_tool_tag(tool: &FunctionToolParam) -> TagFormat {
    const TOOL_CALL_BEGIN_PREFIX: &str = "<|tool_call_begin|>functions.";
    const TOOL_CALL_SUFFIX: &str = ":";
    const TOOL_CALL_ARGUMENT_BEGIN: &str = "<|tool_call_argument_begin|>";
    const TOOL_CALL_END: &str = "<|tool_call_end|>";
    tag(
        format!(
            "{TOOL_CALL_BEGIN_PREFIX}{}{TOOL_CALL_SUFFIX}",
            tool.function.name
        ),
        Format::sequence(vec![
            Format::regex(r"\d+"),
            Format::const_string(TOOL_CALL_ARGUMENT_BEGIN),
            json_schema(schema(&tool.function)),
        ]),
        TOOL_CALL_END,
    )
}

/// Build a Kimi-K2-style structural tag.
///
/// Reference: <https://huggingface.co/moonshotai/Kimi-K2-Instruct/blob/main/docs/tool_call_guidance.md>
///
/// Supports Kimi-K2 and Kimi-K2.5. When `reasoning` is `false`, the reasoning
/// prefix is dropped and only the tool/text part is constrained.
pub(super) fn build_kimi(
    tools: &[FunctionToolParam],
    choice: SimplifiedToolChoice,
    reasoning: bool,
) -> StructuralTag {
    const TOOL_CALL_BEGIN: &str = "<|tool_call_begin|>";
    const TOOL_CALLS_SECTION_BEGIN: &str = "<|tool_calls_section_begin|>";
    const TOOL_CALLS_SECTION_END: &str = "<|tool_calls_section_end|>";
    const THINK_TAG_END: &str = "</think>";
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];

    let suffix = match choice {
        SimplifiedToolChoice::Auto => {
            let tags = tools.iter().map(kimi_tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(THINK_EXCLUDES)
            } else {
                let inner = tools_with_separator(tags, "", true);
                let tool_calls = tag(TOOL_CALLS_SECTION_BEGIN, inner, TOOL_CALLS_SECTION_END);
                triggered_with_excludes(
                    &[TOOL_CALLS_SECTION_BEGIN],
                    vec![tool_calls],
                    &["<think>", "</think>", TOOL_CALL_BEGIN],
                )
            }
        }
        SimplifiedToolChoice::Forced => Format::sequence(vec![
            Format::const_string(TOOL_CALLS_SECTION_BEGIN),
            Format::Tag(kimi_tool_tag(&tools[0])),
            Format::const_string(TOOL_CALLS_SECTION_END),
        ]),
        SimplifiedToolChoice::Required => {
            let tags = tools.iter().map(kimi_tool_tag).collect::<Vec<_>>();
            Format::sequence(vec![
                Format::const_string(TOOL_CALLS_SECTION_BEGIN),
                tools_with_separator(tags, "", true),
                Format::const_string(TOOL_CALLS_SECTION_END),
            ])
        }
    };

    if !reasoning {
        return structural(suffix);
    }
    structural(Format::sequence(vec![
        Format::tag("", Format::any_text(), THINK_TAG_END),
        suffix,
    ]))
}
