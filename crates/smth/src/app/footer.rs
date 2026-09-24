// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Rendering for the session action footer.

use nucleo::Item;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Stylize as _;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::app::sessions;
use crate::app::span::push_shortcut_span;
use crate::model::delete;
use crate::model::session::Session;

/// Available actions for the selected session.
pub(super) struct Footer<'s> {
    sessions: &'s sessions::State,
    delete: Option<&'s delete::Model>,
    matches: &'s [Item<'s, Session>],
}

impl<'s> Footer<'s> {
    /// Create a footer from the current picker state.
    pub(super) fn new(
        sessions: &'s sessions::State,
        delete: Option<&'s delete::Model>,
        matches: &'s [Item<'s, Session>],
    ) -> Self {
        Self {
            sessions,
            delete,
            matches,
        }
    }

    /// Render the available session actions.
    pub(super) fn draw(&self, f: &mut Frame<'_>, area: Rect) {
        let mut line = Line::default();
        let mut prefix = " ";
        let selected = self.sessions.selected();

        if let Some(delete) = self.delete {
            let count = delete.pending();
            let hidden = delete.hidden(self.matches.iter().map(|item| item.data));
            line += Span::raw(prefix).dim();
            push_shortcut_span(&mut line, "C-y");
            let noun = if count == 1 { "session" } else { "sessions" };
            line += Span::raw(format!(" delete {count} {noun}"))
                .light_red()
                .bold();
            if hidden > 0 {
                line += Span::raw(format!(" ({hidden} hidden)")).light_red().bold();
            }
            prefix = ", ";
        }

        if let Some(session) = selected.filter(|s| s.can_delete()) {
            line += Span::raw(prefix).dim();
            push_shortcut_span(&mut line, "C-d");
            line += Span::raw(
                if self.delete.is_some_and(|delete| delete.contains(session)) {
                    " unstage"
                } else {
                    " delete"
                },
            );
            prefix = ", ";
        }

        if selected.is_some_and(|s| s.is_live()) {
            line += Span::raw(prefix).dim();
            push_shortcut_span(&mut line, "C-x");
            line += Span::raw(" close");
            prefix = ", ";
        }

        if let Some(flag) = selected.and_then(|s| s.flag()) {
            line += Span::raw(prefix).dim();
            push_shortcut_span(&mut line, "C-f");
            line += Span::raw(if flag { " unflag" } else { " flag" });
        }

        f.render_widget(line, area);
    }
}
