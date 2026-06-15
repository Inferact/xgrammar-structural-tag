use crate::format::{Format, StructuralTag, TagFormat};
use crate::tool::{FunctionToolParam, SimplifiedToolChoice};

use super::{json_schema, schema, structural, tag, tools_with_separator};

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
