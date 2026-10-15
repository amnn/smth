// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Staged tag editing for the selected live session.

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::style::Style;

use crate::model::tag::Tag;
use crate::model::tag::Tags;

/// Completion of a tag-edit operation.
pub(super) enum Action {
    Accept,
    Cancel,
}

/// Initial and pending memberships for the selected session.
pub(super) struct State {
    pub(super) pending: Tags,
    pub(super) initial: Tags,
    pub(super) batch: bool,
}

impl State {
    /// Start editing the selected live session's memberships.
    pub(super) fn new(initial: Tags) -> Self {
        Self {
            pending: initial,
            initial,
            batch: false,
        }
    }

    /// Lowercase first input commits immediately; uppercase enters explicit-confirmation mode.
    /// Unrecognized input cancels only while awaiting the first tag.
    pub(super) fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        use KeyCode as KC;
        use KeyModifiers as KM;

        const CTRL: KM = KM::CONTROL;

        let ctrl = key.modifiers.contains(CTRL);
        let alt = key.modifiers.contains(KM::ALT);
        let super_ = key.modifiers.contains(KM::SUPER);

        let recognized = match key.code {
            KC::Char('t') if ctrl => return Some(Action::Accept),
            KC::Char('g') if ctrl => return Some(Action::Cancel),

            KC::Char(c @ ('a'..='z' | 'A'..='Z')) if !ctrl && !alt && !super_ => {
                let tag = c
                    .to_ascii_lowercase()
                    .to_string()
                    .parse()
                    .expect("ASCII letter");

                self.pending.toggle(tag);
                if c.is_ascii_uppercase() {
                    self.batch = true;
                }

                true
            }

            _ => false,
        };

        if self.batch {
            None
        } else if recognized {
            Some(Action::Accept)
        } else {
            Some(Action::Cancel)
        }
    }

    /// Distinguish staged additions and removals from unchanged memberships.
    pub(super) fn style(&self, tag: Tag) -> Style {
        if !self.pending.contains(tag) {
            Style::new().dim()
        } else if !self.initial.contains(tag) {
            Style::new().light_blue().bold()
        } else {
            Style::new()
        }
    }
}
