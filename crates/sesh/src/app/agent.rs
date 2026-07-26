// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Compact lifecycle-state summary shared by session rows and their header.

use std::collections::BTreeMap;

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
/// States appear as waiting, failed, succeeded, running, then idle.
pub(super) fn summary(summary: &BTreeMap<AgentState, usize>) -> Line<'static> {
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

        line += Span::styled(text, style(state));
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

/// Return the fixed style for an agent lifecycle state.
fn style(state: AgentState) -> Style {
    match state {
        AgentState::Waiting => Style::new().light_yellow(),
        AgentState::Failed => Style::new().light_red(),
        AgentState::Succeeded => Style::new().light_green(),
        AgentState::Running => Style::new().light_cyan(),
        AgentState::Idle => Style::new(),
    }
}
