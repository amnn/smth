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
use crate::app::sort;
use crate::app::span::push_repo_path_spans;
use crate::app::span::push_shortcut_span;
use crate::model::agent::AgentState;
use crate::model::order::Order;
use crate::model::session::Repo;

/// Header bar component showing counts and repo context.
pub(super) struct Header<'r> {
    /// Lifecycle state counts across all discovered live sessions.
    agents: &'r BTreeMap<AgentState, usize>,
    found: usize,
    repo: Option<&'r Repo>,
    total: usize,
    order: &'r Order,
    selected_order: Option<usize>,
}

impl<'r> Header<'r> {
    /// Create a header from the current picker state.
    pub(super) fn new(
        agents: &'r BTreeMap<AgentState, usize>,
        found: usize,
        repo: Option<&'r Repo>,
        total: usize,
        order: &'r Order,
        selected_order: Option<usize>,
    ) -> Self {
        Self {
            agents,
            found,
            repo,
            total,
            order,
            selected_order,
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

        push_shortcut_span(&mut line, "C-s");
        if let Some(selected) = self.selected_order {
            line.extend(sort::expanded(self.order, selected));
        } else {
            line.extend(sort::summary(self.order));
        }

        line += Span::raw(" | ").dim();
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

        // By default the agent summary takes precedence over the rest of the header, but if we are
        // in sort mode, the header takes precedence to ensure the sort UI is not obscured.
        if self.selected_order.is_none() {
            f.render_widget(line, area);
            f.render_widget(agent::summary(self.agents, false), area);
        } else {
            f.render_widget(agent::summary(self.agents, false), area);
            f.render_widget(line, area);
        }
    }
}
