// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Compact ordering indicator and expanded header controls.

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::style::Style;
use ratatui::style::Stylize as _;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::model::order::Direction;
use crate::model::order::Key;
use crate::model::order::Order;

/// Pending run-local ordering edit, previewed without changing the model until accepted.
pub(super) struct State {
    pub(super) pending: Order,
    pub(super) selected: usize,
}

/// Terminal editor actions; other keys only update pending state.
pub(super) enum Action {
    Accept,
    Cancel,
}

impl State {
    /// Start at the first enabled comparator, or the first comparator when all are disabled.
    pub(super) fn new(order: &Order) -> Self {
        Self {
            pending: order.clone(),
            selected: order
                .entries()
                .iter()
                .position(|entry| entry.direction.is_some())
                .unwrap_or(0),
        }
    }

    /// Handle only ordering keys, leaving query edits and session actions inert.
    pub(super) fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        use KeyCode as KC;
        use KeyModifiers as KM;

        const ALT: KM = KM::ALT;
        const CTRL: KM = KM::CONTROL;

        let alt = key.modifiers.contains(ALT);
        let ctrl = key.modifiers.contains(CTRL);
        let count = self.pending.entries().len();

        match key.code {
            KC::Char('s') if alt => return Some(Action::Accept),
            KC::Char('g') if ctrl => return Some(Action::Cancel),

            KC::Tab => self.selected = (self.selected + 1) % count,
            KC::BackTab => self.selected = (self.selected + count - 1) % count,

            KC::Up => self.pending.toggle(self.selected, Direction::Descending),
            KC::Down => self.pending.toggle(self.selected, Direction::Ascending),

            KC::Left => self.selected = self.pending.move_left(self.selected),
            KC::Right => self.selected = self.pending.move_right(self.selected),

            KC::Char('l') => self.activate(Key::Live),
            KC::Char('r') => self.activate(Key::Recency),
            KC::Char('n') => self.activate(Key::Name),
            KC::Char('a') => self.activate(Key::Attention),

            _ => {}
        }

        None
    }

    fn activate(&mut self, key: Key) {
        let Some(index) = self.pending.entries().iter().position(|e| e.key == key) else {
            return;
        };

        if self.selected == index {
            self.pending.cycle(index);
        } else {
            self.selected = index;
        }
    }
}

/// Render a summary of the ordering criteria. Disabled criteria are omitted, enabled criteria are
/// shown as initials, in the order they are applied, and with direction arrows.
pub(super) fn summary(order: &Order) -> Line<'static> {
    let mut empty = true;
    let mut line = Line::default();
    for entry in order.entries() {
        let Some(direction) = entry.direction else {
            continue;
        };

        empty = false;
        line += Span::raw(" ");
        line += dir(direction);

        let (initial, _) = label(entry.key);
        line += Span::raw(initial);
    }

    if empty {
        line += Span::raw(" order");
    }

    line
}

/// Renders the full ordering control line. Unlike the summary, all criteria are shown with their
/// full labels.
pub(super) fn expanded(order: &Order, selected: usize) -> Line<'static> {
    let mut line = Line::default();

    if selected == 0 {
        line += Span::raw(" ◂").bold();
    } else {
        line += Span::raw("  ");
    }

    for (i, entry) in order.entries().iter().enumerate() {
        line += Span::raw(" ");
        if let Some(direction) = entry.direction {
            line += dir(direction);
        }

        let (initial, rest) = label(entry.key);
        line += Span::styled(initial, Style::new().yellow().bold());

        if selected == i {
            line += Span::raw(rest).underlined();
        } else {
            line += Span::raw(rest);
        };

        if selected == i {
            line += Span::raw(" ▸").bold();
        } else if selected == i + 1 {
            line += Span::raw(" ◂").bold();
        } else if i + 1 == order.entries().len() {
            line += Span::raw("  ");
        } else {
            line += Span::raw(" ·").dim();
        }
    }

    line
}

fn dir(direction: Direction) -> Span<'static> {
    Span::raw(match direction {
        Direction::Ascending => "↓",
        Direction::Descending => "↑",
    })
}

fn label(key: Key) -> (&'static str, &'static str) {
    match key {
        Key::Live => ("l", "ive"),
        Key::Recency => ("r", "ecency"),
        Key::Name => ("n", "ame"),
        Key::Attention => ("a", "ttention"),
    }
}
