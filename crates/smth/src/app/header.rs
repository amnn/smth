// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Rendering for the header bar.

use std::collections::BTreeMap;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Stylize as _;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::app::agent;
use crate::app::highlight::Highlight;
use crate::app::span::push_repo_path_spans;
use crate::app::span::push_shortcut_span;
use crate::model::agent::AgentState;
use crate::model::session::Repo;

/// Header bar component showing counts and repo context.
pub(super) struct Header<'r> {
    /// Lifecycle state counts across all discovered live sessions.
    agents: BTreeMap<AgentState, usize>,
    found: usize,
    repo: Option<&'r Repo>,
    total: usize,
}

impl<'r> Header<'r> {
    /// Create a header from the current picker state.
    pub(super) fn new(
        agents: BTreeMap<AgentState, usize>,
        found: usize,
        repo: Option<&'r Repo>,
        total: usize,
    ) -> Self {
        Self {
            agents,
            found,
            repo,
            total,
        }
    }

    /// Render the header bar into `area`.
    pub(super) fn draw(&self, f: &mut Frame<'_>, area: Rect) {
        let width = if self.total == 0 {
            1
        } else {
            self.total.ilog10() as usize + 1
        };

        let mut line = Line::default();

        line += Span::raw(format!(" {:>width$}", self.found));
        line += Span::raw(format!("/{} | ", self.total)).dim();
        push_shortcut_span(&mut line, "C-r");
        line += Span::raw(" repo: ");

        if let Some(repo) = self.repo {
            push_repo_path_spans(&mut line, repo.path(), &mut Highlight::none());
            line += Span::raw(", ").dim();
            push_shortcut_span(&mut line, "C-o");
            line += Span::raw(" onto: ");
            line += Span::raw(repo.revision().to_owned()).dim();
        } else {
            line += Span::raw("none").dim();
        }

        f.render_widget(line, area);
        f.render_widget(agent::summary(&self.agents, false), area);
    }
}
