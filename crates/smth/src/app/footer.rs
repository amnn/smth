// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Rendering for the session action footer.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Stylize as _;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::app::span::push_shortcut_span;
use crate::model::session::Session;

/// Available actions for the selected session.
pub(super) struct Footer<'s> {
    confirm_delete: bool,
    selected: Option<&'s Session>,
}

impl<'s> Footer<'s> {
    /// Create a footer from the current picker state.
    pub(super) fn new(confirm_delete: bool, selected: Option<&'s Session>) -> Self {
        Self {
            confirm_delete,
            selected,
        }
    }

    /// Render the session actions into `area`.
    pub(super) fn draw(&self, f: &mut Frame<'_>, area: Rect) {
        let mut line = Line::default();
        let mut prefix = " ";

        if self.confirm_delete {
            line += Span::raw(prefix).dim();
            push_shortcut_span(&mut line, "C-y");
            line += Span::raw(" confirm").light_red().bold();
            prefix = ", ";
        } else if self.selected.is_some_and(|s| s.can_delete()) {
            line += Span::raw(prefix).dim();
            push_shortcut_span(&mut line, "C-d");
            line += Span::raw(" delete");
            prefix = ", ";
        }

        if self.selected.is_some_and(|s| s.is_live()) {
            line += Span::raw(prefix).dim();
            push_shortcut_span(&mut line, "C-x");
            line += Span::raw(" close");
            prefix = ", ";
        }

        if let Some(flag) = self.selected.and_then(|s| s.flag()) {
            line += Span::raw(prefix).dim();
            push_shortcut_span(&mut line, "C-f");
            line += Span::raw(if flag { " unflag" } else { " flag" });
        }

        f.render_widget(line, area);
    }
}
