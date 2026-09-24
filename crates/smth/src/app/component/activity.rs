// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Progress widget and retained state for a long-running background activity.

use std::future::Future;
use std::marker::PhantomData;
use std::time::Instant;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::Clear;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget as _;

use crate::app::component::loader;
use crate::app::component::spinner;
use crate::app::component::spinner::Spinner;

/// Spinner and trailing padding.
const SPINNER_REGION_WIDTH: u16 = 2;

/// Progress overlay at the bottom-right of its area, with one column of trailing padding.
pub(crate) struct Activity<V>(PhantomData<fn() -> V>);

/// Retained state for a background activity with owner-styled progress text.
pub(crate) struct State<V> {
    loader: loader::State<V>,
    label: Span<'static>,
    spinner: spinner::State,
}

impl<V> Activity<V> {
    /// Create an activity progress widget.
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<V> State<V>
where
    V: Send + 'static,
{
    /// Start an activity whose progress uses `label`, including its ratatui style.
    pub(crate) fn new<F>(label: impl Into<Span<'static>>, load: F) -> Self
    where
        F: Future<Output = anyhow::Result<V>> + Send + 'static,
    {
        Self {
            loader: loader::State::new(load),
            label: label.into(),
            spinner: spinner::State::new(),
        }
    }
}

impl<V> State<V> {
    /// Return whether the activity was loading when its state was last updated.
    pub(crate) fn is_loading(&self) -> bool {
        self.loader.is_loading()
    }

    /// Take the activity result once its background task has completed.
    ///
    /// Returns `None` while the task is still loading or after its result has already been taken.
    pub(crate) fn take(&mut self) -> Option<anyhow::Result<V>> {
        self.loader.take()
    }
}

impl<V> StatefulWidget for Activity<V> {
    type State = State<V>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        use ratatui::layout::Constraint as C;
        use ratatui::layout::Flex as F;
        use ratatui::layout::Layout as L;

        state.loader.poll();

        let now = Instant::now();
        let is_loading = state.is_loading();
        if !state.spinner.show(is_loading, now) {
            return;
        }

        let area = area.intersection(buf.area);
        if area.width < SPINNER_REGION_WIDTH || area.height == 0 {
            return;
        }

        let [row] = area.layout(&L::vertical([C::Length(1)]).flex(F::End));
        let [label, gap, spinner, padding] = row.layout(
            &L::horizontal([
                // Shrink the label before the spinner and its surrounding spaces.
                C::Max(state.label.width() as u16),
                C::Length(row.width.saturating_sub(SPINNER_REGION_WIDTH).min(1)),
                C::Length(1),
                C::Length(1),
            ])
            .flex(F::End),
        );

        Clear.render(label.union(gap).union(spinner).union(padding), buf);
        state.label.clone().render(label, buf);
        Spinner::new(is_loading).render_at(now, spinner, buf, &mut state.spinner);
    }
}
