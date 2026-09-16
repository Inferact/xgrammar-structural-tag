//! Model-specific structural tag builders.

mod cohere;
mod deepseek_dsml;
mod deepseek_r1;
mod deepseek_v31;
mod exaone;
mod glm_47;
mod harmony;
mod hermes;
mod hy_v3;
mod kimi;
mod kimi_k3;
mod llama;
mod minimax;
mod minimax_m3;
mod qwen_3;
mod qwen_35;

use crate::error::Result;
use crate::format::{
    Format, JsonSchemaFormat, JsonSchemaStyle, StructuralTag, TagFormat, TriggeredTagsFormat,
};
use crate::tool::{
    BuilderToolChoice, BuiltinToolParam, FunctionToolParam, ToolChoice, ToolParam,
    function_parameters, normalize_tool_choice,
};

pub use cohere::CohereBuilder;
pub use deepseek_dsml::{DeepSeekV4Builder, DeepSeekV32Builder, DeepSeekV41Builder};
pub use deepseek_r1::DeepSeekR1Builder;
pub use deepseek_v31::DeepSeekV31Builder;
pub use exaone::ExaoneBuilder;
pub use glm_47::Glm47Builder;
pub use harmony::HarmonyBuilder;
pub use hermes::HermesBuilder;
pub use hy_v3::HyV3Builder;
pub use kimi::KimiBuilder;
pub use kimi_k3::KimiK3Builder;
pub use llama::LlamaBuilder;
pub use minimax::MinimaxBuilder;
pub use minimax_m3::MinimaxM3Builder;
pub use qwen_3::Qwen3Builder;
pub use qwen_35::Qwen35Builder;

/// How a model's reasoning section appears in the generated output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningMode {
    /// Continue the reasoning block opened by the generation prompt.
    #[default]
    Enabled,
    /// Use the model's non-reasoning output format.
    Disabled,
    /// Allow a complete reasoning block or a direct response/tool call.
    Auto,
}

impl From<bool> for ReasoningMode {
    fn from(reasoning: bool) -> Self {
        if reasoning {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }
}

/// Options shared by model-specific structural-tag builders.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralTagOptions {
    /// How the request enables model reasoning sections.
    pub reasoning: ReasoningMode,
    /// Whether a response may contain multiple tool calls.
    ///
    /// When false, generation ends after the first tool call closes. Free text
    /// can precede the call; the model-specific envelope still closes normally.
    pub parallel_tool_calls: bool,
    /// Whether JSON object properties may appear in any order.
    pub any_order: bool,
    /// Whether free-text regions exclude model special tokens.
    pub exclude_special_tokens: bool,
    /// Maximum consecutive whitespace characters in JSON schema regions.
    pub max_whitespace_cnt: Option<i32>,
}

impl StructuralTagOptions {
    /// Configure reasoning mode. Boolean inputs select enabled or disabled mode.
    pub fn with_reasoning(mut self, reasoning: impl Into<ReasoningMode>) -> Self {
        self.reasoning = reasoning.into();
        self
    }

    /// Configure whether a response may contain multiple tool calls.
    pub const fn with_parallel_tool_calls(mut self, parallel_tool_calls: bool) -> Self {
        self.parallel_tool_calls = parallel_tool_calls;
        self
    }

    /// Configure whether JSON object properties may appear in any order.
    pub const fn with_any_order(mut self, any_order: bool) -> Self {
        self.any_order = any_order;
        self
    }

    /// Configure whether free-text regions exclude model special tokens.
    pub const fn with_exclude_special_tokens(mut self, exclude_special_tokens: bool) -> Self {
        self.exclude_special_tokens = exclude_special_tokens;
        self
    }

    /// Limit consecutive whitespace characters in JSON schema regions.
    pub const fn with_max_whitespace_cnt(mut self, max_whitespace_cnt: Option<i32>) -> Self {
        self.max_whitespace_cnt = max_whitespace_cnt;
        self
    }
}

impl Default for StructuralTagOptions {
    fn default() -> Self {
        Self {
            reasoning: ReasoningMode::Enabled,
            parallel_tool_calls: true,
            any_order: false,
            exclude_special_tokens: true,
            max_whitespace_cnt: None,
        }
    }
}

/// Normalized inputs passed to a structural-tag builder.
///
/// Public tool-choice forms are resolved before this context is created: named
/// and builtin choices leave exactly one matching tool and set
/// [`BuilderToolChoice::Forced`], allowed-tools choices filter the tool lists,
/// and `tool_choice=none` clears both lists.
#[derive(Debug, Clone, Copy)]
pub struct StructuralTagContext<'a> {
    /// Function tools remaining after tool-choice normalization.
    pub function_tools: &'a [FunctionToolParam],
    /// Builtin tools remaining after tool-choice normalization.
    pub builtin_tools: &'a [BuiltinToolParam],
    /// Builder-facing tool-choice mode.
    pub tool_choice: BuilderToolChoice,
    /// Shared request options.
    pub options: StructuralTagOptions,
}

/// A model-specific structural-tag template builder.
///
/// Implement this trait to provide structural-tag support outside the built-in
/// [`crate::Model`] catalog.
#[auto_impl::auto_impl(&, Box)]
pub trait StructuralTagBuilder {
    /// Build a structural tag from normalized tools and request flags.
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag>;
}

/// Build a structural tag for a model's reasoning and tool-call output format.
///
/// Use this when a serving engine needs a structural tag that matches a
/// model's tool-call syntax. The API resembles an OpenAI Chat Completions
/// request: pass the model-specific builder, the available `tools`, and a
/// `tool_choice` policy.
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
/// [`StructuralTagOptions::reasoning`] selects enabled, disabled, or adaptive
/// reasoning. Models with a leading reasoning block accept a complete optional
/// block in [`ReasoningMode::Auto`]. Models without reasoning ignore the option;
/// MiniMax M2 keeps its fixed empty-thinking prefix in disabled mode.
///
/// A tool whose `parameters` are omitted, or a function tool with
/// `strict = false`, produces unconstrained-JSON arguments. MiniMax M3's
/// fixed-name XML converter rejects unconstrained schemas at grammar compilation.
///
/// # Errors
///
/// Returns an [`Error`](crate::Error) when the tools or tool choice are
/// inconsistent — for example a named tool is missing, a builtin choice does
/// not match exactly one tool, `required` leaves no tools, or a forced choice
/// does not resolve to exactly one tool.
#[doc(alias = "get_model_structural_tag")]
pub fn build_structural_tag(
    builder: impl StructuralTagBuilder,
    tools: &[ToolParam],
    tool_choice: ToolChoice,
    options: StructuralTagOptions,
) -> Result<StructuralTag> {
    let normalized = normalize_tool_choice(tools, tool_choice)?;
    builder.build(StructuralTagContext {
        function_tools: &normalized.function_tools,
        builtin_tools: &normalized.builtin_tools,
        tool_choice: normalized.choice,
        options,
    })
}

/// Build a structural tag only when tool constraints should be sent downstream.
///
/// This mirrors serving request lowering: empty tools and `tool_choice=none`
/// produce `Ok(None)` so the request can continue without structured outputs.
///
/// Use [`build_structural_tag`] when the caller already knows a structural tag
/// should be built.
#[doc(alias = "maybe_get_model_structural_tag")]
pub fn build_optional_structural_tag(
    builder: impl StructuralTagBuilder,
    tools: &[ToolParam],
    tool_choice: ToolChoice,
    options: StructuralTagOptions,
) -> Result<Option<StructuralTag>> {
    if tools.is_empty()
        || matches!(
            tool_choice,
            ToolChoice::Value(crate::tool::ToolChoiceValue::None)
        )
    {
        return Ok(None);
    }
    build_structural_tag(builder, tools, tool_choice, options).map(Some)
}

pub(super) fn schema(function: &crate::tool::FunctionDefinition) -> serde_json::Value {
    function_parameters(function)
}

pub(super) fn json_schema(value: serde_json::Value, options: StructuralTagOptions) -> Format {
    Format::JsonSchema(
        JsonSchemaFormat::new(value)
            .with_any_order(options.any_order)
            .with_max_whitespace_cnt(options.max_whitespace_cnt),
    )
}

pub(super) fn styled_schema(
    value: serde_json::Value,
    style: JsonSchemaStyle,
    options: StructuralTagOptions,
) -> Format {
    Format::JsonSchema(
        JsonSchemaFormat::new(value)
            .with_style(style)
            .with_any_order(options.any_order)
            .with_max_whitespace_cnt(options.max_whitespace_cnt),
    )
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
    options: StructuralTagOptions,
) -> Format {
    Format::TriggeredTags(
        TriggeredTagsFormat::new(triggers, tags)
            .with_excludes(excludes)
            .with_stop_after_first(!options.parallel_tool_calls),
    )
}

pub(super) fn required_triggered_with_excludes(
    triggers: &[&str],
    tags: Vec<TagFormat>,
    excludes: &[&str],
    options: StructuralTagOptions,
) -> Format {
    Format::TriggeredTags(
        TriggeredTagsFormat::new(triggers, tags)
            .with_excludes(excludes)
            .with_stop_after_first(!options.parallel_tool_calls)
            .require_at_least_one(),
    )
}

pub(super) fn text_excludes<'a>(
    options: StructuralTagOptions,
    excludes: &'a [&'a str],
) -> &'a [&'a str] {
    if options.exclude_special_tokens {
        excludes
    } else {
        &[]
    }
}

pub(super) fn tools_with_separator(
    tags: Vec<TagFormat>,
    separator: &str,
    at_least_one: bool,
    options: StructuralTagOptions,
) -> Format {
    Format::tags_with_separator(tags, separator, at_least_one, !options.parallel_tool_calls)
}

/// Build a conventional reasoning prefix from the model's prompt convention.
pub(super) fn reasoning_prefix(
    options: StructuralTagOptions,
    think_tag_begin: &str,
    think_tag_end: &str,
    excludes: &[&str],
    reasoning_suffix: &str,
) -> Option<Format> {
    if options.reasoning == ReasoningMode::Disabled {
        return None;
    }
    let begin = if options.reasoning == ReasoningMode::Enabled {
        ""
    } else {
        think_tag_begin
    };
    let mut prefix = Format::tag(
        begin,
        Format::any_text_excluding(text_excludes(options, excludes)),
        think_tag_end,
    );
    if !reasoning_suffix.is_empty() {
        prefix = Format::sequence(vec![prefix, Format::const_string(reasoning_suffix)]);
    }
    if options.reasoning == ReasoningMode::Auto {
        prefix = Format::optional(prefix);
    }
    Some(prefix)
}

pub(super) fn assemble(prefix: Option<Format>, suffix: Format) -> StructuralTag {
    match prefix {
        Some(prefix) => structural(Format::sequence(vec![prefix, suffix])),
        None => structural(suffix),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use strum::VariantArray;

    use super::*;
    use crate::{
        AllowedToolRef, AllowedToolsMode, FunctionDefinition, FunctionToolParam, Model, ToolChoice,
        ToolParam,
        tool::{BuilderToolChoice, normalize_tool_choice},
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
        let options = StructuralTagOptions::default().with_reasoning(false);
        for model in Model::VARIANTS {
            let required =
                build_structural_tag(*model, &tools, ToolChoice::required(), options).unwrap();
            let forced =
                build_structural_tag(*model, &tools, ToolChoice::function("search"), options)
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
    fn minimax_m3_rejects_builtin_tools() {
        let error = build_structural_tag(
            Model::MinimaxM3,
            &[ToolParam::Builtin(BuiltinToolParam::new(
                "web_search_preview",
            ))],
            ToolChoice::auto(),
            StructuralTagOptions::default(),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            crate::Error::UnsupportedBuiltinTools {
                model: "minimax_m3"
            }
        ));
    }

    #[test]
    fn optional_skips_empty_tools_and_none_choice() {
        assert!(
            build_optional_structural_tag(
                Model::Llama,
                &[],
                ToolChoice::auto(),
                StructuralTagOptions::default(),
            )
            .unwrap()
            .is_none()
        );
        assert!(
            build_optional_structural_tag(
                Model::Llama,
                &[tool("search")],
                ToolChoice::none(),
                StructuralTagOptions::default(),
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
        assert_eq!(normalized.choice, BuilderToolChoice::Required);
        assert_eq!(normalized.function_tools.len(), 1);
        assert_eq!(normalized.function_tools[0].function.name, "alt");
    }

    #[test]
    fn qwen_35_required_uses_xml_style() {
        let tag = build_structural_tag(
            Model::Qwen35,
            &[tool("run_sql")],
            ToolChoice::required(),
            StructuralTagOptions::default().with_reasoning(false),
        )
        .unwrap();
        let value: Value = serde_json::to_value(tag).unwrap();
        assert_eq!(value["format"]["type"], "triggered_tags");
        assert_eq!(value["format"]["at_least_one"], true);
        assert_eq!(value["format"]["tags"][0]["content"]["style"], "qwen_xml");
    }

    #[test]
    fn request_options_flow_to_schema_and_text_regions() {
        let options = StructuralTagOptions::default()
            .with_reasoning(false)
            .with_any_order(true)
            .with_exclude_special_tokens(false)
            .with_max_whitespace_cnt(Some(2));
        let tag =
            build_structural_tag(Model::Llama, &[tool("search")], ToolChoice::auto(), options)
                .unwrap();
        let value: Value = serde_json::to_value(tag).unwrap();
        assert_eq!(value["format"]["excludes"], json!([]));
        assert_eq!(value["format"]["tags"][0]["content"]["any_order"], true);
        assert_eq!(
            value["format"]["tags"][0]["content"]["max_whitespace_cnt"],
            2
        );
    }

    #[test]
    fn glm_text_regions_exclude_control_tokens() {
        let tag = build_structural_tag(
            Model::Glm47,
            &[tool("search")],
            ToolChoice::auto(),
            StructuralTagOptions::default(),
        )
        .unwrap();
        let value: Value = serde_json::to_value(tag).unwrap();
        let reasoning_excludes = value["format"]["elements"][0]["content"]["excludes"]
            .as_array()
            .unwrap();
        let text_excludes = value["format"]["elements"][1]["excludes"]
            .as_array()
            .unwrap();
        assert!(reasoning_excludes.len() > text_excludes.len());
        assert!(text_excludes.contains(&json!("<arg_key>")));
        assert!(!text_excludes.contains(&json!("<tool_call>")));
    }

    #[test]
    fn builtin_model_builder_matches_model_adapter() {
        let tools = vec![tool("run_sql")];
        let options = StructuralTagOptions::default().with_reasoning(false);
        let from_model =
            build_structural_tag(Model::Qwen35, &tools, ToolChoice::required(), options).unwrap();
        let from_builder = build_structural_tag(
            Model::Qwen35.builder(),
            &tools,
            ToolChoice::required(),
            options,
        )
        .unwrap();
        assert_eq!(from_builder, from_model);
    }

    struct CustomXmlBuilder;

    impl StructuralTagBuilder for CustomXmlBuilder {
        fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
            let tags = ctx
                .function_tools
                .iter()
                .map(|tool| {
                    tag(
                        format!("<call name=\"{}\">", tool.function.name),
                        json_schema(schema(&tool.function), ctx.options),
                        "</call>",
                    )
                })
                .collect();
            Ok(structural(tools_with_separator(
                tags,
                "",
                ctx.tool_choice.requires_tool_call(),
                ctx.options,
            )))
        }
    }

    #[test]
    fn custom_builder_receives_normalized_context() {
        let tag = build_structural_tag(
            CustomXmlBuilder,
            &[tool("search"), tool("other")],
            ToolChoice::function("search"),
            StructuralTagOptions::default().with_reasoning(false),
        )
        .unwrap();
        let value: Value = serde_json::to_value(tag).unwrap();
        assert_eq!(value["format"]["at_least_one"], true);
        assert_eq!(
            value["format"]["tags"][0]["begin"],
            "<call name=\"search\">"
        );
        assert_eq!(value["format"]["tags"].as_array().unwrap().len(), 1);
    }

    struct FailingBuilder;

    impl StructuralTagBuilder for FailingBuilder {
        fn build(&self, _ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
            Err(crate::Error::Custom("unsupported template".into()))
        }
    }

    #[test]
    fn custom_builder_error_is_propagated() {
        let error = build_structural_tag(
            FailingBuilder,
            &[tool("search")],
            ToolChoice::required(),
            StructuralTagOptions::default().with_reasoning(false),
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "unsupported template");
    }

    #[test]
    fn hy_v3_required_uses_glm_xml_arguments() {
        let tag = build_structural_tag(
            Model::HyV3,
            &[tool("search")],
            ToolChoice::required(),
            StructuralTagOptions::default().with_reasoning(false),
        )
        .unwrap();
        let value: Value = serde_json::to_value(tag).unwrap();
        assert!(value.to_string().contains("<tool_calls>"));
        assert!(value.to_string().contains("<tool_sep>"));
        assert!(value.to_string().contains("glm_xml"));
    }
}
