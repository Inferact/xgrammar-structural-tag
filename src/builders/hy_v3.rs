use crate::Result;
use crate::format::{Format, JsonSchemaStyle, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};

use super::{
    StructuralTagBuilder, StructuralTagContext, schema, structural, styled_schema, tag,
    tools_with_separator, triggered_with_excludes,
};

/// HY3 XML tool-calling structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct HyV3Builder;

impl StructuralTagBuilder for HyV3Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        Ok(build_hy_v3(ctx.function_tools, ctx.tool_choice))
    }
}

/// Build an HY3-style structural tag.
///
/// Local extension not present in upstream xgrammar. Uses GLM-style XML
/// arguments wrapped in `<tool_calls>` / `<tool_call>` / `<tool_sep>` markers,
/// with no reasoning part.
pub(super) fn build_hy_v3(tools: &[FunctionToolParam], choice: BuilderToolChoice) -> StructuralTag {
    const TOOL_CALLS_BEGIN: &str = "<tool_calls>\n";
    const TOOL_CALLS_TRIGGER: &str = "<tool_calls>";
    const TOOL_CALLS_END: &str = "</tool_calls>";
    const TOOL_CALL_BEGIN_PREFIX: &str = "<tool_call>";
    const TOOL_SEP: &str = "<tool_sep>";
    const TOOL_CALL_END: &str = "</tool_call>";

    let tool_tag = |tool: &FunctionToolParam| {
        tag(
            format!("{TOOL_CALL_BEGIN_PREFIX}{}{TOOL_SEP}", tool.function.name),
            styled_schema(schema(&tool.function), JsonSchemaStyle::GlmXml),
            TOOL_CALL_END,
        )
    };
    let suffix = match choice {
        BuilderToolChoice::Auto => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            if tags.is_empty() {
                Format::any_text()
            } else {
                let outer = tag(
                    TOOL_CALLS_BEGIN,
                    tools_with_separator(tags, "\n", true),
                    TOOL_CALLS_END,
                );
                triggered_with_excludes(&[TOOL_CALLS_TRIGGER], vec![outer], &[])
            }
        }
        BuilderToolChoice::Forced => Format::sequence(vec![
            Format::const_string(TOOL_CALLS_BEGIN),
            Format::Tag(tool_tag(&tools[0])),
            Format::const_string(format!("\n{TOOL_CALLS_END}")),
        ]),
        BuilderToolChoice::Required => {
            let tags = tools.iter().map(tool_tag).collect::<Vec<_>>();
            Format::sequence(vec![
                Format::const_string(TOOL_CALLS_BEGIN),
                tools_with_separator(tags, "\n", true),
                Format::const_string(format!("\n{TOOL_CALLS_END}")),
            ])
        }
    };
    structural(suffix)
}
