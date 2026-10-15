// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Rendering for the session action footer.

use nucleo::Item;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Stylize as _;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::app::Mode;
use crate::app::actions::AvailableActions;
use crate::app::span::push_shortcut_span;
use crate::app::span::tag_span;
use crate::config::Alphabet;
use crate::model::delete;
use crate::model::session::Session;

/// Available actions for the selected session.
pub(super) struct Footer<'s> {
    delete: Option<&'s delete::Model>,
    matches: &'s [Item<'s, Session>],
    selected: Option<&'s Session>,
    mode: &'s Mode,
    alphabet: &'s Alphabet,
}

impl<'s> Footer<'s> {
    /// Create a footer from the current picker state.
    pub(super) fn new(
        delete: Option<&'s delete::Model>,
        matches: &'s [Item<'s, Session>],
        selected: Option<&'s Session>,
        mode: &'s Mode,
        alphabet: &'s Alphabet,
    ) -> Self {
        Self {
            delete,
            matches,
            selected,
            mode,
            alphabet,
        }
    }

    /// Render session actions beside an optional right-aligned modal cancellation hint.
    pub(super) fn draw(&self, f: &mut Frame<'_>, mut area: Rect, actions: &AvailableActions) {
        use ratatui::layout::Constraint as C;
        use ratatui::layout::Layout as L;

        if let Some(line) = self.mode(actions.deleting) {
            let [left, _, right] = area.layout(&L::horizontal([
                C::Fill(1),
                C::Max(1),
                C::Length(line.width() as u16),
            ]));

            f.render_widget(line, right);
            area = left;
        }

        let mut line = Line::default();
        let mut prefix = " ";

        if let Some(staged) = actions.delete {
            line += Span::raw(prefix).dim();
            push_shortcut_span(&mut line, "C-d");
            line += Span::raw(if staged { " unstage" } else { " delete" });
            prefix = ", ";
        }

        if actions.close {
            line += Span::raw(prefix).dim();
            push_shortcut_span(&mut line, "C-x");
            line += Span::raw(" close");
            prefix = ", ";
        }

        if let Some(flag) = actions.flag {
            line += Span::raw(prefix).dim();
            push_shortcut_span(&mut line, "C-f");
            line += Span::raw(if flag { " unflag" } else { " flag" });
            prefix = ", ";
        }

        if actions.tag {
            let tags = self.selected.and_then(Session::tags).unwrap_or_default();
            let text = tags.display(self.alphabet);

            line += Span::raw(prefix).dim();
            push_shortcut_span(&mut line, "C-t");
            line += Span::raw(" tags");
            if !text.is_empty() {
                line += Span::raw(": ");
                line += tag_span(text, false);
            }
        }

        f.render_widget(line, area);
    }

    /// Render the modal footer hint.
    fn mode(&self, deleting: bool) -> Option<Line<'static>> {
        let label = match self.mode {
            Mode::Normal if deleting => "delete ",
            Mode::Normal => return None,
            Mode::Onto(_) => "onto ",
            Mode::Sort(_) => "sort ",
            Mode::Tag(_) => "tags",
        };

        let mut line = Line::from(Span::raw(label).dim());

        if let Mode::Tag(state) = self.mode {
            let tags = state.initial | state.pending;
            let text = tags.display(self.alphabet);
            if !text.is_empty() {
                line += Span::raw(": ").dim();
                for (tag, glyph) in tags.iter().zip(text.chars()) {
                    line += Span::styled(glyph.to_string(), state.style(tag));
                }
            }

            line += Span::raw(" ");
            if state.batch {
                push_shortcut_span(&mut line, "C-t");
                line += Span::raw(" accept ");
            }
        } else if deleting && let Some(delete) = self.delete {
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
        Some(line)
    }
}
