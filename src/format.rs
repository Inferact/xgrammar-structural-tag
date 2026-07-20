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

/// How the schema-constrained content of a [`JsonSchemaFormat`] is rendered.
///
/// Most models emit JSON arguments, but several tool-call syntaxes wrap the
/// arguments in model-specific XML instead.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonSchemaStyle {
    /// Standard JSON object syntax.
    #[default]
    Json,
    /// Qwen XML: `<parameter=key>value</parameter>`.
    QwenXml,
    /// MiniMax XML: `<parameter name="key">value</parameter>`.
    MinimaxXml,
    /// DeepSeek DSML: `<…parameter name="key" string="true|false">value</…parameter>`.
    DeepseekXml,
    /// GLM key-value XML: `<arg_key>key</arg_key><arg_value>value</arg_value>`.
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
    /// Whether object properties may appear in any order.
    ///
    /// When enabled, key validity and value schemas are enforced while required
    /// key presence and key uniqueness are relaxed. This applies recursively to
    /// nested objects.
    #[serde(default)]
    pub any_order: bool,
    /// Maximum consecutive whitespace characters, or no limit when unset.
    pub max_whitespace_cnt: Option<i32>,
}

impl JsonSchemaFormat {
    /// Build a standard JSON schema format.
    pub fn new(json_schema: Value) -> Self {
        Self {
            json_schema,
            style: JsonSchemaStyle::Json,
            any_order: false,
            max_whitespace_cnt: None,
        }
    }

    /// Set the model-specific schema rendering style.
    pub fn with_style(mut self, style: JsonSchemaStyle) -> Self {
        self.style = style;
        self
    }

    /// Configure whether object properties may appear in any order.
    pub fn with_any_order(mut self, any_order: bool) -> Self {
        self.any_order = any_order;
        self
    }

    /// Limit consecutive whitespace characters in the generated schema grammar.
    pub fn with_max_whitespace_cnt(mut self, max_whitespace_cnt: Option<i32>) -> Self {
        self.max_whitespace_cnt = max_whitespace_cnt;
        self
    }
}

/// A format that matches arbitrary text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnyTextFormat {
    /// Strings that must not appear in the arbitrary text region.
    #[serde(default)]
    pub excludes: Vec<String>,
}

/// A format that matches a single token, by ID or string representation.
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

/// A format that matches `begin`, then `content`, then `end`.
///
/// The end boundary may be a single string, one of several strings (any of
/// which closes the tag), or a token; see [`EndBoundary`].
///
/// # Examples
///
/// ```
/// use xgrammar_structural_tag::format::{Format, TagFormat};
///
/// // A single end string.
/// let tag = TagFormat::new("<response>", Format::any_text(), "</response>");
///
/// // Either end string closes the tag.
/// let tag = TagFormat::new(
///     "<response>",
///     Format::any_text(),
///     vec!["</response>", "</answer>"],
/// );
/// # let _ = tag;
/// ```
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

/// A format that allows free text until a trigger string appears, then
/// dispatches to the matching tag; after the tag's end boundary it resumes
/// free text until the next trigger.
///
/// Each tag must be matched by exactly one trigger, where the trigger is a
/// prefix of the tag's begin string. Tags must use string begin boundaries;
/// for token-level dispatch use [`TokenTriggeredTagsFormat`].
///
/// # Examples
///
/// With triggers `["<function="]` and one tag per function, accepted outputs
/// include:
///
/// ```text
/// <function=func1>{"name": "John", "age": 30}</function>
/// text<function=func1>{...}</function>more<function=func2>{...}</function>tail
/// ```
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

impl TriggeredTagsFormat {
    /// Build a triggered-tags format with optional tool calls and unrestricted text.
    pub fn new(triggers: &[&str], tags: Vec<TagFormat>) -> Self {
        Self {
            triggers: triggers.iter().map(|s| (*s).to_string()).collect(),
            tags,
            at_least_one: false,
            stop_after_first: false,
            excludes: vec![],
        }
    }

    /// Set strings excluded from free-text regions.
    pub fn with_excludes(mut self, excludes: &[&str]) -> Self {
        self.excludes = excludes.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Require at least one triggered tag.
    pub fn require_at_least_one(mut self) -> Self {
        self.at_least_one = true;
        self
    }
}

/// A token-level triggered tag dispatcher.
///
/// Like [`TriggeredTagsFormat`] but dispatches on trigger token IDs or strings
/// instead of free text. Tags must use token begin boundaries ([`TokenBoundary`]).
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

/// A format that matches zero, one, or more tags separated by a fixed
/// separator, with no other text allowed.
///
/// # Examples
///
/// With two function tags and separator `","`, the empty string is accepted,
/// as well as:
///
/// ```text
/// <function=func1>{...}</function>
/// <function=func1>{...}</function>,<function=func2>{...}</function>
/// ```
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

/// A format that matches its child 0 or 1 time (EBNF optional).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionalFormat {
    /// Child format.
    pub content: Box<Format>,
}

/// A format that matches its child 1 or more times (EBNF plus).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlusFormat {
    /// Child format.
    pub content: Box<Format>,
}

/// A format that matches its child 0 or more times (EBNF star).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StarFormat {
    /// Child format.
    pub content: Box<Format>,
}

/// A format that matches its child between `min` and `max` times, inclusive.
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
///
/// The model may emit any free text, but once one of the `rules` patterns
/// appears, the following output must match that rule's format. When `loop`
/// is set, matching continues afterwards, alternating free text and pattern
/// detection; otherwise the format ends after the first dispatched rule.
/// `excludes` lists strings that may not appear in the free-text regions; it
/// can also terminate the format, e.g. wrapping the dispatch in a tag with an
/// empty end boundary.
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
///
/// The token-level analogue of [`DispatchFormat`]: free tokens are allowed
/// until one of the `rules` trigger tokens appears, after which the output
/// must match that rule's format. `loop` and `exclude_tokens` behave like
/// their string counterparts on [`DispatchFormat`].
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
        Self::JsonSchema(JsonSchemaFormat::new(json_schema))
    }

    /// Build a JSON schema format with an explicit style.
    pub fn json_schema_style(json_schema: Value, style: JsonSchemaStyle) -> Self {
        Self::JsonSchema(JsonSchemaFormat::new(json_schema).with_style(style))
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
        Self::TriggeredTags(TriggeredTagsFormat::new(triggers, tags))
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

    /// Build an optional child format.
    pub fn optional(content: Format) -> Self {
        Self::Optional(OptionalFormat {
            content: Box::new(content),
        })
    }

    /// Build a one-or-more child format.
    pub fn plus(content: Format) -> Self {
        Self::Plus(PlusFormat {
            content: Box::new(content),
        })
    }

    /// Build a zero-or-more child format.
    pub fn star(content: Format) -> Self {
        Self::Star(StarFormat {
            content: Box::new(content),
        })
    }

    /// Build a bounded or unbounded repeated child format.
    pub fn repeat(content: Format, min: i64, max: i64) -> Self {
        Self::Repeat(RepeatFormat {
            min,
            max,
            content: Box::new(content),
        })
    }
}

/// Top-level structural tag object accepted by xgrammar.
///
/// Corresponds to `{"type": "structural_tag", "format": {...}}` in the
/// `response_format` API field.
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

    /// Serialize this tag into the JSON string expected by structural-output backends.
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
        round_trip(Format::optional(Format::const_string("x")));
        round_trip(Format::plus(Format::const_string("x")));
        round_trip(Format::star(Format::const_string("x")));
        round_trip(Format::repeat(Format::const_string("x"), 1, 3));
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

    #[test]
    fn json_schema_options_use_upstream_wire_fields() {
        let format = Format::JsonSchema(
            JsonSchemaFormat::new(json!({"type": "object"}))
                .with_any_order(true)
                .with_max_whitespace_cnt(Some(2)),
        );
        let value = serde_json::to_value(format).unwrap();
        assert_eq!(value["any_order"], true);
        assert_eq!(value["max_whitespace_cnt"], 2);
    }
}
