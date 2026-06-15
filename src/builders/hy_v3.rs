use crate::format::{Format, JsonSchemaStyle, StructuralTag};
use crate::tool::{FunctionToolParam, SimplifiedToolChoice};

use super::{
    schema, structural, styled_schema, tag, tools_with_separator, triggered_with_excludes,
};

pub(super) fn build_hy_v3(
    tools: &[FunctionToolParam],
    choice: SimplifiedToolChoice,
) -> StructuralTag {
    const TOOL_CALLS_BEGIN: &str = "<tool_calls>\n";
    const TOOL_CALLS_TRIGGER: &str = "<tool_calls>";
    const TOOL_CALLS_END: &str = "</tool_calls>";
    const TOOL_CALL_BEGIN_PREFIX: &str = "<tool_call>";
    const TOOL_SEP: &str = "<tool_sep>";
    const TOOL_CALL_END: &str = "</tool_call>";

    let tool_tag = |tool: &FunctionToolParam| {
        tag(
            format!("{TOOL_CALL_BEGIN_PREFIX}{}{TOOL_SEP}", tool.function.name),
            styled_schema(schema(&tool.function), JsonSchemaStyle::GlmXml),
            TOOL_CALL_END,
        )
    };
    let suffix = match choice {
        SimplifiedToolChoice::Auto => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text()
            } else {
                let outer = tag(
                    TOOL_CALLS_BEGIN,
                    tools_with_separator(tags, "\n", true),
                    TOOL_CALLS_END,
                );
                triggered_with_excludes(&[TOOL_CALLS_TRIGGER], vec![outer], &[])
            }
        }
        SimplifiedToolChoice::Forced => Format::sequence(vec![
            Format::const_string(TOOL_CALLS_BEGIN),
            Format::Tag(tool_tag(&tools[0])),
            Format::const_string(format!("\n{TOOL_CALLS_END}")),
        ]),
        SimplifiedToolChoice::Required => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            Format::sequence(vec![
                Format::const_string(TOOL_CALLS_BEGIN),
                tools_with_separator(tags, "\n", true),
                Format::const_string(format!("\n{TOOL_CALLS_END}")),
            ])
        }
    };
    structural(suffix)
}
