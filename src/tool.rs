//! OpenAI-style tool and tool-choice DTOs.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::{Error, Result};

/// One function definition nested under a function tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Function name emitted by the model.
    pub name: String,
    /// Optional natural-language description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON schema for function arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
    /// Whether strict schema adherence is requested by the caller.
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

/// One OpenAI Chat Completions function tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionToolParam {
    /// Tool type, always `function`.
    #[serde(rename = "type")]
    pub tool_type: FunctionToolType,
    /// Function definition.
    pub function: FunctionDefinition,
}

impl FunctionToolParam {
    /// Build a function tool.
    pub fn new(function: FunctionDefinition) -> Self {
        Self {
            tool_type: FunctionToolType::Function,
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

/// A provider/server builtin tool declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuiltinToolParam {
    /// Provider-facing builtin tool type.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Optional model-output tool name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// JSON schema for builtin tool arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
}

impl BuiltinToolParam {
    /// Build a builtin tool declaration.
    pub fn new(tool_type: impl Into<String>) -> Self {
        Self {
            tool_type: tool_type.into(),
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

/// Nested function reference used by named tool choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedToolChoiceFunction {
    /// Function name.
    pub name: String,
}

/// Named function tool choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedToolChoiceParam {
    /// Choice type, normally `function`.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function reference.
    pub function: NamedToolChoiceFunction,
}

/// Builtin tool choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuiltinToolChoiceParam {
    /// Builtin tool type.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Optional model-output tool name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// One allowed function or builtin tool reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowedToolRef {
    /// Tool type, usually `function` or a builtin tool type.
    #[serde(rename = "type")]
    pub tool_type: String,
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
            tool_type: "function".to_string(),
            function: Some(NamedToolChoiceFunction { name: name.into() }),
            name: None,
        }
    }

    /// Build a builtin tool reference.
    pub fn builtin(tool_type: impl Into<String>) -> Self {
        Self {
            tool_type: tool_type.into(),
            function: None,
            name: None,
        }
    }

    fn function_name(&self) -> Option<&str> {
        if self.tool_type != "function" {
            return None;
        }
        self.function
            .as_ref()
            .map(|f| f.name.as_str())
            .or(self.name.as_deref())
    }
}

/// Nested allowed-tools payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowedToolsParam {
    /// Allowed-tools mode, `auto` or `required`.
    pub mode: String,
    /// Allowed tool references.
    pub tools: Vec<AllowedToolRef>,
}

/// Chat Completions shaped allowed-tools choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowedToolChoiceParam {
    /// Choice type, always `allowed_tools`.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Nested allowed-tools payload.
    pub allowed_tools: AllowedToolsParam,
}

/// Flat Responses-style allowed-tools choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlatAllowedToolChoiceParam {
    /// Choice type, always `allowed_tools`.
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Allowed-tools mode, `auto` or `required`.
    pub mode: String,
    /// Allowed tool references.
    pub tools: Vec<AllowedToolRef>,
}

/// Tool choice accepted by structural tag builders.
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
            tool_type: "function".to_string(),
            function: NamedToolChoiceFunction { name: name.into() },
        })
    }

    /// Build a builtin tool choice.
    pub fn builtin(tool_type: impl Into<String>) -> Self {
        Self::Builtin(BuiltinToolChoiceParam {
            tool_type: tool_type.into(),
            name: None,
        })
    }

    /// Build a nested allowed-tools choice.
    pub fn allowed_tools(mode: impl Into<String>, tools: Vec<AllowedToolRef>) -> Self {
        Self::AllowedTools(AllowedToolChoiceParam {
            tool_type: "allowed_tools".to_string(),
            allowed_tools: AllowedToolsParam {
                mode: mode.into(),
                tools,
            },
        })
    }
}

/// Internal simplified tool-choice value used by model builders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimplifiedToolChoice {
    /// Allow text or tool calls.
    Auto,
    /// Require at least one tool call.
    Required,
    /// Force exactly one tool.
    Forced,
}

/// Normalized inputs for model-specific builders.
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedToolChoice {
    /// Function tools remaining after tool-choice filtering.
    pub function_tools: Vec<FunctionToolParam>,
    /// Builtin tools remaining after tool-choice filtering.
    pub builtin_tools: Vec<BuiltinToolParam>,
    /// Simplified builder choice.
    pub choice: SimplifiedToolChoice,
}

/// Normalize public tools and tool choice into builder-ready inputs.
pub fn normalize_tool_choice(
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
            builtin_tools.retain(|tool| tool.tool_type == choice.tool_type);
            if builtin_tools.len() != 1 {
                return Err(Error::BuiltinToolChoiceAmbiguous {
                    tool_type: choice.tool_type,
                    matches: builtin_tools.len(),
                });
            }
            SimplifiedToolChoice::Forced
        }
        ToolChoice::AllowedTools(choice) => {
            let mode = filter_allowed_tools(
                &mut function_tools,
                &mut builtin_tools,
                &choice.allowed_tools.mode,
                &choice.allowed_tools.tools,
            )?;
            mode
        }
        ToolChoice::FlatAllowedTools(choice) => filter_allowed_tools(
            &mut function_tools,
            &mut builtin_tools,
            &choice.mode,
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
    mode: &str,
    allowed_tools: &[AllowedToolRef],
) -> Result<SimplifiedToolChoice> {
    let choice = match mode {
        "auto" => SimplifiedToolChoice::Auto,
        "required" => SimplifiedToolChoice::Required,
        other => {
            return Err(Error::InvalidAllowedToolsMode {
                mode: other.to_string(),
            });
        }
    };

    let mut allowed_function_names = BTreeSet::new();
    let mut allowed_builtin_types = BTreeSet::new();
    for allowed in allowed_tools {
        if allowed.tool_type == "function" {
            if let Some(name) = allowed.function_name() {
                allowed_function_names.insert(name.to_string());
            }
        } else {
            allowed_builtin_types.insert(allowed.tool_type.clone());
        }
    }

    let available_function_names = function_tools
        .iter()
        .map(|tool| tool.function.name.clone())
        .collect::<BTreeSet<_>>();
    let available_builtin_types = builtin_tools
        .iter()
        .map(|tool| tool.tool_type.clone())
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
    builtin_tools.retain(|tool| allowed_builtin_types.contains(&tool.tool_type));
    Ok(choice)
}

/// Return the JSON schema used to constrain a function's emitted arguments.
pub fn function_parameters(function: &FunctionDefinition) -> Value {
    if function.strict == Some(false) {
        return json!(true);
    }
    function.parameters.clone().unwrap_or_else(|| json!(true))
}

/// Return the JSON schema used to constrain a builtin tool's emitted arguments.
pub fn builtin_parameters(tool: &BuiltinToolParam) -> Value {
    tool.parameters.clone().unwrap_or_else(|| json!(true))
}

/// Return the model-output name for a builtin tool.
pub fn builtin_tool_name(tool: &BuiltinToolParam) -> &str {
    tool.name.as_deref().unwrap_or(&tool.tool_type)
}
