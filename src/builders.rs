//! Model-specific structural tag builders.

mod deepseek_dsml;
mod deepseek_r1;
mod deepseek_v31;
mod glm_47;
mod harmony;
mod hermes;
mod hy_v3;
mod kimi;
mod llama;
mod minimax;
mod qwen_3;
mod qwen_35;

use crate::error::Result;
use crate::format::{Format, JsonSchemaStyle, StructuralTag, TagFormat, TriggeredTagsFormat};
use crate::model::Model;
use crate::tool::{ToolChoice, ToolParam, function_parameters, normalize_tool_choice};

use deepseek_dsml::{build_deepseek_v4, build_deepseek_v32};
use deepseek_r1::build_deepseek_r1;
use deepseek_v31::build_deepseek_v31;
use glm_47::build_glm_47;
use harmony::build_harmony;
use hermes::build_hermes;
use hy_v3::build_hy_v3;
use kimi::build_kimi;
use llama::build_llama;
use minimax::build_minimax;
use qwen_3::build_qwen_3;
use qwen_35::build_qwen_35;

/// Build a structural tag for a model's reasoning and tool-call output format.
///
/// Use this when a serving engine needs a structural tag that matches a
/// model's tool-call syntax. The API resembles an OpenAI Chat Completions
/// request: pass the model template, the available `tools`, and a `tool_choice`
/// policy.
///
/// `tool_choice` controls whether the model may or must call tools:
///
/// - [`ToolChoice::auto`] lets the model choose between text and tool calls.
/// - [`ToolChoice::none`] disables all tools.
/// - [`ToolChoice::required`] requires at least one available tool.
/// - [`ToolChoice::function`] forces one named function tool.
/// - [`ToolChoice::builtin`] forces one builtin tool, matched by type.
/// - [`ToolChoice::allowed_tools`] restricts the tools before applying its mode.
///
/// `reasoning` toggles the reasoning part for models that support both modes
/// (e.g. Qwen 3.6, DeepSeek V4). It has no effect on models without a reasoning
/// part, and for reasoning-only models `false` keeps the reasoning section with
/// empty content.
///
/// A tool whose `parameters` are omitted, or a function tool with
/// `strict = false`, produces unconstrained-JSON arguments.
///
/// # Errors
///
/// Returns an [`Error`](crate::Error) when the tools or tool choice are
/// inconsistent — for example a named tool is missing, a builtin choice does
/// not match exactly one tool, `required` leaves no tools, or a forced choice
/// does not resolve to exactly one tool.
#[doc(alias = "get_model_structural_tag")]
pub fn build_structural_tag(
    model: Model,
    tools: &[ToolParam],
    tool_choice: ToolChoice,
    reasoning: bool,
) -> Result<StructuralTag> {
    let normalized = normalize_tool_choice(tools, tool_choice)?;
    let tag = match model {
        Model::Llama => build_llama(&normalized.function_tools, normalized.choice, reasoning),
        Model::Kimi => build_kimi(&normalized.function_tools, normalized.choice, reasoning),
        Model::DeepSeekR1 => {
            build_deepseek_r1(&normalized.function_tools, normalized.choice, reasoning)
        }
        Model::DeepSeekV31 => {
            build_deepseek_v31(&normalized.function_tools, normalized.choice, reasoning)
        }
        Model::Qwen35 | Model::Qwen3Coder => {
            build_qwen_35(&normalized.function_tools, normalized.choice, reasoning)
        }
        Model::Qwen3 => build_qwen_3(&normalized.function_tools, normalized.choice, reasoning),
        Model::Harmony => build_harmony(
            &normalized.function_tools,
            &normalized.builtin_tools,
            normalized.choice,
            reasoning,
        ),
        Model::DeepSeekV32 => {
            build_deepseek_v32(&normalized.function_tools, normalized.choice, reasoning)
        }
        Model::Minimax => build_minimax(&normalized.function_tools, normalized.choice, reasoning),
        Model::Glm47 => build_glm_47(&normalized.function_tools, normalized.choice, reasoning),
        Model::DeepSeekV4 => {
            build_deepseek_v4(&normalized.function_tools, normalized.choice, reasoning)
        }
        Model::Hermes => build_hermes(&normalized.function_tools, normalized.choice),
        Model::HyV3 => build_hy_v3(&normalized.function_tools, normalized.choice),
    };
    Ok(tag)
}

/// Build a structural tag only when tool constraints should be sent downstream.
///
/// This mirrors serving request lowering: empty tools and `tool_choice=none`
/// produce `Ok(None)` so the request can continue without structured outputs.
#[doc(alias = "maybe_get_model_structural_tag")]
pub fn build_optional_structural_tag(
    model: Model,
    tools: &[ToolParam],
    tool_choice: ToolChoice,
    reasoning: bool,
) -> Result<Option<StructuralTag>> {
    if tools.is_empty()
        || matches!(
            tool_choice,
            ToolChoice::Value(crate::tool::ToolChoiceValue::None)
        )
    {
        return Ok(None);
    }
    build_structural_tag(model, tools, tool_choice, reasoning).map(Some)
}

pub(super) fn schema(function: &crate::tool::FunctionDefinition) -> serde_json::Value {
    function_parameters(function)
}

pub(super) fn json_schema(value: serde_json::Value) -> Format {
    Format::json_schema(value)
}

pub(super) fn styled_schema(value: serde_json::Value, style: JsonSchemaStyle) -> Format {
    Format::json_schema_style(value, style)
}

pub(super) fn tag(
    begin: impl Into<crate::format::TagBoundary>,
    content: Format,
    end: impl Into<crate::format::EndBoundary>,
) -> TagFormat {
    TagFormat::new(begin, content, end)
}

pub(super) fn structural(format: Format) -> StructuralTag {
    StructuralTag::new(format)
}

pub(super) fn triggered_with_excludes(
    triggers: &[&str],
    tags: Vec<TagFormat>,
    excludes: &[&str],
) -> Format {
    Format::TriggeredTags(TriggeredTagsFormat {
        triggers: triggers.iter().map(|s| (*s).to_string()).collect(),
        tags,
        at_least_one: false,
        stop_after_first: false,
        excludes: excludes.iter().map(|s| (*s).to_string()).collect(),
    })
}

pub(super) fn tools_with_separator(
    tags: Vec<TagFormat>,
    separator: &str,
    at_least_one: bool,
) -> Format {
    Format::tags_with_separator(tags, separator, at_least_one, false)
}

pub(super) fn with_optional_reasoning(
    suffix: Format,
    reasoning: bool,
    think_tag_end: &str,
) -> StructuralTag {
    if !reasoning {
        return structural(suffix);
    }
    structural(Format::sequence(vec![
        Format::tag("", Format::any_text(), think_tag_end),
        suffix,
    ]))
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use strum::VariantArray;

    use super::*;
    use crate::{
        AllowedToolRef, AllowedToolsMode, FunctionDefinition, FunctionToolParam, ToolChoice,
        ToolParam,
        tool::{SimplifiedToolChoice, normalize_tool_choice},
    };

    fn tool(name: &str) -> ToolParam {
        ToolParam::Function(FunctionToolParam::new(
            FunctionDefinition::new(name).with_parameters(json!({
                "type": "object",
                "properties": { "q": { "type": "string" } },
                "required": ["q"]
            })),
        ))
    }

    #[test]
    fn every_model_builds_required_and_forced() {
        let tools = vec![tool("search"), tool("alt")];
        for model in Model::VARIANTS {
            let required =
                build_structural_tag(*model, &tools, ToolChoice::required(), false).unwrap();
            let forced =
                build_structural_tag(*model, &tools, ToolChoice::function("search"), false)
                    .unwrap();
            assert_eq!(
                serde_json::to_value(required).unwrap()["type"],
                "structural_tag"
            );
            assert_eq!(
                serde_json::to_value(forced).unwrap()["type"],
                "structural_tag"
            );
        }
    }

    #[test]
    fn optional_skips_empty_tools_and_none_choice() {
        assert!(
            build_optional_structural_tag(Model::Llama, &[], ToolChoice::auto(), false)
                .unwrap()
                .is_none()
        );
        assert!(
            build_optional_structural_tag(
                Model::Llama,
                &[tool("search")],
                ToolChoice::none(),
                false
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn normalize_strict_false_and_missing_parameters_to_true_schema() {
        let strict_false = FunctionToolParam::new(
            FunctionDefinition::new("loose")
                .with_parameters(json!({"type": "object"}))
                .with_strict(false),
        );
        let missing = FunctionToolParam::new(FunctionDefinition::new("missing"));
        assert_eq!(schema(&strict_false.function), json!(true));
        assert_eq!(schema(&missing.function), json!(true));
    }

    #[test]
    fn normalize_allowed_tools_filters_functions() {
        let tools = vec![tool("search"), tool("alt")];
        let normalized = normalize_tool_choice(
            &tools,
            ToolChoice::allowed_tools(
                AllowedToolsMode::Required,
                vec![AllowedToolRef::function("alt")],
            ),
        )
        .unwrap();
        assert_eq!(normalized.choice, SimplifiedToolChoice::Required);
        assert_eq!(normalized.function_tools.len(), 1);
        assert_eq!(normalized.function_tools[0].function.name, "alt");
    }

    #[test]
    fn qwen_35_required_uses_xml_style() {
        let tag = build_structural_tag(
            Model::Qwen35,
            &[tool("run_sql")],
            ToolChoice::required(),
            false,
        )
        .unwrap();
        let value: Value = serde_json::to_value(tag).unwrap();
        assert_eq!(value["format"]["type"], "tags_with_separator");
        assert_eq!(value["format"]["tags"][0]["content"]["style"], "qwen_xml");
    }

    #[test]
    fn hy_v3_required_uses_glm_xml_arguments() {
        let tag = build_structural_tag(
            Model::HyV3,
            &[tool("search")],
            ToolChoice::required(),
            false,
        )
        .unwrap();
        let value: Value = serde_json::to_value(tag).unwrap();
        assert!(value.to_string().contains("<tool_calls>"));
        assert!(value.to_string().contains("<tool_sep>"));
        assert!(value.to_string().contains("glm_xml"));
    }
}
