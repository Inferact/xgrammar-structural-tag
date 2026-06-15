//! Data transfer objects for xgrammar structural tag JSON.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A token reference used by token-aware structural tag formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
    /// A tokenizer token ID.
    Id(i64),
    /// A token string representation.
    Text(String),
}

impl From<i64> for TokenValue {
    fn from(value: i64) -> Self {
        Self::Id(value)
    }
}

impl From<&str> for TokenValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for TokenValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

/// JSON schema serialization style used inside [`JsonSchemaFormat`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonSchemaStyle {
    /// Standard JSON object syntax.
    #[default]
    Json,
    /// Qwen XML parameters: `<parameter=name>value</parameter>`.
    QwenXml,
    /// MiniMax XML parameters: `<parameter name="name">value</parameter>`.
    MinimaxXml,
    /// DeepSeek DSML parameters.
    DeepseekXml,
    /// GLM/HY3 key-value XML parameters.
    GlmXml,
}

/// A format that matches a constant string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstStringFormat {
    /// Constant string value.
    pub value: String,
}

/// A format that matches a JSON schema with an optional model-specific style.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonSchemaFormat {
    /// JSON schema object or `true` for unconstrained JSON.
    pub json_schema: Value,
    /// Conversion style for non-JSON tool argument syntaxes.
    #[serde(default)]
    pub style: JsonSchemaStyle,
}

/// A format that matches arbitrary text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnyTextFormat {
    /// Strings that must not appear in the arbitrary text region.
    #[serde(default)]
    pub excludes: Vec<String>,
}

/// A format that matches a single token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenFormat {
    /// Token ID or token string.
    pub token: TokenValue,
}

/// A token boundary object used in tag begin/end fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenBoundary {
    r#type: TokenBoundaryKind,
    /// Token ID or token string.
    pub token: TokenValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TokenBoundaryKind {
    Token,
}

impl TokenBoundary {
    /// Build a token boundary.
    pub fn new(token: impl Into<TokenValue>) -> Self {
        Self {
            r#type: TokenBoundaryKind::Token,
            token: token.into(),
        }
    }
}

/// A format that matches one token excluding a set of tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExcludeTokenFormat {
    /// Tokens excluded from the match.
    #[serde(default)]
    pub exclude_tokens: Vec<TokenValue>,
}

/// A format that matches zero or more tokens excluding a set of tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnyTokensFormat {
    /// Tokens excluded from the match.
    #[serde(default)]
    pub exclude_tokens: Vec<TokenValue>,
}

/// A format that embeds an EBNF grammar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarFormat {
    /// EBNF grammar text.
    pub grammar: String,
}

/// A format that matches a regex pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegexFormat {
    /// Regex pattern.
    pub pattern: String,
}

/// A format that matches a sequence of child formats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SequenceFormat {
    /// Ordered child formats.
    pub elements: Vec<Format>,
}

/// A format that matches one of multiple child formats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrFormat {
    /// Alternative child formats.
    pub elements: Vec<Format>,
}

/// A tag begin boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TagBoundary {
    /// Literal string boundary.
    Text(String),
    /// Token-level boundary.
    Token(TokenBoundary),
}

impl From<&str> for TagBoundary {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for TagBoundary {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<TokenBoundary> for TagBoundary {
    fn from(value: TokenBoundary) -> Self {
        Self::Token(value)
    }
}

/// A tag end boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EndBoundary {
    /// Literal string boundary.
    Text(String),
    /// One of multiple literal string boundaries.
    Texts(Vec<String>),
    /// Token-level boundary.
    Token(TokenBoundary),
}

impl From<&str> for EndBoundary {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for EndBoundary {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<Vec<String>> for EndBoundary {
    fn from(value: Vec<String>) -> Self {
        Self::Texts(value)
    }
}

impl From<Vec<&str>> for EndBoundary {
    fn from(value: Vec<&str>) -> Self {
        Self::Texts(value.into_iter().map(str::to_string).collect())
    }
}

impl From<TokenBoundary> for EndBoundary {
    fn from(value: TokenBoundary) -> Self {
        Self::Token(value)
    }
}

/// A format that matches `begin + content + end`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagFormat {
    /// Begin boundary.
    pub begin: TagBoundary,
    /// Content format inside the tag.
    pub content: Box<Format>,
    /// End boundary.
    pub end: EndBoundary,
}

impl TagFormat {
    /// Build a tag format.
    pub fn new(
        begin: impl Into<TagBoundary>,
        content: Format,
        end: impl Into<EndBoundary>,
    ) -> Self {
        Self {
            begin: begin.into(),
            content: Box::new(content),
            end: end.into(),
        }
    }
}

/// A format that dispatches to tags after seeing trigger strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggeredTagsFormat {
    /// Trigger strings.
    pub triggers: Vec<String>,
    /// Tag formats reached by triggers.
    pub tags: Vec<TagFormat>,
    /// Whether at least one tag must be produced.
    #[serde(default)]
    pub at_least_one: bool,
    /// Whether to stop after the first tag is produced.
    #[serde(default)]
    pub stop_after_first: bool,
    /// Strings excluded from free text regions.
    #[serde(default)]
    pub excludes: Vec<String>,
}

/// A token-level triggered tag dispatcher.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenTriggeredTagsFormat {
    /// Trigger token IDs or strings.
    pub trigger_tokens: Vec<TokenValue>,
    /// Tag formats reached by triggers.
    pub tags: Vec<TagFormat>,
    /// Tokens excluded from free token regions.
    #[serde(default)]
    pub exclude_tokens: Vec<TokenValue>,
    /// Whether at least one tag must be produced.
    #[serde(default)]
    pub at_least_one: bool,
    /// Whether to stop after the first tag is produced.
    #[serde(default)]
    pub stop_after_first: bool,
}

/// A format that matches tags separated by a fixed separator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagsWithSeparatorFormat {
    /// Candidate tag formats.
    pub tags: Vec<TagFormat>,
    /// Separator between adjacent tags.
    pub separator: String,
    /// Whether at least one tag must be produced.
    #[serde(default)]
    pub at_least_one: bool,
    /// Whether to stop after the first tag is produced.
    #[serde(default)]
    pub stop_after_first: bool,
}

/// Optional child format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionalFormat {
    /// Child format.
    pub content: Box<Format>,
}

/// One-or-more child format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlusFormat {
    /// Child format.
    pub content: Box<Format>,
}

/// Zero-or-more child format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StarFormat {
    /// Child format.
    pub content: Box<Format>,
}

/// Repeated child format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepeatFormat {
    /// Minimum number of repetitions.
    pub min: i64,
    /// Maximum number of repetitions, or `-1` for unbounded.
    pub max: i64,
    /// Child format.
    pub content: Box<Format>,
}

/// A string-pattern dispatcher used inside free text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatchFormat {
    /// `(pattern, format)` rules.
    pub rules: Vec<(String, Format)>,
    /// Whether matching continues after one dispatched format.
    #[serde(default = "default_true")]
    pub r#loop: bool,
    /// Strings excluded from free text regions.
    #[serde(default)]
    pub excludes: Vec<String>,
}

/// A token-pattern dispatcher used inside free token regions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenDispatchFormat {
    /// `(token, format)` rules.
    pub rules: Vec<(TokenValue, Format)>,
    /// Whether matching continues after one dispatched format.
    #[serde(default = "default_true")]
    pub r#loop: bool,
    /// Tokens excluded from free token regions.
    #[serde(default)]
    pub exclude_tokens: Vec<TokenValue>,
}

/// Deprecated Qwen XML parameter format retained for wire compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QwenXmlParameterFormat {
    /// JSON schema object or `true` for unconstrained JSON.
    pub json_schema: Value,
}

fn default_true() -> bool {
    true
}

/// Any xgrammar structural tag format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Format {
    /// Match a constant string.
    ConstString(ConstStringFormat),
    /// Match a JSON schema.
    JsonSchema(JsonSchemaFormat),
    /// Match arbitrary text.
    AnyText(AnyTextFormat),
    /// Match one token.
    Token(TokenFormat),
    /// Match one token excluding a set.
    ExcludeToken(ExcludeTokenFormat),
    /// Match arbitrary tokens excluding a set.
    AnyTokens(AnyTokensFormat),
    /// Match an EBNF grammar.
    Grammar(GrammarFormat),
    /// Match a regex pattern.
    Regex(RegexFormat),
    /// Match a sequence.
    Sequence(SequenceFormat),
    /// Match one alternative.
    Or(OrFormat),
    /// Match a tag.
    Tag(TagFormat),
    /// Match triggered tags.
    TriggeredTags(TriggeredTagsFormat),
    /// Match token-triggered tags.
    TokenTriggeredTags(TokenTriggeredTagsFormat),
    /// Match tags with a separator.
    TagsWithSeparator(TagsWithSeparatorFormat),
    /// Match an optional child.
    Optional(OptionalFormat),
    /// Match one-or-more children.
    Plus(PlusFormat),
    /// Match zero-or-more children.
    Star(StarFormat),
    /// Match a repeated child.
    Repeat(RepeatFormat),
    /// Dispatch on string patterns.
    Dispatch(DispatchFormat),
    /// Dispatch on token patterns.
    TokenDispatch(TokenDispatchFormat),
    /// Deprecated Qwen XML parameter format.
    QwenXmlParameter(QwenXmlParameterFormat),
}

impl Format {
    /// Build a constant string format.
    pub fn const_string(value: impl Into<String>) -> Self {
        Self::ConstString(ConstStringFormat {
            value: value.into(),
        })
    }

    /// Build a standard JSON schema format.
    pub fn json_schema(json_schema: Value) -> Self {
        Self::JsonSchema(JsonSchemaFormat {
            json_schema,
            style: JsonSchemaStyle::Json,
        })
    }

    /// Build a JSON schema format with an explicit style.
    pub fn json_schema_style(json_schema: Value, style: JsonSchemaStyle) -> Self {
        Self::JsonSchema(JsonSchemaFormat { json_schema, style })
    }

    /// Build an arbitrary text format.
    pub fn any_text() -> Self {
        Self::AnyText(AnyTextFormat { excludes: vec![] })
    }

    /// Build an arbitrary text format with excluded strings.
    pub fn any_text_excluding(excludes: &[&str]) -> Self {
        Self::AnyText(AnyTextFormat {
            excludes: excludes.iter().map(|s| (*s).to_string()).collect(),
        })
    }

    /// Build a regex format.
    pub fn regex(pattern: impl Into<String>) -> Self {
        Self::Regex(RegexFormat {
            pattern: pattern.into(),
        })
    }

    /// Build a sequence format.
    pub fn sequence(elements: Vec<Format>) -> Self {
        Self::Sequence(SequenceFormat { elements })
    }

    /// Build an `or` format.
    pub fn or(elements: Vec<Format>) -> Self {
        Self::Or(OrFormat { elements })
    }

    /// Build a tag format.
    pub fn tag(
        begin: impl Into<TagBoundary>,
        content: Format,
        end: impl Into<EndBoundary>,
    ) -> Self {
        Self::Tag(TagFormat::new(begin, content, end))
    }

    /// Build a triggered-tags format.
    pub fn triggered_tags(triggers: &[&str], tags: Vec<TagFormat>) -> Self {
        Self::TriggeredTags(TriggeredTagsFormat {
            triggers: triggers.iter().map(|s| (*s).to_string()).collect(),
            tags,
            at_least_one: false,
            stop_after_first: false,
            excludes: vec![],
        })
    }

    /// Build tags with a separator.
    pub fn tags_with_separator(
        tags: Vec<TagFormat>,
        separator: impl Into<String>,
        at_least_one: bool,
        stop_after_first: bool,
    ) -> Self {
        Self::TagsWithSeparator(TagsWithSeparatorFormat {
            tags,
            separator: separator.into(),
            at_least_one,
            stop_after_first,
        })
    }
}

/// Deprecated structural tag item shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralTagItem {
    /// Begin tag.
    pub begin: String,
    /// JSON schema payload.
    pub schema: Value,
    /// End tag.
    pub end: String,
}

/// Top-level structural tag object accepted by xgrammar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralTag {
    r#type: StructuralTagKind,
    /// Structural tag format.
    pub format: Format,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StructuralTagKind {
    StructuralTag,
}

impl StructuralTag {
    /// Build a top-level structural tag from a format.
    pub fn new(format: Format) -> Self {
        Self {
            r#type: StructuralTagKind::StructuralTag,
            format,
        }
    }

    /// Return the top-level type string.
    pub fn kind(&self) -> &'static str {
        "structural_tag"
    }

    /// Serialize this tag into the JSON string expected by vLLM engine-core.
    pub fn to_json_string(&self) -> crate::Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn round_trip(format: Format) {
        let value = serde_json::to_value(&format).unwrap();
        let decoded: Format = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(decoded, format, "round-trip failed for {value}");
    }

    #[test]
    fn every_format_variant_round_trips() {
        let tag = TagFormat::new("<x>", Format::any_text(), "</x>");
        round_trip(Format::const_string("x"));
        round_trip(Format::json_schema(json!({"type": "object"})));
        round_trip(Format::any_text_excluding(&["</x>"]));
        round_trip(Format::Token(TokenFormat {
            token: TokenValue::Id(1),
        }));
        round_trip(Format::ExcludeToken(ExcludeTokenFormat {
            exclude_tokens: vec![TokenValue::Text("</x>".to_string())],
        }));
        round_trip(Format::AnyTokens(AnyTokensFormat {
            exclude_tokens: vec![TokenValue::Id(2)],
        }));
        round_trip(Format::Grammar(GrammarFormat {
            grammar: "root ::= \"x\"".to_string(),
        }));
        round_trip(Format::regex("[a-z]+"));
        round_trip(Format::sequence(vec![Format::const_string("x")]));
        round_trip(Format::or(vec![
            Format::const_string("x"),
            Format::const_string("y"),
        ]));
        round_trip(Format::Tag(tag.clone()));
        round_trip(Format::triggered_tags(&["<x>"], vec![tag.clone()]));
        round_trip(Format::TokenTriggeredTags(TokenTriggeredTagsFormat {
            trigger_tokens: vec![TokenValue::Text("<x>".to_string())],
            tags: vec![TagFormat::new(
                TokenBoundary::new("<x>"),
                Format::any_text(),
                TokenBoundary::new("</x>"),
            )],
            exclude_tokens: vec![],
            at_least_one: true,
            stop_after_first: false,
        }));
        round_trip(Format::tags_with_separator(
            vec![tag.clone()],
            "\n",
            true,
            false,
        ));
        round_trip(Format::Optional(OptionalFormat {
            content: Box::new(Format::const_string("x")),
        }));
        round_trip(Format::Plus(PlusFormat {
            content: Box::new(Format::const_string("x")),
        }));
        round_trip(Format::Star(StarFormat {
            content: Box::new(Format::const_string("x")),
        }));
        round_trip(Format::Repeat(RepeatFormat {
            min: 1,
            max: 3,
            content: Box::new(Format::const_string("x")),
        }));
        round_trip(Format::Dispatch(DispatchFormat {
            rules: vec![("<x>".to_string(), Format::Tag(tag.clone()))],
            r#loop: true,
            excludes: vec!["</x>".to_string()],
        }));
        round_trip(Format::TokenDispatch(TokenDispatchFormat {
            rules: vec![(
                TokenValue::Text("<x>".to_string()),
                Format::const_string("x"),
            )],
            r#loop: true,
            exclude_tokens: vec![TokenValue::Id(3)],
        }));
        round_trip(Format::QwenXmlParameter(QwenXmlParameterFormat {
            json_schema: json!(true),
        }));
    }

    #[test]
    fn dispatch_uses_xgrammar_loop_field_name() {
        let value = serde_json::to_value(Format::Dispatch(DispatchFormat {
            rules: vec![],
            r#loop: false,
            excludes: vec![],
        }))
        .unwrap();
        assert_eq!(value["loop"], json!(false));
        assert!(value.get("loop_").is_none());
    }
}
