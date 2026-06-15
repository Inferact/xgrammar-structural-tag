#![deny(missing_docs)]

//! Rust builders for xgrammar `structural_tag` tool-calling constraints.
//!
//! This crate builds the JSON object accepted by xgrammar's structural tag
//! backend. It does not compile grammars, allocate token masks, or parse model
//! output. Serving frontends can use it to construct
//! `StructuredOutputsParams.structural_tag`, then let their existing xgrammar
//! backend perform constrained decoding.
//!
//! The model templates are ported from xgrammar's Apache-2.0 structural tag
//! builders at commit `4d145cc13d878c751ebeed36af1c013074be76bc`, with
//! vLLM/Rust frontend extensions for `hermes` and `hy_v3`.
//!
//! # Examples
//!
//! Build an automatic tool-calling structural tag:
//!
//! ```
//! use serde_json::json;
//! use xgrammar_structural_tag::{
//!     FunctionDefinition, FunctionToolParam, ToolChoice, ToolParam,
//!     get_model_structural_tag,
//! };
//!
//! let tools = vec![ToolParam::Function(FunctionToolParam::new(
//!     FunctionDefinition::new("get_weather")
//!         .with_parameters(json!({
//!             "type": "object",
//!             "properties": { "city": { "type": "string" } },
//!             "required": ["city"]
//!         })),
//! ))];
//!
//! let tag = get_model_structural_tag("qwen_3_5", &tools, ToolChoice::auto(), true)?;
//! let structural_tag_json = tag.to_json_string()?;
//! assert!(structural_tag_json.contains("structural_tag"));
//! # Ok::<(), xgrammar_structural_tag::Error>(())
//! ```
//!
//! Require at least one tool call:
//!
//! ```
//! use xgrammar_structural_tag::{ToolChoice, get_model_structural_tag};
//! # use xgrammar_structural_tag::{FunctionDefinition, FunctionToolParam, ToolParam};
//! # let tools = vec![ToolParam::Function(FunctionToolParam::new(FunctionDefinition::new("ping")))];
//! let tag = get_model_structural_tag("llama", &tools, ToolChoice::required(), false)?;
//! assert!(tag.to_json_string()?.contains("structural_tag"));
//! # Ok::<(), xgrammar_structural_tag::Error>(())
//! ```
//!
//! Force a named function:
//!
//! ```
//! use xgrammar_structural_tag::{ToolChoice, get_model_structural_tag};
//! # use xgrammar_structural_tag::{FunctionDefinition, FunctionToolParam, ToolParam};
//! # let tools = vec![ToolParam::Function(FunctionToolParam::new(FunctionDefinition::new("ping")))];
//! let tag = get_model_structural_tag("qwen_3", &tools, ToolChoice::function("ping"), false)?;
//! assert!(tag.to_json_string()?.contains("structural_tag"));
//! # Ok::<(), xgrammar_structural_tag::Error>(())
//! ```
//!
//! Build only when the request actually needs tool constraints:
//!
//! ```
//! use xgrammar_structural_tag::{ToolChoice, maybe_get_model_structural_tag};
//! let tag = maybe_get_model_structural_tag("llama", &[], ToolChoice::none(), false)?;
//! assert!(tag.is_none());
//! # Ok::<(), xgrammar_structural_tag::Error>(())
//! ```

pub mod builders;
pub mod error;
pub mod format;
pub mod model;
pub mod tool;

pub use builders::{get_model_structural_tag, maybe_get_model_structural_tag, supported_models};
pub use error::{Error, Result};
pub use format::StructuralTag;
pub use tool::{
    AllowedToolRef, BuiltinToolParam, FunctionDefinition, FunctionToolParam, ToolChoice, ToolParam,
};
