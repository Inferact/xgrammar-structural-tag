//! Model-specific structural tag builders.
//!
//! Most builders in this module tree are direct Rust ports of xgrammar's
//! `python/xgrammar/builtin_structural_tag.py` at commit
//! `4d145cc13d878c751ebeed36af1c013074be76bc`.
//! The `hermes` builder is adapted from vLLM's Apache-2.0
//! `vllm/tool_parsers/structural_tag_registry.py`, and `hy_v3` follows the
//! Rust frontend HY3 tool parser syntax.

mod deepseek;
mod extensions;
mod harmony;
mod kimi;
mod llama;
mod qwen;
mod xml;

use crate::error::{Error, Result};
use crate::format::{Format, JsonSchemaStyle, StructuralTag, TagFormat, TriggeredTagsFormat};
use crate::model::Model;
use crate::tool::{ToolChoice, ToolParam, function_parameters, normalize_tool_choice};

use deepseek::{build_deepseek_r1, build_deepseek_v4, build_deepseek_v31, build_deepseek_v32};
use extensions::{build_hermes, build_hy_v3};
use harmony::build_harmony;
use kimi::build_kimi;
use llama::build_llama;
use qwen::{build_qwen_3, build_qwen_35};
use xml::{build_glm_47, build_minimax};

/// xgrammar builtin model keys covered by this crate.
pub const XGRAMMAR_BUILTIN_MODELS: &[&str] = &[
    "llama",
    "kimi",
    "deepseek_r1",
    "deepseek_v3_1",
    "qwen_3_5",
    "qwen_3_coder",
    "qwen_3",
    "harmony",
    "deepseek_v3_2",
    "minimax",
    "glm_4_7",
    "deepseek_v4",
];

/// vLLM/Rust frontend extension model keys covered by this crate.
pub const EXTENSION_MODELS: &[&str] = &["hermes", "hy_v3"];

/// Return xgrammar builtin model keys.
pub fn xgrammar_builtin_models() -> &'static [&'static str] {
    XGRAMMAR_BUILTIN_MODELS
}

/// Return extension model keys.
pub fn extension_models() -> &'static [&'static str] {
    EXTENSION_MODELS
}

/// Return every supported model key.
pub fn supported_models() -> Vec<&'static str> {
    XGRAMMAR_BUILTIN_MODELS
        .iter()
        .chain(EXTENSION_MODELS.iter())
        .copied()
        .collect()
}

/// Build a structural tag for a supported model.
pub fn get_model_structural_tag(
    model: &str,
    tools: &[ToolParam],
    tool_choice: ToolChoice,
    reasoning: bool,
) -> Result<StructuralTag> {
    let normalized = normalize_tool_choice(tools, tool_choice)?;
    let model = Model::parse(model).ok_or_else(|| Error::UnknownModel {
        model: model.to_string(),
        supported: supported_models(),
    })?;

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
/// This mirrors vLLM frontend behavior: empty tools and `tool_choice=none`
/// produce `Ok(None)` so the request can continue without structured outputs.
pub fn maybe_get_model_structural_tag(
    model: &str,
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
    get_model_structural_tag(model, tools, tool_choice, reasoning).map(Some)
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

    use super::*;
    use crate::{
        AllowedToolRef, FunctionDefinition, FunctionToolParam, ToolChoice, ToolParam,
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
    fn registry_lists_all_expected_models() {
        let models = supported_models();
        for key in [
            "llama",
            "kimi",
            "deepseek_r1",
            "deepseek_v3_1",
            "qwen_3_5",
            "qwen_3_coder",
            "qwen_3",
            "harmony",
            "deepseek_v3_2",
            "minimax",
            "glm_4_7",
            "deepseek_v4",
            "hermes",
            "hy_v3",
        ] {
            assert!(models.contains(&key));
        }
        assert!(!models.contains(&"gemma_4"));
    }

    #[test]
    fn every_model_builds_required_and_forced() {
        let tools = vec![tool("search"), tool("alt")];
        for model in supported_models() {
            let required =
                get_model_structural_tag(model, &tools, ToolChoice::required(), false).unwrap();
            let forced =
                get_model_structural_tag(model, &tools, ToolChoice::function("search"), false)
                    .unwrap();
            assert_eq!(required.kind(), "structural_tag");
            assert_eq!(forced.kind(), "structural_tag");
        }
    }

    #[test]
    fn maybe_skips_empty_tools_and_none_choice() {
        assert!(
            maybe_get_model_structural_tag("llama", &[], ToolChoice::auto(), false)
                .unwrap()
                .is_none()
        );
        assert!(
            maybe_get_model_structural_tag("llama", &[tool("search")], ToolChoice::none(), false)
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
            ToolChoice::allowed_tools("required", vec![AllowedToolRef::function("alt")]),
        )
        .unwrap();
        assert_eq!(normalized.choice, SimplifiedToolChoice::Required);
        assert_eq!(normalized.function_tools.len(), 1);
        assert_eq!(normalized.function_tools[0].function.name, "alt");
    }

    #[test]
    fn qwen_35_required_uses_xml_style() {
        let tag = get_model_structural_tag(
            "qwen_3_5",
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
        let tag =
            get_model_structural_tag("hy_v3", &[tool("search")], ToolChoice::required(), false)
                .unwrap();
        let value: Value = serde_json::to_value(tag).unwrap();
        assert!(value.to_string().contains("<tool_calls>"));
        assert!(value.to_string().contains("<tool_sep>"));
        assert!(value.to_string().contains("glm_xml"));
    }
}
