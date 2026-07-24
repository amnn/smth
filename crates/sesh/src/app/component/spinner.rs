// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Widget for representing an animated spinner.

use std::time::Duration;
use std::time::Instant;

use ratatui::prelude::Buffer;
use ratatui::prelude::Rect;
use ratatui::widgets::StatefulWidget;

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
const FRAME_DURATION: Duration = Duration::from_millis(100);

/// An animated spinner.
pub(crate) struct Spinner(bool);

/// The state of the spinner. This remembers when the animation started. Animation duration and
/// therefore frame calculation is based on this start time.
pub(crate) struct State {
    start: Instant,
}

impl Spinner {
    /// Create a spinner, enabled only when `enabled` is true.
    pub(crate) fn new(enabled: bool) -> Self {
        Self(enabled)
    }
}

impl State {
    /// Create a fresh spinner state, for an inactive spinner.
    pub(crate) fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Pick the frame for instant `now` from `frames`.
    ///
    /// `now` must not precede the state start, `frame_duration` must be at least one millisecond,
    /// and `frames` must not be empty.
    pub(crate) fn frame<'f>(
        &self,
        now: Instant,
        frame_duration: Duration,
        frames: &[&'f str],
    ) -> &'f str {
        let delta = now - self.start;
        let index = (delta.as_millis() / frame_duration.as_millis()) as usize;
        frames[index % frames.len()]
    }
}

impl StatefulWidget for Spinner {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }

        let Some(cell) = buf.cell_mut(area) else {
            return;
        };

        if !self.0 {
            cell.set_symbol(" ");
            return;
        }

        cell.set_symbol(state.frame(Instant::now(), FRAME_DURATION, FRAMES));
    }
}
