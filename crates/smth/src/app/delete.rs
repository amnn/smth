// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Actions and modal key handling for persisted workspace deletions.

use std::path::PathBuf;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

/// A request to stage, confirm, or cancel workspace deletions.
pub(super) enum Action {
    /// Delete staged workspace checkouts and close their discovered live sessions.
    Accept,
    /// Clear persisted deletion markers.
    Cancel,
    /// Toggle the selected checkout's persisted deletion marker.
    Toggle(PathBuf),
}

/// Handle modal confirmation and cancellation keys when pending deletions exist.
pub(super) fn handle_key(key: KeyEvent) -> Option<Action> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char('y') if ctrl => Some(Action::Accept),
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Char('c' | 'g') if ctrl => Some(Action::Cancel),
        _ => None,
    }
}
