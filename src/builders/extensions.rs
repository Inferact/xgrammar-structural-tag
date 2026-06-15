use crate::format::{Format, JsonSchemaStyle, StructuralTag, TagFormat};
use crate::tool::{FunctionToolParam, SimplifiedToolChoice};

use super::{
    json_schema, schema, structural, styled_schema, tag, tools_with_separator,
    triggered_with_excludes,
};

pub(super) fn build_hermes(
    tools: &[FunctionToolParam],
    choice: SimplifiedToolChoice,
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
        SimplifiedToolChoice::Auto => {
            let tags = hermes_tool_tags(tools);
            if tags.is_empty() {
                Format::any_text()
            } else {
                Format::triggered_tags(&["<tool_call>"], tags)
            }
        }
        SimplifiedToolChoice::Forced => {
            Format::tags_with_separator(hermes_tool_tags(tools), "", true, true)
        }
        SimplifiedToolChoice::Required => tools_with_separator(hermes_tool_tags(tools), "", true),
    };
    structural(suffix)
}

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
