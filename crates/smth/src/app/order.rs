// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Compact ordering indicator and expanded header controls.

use ratatui::style::Style;
use ratatui::style::Stylize as _;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::model::order::Direction;
use crate::model::order::Key;
use crate::model::order::Order;

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
