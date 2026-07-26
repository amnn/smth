// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Session-list row widget.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

/// One visible session row with scrollable text, an optional sticky sigil, and an overlay.
#[derive(Default)]
pub(crate) struct Row {
    line: Line<'static>,
    overlay: Option<Line<'static>>,
    right_margin: Option<u16>,
    sigil: Option<Span<'static>>,
}

impl Row {
    /// Create an empty spacer row.
    pub(crate) fn empty() -> Self {
        Self::default()
    }

    /// Create a row from scrollable line content.
    pub(crate) fn new(line: Line<'static>) -> Self {
        Self {
            line,
            overlay: None,
            right_margin: None,
            sigil: None,
        }
    }

    /// Set content to overdraw on top of the scrollable line.
    pub(crate) fn with_overlay(mut self, overlay: Line<'static>) -> Self {
        self.overlay = Some(overlay);
        self
    }

    /// Set the rightmost scrollable content column that should stay visible.
    pub(crate) fn with_right_margin(mut self, right_margin: Option<u16>) -> Self {
        self.right_margin = right_margin;
        self
    }

    /// Set a sticky sigil to render in the first cell before the scrollable content.
    pub(crate) fn with_sigil(mut self, sigil: Span<'static>) -> Self {
        self.sigil = Some(sigil);
        self
    }
}

impl Widget for Row {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::layout::Constraint as C;
        use ratatui::layout::Direction as D;
        use ratatui::layout::Layout as L;

        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }

        let [prefix, _, rest] = area.layout(&L::new(
            D::Horizontal,
            [C::Length(1), C::Length(1), C::Min(0)],
        ));

        if let Some(sigil) = self.sigil {
            sigil.render(prefix, buf);
        }

        let left_margin = self
            .right_margin
            .unwrap_or_default()
            .saturating_sub(rest.width);

        Paragraph::new(self.line)
            .scroll((0, left_margin))
            .render(rest, buf);

        if let Some(overlay) = self.overlay {
            overlay.render(rest, buf);
        }
    }
}
