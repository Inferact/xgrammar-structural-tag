//! Demonstrate a caller-provided structural-tag builder.
//!
//! The custom builder emits `<custom_tool name="...">...</custom_tool>` tags
//! using only the public `StructuralTagBuilder` and `format` APIs.

use serde_json::json;
use xgrammar_structural_tag::{
    FunctionDefinition, FunctionToolParam, StructuralTag, ToolChoice, ToolParam,
    build_structural_tag,
    builders::{StructuralTagBuilder, StructuralTagContext, StructuralTagOptions},
    format::{Format, TagFormat},
    tool::function_parameters,
};

#[derive(Debug, Clone, Copy)]
struct CustomXmlBuilder;

impl StructuralTagBuilder for CustomXmlBuilder {
    fn build(
        &self,
        ctx: StructuralTagContext<'_>,
    ) -> xgrammar_structural_tag::Result<StructuralTag> {
        let tags = ctx
            .function_tools
            .iter()
            .map(|tool| {
                TagFormat::new(
                    format!("<custom_tool name=\"{}\">", tool.function.name),
                    Format::json_schema(function_parameters(&tool.function)),
                    "</custom_tool>",
                )
            })
            .collect();

        Ok(StructuralTag::new(Format::tags_with_separator(
            tags,
            "\n",
            ctx.tool_choice.requires_tool_call(),
            ctx.tool_choice.is_forced(),
        )))
    }
}

fn main() -> xgrammar_structural_tag::Result<()> {
    let tools = vec![
        ToolParam::Function(FunctionToolParam::new(
            FunctionDefinition::new("search").with_parameters(json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            })),
        )),
        ToolParam::Function(FunctionToolParam::new(FunctionDefinition::new("lookup"))),
    ];

    let tag = build_structural_tag(
        CustomXmlBuilder,
        &tools,
        ToolChoice::function("search"),
        StructuralTagOptions::default().with_reasoning(false),
    )?;

    let rendered = tag.to_json_string()?;
    assert!(rendered.contains("<custom_tool name=\\\"search\\\">"));
    assert!(!rendered.contains("<custom_tool name=\\\"lookup\\\">"));
    println!("{rendered}");

    Ok(())
}
