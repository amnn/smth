// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Transient match-counter overlay for onto revision navigation.

use std::time::Duration;
use std::time::Instant;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget as _;

/// How long the position remains visible after jumping to a fuzzy match.
const VISIBLE_FOR: Duration = Duration::from_secs(1);

/// Stateless widget for transient fuzzy-match position feedback.
pub(super) struct MatchCounter;

/// One fuzzy-match jump and the time at which it occurred.
///
/// `current` is a valid one-based position in `1..=total`.
pub(super) struct State {
    /// One-based position of the selected commit among matching commits.
    current: usize,
    /// Number of matching commits in the navigation cycle.
    total: usize,
    /// Time at which match navigation selected this commit.
    shown_at: Instant,
}

impl MatchCounter {
    /// Render the position when it is still visible at `now`.
    ///
    /// This component is designed to be overdrawn over an existing view that potentially contains a
    /// scrollbar. It renders in the top right, reserving the furthest right column for that
    /// scrollbar.
    fn render_at(self, now: Instant, area: Rect, buf: &mut Buffer, state: &State) {
        if now.saturating_duration_since(state.shown_at) >= VISIBLE_FOR {
            return;
        }

        let area = Rect {
            width: area.width.saturating_sub(1),
            ..area
        };

        let text = format!(" {}/{} ", state.current, state.total);
        let line = Line::from(Span::styled(text, Style::reset().reversed())).right_aligned();
        line.render(area, buf);
    }
}

impl State {
    /// Record a fuzzy-match jump for transient display.
    pub(super) fn new(current: usize, total: usize) -> Self {
        Self {
            current,
            total,
            shown_at: Instant::now(),
        }
    }
}

impl StatefulWidget for MatchCounter {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_at(Instant::now(), area, buf, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_at_timeout() {
        let shown_at = Instant::now();
        let state = State {
            current: 5,
            total: 10,
            shown_at,
        };
        let area = Rect::new(0, 0, 8, 1);
        let mut buf = Buffer::empty(area);

        MatchCounter.render_at(shown_at + VISIBLE_FOR, area, &mut buf, &state);

        let rendered: String = buf.content().iter().map(|cell| cell.symbol()).collect();
        assert_eq!(rendered, "        ");
    }
}
