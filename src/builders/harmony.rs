use crate::Result;
use crate::format::{Format, StructuralTag, TagFormat};
use crate::tool::{
    BuilderToolChoice, BuiltinToolParam, FunctionToolParam, builtin_parameters, builtin_tool_name,
};

use super::{
    StructuralTagBuilder, StructuralTagContext, StructuralTagOptions, json_schema, schema,
    structural, tag, tools_with_separator,
};

const CALL_END: &str = "<|call|>";

/// Harmony / GPT-OSS structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct HarmonyBuilder;

impl StructuralTagBuilder for HarmonyBuilder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_harmony(
            ctx.function_tools,
            ctx.builtin_tools,
            ctx.tool_choice,
            ctx.options,
        ))
    }
}

fn function_tool_tags(
    name: &str,
    parameters: serde_json::Value,
    options: StructuralTagOptions,
) -> Vec<TagFormat> {
    let content = json_schema(parameters, options);
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

fn builtin_tool_tags(
    name: &str,
    parameters: serde_json::Value,
    options: StructuralTagOptions,
) -> Vec<TagFormat> {
    let content = json_schema(parameters, options);
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

/// Build a Harmony (gpt-oss) structural tag.
///
/// Reference: <https://developers.openai.com/cookbook/articles/openai-harmony>
///
/// Used by gpt-oss. Unlike the other builders this also accepts builtin tools,
/// and `reasoning` enables the analysis channel.
pub(super) fn build_harmony(
    tools: &[FunctionToolParam],
    builtin_tools: &[BuiltinToolParam],
    choice: BuilderToolChoice,
    options: StructuralTagOptions,
) -> StructuralTag {
    const FINAL_BEGIN: &str = "<|channel|>final<|message|>";
    const ANALYSIS_BEGIN: &str = "<|channel|>analysis<|message|>";
    const TAG_SEPARATOR: &str = "<|start|>assistant";

    let mut tags = Vec::new();
    match choice {
        BuilderToolChoice::Auto => {
            for tool in tools {
                tags.extend(function_tool_tags(
                    &tool.function.name,
                    schema(&tool.function),
                    options,
                ));
            }
            for tool in builtin_tools {
                tags.extend(builtin_tool_tags(
                    builtin_tool_name(tool),
                    builtin_parameters(tool),
                    options,
                ));
            }
            tags.push(tag(
                FINAL_BEGIN,
                Format::any_text(),
                vec!["<|end|>", "<|return|>"],
            ));
        }
        BuilderToolChoice::Forced => {
            if let Some(tool) = builtin_tools.first() {
                tags.extend(builtin_tool_tags(
                    builtin_tool_name(tool),
                    builtin_parameters(tool),
                    options,
                ));
            } else {
                let function = &tools[0].function;
                tags.extend(function_tool_tags(
                    &function.name,
                    schema(function),
                    options,
                ));
            }
        }
        BuilderToolChoice::Required => {
            for tool in builtin_tools {
                tags.extend(builtin_tool_tags(
                    builtin_tool_name(tool),
                    builtin_parameters(tool),
                    options,
                ));
            }
            for tool in tools {
                tags.extend(function_tool_tags(
                    &tool.function.name,
                    schema(&tool.function),
                    options,
                ));
            }
        }
    }
    if options.reasoning {
        tags.push(tag(
            ANALYSIS_BEGIN,
            Format::any_text(),
            vec!["<|end|>", "<|return|>"],
        ));
    }
    structural(tools_with_separator(tags, TAG_SEPARATOR, false))
}
