// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Fuzzy-match highlighting helpers.

use std::iter::Peekable;
use std::vec;

use ratatui::style::Style;
use ratatui::text::Span;

/// Stateful highlighter for character indices matched by the active fuzzy query.
pub(crate) struct Highlight<F> {
    indices: Peekable<vec::IntoIter<u32>>,
    processed: u32,
    transform: F,
}

impl Highlight<fn(Style) -> Style> {
    /// An instance that applies no highlighting (spans are returned unchanged).
    pub(crate) fn none() -> Self {
        Self::new(Vec::new(), |s| s)
    }
}

impl<F> Highlight<F>
where
    F: Fn(Style) -> Style,
{
    /// Construct a highlighter from character positions and a matched-style transform.
    pub(crate) fn new(mut indices: Vec<u32>, transform: F) -> Self {
        indices.sort_unstable();
        indices.dedup();

        Self {
            indices: indices.into_iter().peekable(),
            processed: 0,
            transform,
        }
    }

    /// Overlay fuzzy match highlighting onto `span`, preserving its style when unmatched.
    pub(crate) fn highlight(&mut self, span: Span<'static>) -> Vec<Span<'static>> {
        let mut output = Vec::new();
        let mut chars = span.content.chars().peekable();

        while chars.peek().is_some() {
            let next = self.indices.peek().copied().unwrap_or(u32::MAX);
            debug_assert!(next >= self.processed, "out-of-order highlight index");

            // Consume all non-highlighted characters before the next matched character.
            let mut plain = String::new();
            while let Some(&ch) = chars.peek()
                && self.processed < next
            {
                plain.push(ch);
                chars.next();
                self.processed += 1;
            }

            if !plain.is_empty() {
                output.push(Span::styled(plain, span.style));
            }

            // Gather the prefix of highlighted characters and apply the configured style.
            let mut highlighted = String::new();
            while let (Some(&ch), Some(&ix)) = (chars.peek(), self.indices.peek())
                && ix == self.processed
            {
                highlighted.push(ch);
                chars.next();
                self.indices.next();
                self.processed += 1;
            }

            if !highlighted.is_empty() {
                output.push(Span::styled(highlighted, (self.transform)(span.style)));
            }
        }

        output
    }

    /// Advance to an absolute character offset in the searchable text without rendering it.
    /// Earlier or equal offsets are no-ops. Matches before the new offset are discarded,
    /// while a match at the offset is retained.
    pub(crate) fn skip_to(&mut self, offset: u32) {
        if offset <= self.processed {
            return;
        }

        self.processed = offset;
        while let Some(index) = self.indices.peek()
            && *index < self.processed
        {
            self.indices.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::Stylize as _;

    use super::*;

    #[test]
    fn highlight_match_indices_replaces_existing_span_styles() {
        let style = Style::new().blue().bold();
        let mut hl = Highlight::new(vec![2, 1, 1], |_| style);
        let mut spans = hl.highlight(Span::raw("ab").dim());
        spans.extend(hl.highlight(Span::raw("cd").green()));

        assert_eq!(
            spans,
            vec![
                Span::raw("a").dim(),
                Span::styled("b", style),
                Span::styled("c", style),
                Span::raw("d").green(),
            ]
        );
    }

    #[test]
    fn skip_to_advances_character_indices_and_discards_skipped_matches() {
        let style = Style::new().blue().bold();
        let mut hl = Highlight::new(vec![0, 1, 2, 3, 5], |_| style);
        assert_eq!(
            hl.highlight(Span::raw("界")),
            vec![Span::styled("界", style)]
        );

        hl.skip_to(3);
        assert_eq!(
            hl.highlight(Span::raw("abc")),
            vec![
                Span::styled("a", style),
                Span::raw("b"),
                Span::styled("c", style)
            ]
        );
    }

    #[test]
    fn skip_to_earlier_or_equal_offsets_preserves_position_and_matches() {
        let style = Style::new().blue().bold();
        let mut hl = Highlight::new(vec![2, 3], |_| style);
        hl.skip_to(2);
        hl.skip_to(1);
        hl.skip_to(2);
        assert_eq!(
            hl.highlight(Span::raw("ab")),
            vec![Span::styled("ab", style)]
        );
    }

    #[test]
    fn skip_to_zero_preserves_matches_and_skip_past_end_exhausts_them() {
        let style = Style::new().blue().bold();
        let mut hl = Highlight::new(vec![0, 2], |_| style);
        hl.skip_to(0);
        assert_eq!(hl.highlight(Span::raw("a")), vec![Span::styled("a", style)]);

        hl.skip_to(10);
        assert_eq!(hl.highlight(Span::raw("bc")), vec![Span::raw("bc")]);
    }

    #[test]
    fn transform_match_indices_preserves_existing_span_styles() {
        let ul = Style::new().underlined();
        let mut hl = Highlight::new(vec![1], |style| style.patch(ul));
        let spans = hl.highlight(Span::raw("ab").green());

        assert_eq!(
            spans,
            vec![Span::raw("a").green(), Span::raw("b").green().underlined()]
        );
    }
}
