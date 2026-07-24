// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Stateful component and retained task state for values loaded in the background.

use std::future::Future;
use std::marker::PhantomData;

use anyhow::anyhow;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget as _;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;
use tokio_util::task::AbortOnDropHandle;

/// View renderer for a background-loaded stateful view of type `V`.
pub(crate) struct Loader<'s, V, S> {
    state: &'s mut S,
    _view: PhantomData<fn() -> V>,
}

/// Retained loading state for a value produced by a background task.
pub(crate) struct State<V> {
    status: Status<V>,
}

/// Current result state for a value produced by a background task.
pub(crate) enum Status<V> {
    /// The background task is still running.
    Loading(Task<V>),

    /// The background task failed or ended before returning a value.
    Error(anyhow::Error),

    /// The value has loaded. `done` is true if the value has been acknowledged.
    Loaded { view: V, done: bool },

    /// The completed result has been taken by the loader's owner.
    Finished,
}

/// In-flight background task state while a value is loading.
pub(crate) struct Task<V> {
    rx: oneshot::Receiver<anyhow::Result<V>>,
    _h: AbortOnDropHandle<()>,
}

impl<'s, V, S> Loader<'s, V, S> {
    /// Create a renderer for a background-loaded view that uses `state` for the loaded view.
    pub(crate) fn new(state: &'s mut S) -> Self {
        Self {
            state,
            _view: PhantomData,
        }
    }
}

impl<V> State<V>
where
    V: Send + 'static,
{
    /// Start producing a value in the background.
    pub(crate) fn new<F>(load: F) -> Self
    where
        F: Future<Output = anyhow::Result<V>> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let worker = tokio::task::spawn(async move {
            let _ = tx.send(load.await);
        });

        Self {
            status: Status::Loading(Task {
                rx,
                _h: AbortOnDropHandle::new(worker),
            }),
        }
    }
}

impl<V> State<V> {
    /// Mark a loaded value as handled without changing its rendered output.
    pub(crate) fn finish(&mut self) -> bool {
        self.poll();

        use Status as S;
        match &mut self.status {
            S::Loading(_) | S::Error(_) | S::Finished => false,
            S::Loaded { done: true, .. } => false,
            S::Loaded { done, .. } => {
                *done = true;
                true
            }
        }
    }

    /// Return whether the retained background task state is still loading.
    pub(crate) fn is_loading(&self) -> bool {
        matches!(self.status, Status::Loading(_))
    }

    /// Return the value if it has loaded and has not yet been marked handled.
    pub(crate) fn pending(&self) -> Option<&V> {
        if let Status::Loaded { view, done: false } = &self.status {
            Some(view)
        } else {
            None
        }
    }

    /// Take the completed result, leaving this state finished.
    ///
    /// Requires the loader to have been polled previously to update its status. Returns `None`
    /// while the task is still loading or after its result has already been taken.
    pub(crate) fn take(&mut self) -> Option<anyhow::Result<V>> {
        match std::mem::replace(&mut self.status, Status::Finished) {
            Status::Loading(task) => {
                self.status = Status::Loading(task);
                None
            }
            Status::Error(err) => Some(Err(err)),
            Status::Loaded { view, .. } => Some(Ok(view)),
            Status::Finished => None,
        }
    }

    /// Return the loaded value, including after it has been marked handled.
    pub(crate) fn view(&self) -> Option<&V> {
        if let Status::Loaded { view, .. } = &self.status {
            Some(view)
        } else {
            None
        }
    }

    /// Poll the background task and retain any completed result.
    pub(super) fn poll(&mut self) {
        match &mut self.status {
            Status::Loading(task) => match task.rx.try_recv() {
                Ok(Ok(view)) => self.status = Status::Loaded { view, done: false },
                Ok(Err(err)) => self.status = Status::Error(err),
                Err(TryRecvError::Empty) => { /* nop */ }
                Err(TryRecvError::Closed) => {
                    self.status = Status::Error(anyhow!("background task ended without a value"))
                }
            },
            Status::Error(_) | Status::Loaded { .. } | Status::Finished => { /* nop */ }
        }
    }
}

impl<V, S> StatefulWidget for Loader<'_, V, S>
where
    for<'a> &'a V: StatefulWidget<State = S>,
{
    type State = State<V>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.poll();

        match &state.status {
            Status::Loading(_) => "Loading...".render(area, buf),
            Status::Error(err) => format!("Error: {err}").render(area, buf),
            Status::Loaded { view, .. } => {
                view.render(area, buf, self.state);
            }
            Status::Finished => {}
        }
    }
}
