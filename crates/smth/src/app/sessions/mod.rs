// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Component and state for rendering the session list.

pub(super) mod preview;
mod session;

use nucleo::Config;
use nucleo::Item;
use nucleo::Matcher;
use nucleo::Utf32Str;
use nucleo::pattern::Pattern;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use ratatui::widgets::ScrollbarState;
use unicode_width::UnicodeWidthStr as _;

use crate::app::component::list::List;
use crate::app::component::row::Row;
use crate::app::component::scrollbar;
use crate::model::session::Session;

/// Session-list component, backed by fuzzy-matched rows and prospective session candidates.
pub(super) struct Sessions<'s> {
    sigil: char,
    new: &'s [Session],
    rest: &'s [Item<'s, Session>],
    pattern: &'s Pattern,
}

/// Persistent selection and scroll state for the session list.
#[derive(Default)]
pub(super) struct State {
    deleting: bool,
    list: ListState,
    selected: Option<Session>,
}

impl<'s> Sessions<'s> {
    /// Create a new `Sessions` component with `new` representing the potential new sessions, and
    /// `rest` being the other candidates. The `pattern` is what was used to filter down to these
    /// candidates, and is used to highlight the matching parts of candidate text.
    pub(super) fn new(
        sigil: char,
        new: &'s [Session],
        rest: &'s [Item<'s, Session>],
        pattern: &'s Pattern,
    ) -> Self {
        Self {
            sigil,
            new,
            rest,
            pattern,
        }
    }

    /// Render the session rows and keep the selected session state in sync with the list.
    pub(super) fn draw(&self, f: &mut Frame<'_>, list: Rect, scroll: Rect, state: &mut State) {
        let start = self.new.len().max(1);
        let padding = start - self.new.len();
        let mut rows = Vec::with_capacity(start + self.rest.len());

        state.selected = match (state.list.selected_mut(), self.new, self.rest) {
            // With no candidates or matches, there is nothing to select.
            (s, [], []) => {
                *s = None;
                None
            }

            // Without matches, default to the last prospective candidate.
            (s @ None, new, []) => {
                *s = Some(start - 1);
                new.last().cloned()
            }

            // Otherwise, default to the first discovered match.
            (s @ None, _, [fst, ..]) => {
                *s = Some(start);
                Some(fst.data.clone())
            }

            // Clamp navigation to populated rows, skipping any leading padding.
            (Some(s), new, rest) => {
                *s = (*s).clamp(padding, start + rest.len() - 1);
                if *s < start {
                    new.get(*s - padding).cloned()
                } else {
                    rest.get(*s - start).map(|i| i.data.clone())
                }
            }
        };

        let selected = state.list.selected();
        rows.extend((0..padding).map(|_| Row::empty()));
        for (i, session) in (padding..).zip(self.new) {
            rows.push(session::row(
                self.sigil,
                session,
                selected == Some(i),
                false,
                &[],
            ));
        }

        // Reuse the matcher's scratch buffer across all candidates.
        let mut matcher = Matcher::new(Config::DEFAULT);

        for (i, item) in (start..).zip(self.rest) {
            let mut indices = Vec::new();
            let text = item.matcher_columns[0].slice(..);

            self.pattern.indices(text, &mut matcher, &mut indices);
            indices.sort_unstable();
            indices.dedup();

            let highlighted = selected == Some(i);
            let row = session::row(self.sigil, item.data, highlighted, state.deleting, &indices);

            let margin = indices.last().copied().map(|off| right_margin(text, off));
            rows.push(row.with_right_margin(margin));
        }

        let height = list.height as usize;
        let mut scroll_state = ScrollbarState::default()
            .content_length(rows.len().saturating_sub(height) + 1)
            .viewport_content_length(height)
            .position(state.list.offset());

        f.render_stateful_widget(List::new(rows), list, &mut state.list);
        f.render_stateful_widget(scrollbar::widget(), scroll, &mut scroll_state);
    }
}

impl State {
    /// Create session-list state with an optional initially selected list row.
    pub(super) fn new(selected: Option<usize>) -> Self {
        Self {
            list: ListState::default().with_selected(selected),
            ..Self::default()
        }
    }

    /// Whether the currently selected session can be deleted.
    pub(super) fn can_delete(&self) -> bool {
        self.selected.as_ref().is_some_and(Session::can_delete)
    }

    /// Whether the currently selected session can be flagged or unflagged.
    pub(super) fn can_flag(&self) -> bool {
        self.selected.as_ref().and_then(Session::flag).is_some()
    }

    /// Whether the selected session is marked for deletion.
    pub(super) fn is_deleting(&self) -> bool {
        self.deleting
    }

    /// Whether the currently selected session is live.
    pub(super) fn is_live(&self) -> bool {
        self.selected.as_ref().is_some_and(Session::is_live)
    }

    /// The session to preview, if one is currently selected.
    pub(super) fn preview(&self) -> Option<&Session> {
        self.selected.as_ref()
    }

    /// Cancel any pending deletion.
    pub(super) fn reset_delete(&mut self) {
        self.deleting = false;
    }

    /// Clear the selection so the next render selects the default row.
    pub(super) fn reset_selection(&mut self) {
        self.list.select(None);
    }

    /// Move selection to the beginning of the list.
    ///
    /// During rendering this is nudged past any empty leading rows.
    pub(super) fn select_first(&mut self) {
        self.list.select_first();
    }

    /// Move selection to the end of the list.
    pub(super) fn select_last(&mut self) {
        self.list.select_last();
    }

    /// Move selection down by one, unless already at the end of the list.
    pub(super) fn select_next(&mut self) {
        self.list.select_next();
    }

    /// Move selection up by one, unless already at the start of the list.
    pub(super) fn select_previous(&mut self) {
        self.list.select_previous();
    }

    /// A reference to the currently selected session, if there is one.
    pub(super) fn selected(&self) -> Option<&Session> {
        self.selected.as_ref()
    }

    /// Mark the selected session for deletion.
    pub(super) fn start_delete(&mut self) {
        self.deleting = true;
    }

    /// Take the selected session.
    ///
    /// Subsequent calls will return `None` until the next `draw` which will replenish this value.
    pub(super) fn take_selected(&mut self) -> Option<Session> {
        self.selected.take()
    }
}

/// If `text` were laid out on a single line, calculate the column that would contain the glyph
/// after the glyph corresponding to the character at `offset` (an offset in terms of code points,
/// not bytes).
fn right_margin(text: Utf32Str<'_>, offset: u32) -> u16 {
    let matched: String = text.chars().take(offset as usize + 1).collect();
    matched.width() as u16
}
