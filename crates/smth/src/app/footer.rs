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

/// Active mode whose confirmation or cancellation controls appear in the footer.
#[derive(Clone, Copy)]
pub(super) enum Mode {
    Delete,
    Onto,
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

    /// Render session actions beside an optional right-aligned modal cancellation hint.
    pub(super) fn draw(&self, f: &mut Frame<'_>, mut area: Rect, mode: Option<Mode>) {
        use ratatui::layout::Constraint as C;
        use ratatui::layout::Layout as L;

        if let Some(mode) = mode {
            let line = self.mode(mode);
            let [left, _, right] = area.layout(&L::horizontal([
                C::Fill(1),
                C::Max(1),
                C::Length(line.width() as u16),
            ]));

            f.render_widget(line, right);
            area = left;
        }

        if matches!(mode, Some(Mode::Onto)) {
            return;
        }

        let mut line = Line::default();
        let mut prefix = " ";
        let selected = self.sessions.selected();

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

    /// Render the modal footer hint.
    fn mode(&self, mode: Mode) -> Line<'static> {
        let label = match mode {
            Mode::Delete => "delete",
            Mode::Onto => "onto",
        };

        let mut line = Line::from(Span::raw(format!("{label} ")).dim());

        if let Mode::Delete = mode
            && let Some(delete) = self.delete
        {
            push_shortcut_span(&mut line, "C-y");

            let count = delete.pending();
            let hidden = delete.hidden(self.matches.iter().map(|item| item.data));
            let s = if count == 1 { "" } else { "s" };
            line += Span::raw(format!(" {count} session{s} "))
                .light_red()
                .bold();

            if hidden > 0 {
                line += Span::raw(format!("({hidden} hidden) ")).light_red().bold();
            }
        }

        push_shortcut_span(&mut line, "C-g");
        line += Span::raw(" cancel");
        line
    }
}
