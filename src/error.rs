//! Error types returned by structural tag builders.

use thiserror::Error as ThisError;

/// Result alias used by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors produced while normalizing tools or building a model structural tag.
#[derive(Debug, ThisError)]
pub enum Error {
    /// The requested model key is not supported by this crate.
    #[error("unknown structural tag model '{model}', supported models: {supported:?}")]
    UnknownModel {
        /// Requested model key.
        model: String,
        /// Supported model keys.
        supported: Vec<&'static str>,
    },

    /// A named tool choice referenced a function absent from `tools`.
    #[error("the tool with name '{name}' is not found in the tools list")]
    ToolNotFound {
        /// Missing function name.
        name: String,
    },

    /// A builtin tool choice matched zero or multiple builtin tool entries.
    #[error("builtin tool choice '{tool_type}' must match exactly one builtin tool, got {matches}")]
    BuiltinToolChoiceAmbiguous {
        /// Builtin tool type used by the choice.
        tool_type: String,
        /// Number of matched builtin tools.
        matches: usize,
    },

    /// `tool_choice=required` was requested with no remaining tools.
    #[error("tool_choice=required requires at least one available tool")]
    RequiredWithoutTools,

    /// A forced choice failed to resolve to exactly one function or builtin tool.
    #[error("forced tool choice must resolve to exactly one tool, got {count}")]
    ForcedToolChoiceInvalid {
        /// Number of tools remaining after choice normalization.
        count: usize,
    },

    /// An allowed-tools choice referenced functions or builtins absent from `tools`.
    #[error("allowed tool references are not found in the tools list: {names:?}")]
    AllowedToolMissing {
        /// Missing tool names or builtin types.
        names: Vec<String>,
    },

    /// The allowed-tools mode is not one of xgrammar's supported modes.
    #[error("allowed_tools.mode must be 'auto' or 'required', got '{mode}'")]
    InvalidAllowedToolsMode {
        /// Invalid mode value.
        mode: String,
    },

    /// JSON serialization failed.
    #[error("failed to serialize structural tag: {0}")]
    Serialize(#[from] serde_json::Error),
}
