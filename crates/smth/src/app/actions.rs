// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Retained action eligibility for picker input and footer rendering.

use crate::app::Mode;
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
    /// Whether tag editing can begin for the selected session.
    pub(super) tag: bool,
    /// Whether staged deletion controls are available without an active editor or mutation.
    pub(super) deleting: bool,
}

impl AvailableActions {
    /// Derive eligibility after rendering updates selection and activity state.
    pub(super) fn new(
        selected: Option<&Session>,
        loading: bool,
        mode: &Mode,
        delete: Option<&delete::Model>,
    ) -> Self {
        let session_actions = !loading && matches!(mode, Mode::Normal);
        let live = selected.is_some_and(|s| s.is_live());
        Self {
            exit: !loading,
            switch: session_actions && selected.is_some(),
            create: session_actions && selected.is_some() && !live,
            close: session_actions && live,
            delete: selected
                .filter(|s| session_actions && s.can_delete())
                .map(|s| delete.is_some_and(|delete| delete.contains(s))),
            flag: selected.and_then(|s| s.flag()).filter(|_| session_actions),
            tag: session_actions && delete.is_none() && live,
            deleting: session_actions && delete.is_some(),
        }
    }
}
