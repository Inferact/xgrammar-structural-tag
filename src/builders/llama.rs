use crate::format::{Format, StructuralTag};
use crate::tool::{FunctionToolParam, SimplifiedToolChoice};

use super::{json_schema, schema, structural, tag, tools_with_separator, triggered_with_excludes};

/// Build a Llama-style structural tag.
///
/// Reference: <https://www.llama.com/docs/model-cards-and-prompt-formats/llama3_1/>
///
/// Supports Meta-Llama-3, Llama-3.1, and Llama-3.2. This format has no
/// reasoning part, so `reasoning` is ignored.
pub(super) fn build_llama(
    tools: &[FunctionToolParam],
    choice: SimplifiedToolChoice,
    _reasoning: bool,
) -> StructuralTag {
    const TOOL_NAME_PREFIX: &str = "{\"name\": \"";
    const PARAMETERS_FIELD_PREFIX: &str = "\", \"parameters\": ";
    const TOOL_OBJECT_BEGIN_PREFIX: &str = "{\"name\": \"";
    const TOOL_OBJECT_PARAMETERS_PREFIX: &str = "\", \"parameters\": ";
    const TOOLS_TRIGGER: &str = "{\"name\": ";
    const THINK_EXCLUDES: &[&str] = &["<think>", "</think>"];

    let suffix = match choice {
        SimplifiedToolChoice::Auto => {
            let tags = tools
                .iter()
                .map(|tool| {
                    tag(
                        format!(
                            "{TOOL_OBJECT_BEGIN_PREFIX}{}{TOOL_OBJECT_PARAMETERS_PREFIX}",
                            tool.function.name
                        ),
                        json_schema(schema(&tool.function)),
                        "}",
                    )
                })
                .collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text_excluding(THINK_EXCLUDES)
            } else {
                triggered_with_excludes(&[TOOLS_TRIGGER], tags, THINK_EXCLUDES)
            }
        }
        SimplifiedToolChoice::Forced => {
            let function = &tools[0].function;
            Format::tag(
                format!(
                    "{TOOL_NAME_PREFIX}{}{PARAMETERS_FIELD_PREFIX}",
                    function.name
                ),
                json_schema(schema(function)),
                "}",
            )
        }
        SimplifiedToolChoice::Required => {
            let tags = tools
                .iter()
                .map(|tool| {
                    tag(
                        format!(
                            "{TOOL_OBJECT_BEGIN_PREFIX}{}{TOOL_OBJECT_PARAMETERS_PREFIX}",
                            tool.function.name
                        ),
                        json_schema(schema(&tool.function)),
                        "}",
                    )
                })
                .collect();
            tools_with_separator(tags, "", true)
        }
    };
    structural(suffix)
}
