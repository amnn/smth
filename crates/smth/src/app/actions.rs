// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Retained action eligibility for picker input and footer rendering.

use crate::app::sessions;
use crate::model::delete;
use crate::model::session::Session;

/// Actions available from the state used by the last render pass.
#[derive(Default)]
pub(super) struct AvailableActions {
    /// Whether the picker can exit without selecting a session.
    pub(super) exit: bool,
    /// Whether the selected session can be switched to, creating it first if needed.
    pub(super) switch: bool,
    /// Whether the selected session can be created without switching to it.
    pub(super) create: bool,
    /// Whether the selected live session can be closed without deleting its workspace.
    pub(super) close: bool,
    /// Current staged state, or `None` when deletion cannot be toggled.
    pub(super) delete: Option<bool>,
    /// Current flag state, or `None` when the flag cannot be toggled.
    pub(super) flag: Option<bool>,
    /// Modal actions taking precedence over session actions, or `None` when none are available.
    pub(super) mode: Option<Mode>,
}

/// Modal actions taking precedence over session actions.
#[derive(Clone, Copy)]
pub(super) enum Mode {
    Delete,
    Onto,
}

impl AvailableActions {
    /// Derive eligibility after rendering updates selection and activity state.
    pub(super) fn new(
        sessions: &sessions::State,
        loading: bool,
        onto: bool,
        delete: Option<&delete::Model>,
    ) -> Self {
        let session_actions = !loading && !onto;
        let selected = sessions.selected();
        Self {
            exit: !loading,
            switch: session_actions && sessions.selected().is_some(),
            create: session_actions && sessions.selected().is_some() && !sessions.is_live(),
            close: session_actions && sessions.is_live(),
            delete: selected
                .filter(|_| session_actions && sessions.can_delete())
                .map(|session| delete.is_some_and(|delete| delete.contains(session))),
            flag: selected.and_then(Session::flag).filter(|_| session_actions),
            mode: if onto {
                Some(Mode::Onto)
            } else if !loading && delete.is_some() {
                Some(Mode::Delete)
            } else {
                None
            },
        }
    }
}
