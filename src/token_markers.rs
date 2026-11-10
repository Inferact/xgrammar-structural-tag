//! Binding of a model's control markers to their dedicated tokens.

use crate::error::{Error, Result};
use crate::format::{
    AnyTokensFormat, EndBoundary, Format, SequenceFormat, StructuralTag, TagBoundary, TagFormat,
    TokenBoundary, TokenFormat, TokenTriggeredTagsFormat, TokenValue, TriggeredTagsFormat,
};

/// Match the given marker strings as their dedicated tokens instead of as text.
///
/// Markers in triggers, tag `begin`/`end` and constant strings become token formats; the
/// surrounding text is kept, and markers inside schema-driven content stay strings. Free text under
/// a token end becomes [`AnyTokensFormat`] (its excludes must be markers), and in token-triggered
/// free text marker excludes become token-level (other excludes are dropped), so free text may
/// contain a marker spelled from sub-tokens: use this only with parsers that recognize markers by
/// token ID. Markers under a [`Format::Repeat`] stay strings, because token formats beneath it are
/// not resolved against the tokenizer.
///
/// # Errors
///
/// Returns [`Error::UnbindableTokenMarkers`] for a token end over a repeat or dispatch format, a
/// list-valued `end` with an alternative ending in a marker, or free text under a token end that
/// cannot become token-level.
pub(crate) fn bind_marker_tokens(tag: StructuralTag, markers: &[&str]) -> Result<StructuralTag> {
    let mut markers: Vec<&str> = markers.iter().copied().filter(|m| !m.is_empty()).collect();
    // Longest first, so a marker that extends another marker wins.
    markers.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    markers.dedup();
    if markers.is_empty() {
        return Ok(tag);
    }
    let binder = MarkerBinder { markers };
    Ok(StructuralTag::new(binder.bind(tag.format)?))
}

struct MarkerBinder<'a> {
    markers: Vec<&'a str>,
}

fn token(marker: &str) -> Format {
    Format::Token(TokenFormat {
        token: TokenValue::Text(marker.to_string()),
    })
}

fn unbindable(reason: impl Into<String>) -> Error {
    Error::UnbindableTokenMarkers {
        reason: reason.into(),
    }
}

impl MarkerBinder<'_> {
    fn bind(&self, format: Format) -> Result<Format> {
        Ok(match format {
            Format::ConstString(constant) => self.bind_const_string(constant.value),
            Format::Tag(tag) => Format::Tag(self.bind_tag(tag, None, false)?),
            Format::TriggeredTags(triggered) => self.bind_triggered_tags(triggered)?,
            Format::TagsWithSeparator(mut tags) => {
                tags.tags = self.bind_tags(tags.tags, None, false)?;
                Format::TagsWithSeparator(tags)
            }
            Format::Sequence(mut sequence) => {
                sequence.elements = self.bind_all(sequence.elements, Self::bind)?;
                Format::Sequence(sequence)
            }
            Format::Or(mut or) => {
                or.elements = self.bind_all(or.elements, Self::bind)?;
                Format::Or(or)
            }
            Format::Optional(mut optional) => {
                optional.content = Box::new(self.bind(*optional.content)?);
                Format::Optional(optional)
            }
            Format::Plus(mut plus) => {
                plus.content = Box::new(self.bind(*plus.content)?);
                Format::Plus(plus)
            }
            Format::Star(mut star) => {
                star.content = Box::new(self.bind(*star.content)?);
                Format::Star(star)
            }
            other => other,
        })
    }

    fn bind_all(
        &self,
        formats: Vec<Format>,
        bind: impl Fn(&Self, Format) -> Result<Format>,
    ) -> Result<Vec<Format>> {
        formats
            .into_iter()
            .map(|format| bind(self, format))
            .collect()
    }

    fn bind_tags(
        &self,
        tags: Vec<TagFormat>,
        begin_marker: Option<&str>,
        keep_begin: bool,
    ) -> Result<Vec<TagFormat>> {
        tags.into_iter()
            .map(|tag| self.bind_tag(tag, begin_marker, keep_begin))
            .collect()
    }

    fn leading_marker(&self, text: &str) -> Option<&str> {
        self.markers.iter().copied().find(|m| text.starts_with(m))
    }

    fn trailing_marker(&self, text: &str) -> Option<&str> {
        self.markers.iter().copied().find(|m| text.ends_with(m))
    }

    /// Split a literal into const strings and marker tokens.
    fn split_literal(&self, mut text: &str) -> Vec<Format> {
        let mut parts = Vec::new();
        while !text.is_empty() {
            // The earliest occurrence wins; at the same index the longest marker wins.
            let best = self
                .markers
                .iter()
                .filter_map(|marker| text.find(marker).map(|index| (index, *marker)))
                .min_by_key(|(index, _)| *index);
            let Some((index, marker)) = best else {
                parts.push(Format::const_string(text));
                break;
            };
            if index > 0 {
                parts.push(Format::const_string(&text[..index]));
            }
            parts.push(token(marker));
            text = &text[index + marker.len()..];
        }
        parts
    }

    fn bind_const_string(&self, value: String) -> Format {
        let mut parts = self.split_literal(&value);
        match parts.len() {
            0 => Format::const_string(value),
            1 => parts.pop().unwrap(),
            _ => Format::sequence(parts),
        }
    }

    /// Move a leading begin marker and a trailing end marker onto tokens.
    ///
    /// The rest of `begin` and `end` is folded into the tag content so the accepted text stays
    /// the same.
    fn bind_tag(
        &self,
        tag: TagFormat,
        begin_marker: Option<&str>,
        keep_begin: bool,
    ) -> Result<TagFormat> {
        let TagFormat {
            mut begin,
            content,
            mut end,
        } = tag;
        let mut lead = Vec::new();
        let mut tail = Vec::new();
        if let TagBoundary::Text(text) = &begin
            && !keep_begin
            && let Some(marker) = begin_marker.or_else(|| self.leading_marker(text))
            && let Some(rest) = text.strip_prefix(marker)
        {
            lead = self.split_literal(rest);
            begin = TagBoundary::Token(TokenBoundary::new(marker));
        }
        if let EndBoundary::Texts(alternatives) = &end {
            let bound: Vec<&String> = alternatives
                .iter()
                .filter(|alt| self.trailing_marker(alt).is_some())
                .collect();
            if !bound.is_empty() {
                return Err(unbindable(format!(
                    "a tag whose `end` is a list of alternatives cannot be bound when one of them \
                     ends with a marker (got {bound:?}); use a single end string or leave that \
                     tag unbound"
                )));
            }
        }
        let mut end_is_token = false;
        if let EndBoundary::Text(text) = &end
            && let Some(marker) = self.trailing_marker(text)
        {
            tail = self.split_literal(&text[..text.len() - marker.len()]);
            end = EndBoundary::Token(TokenBoundary::new(marker));
            end_is_token = true;
        }
        let mut content = self.bind(*content)?;
        if end_is_token {
            // String-level free text does not see a token end; make it token-level so the end
            // token stops it.
            content = self.token_level(content)?;
        }
        if !lead.is_empty() || !tail.is_empty() {
            content = Format::Sequence(SequenceFormat {
                elements: lead.into_iter().chain([content]).chain(tail).collect(),
            });
        }
        Ok(TagFormat {
            begin,
            content: Box::new(content),
            end,
        })
    }

    /// Dispatch on the marker token when every trigger starts with it.
    fn bind_triggered_tags(&self, triggered: TriggeredTagsFormat) -> Result<Format> {
        let marker = self.markers.iter().copied().find(|marker| {
            triggered.triggers.iter().all(|t| t.starts_with(marker))
                && triggered.tags.iter().all(
                    |tag| matches!(&tag.begin, TagBoundary::Text(text) if text.starts_with(marker)),
                )
        });
        let Some(marker) = marker else {
            // The trigger is ordinary text; keep string dispatch but still bind the markers
            // inside the tags.
            return Ok(Format::TriggeredTags(TriggeredTagsFormat {
                tags: self.bind_tags(triggered.tags, None, true)?,
                ..triggered
            }));
        };
        Ok(Format::TokenTriggeredTags(TokenTriggeredTagsFormat {
            trigger_tokens: vec![TokenValue::Text(marker.to_string())],
            tags: self.bind_tags(triggered.tags, Some(marker), false)?,
            // Only markers are known to be single tokens; other string excludes cannot be
            // expressed at the token level and are dropped.
            exclude_tokens: triggered
                .excludes
                .into_iter()
                .filter(|text| self.markers.contains(&text.as_str()))
                .map(TokenValue::Text)
                .collect(),
            at_least_one: triggered.at_least_one,
            stop_after_first: triggered.stop_after_first,
        }))
    }

    /// Turn the free text under a token end into token-level formats.
    fn token_level(&self, format: Format) -> Result<Format> {
        Ok(match format {
            Format::AnyText(text) => {
                if text.max_chars.is_some()
                    || text
                        .excludes
                        .iter()
                        .any(|exclude| !self.markers.contains(&exclude.as_str()))
                {
                    return Err(unbindable(format!(
                        "the free text under a token end must be token-level: its excludes must \
                         be markers and max_chars unset (got {text:?})"
                    )));
                }
                Format::AnyTokens(AnyTokensFormat {
                    exclude_tokens: text.excludes.into_iter().map(TokenValue::Text).collect(),
                    max_tokens: text.max_tokens,
                })
            }
            Format::TriggeredTags(_) => {
                return Err(unbindable(
                    "string-triggered tags cannot stay under a token end; their triggers must \
                     start with a marker",
                ));
            }
            Format::Sequence(mut sequence) => {
                sequence.elements = self.bind_all(sequence.elements, Self::token_level)?;
                Format::Sequence(sequence)
            }
            Format::Or(mut or) => {
                or.elements = self.bind_all(or.elements, Self::token_level)?;
                Format::Or(or)
            }
            Format::Optional(mut optional) => {
                optional.content = Box::new(self.token_level(*optional.content)?);
                Format::Optional(optional)
            }
            Format::Plus(mut plus) => {
                plus.content = Box::new(self.token_level(*plus.content)?);
                Format::Plus(plus)
            }
            Format::Star(mut star) => {
                star.content = Box::new(self.token_level(*star.content)?);
                Format::Star(star)
            }
            Format::Repeat(_) | Format::Dispatch(_) => {
                return Err(unbindable(
                    "a token end cannot be put over a repeat or dispatch format; the free text \
                     inside it would stay string-level",
                ));
            }
            other => other,
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::format::AnyTextFormat;

    const MARKERS: &[&str] = &["<think>", "</think>", "<tool_call>", "</tool_call>"];

    fn bind(format: Format) -> Result<Format> {
        bind_marker_tokens(StructuralTag::new(format), MARKERS).map(|tag| tag.format)
    }

    #[test]
    fn literal_markers_split_into_tokens() {
        let bound = bind(Format::const_string("a<think>b</think>")).unwrap();
        assert_eq!(
            serde_json::to_value(bound).unwrap(),
            json!({"type": "sequence", "elements": [
                {"type": "const_string", "value": "a"},
                {"type": "token", "token": "<think>"},
                {"type": "const_string", "value": "b"},
                {"type": "token", "token": "</think>"},
            ]})
        );
        assert_eq!(
            bind(Format::const_string("plain")).unwrap(),
            Format::const_string("plain")
        );
    }

    #[test]
    fn token_end_turns_free_text_token_level() {
        let tag = Format::tag(
            "<think>",
            Format::any_text_excluding(&["<think>"]),
            "</think>\n",
        );
        // The end ends with "\n", not a marker, so the free text stays string-level.
        assert!(matches!(
            bind(tag).unwrap(),
            Format::Tag(TagFormat { ref content, .. }) if matches!(**content, Format::AnyText(_))
        ));

        let tag = Format::tag("", Format::any_text_excluding(&["<think>"]), "</think>");
        let Format::Tag(bound) = bind(tag).unwrap() else {
            panic!("expected a tag");
        };
        assert_eq!(
            bound.end,
            EndBoundary::Token(TokenBoundary::new("</think>"))
        );
        assert_eq!(
            *bound.content,
            Format::AnyTokens(AnyTokensFormat {
                exclude_tokens: vec![TokenValue::Text("<think>".to_string())],
                max_tokens: None,
            })
        );
    }

    #[test]
    fn unbindable_shapes_are_rejected() {
        let non_marker_exclude = Format::tag("", Format::any_text_excluding(&["x"]), "</think>");
        let char_budget = Format::tag(
            "",
            Format::AnyText(AnyTextFormat {
                max_chars: Some(1),
                ..Default::default()
            }),
            "</think>",
        );
        let repeat = Format::tag("", Format::repeat(Format::any_text(), 0, 2), "</tool_call>");
        let ends = Format::tag("<a>", Format::any_text(), vec!["</think>", "</a>"]);
        for format in [non_marker_exclude, char_budget, repeat, ends] {
            assert!(
                matches!(
                    bind(format.clone()),
                    Err(Error::UnbindableTokenMarkers { .. })
                ),
                "{format:?}"
            );
        }
    }
}
