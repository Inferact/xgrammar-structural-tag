use crate::format::{Format, StructuralTag, TagFormat};
use crate::tool::{
    BuiltinToolParam, FunctionToolParam, SimplifiedToolChoice, builtin_parameters,
    builtin_tool_name,
};

use super::{json_schema, schema, structural, tag, tools_with_separator};

const CALL_END: &str = "<|call|>";

fn function_tool_tags(name: &str, parameters: serde_json::Value) -> Vec<TagFormat> {
    let content = json_schema(parameters);
    vec![
        tag(
            format!("<|channel|>commentary to=functions.{name}<|constrain|>json<|message|>"),
            content.clone(),
            CALL_END,
        ),
        tag(
            format!(" to=functions.{name}<|channel|>commentary <|constrain|>json<|message|>"),
            content.clone(),
            CALL_END,
        ),
        tag(
            format!(" to=functions.{name}<|channel|>commentary json<|message|>"),
            content,
            CALL_END,
        ),
    ]
}

fn builtin_tool_tags(name: &str, parameters: serde_json::Value) -> Vec<TagFormat> {
    let content = json_schema(parameters);
    vec![
        tag(
            format!("<|channel|>commentary to={name} code<|message|>"),
            content.clone(),
            CALL_END,
        ),
        tag(
            format!(" to={name}<|channel|>commentary code<|message|>"),
            content,
            CALL_END,
        ),
    ]
}

pub(super) fn build_harmony(
    tools: &[FunctionToolParam],
    builtin_tools: &[BuiltinToolParam],
    choice: SimplifiedToolChoice,
    reasoning: bool,
) -> StructuralTag {
    const FINAL_BEGIN: &str = "<|channel|>final<|message|>";
    const ANALYSIS_BEGIN: &str = "<|channel|>analysis<|message|>";
    const TAG_SEPARATOR: &str = "<|start|>assistant";

    let mut tags = Vec::new();
    match choice {
        SimplifiedToolChoice::Auto => {
            for tool in tools {
                tags.extend(function_tool_tags(
                    &tool.function.name,
                    schema(&tool.function),
                ));
            }
            for tool in builtin_tools {
                tags.extend(builtin_tool_tags(
                    builtin_tool_name(tool),
                    builtin_parameters(tool),
                ));
            }
            tags.push(tag(
                FINAL_BEGIN,
                Format::any_text(),
                vec!["<|end|>", "<|return|>"],
            ));
        }
        SimplifiedToolChoice::Forced => {
            if let Some(tool) = builtin_tools.first() {
                tags.extend(builtin_tool_tags(
                    builtin_tool_name(tool),
                    builtin_parameters(tool),
                ));
            } else {
                let function = &tools[0].function;
                tags.extend(function_tool_tags(&function.name, schema(function)));
            }
        }
        SimplifiedToolChoice::Required => {
            for tool in builtin_tools {
                tags.extend(builtin_tool_tags(
                    builtin_tool_name(tool),
                    builtin_parameters(tool),
                ));
            }
            for tool in tools {
                tags.extend(function_tool_tags(
                    &tool.function.name,
                    schema(&tool.function),
                ));
            }
        }
    }
    if reasoning {
        tags.push(tag(
            ANALYSIS_BEGIN,
            Format::any_text(),
            vec!["<|end|>", "<|return|>"],
        ));
    }
    structural(tools_with_separator(tags, TAG_SEPARATOR, false))
}
