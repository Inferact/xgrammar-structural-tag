use serde_json::Value;

/// Rust-emitted region budgets consumed by the real XGrammar compatibility test.
pub fn build_cases() -> Value {
    use xgrammar_structural_tag::format::{
        AnyTextFormat, AnyTokensFormat, Format, StructuralTag, TokenBoundary,
    };

    let text = |tokens, chars| {
        StructuralTag::new(Format::tag(
            "<think>",
            Format::AnyText(AnyTextFormat {
                max_tokens: tokens,
                max_chars: chars,
                ..Default::default()
            }),
            "</think>",
        ))
    };
    let tokens = |limit| {
        StructuralTag::new(Format::tag(
            TokenBoundary::new("<think>"),
            Format::AnyTokens(AnyTokensFormat {
                max_tokens: Some(limit),
                ..Default::default()
            }),
            TokenBoundary::new("</think>"),
        ))
    };
    serde_json::json!({
        "any_text_tokens": text(Some(2), None),
        "any_text_chars": text(None, Some(3)),
        "any_text_both": text(Some(2), Some(3)),
        "any_text_zero": text(Some(0), None),
        "any_tokens": tokens(2),
        "any_tokens_zero": tokens(0),
    })
}
