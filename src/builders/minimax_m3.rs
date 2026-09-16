use crate::format::{Format, JsonSchemaStyle, StructuralTag};
use crate::tool::{BuilderToolChoice, FunctionToolParam};
use crate::{Error, Result};

use super::{
    StructuralTagBuilder, StructuralTagContext, assemble, reasoning_prefix,
    required_triggered_with_excludes, schema, styled_schema, tag, text_excludes,
    tools_with_separator, triggered_with_excludes,
};

/// MiniMax M3 namespace-XML structural-tag builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct MinimaxM3Builder;

impl StructuralTagBuilder for MinimaxM3Builder {
    fn build(&self, ctx: StructuralTagContext<'_>) -> Result<StructuralTag> {
        if !ctx.builtin_tools.is_empty() {
            return Err(Error::UnsupportedBuiltinTools {
                model: "minimax_m3",
            });
        }
        Ok(build_minimax_m3(
            ctx.function_tools,
            ctx.tool_choice,
            ctx.options,
        ))
    }
}

/// Build a MiniMax-M3 structural tag with recursive namespace-prefixed XML arguments.
///
/// Enabled reasoning continues the prompt's `<mm:think>` opener; disabled
/// reasoning continues after the prompt's close marker. Adaptive mode accepts a
/// complete reasoning block or a direct response/tool call, optionally preceded
/// by `</mm:think>` when skipping reasoning.
///
/// The fixed-name XML converter in XGrammar requires a constrained argument
/// schema; unconstrained schemas such as those from `strict=false` fail at compilation.
pub(super) fn build_minimax_m3(
    tools: &[FunctionToolParam],
    choice: BuilderToolChoice,
    options: super::StructuralTagOptions,
) -> StructuralTag {
    const BEGIN: &str = "]<]minimax[>[<tool_call>\n";
    const END: &str = "]<]minimax[>[</tool_call>";
    const TRIGGER: &str = "]<]minimax[>[<tool_call>";
    const THINK_BEGIN: &str = "<mm:think>";
    const THINK_END: &str = "</mm:think>";
    const SUFFIX_EXCLUDES: &[&str] = &[
        THINK_BEGIN,
        THINK_END,
        END,
        "]<]minimax[>[<invoke",
        "]<]minimax[>[</invoke>",
    ];
    const REASONING_EXCLUDES: &[&str] = &[
        TRIGGER,
        THINK_BEGIN,
        THINK_END,
        END,
        "]<]minimax[>[<invoke",
        "]<]minimax[>[</invoke>",
    ];
    let invoke_tags = tools
        .iter()
        .map(|tool| {
            tag(
                format!("]<]minimax[>[<invoke name=\"{}\">", tool.function.name),
                styled_schema(
                    schema(&tool.function),
                    JsonSchemaStyle::MinimaxM3Xml,
                    options,
                ),
                "]<]minimax[>[</invoke>\n",
            )
        })
        .collect::<Vec<_>>();
    let suffix = match choice {
        BuilderToolChoice::Auto if invoke_tags.is_empty() => {
            Format::any_text_excluding(text_excludes(options, REASONING_EXCLUDES))
        }
        BuilderToolChoice::Auto => triggered_with_excludes(
            &[TRIGGER],
            vec![tag(
                BEGIN,
                tools_with_separator(invoke_tags, "", true, options),
                END,
            )],
            text_excludes(options, SUFFIX_EXCLUDES),
            options,
        ),
        BuilderToolChoice::Forced => Format::tag(BEGIN, Format::Tag(invoke_tags[0].clone()), END),
        BuilderToolChoice::Required => required_triggered_with_excludes(
            &[TRIGGER],
            vec![tag(
                BEGIN,
                tools_with_separator(invoke_tags, "", true, options),
                END,
            )],
            text_excludes(options, SUFFIX_EXCLUDES),
            options,
        ),
    };
    let mut prefix = reasoning_prefix(options, THINK_BEGIN, THINK_END, REASONING_EXCLUDES, "");
    if let Some(Format::Optional(optional)) = prefix.as_mut() {
        *optional.content = Format::or(vec![
            *optional.content.clone(),
            Format::const_string(THINK_END),
        ]);
    }
    assemble(prefix, suffix)
}
