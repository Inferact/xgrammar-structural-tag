#![deny(missing_docs)]

//! Rust builders for xgrammar `structural_tag` tool-calling constraints.
//!
//! This crate builds the JSON object accepted by xgrammar's structural tag
//! backend. It does not compile grammars, allocate token masks, or parse model
//! output. Serving frontends can use it to construct
//! `StructuredOutputsParams.structural_tag`, then let their existing xgrammar
//! backend perform constrained decoding.
//!
//! The model templates follow xgrammar's structural tag builders, with
//! extensions like `hermes` and `hy_v3`.
//!
//! # Examples
//!
//! Build an automatic tool-calling structural tag:
//!
//! ```
//! use serde_json::json;
//! use xgrammar_structural_tag::{
//!     FunctionDefinition, FunctionToolParam, Model, ToolChoice, ToolParam,
//!     build_structural_tag,
//!     builders::StructuralTagOptions,
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
//! let tag = build_structural_tag(
//!     Model::Qwen35,
//!     &tools,
//!     ToolChoice::auto(),
//!     StructuralTagOptions::default(),
//! )?;
//! let structural_tag_json = tag.to_json_string()?;
//! assert!(structural_tag_json.contains("structural_tag"));
//! # Ok::<(), xgrammar_structural_tag::Error>(())
//! ```
//!
//! Require at least one tool call:
//!
//! ```
//! use xgrammar_structural_tag::{
//!     Model, ToolChoice, build_structural_tag,
//!     builders::StructuralTagOptions,
//! };
//! # use xgrammar_structural_tag::{FunctionDefinition, FunctionToolParam, ToolParam};
//! # let tools = vec![ToolParam::Function(FunctionToolParam::new(FunctionDefinition::new("ping")))];
//! let tag = build_structural_tag(
//!     Model::Llama,
//!     &tools,
//!     ToolChoice::required(),
//!     StructuralTagOptions::default().with_reasoning(false),
//! )?;
//! assert!(tag.to_json_string()?.contains("structural_tag"));
//! # Ok::<(), xgrammar_structural_tag::Error>(())
//! ```
//!
//! Force a named function:
//!
//! ```
//! use xgrammar_structural_tag::{
//!     Model, ToolChoice, build_structural_tag,
//!     builders::StructuralTagOptions,
//! };
//! # use xgrammar_structural_tag::{FunctionDefinition, FunctionToolParam, ToolParam};
//! # let tools = vec![ToolParam::Function(FunctionToolParam::new(FunctionDefinition::new("ping")))];
//! let tag = build_structural_tag(
//!     Model::Qwen3,
//!     &tools,
//!     ToolChoice::function("ping"),
//!     StructuralTagOptions::default().with_reasoning(false),
//! )?;
//! assert!(tag.to_json_string()?.contains("structural_tag"));
//! # Ok::<(), xgrammar_structural_tag::Error>(())
//! ```
//!
//! Build only when the request actually needs tool constraints:
//!
//! ```
//! use xgrammar_structural_tag::{
//!     Model, ToolChoice, build_optional_structural_tag,
//!     builders::StructuralTagOptions,
//! };
//! let tag = build_optional_structural_tag(
//!     Model::Llama,
//!     &[],
//!     ToolChoice::none(),
//!     StructuralTagOptions::default(),
//! )?;
//! assert!(tag.is_none());
//! # Ok::<(), xgrammar_structural_tag::Error>(())
//! ```

pub mod builders;
pub mod error;
pub mod format;
pub mod model;
pub mod tool;

pub use builders::{build_optional_structural_tag, build_structural_tag};
pub use error::{Error, Result};
pub use format::StructuralTag;
pub use model::Model;
pub use tool::{
    AllowedToolRef, AllowedToolsMode, BuiltinToolParam, FunctionDefinition, FunctionToolParam,
    ToolChoice, ToolParam,
};
