//! OpenAI-style tool and tool-choice DTOs.
//!
//! These types model the OpenAI Chat Completions tool-call shape. The
//! Responses API uses flatter tool and tool-choice shapes; where the two
//! differ, the wire shape is noted on the individual types.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::{Error, Result};

/// Open-ended provider/server builtin tool type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuiltinToolType(String);

impl BuiltinToolType {
    /// Build a builtin tool type.
    pub fn new(tool_type: impl Into<String>) -> Self {
        Self(tool_type.into())
    }

    /// Return the provider-facing builtin tool type string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for BuiltinToolType {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for BuiltinToolType {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for BuiltinToolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A JSON-Schema-based function definition.
///
/// In the Chat Completions shape this is nested under `tools[].function`; in
/// the Responses API the same fields are flattened onto the tool object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Function name emitted by the model.
    ///
    /// Must be `a-z`, `A-Z`, `0-9`, underscores, or dashes, with a maximum
    /// length of 64.
    pub name: String,
    /// Description of what the function does, used by the model to decide when
    /// and how to call it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON schema for the function arguments.
    ///
    /// When omitted, the generated arguments are unconstrained.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
    /// Whether strict schema adherence is requested.
    ///
    /// When `true`, the model follows the exact `parameters` schema. When
    /// `false`, the arguments are treated as unconstrained.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

impl FunctionDefinition {
    /// Build a function definition with no description or parameters.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            parameters: None,
            strict: None,
        }
    }

    /// Set the function description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the function parameter JSON schema.
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Set the strict flag.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }
}

/// One OpenAI Chat Completions function tool: `{"type": "function", "function": {...}}`.
///
/// The Responses API uses a flat shape with the function fields directly on
/// the tool object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionToolParam {
    /// Tool type, always `function`.
    pub r#type: FunctionToolType,
    /// Function definition.
    pub function: FunctionDefinition,
}

impl FunctionToolParam {
    /// Build a function tool.
    pub fn new(function: FunctionDefinition) -> Self {
        Self {
            r#type: FunctionToolType::Function,
            function,
        }
    }
}

/// Function tool type marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionToolType {
    /// Function tool marker.
    Function,
}

/// A provider/server builtin tool whose output should be constrained.
///
/// Mirrors hosted tool declarations from APIs such as OpenAI Responses or
/// Anthropic Messages. `type` is the provider-facing builtin tool type; `name`
/// and `parameters` are xgrammar-specific fields needed for constrained
/// decoding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuiltinToolParam {
    /// Provider-facing builtin tool type.
    pub r#type: BuiltinToolType,
    /// Tool name as it appears in model output.
    ///
    /// Use this when the emitted name differs from `type` — for example an
    /// OpenAI `web_search_preview` builtin emitted as `browser.search` by a
    /// Harmony-style model. Defaults to `type` when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Argument schema used to constrain the builtin tool's output.
    ///
    /// Hosted tool APIs often omit this, but xgrammar needs it to constrain
    /// the emitted arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
}

impl BuiltinToolParam {
    /// Build a builtin tool declaration.
    pub fn new(tool_type: impl Into<BuiltinToolType>) -> Self {
        Self {
            r#type: tool_type.into(),
            name: None,
            parameters: None,
        }
    }

    /// Set the model-output tool name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the builtin tool parameter JSON schema.
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = Some(parameters);
        self
    }
}

/// Function or builtin tool input accepted by the public builder API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolParam {
    /// Function tool.
    Function(FunctionToolParam),
    /// Builtin tool.
    Builtin(BuiltinToolParam),
}

impl From<FunctionToolParam> for ToolParam {
    fn from(value: FunctionToolParam) -> Self {
        Self::Function(value)
    }
}

impl From<BuiltinToolParam> for ToolParam {
    fn from(value: BuiltinToolParam) -> Self {
        Self::Builtin(value)
    }
}

/// Simple string tool-choice values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceValue {
    /// Disable tool calling.
    None,
    /// Allow text or tool calls.
    Auto,
    /// Require at least one tool call.
    Required,
}

/// Named tool-choice type marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedToolChoiceType {
    /// Function tool choice marker.
    Function,
}

/// The nested function reference used by Chat Completions named tool choice.
///
/// The Responses API uses a flat `{"type": "function", "name": "..."}` shape
/// without this nested object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedToolChoiceFunction {
    /// Function name.
    pub name: String,
}

/// Forces the model to call a specific function.
///
/// Chat Completions shape: `{"type": "function", "function": {"name": "..."}}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedToolChoiceParam {
    /// Choice type, always `function`.
    pub r#type: NamedToolChoiceType,
    /// Function reference.
    pub function: NamedToolChoiceFunction,
}

/// Forces the model to use a specific builtin tool.
///
/// Matching is by `type`; `name` is accepted for API compatibility but is not
/// used for matching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuiltinToolChoiceParam {
    /// Builtin tool type.
    pub r#type: BuiltinToolType,
    /// Optional model-output tool name. Builtin choices are matched by `type`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Allowed-tools choice type marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllowedToolChoiceType {
    /// Allowed-tools choice marker.
    AllowedTools,
}

/// Allowed-tools mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllowedToolsMode {
    /// Allow text output or one of the allowed tools.
    #[default]
    Auto,
    /// Require at least one of the allowed tools.
    Required,
}

impl AllowedToolsMode {
    fn simplified(self) -> SimplifiedToolChoice {
        match self {
            Self::Auto => SimplifiedToolChoice::Auto,
            Self::Required => SimplifiedToolChoice::Required,
        }
    }
}

/// A reference to a function or builtin tool allowed in this turn.
///
/// Chat Completions refs nest the function name
/// (`{"type": "function", "function": {"name": "..."}}`); Responses API refs
/// are flat (`{"type": "function", "name": "..."}`). Builtin refs are matched
/// by `type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowedToolRef {
    /// Tool type, usually `function` or a builtin tool type.
    pub r#type: String,
    /// Nested function reference for Chat Completions shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<NamedToolChoiceFunction>,
    /// Flat function name or builtin model-output name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl AllowedToolRef {
    /// Build a function reference.
    pub fn function(name: impl Into<String>) -> Self {
        Self {
            r#type: "function".to_string(),
            function: Some(NamedToolChoiceFunction { name: name.into() }),
            name: None,
        }
    }

    /// Build a builtin tool reference.
    pub fn builtin(tool_type: impl Into<BuiltinToolType>) -> Self {
        let tool_type = tool_type.into();
        Self {
            r#type: tool_type.to_string(),
            function: None,
            name: None,
        }
    }

    fn function_name(&self) -> Option<&str> {
        if self.r#type != "function" {
            return None;
        }
        self.function
            .as_ref()
            .map(|f| f.name.as_str())
            .or(self.name.as_deref())
    }
}

/// Constrains the available tools to a predefined set (Chat Completions nested payload).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowedToolsParam {
    /// Allowed-tools mode, `auto` or `required`.
    pub mode: AllowedToolsMode,
    /// Allowed tool references.
    pub tools: Vec<AllowedToolRef>,
}

/// Chat Completions allowed-tools choice: `{"type": "allowed_tools", "allowed_tools": {...}}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowedToolChoiceParam {
    /// Choice type, always `allowed_tools`.
    pub r#type: AllowedToolChoiceType,
    /// Nested allowed-tools payload.
    pub allowed_tools: AllowedToolsParam,
}

/// Responses-style flat allowed-tools choice: `{"type": "allowed_tools", "mode": ..., "tools": [...]}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlatAllowedToolChoiceParam {
    /// Choice type, always `allowed_tools`.
    pub r#type: AllowedToolChoiceType,
    /// Allowed-tools mode, `auto` or `required`.
    pub mode: AllowedToolsMode,
    /// Allowed tool references.
    pub tools: Vec<AllowedToolRef>,
}

/// Controls which (if any) tool the model calls.
///
/// `none` disables tools and generates a message, `auto` lets the model
/// choose between a message and tool calls, and `required` forces at least one
/// tool call. The named, builtin, and allowed-tools variants restrict the set
/// further, down to forcing a single tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Simple string choice.
    Value(ToolChoiceValue),
    /// Named function choice.
    NamedFunction(NamedToolChoiceParam),
    /// Chat Completions shaped allowed-tools choice.
    AllowedTools(AllowedToolChoiceParam),
    /// Responses-style flat allowed-tools choice.
    FlatAllowedTools(FlatAllowedToolChoiceParam),
    /// Builtin tool choice.
    Builtin(BuiltinToolChoiceParam),
}

impl Default for ToolChoice {
    fn default() -> Self {
        Self::Value(ToolChoiceValue::Auto)
    }
}

impl ToolChoice {
    /// Build `tool_choice="auto"`.
    pub fn auto() -> Self {
        Self::Value(ToolChoiceValue::Auto)
    }

    /// Build `tool_choice="none"`.
    pub fn none() -> Self {
        Self::Value(ToolChoiceValue::None)
    }

    /// Build `tool_choice="required"`.
    pub fn required() -> Self {
        Self::Value(ToolChoiceValue::Required)
    }

    /// Build a named function tool choice.
    pub fn function(name: impl Into<String>) -> Self {
        Self::NamedFunction(NamedToolChoiceParam {
            r#type: NamedToolChoiceType::Function,
            function: NamedToolChoiceFunction { name: name.into() },
        })
    }

    /// Build a builtin tool choice.
    pub fn builtin(tool_type: impl Into<BuiltinToolType>) -> Self {
        Self::Builtin(BuiltinToolChoiceParam {
            r#type: tool_type.into(),
            name: None,
        })
    }

    /// Build a nested allowed-tools choice.
    pub fn allowed_tools(mode: AllowedToolsMode, tools: Vec<AllowedToolRef>) -> Self {
        Self::AllowedTools(AllowedToolChoiceParam {
            r#type: AllowedToolChoiceType::AllowedTools,
            allowed_tools: AllowedToolsParam { mode, tools },
        })
    }
}

/// Internal simplified tool-choice value used by model builders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SimplifiedToolChoice {
    /// Allow text or tool calls.
    Auto,
    /// Require at least one tool call.
    Required,
    /// Force exactly one tool.
    Forced,
}

/// Normalized inputs for model-specific builders.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NormalizedToolChoice {
    /// Function tools remaining after tool-choice filtering.
    pub function_tools: Vec<FunctionToolParam>,
    /// Builtin tools remaining after tool-choice filtering.
    pub builtin_tools: Vec<BuiltinToolParam>,
    /// Simplified builder choice.
    pub choice: SimplifiedToolChoice,
}

/// Normalize public tools and tool choice into builder-ready inputs.
///
/// Splits tools into function and builtin lists and simplifies the public tool
/// choice to one of `Auto`, `Required`, or `Forced`: `none` clears the tools
/// and yields `Auto` (builders treat auto-with-no-tools as text-only); named
/// and builtin choices filter to a single tool and yield `Forced`; allowed
/// tools filter to the referenced set and carry the nested mode through.
pub(crate) fn normalize_tool_choice(
    tools: &[ToolParam],
    tool_choice: ToolChoice,
) -> Result<NormalizedToolChoice> {
    let mut function_tools = Vec::new();
    let mut builtin_tools = Vec::new();
    for tool in tools {
        match tool {
            ToolParam::Function(tool) => function_tools.push(tool.clone()),
            ToolParam::Builtin(tool) => builtin_tools.push(tool.clone()),
        }
    }

    let choice = match tool_choice {
        ToolChoice::Value(ToolChoiceValue::None) => {
            function_tools.clear();
            builtin_tools.clear();
            SimplifiedToolChoice::Auto
        }
        ToolChoice::Value(ToolChoiceValue::Auto) => SimplifiedToolChoice::Auto,
        ToolChoice::Value(ToolChoiceValue::Required) => SimplifiedToolChoice::Required,
        ToolChoice::NamedFunction(choice) => {
            let tool_name = choice.function.name;
            function_tools.retain(|tool| tool.function.name == tool_name);
            if function_tools.is_empty() {
                return Err(Error::ToolNotFound { name: tool_name });
            }
            builtin_tools.clear();
            SimplifiedToolChoice::Forced
        }
        ToolChoice::Builtin(choice) => {
            function_tools.clear();
            let tool_type = choice.r#type;
            builtin_tools.retain(|tool| tool.r#type.as_str() == tool_type.as_str());
            if builtin_tools.len() != 1 {
                return Err(Error::BuiltinToolChoiceAmbiguous {
                    tool_type: tool_type.to_string(),
                    matches: builtin_tools.len(),
                });
            }
            SimplifiedToolChoice::Forced
        }
        ToolChoice::AllowedTools(choice) => filter_allowed_tools(
            &mut function_tools,
            &mut builtin_tools,
            choice.allowed_tools.mode,
            &choice.allowed_tools.tools,
        )?,
        ToolChoice::FlatAllowedTools(choice) => filter_allowed_tools(
            &mut function_tools,
            &mut builtin_tools,
            choice.mode,
            &choice.tools,
        )?,
    };

    if choice == SimplifiedToolChoice::Required
        && function_tools.is_empty()
        && builtin_tools.is_empty()
    {
        return Err(Error::RequiredWithoutTools);
    }
    if choice == SimplifiedToolChoice::Forced && function_tools.len() + builtin_tools.len() != 1 {
        return Err(Error::ForcedToolChoiceInvalid {
            count: function_tools.len() + builtin_tools.len(),
        });
    }

    Ok(NormalizedToolChoice {
        function_tools,
        builtin_tools,
        choice,
    })
}

fn filter_allowed_tools(
    function_tools: &mut Vec<FunctionToolParam>,
    builtin_tools: &mut Vec<BuiltinToolParam>,
    mode: AllowedToolsMode,
    allowed_tools: &[AllowedToolRef],
) -> Result<SimplifiedToolChoice> {
    let choice = mode.simplified();

    let mut allowed_function_names = BTreeSet::new();
    let mut allowed_builtin_types = BTreeSet::new();
    for allowed in allowed_tools {
        if allowed.r#type == "function" {
            if let Some(name) = allowed.function_name() {
                allowed_function_names.insert(name.to_string());
            }
        } else {
            allowed_builtin_types.insert(allowed.r#type.clone());
        }
    }

    let available_function_names = function_tools
        .iter()
        .map(|tool| tool.function.name.clone())
        .collect::<BTreeSet<_>>();
    let available_builtin_types = builtin_tools
        .iter()
        .map(|tool| tool.r#type.as_str().to_string())
        .collect::<BTreeSet<_>>();

    let mut missing = Vec::new();
    for name in allowed_function_names.difference(&available_function_names) {
        missing.push(name.clone());
    }
    for tool_type in allowed_builtin_types.difference(&available_builtin_types) {
        missing.push(tool_type.clone());
    }
    if !missing.is_empty() {
        return Err(Error::AllowedToolMissing { names: missing });
    }

    function_tools.retain(|tool| allowed_function_names.contains(&tool.function.name));
    builtin_tools.retain(|tool| allowed_builtin_types.contains(tool.r#type.as_str()));
    Ok(choice)
}

/// Return the JSON schema used to constrain a function's emitted arguments.
///
/// Missing `parameters` and non-strict functions map to `true`, so the
/// arguments stay syntactically valid JSON but schema-unconstrained.
pub(crate) fn function_parameters(function: &FunctionDefinition) -> Value {
    if function.strict == Some(false) {
        return json!(true);
    }
    function.parameters.clone().unwrap_or_else(|| json!(true))
}

/// Return the JSON schema used to constrain a builtin tool's emitted arguments.
pub(crate) fn builtin_parameters(tool: &BuiltinToolParam) -> Value {
    tool.parameters.clone().unwrap_or_else(|| json!(true))
}

/// Return the model-output name for a builtin tool.
pub(crate) fn builtin_tool_name(tool: &BuiltinToolParam) -> &str {
    tool.name.as_deref().unwrap_or(tool.r#type.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_tool_choice_serializes_to_openai_wire_shape() {
        assert_eq!(
            serde_json::to_value(ToolChoice::function("search")).unwrap(),
            json!({
                "type": "function",
                "function": { "name": "search" }
            })
        );

        assert_eq!(
            serde_json::to_value(ToolChoice::allowed_tools(
                AllowedToolsMode::Required,
                vec![AllowedToolRef::function("search")]
            ))
            .unwrap(),
            json!({
                "type": "allowed_tools",
                "allowed_tools": {
                    "mode": "required",
                    "tools": [
                        { "type": "function", "function": { "name": "search" } }
                    ]
                }
            })
        );
    }

    #[test]
    fn typed_tool_choice_deserializes_openai_wire_shape() {
        let builtin_choice: ToolChoice =
            serde_json::from_value(json!({ "type": "web_search_preview" })).unwrap();
        match builtin_choice {
            ToolChoice::Builtin(choice) => {
                assert_eq!(choice.r#type.as_str(), "web_search_preview");
            }
            other => panic!("expected builtin tool choice, got {other:?}"),
        }

        let allowed_choice: ToolChoice = serde_json::from_value(json!({
            "type": "allowed_tools",
            "mode": "auto",
            "tools": [{ "type": "function", "name": "search" }]
        }))
        .unwrap();
        match allowed_choice {
            ToolChoice::FlatAllowedTools(choice) => {
                assert_eq!(choice.r#type, AllowedToolChoiceType::AllowedTools);
                assert_eq!(choice.mode, AllowedToolsMode::Auto);
            }
            other => panic!("expected flat allowed-tools choice, got {other:?}"),
        }
    }
}
