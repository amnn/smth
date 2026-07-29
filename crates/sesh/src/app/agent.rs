// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Compact lifecycle-state summary shared by session rows and their header.

use std::collections::BTreeMap;

use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::model::agent::AgentState;

const STATES: [AgentState; 5] = [
    AgentState::Waiting,
    AgentState::Failed,
    AgentState::Succeeded,
    AgentState::Running,
    AgentState::Idle,
];

/// Render a right-aligned lifecycle summary, or an empty line when no agents are tracked.
///
/// States appear as waiting, failed, succeeded, running, then idle. When `selected`, state colours
/// are pre-inverted so the surrounding row inversion keeps them in the foreground.
pub(super) fn summary(summary: &BTreeMap<AgentState, usize>, selected: bool) -> Line<'static> {
    let mut line = Line::default();

    let mut separator = " ";
    for state in STATES {
        let Some(&count) = summary.get(&state) else {
            continue;
        };

        line += Span::raw(separator);
        separator = " · ";

        let glyph = glyph(state);
        let text = if count == 1 {
            glyph.to_owned()
        } else {
            format!("{glyph} {count}")
        };

        line += Span::styled(text, style(state, selected));
    }

    if separator != " " {
        line += Span::raw(" ");
    }

    for span in &mut line.spans {
        span.style = span.style.dim();
    }

    line.right_aligned()
}

/// Return the fixed glyph for an agent lifecycle state.
fn glyph(state: AgentState) -> &'static str {
    match state {
        AgentState::Waiting => "⏸",
        AgentState::Failed => "×",
        AgentState::Succeeded => "✔",
        AgentState::Running => "▶",
        AgentState::Idle => "○",
    }
}

/// Return a lifecycle-state style, pre-inverted when rendered on a selected row.
fn style(state: AgentState, selected: bool) -> Style {
    let colour = match state {
        AgentState::Waiting => Color::LightYellow,
        AgentState::Failed => Color::LightRed,
        AgentState::Succeeded => Color::LightGreen,
        AgentState::Running => Color::LightCyan,
        AgentState::Idle => return Style::new(),
    };

    if selected {
        Style::new().bg(colour)
    } else {
        Style::new().fg(colour)
    }
}
