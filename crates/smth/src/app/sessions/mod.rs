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
use crate::config::Alphabet;
use crate::model::delete;
use crate::model::order::Order;
use crate::model::session::Session;

/// Minimum rows reserved for prospective sessions, with unused rows padded above the candidates.
pub(super) const NEW_SESSION_ROWS: usize = 2;

/// Session-list component, backed by fuzzy-matched rows and prospective session candidates.
pub(super) struct Sessions<'s> {
    sigil: char,
    alphabet: &'s Alphabet,
    new: &'s [Session],
    rest: &'s [Item<'s, Session>],
    order: &'s Order,
    pattern: &'s Pattern,
    delete: Option<&'s delete::Model>,
}

/// Selection and scroll state, including the ordering of the last rendered rows.
#[derive(Default)]
pub(super) struct State {
    list: ListState,
    /// Invoking session to skip once while the initial matches are arriving.
    initial: Option<Session>,
    /// Comparator stack used for the last rendered rows; absent before the first render.
    order: Option<Order>,
}

impl<'s> Sessions<'s> {
    /// Create a new `Sessions` component with `new` representing the potential new sessions, and
    /// `rest` being the other candidates. The `pattern` is what was used to filter down to these
    /// candidates, and is used to highlight the matching parts of candidate text. `order` is the
    /// stack used to sort `rest`; changes preserve the selected session's identity unless selection
    /// was explicitly reset.
    pub(super) fn new(
        sigil: char,
        alphabet: &'s Alphabet,
        new: &'s [Session],
        rest: &'s [Item<'s, Session>],
        order: &'s Order,
        pattern: &'s Pattern,
        delete: Option<&'s delete::Model>,
    ) -> Self {
        Self {
            sigil,
            alphabet,
            new,
            rest,
            order,
            pattern,
            delete,
        }
    }

    /// Render the session rows and keep the selected session state in sync with the list.
    pub(super) fn draw(
        &self,
        f: &mut Frame<'_>,
        list: Rect,
        scroll: Rect,
        state: &mut State,
        selected: &mut Option<Session>,
        running: bool,
    ) {
        let start = self.new.len().max(NEW_SESSION_ROWS);
        let padding = start - self.new.len();
        let mut rows = Vec::with_capacity(start + self.rest.len());

        self.adjust_selection(state, selected, running);
        let selected = state.list.selected();
        rows.extend((0..padding).map(|_| Row::empty()));
        for (i, session) in (padding..).zip(self.new) {
            rows.push(session::row(
                self.sigil,
                session,
                selected == Some(i),
                false,
                &[],
                "",
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
            let deleting = self.delete.is_some_and(|delete| delete.contains(item.data));
            let tags = item.data.tags().unwrap_or_default();
            let tags = tags.display(self.alphabet);
            let row = session::row(
                self.sigil,
                item.data,
                highlighted,
                deleting,
                &indices,
                &tags,
            );

            let margin = indices.last().copied().map(|off| right_margin(text, off));
            rows.push(row.with_right_margin(margin));
        }

        let height = list.height as usize;
        let length = rows.len().saturating_sub(height) + 1;
        f.render_stateful_widget(List::new(rows), list, &mut state.list);

        // Rendering the list may move its offset, so scrollbar state is calculated after, to avoid
        // a one frame desync.
        let mut scroll_state = ScrollbarState::default()
            .content_length(length)
            .viewport_content_length(height)
            .position(state.list.offset());

        f.render_stateful_widget(scrollbar::widget(), scroll, &mut scroll_state);
    }

    /// Adjust the selected row for the current matches and keep its session identity in sync.
    fn adjust_selection(&self, state: &mut State, selected: &mut Option<Session>, running: bool) {
        let start = self.new.len().max(NEW_SESSION_ROWS);
        let padding = start - self.new.len();

        // If ordering has changed since the last render, move the selected index to the new
        // position of the previously selected session, as long as it still exists.
        if state.order.as_ref() != Some(self.order) {
            if state.list.selected().is_some()
                && let Some(selected) = selected.as_ref()
                && let Some(index) = self
                    .new
                    .iter()
                    .chain(self.rest.iter().map(|item| item.data))
                    .position(|s| s == selected)
            {
                state.list.select(Some(index + padding));
            }

            state.order = Some(self.order.clone());
        }

        *selected = match (state.list.selected_mut(), self.new, self.rest) {
            // Skip the invoking first match once, unless input already chose another row.
            (s, _, [fst, snd, ..])
                if state.initial.as_ref() == Some(fst.data)
                    && s.is_none_or(|index| index == start) =>
            {
                *s = Some(start + 1);
                state.initial = None;
                Some(snd.data.clone())
            }

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

        // Once initial matching finishes, discard an unused adjustment even for empty or
        // singleton results. Otherwise later discovery could unexpectedly move the selection.
        if !running {
            state.initial = None;
        }
    }
}

impl State {
    /// Create session-list state with an optional invoking session to skip at startup.
    pub(super) fn new(initial: Option<Session>) -> Self {
        Self {
            initial,
            ..Self::default()
        }
    }

    /// End automatic startup selection on any key press, even an unrecognized key.
    ///
    /// User input takes precedence over skipping the invoking session as matches arrive.
    /// This leaves the current row and scroll position unchanged.
    pub(super) fn key_pressed(&mut self) {
        self.initial = None;
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
}

/// If `text` were laid out on a single line, calculate the column that would contain the glyph
/// after the glyph corresponding to the character at `offset` (an offset in terms of code points,
/// not bytes).
fn right_margin(text: Utf32Str<'_>, offset: u32) -> u16 {
    let matched: String = text.chars().take(offset as usize + 1).collect();
    matched.width() as u16
}

#[cfg(test)]
mod tests {
    use nucleo::Utf32String;

    use crate::cmd::tmux;
    use crate::model::session::LiveKind;

    use super::*;

    /// Supply controlled matches to exercise startup transitions without worker timing assumptions.
    fn adjust(state: &mut State, selected: &mut Option<Session>, rest: &[Session], running: bool) {
        let columns: Vec<_> = rest.iter().map(|s| [Utf32String::from(s.name())]).collect();
        let items: Vec<_> = rest
            .iter()
            .zip(&columns)
            .map(|(data, columns)| Item {
                data,
                matcher_columns: columns,
            })
            .collect();

        let alphabet = Alphabet::default();
        let order = "+name".parse().unwrap();
        let pattern = Pattern::default();
        let sessions = Sessions::new(' ', &alphabet, &[], &items, &order, &pattern, None);
        sessions.adjust_selection(state, selected, running);
    }

    /// Construct a live row with no external process or filesystem dependency.
    fn session(name: &str) -> Session {
        LiveKind::new(name.to_owned(), None, tmux::SessionInfo::default()).into()
    }

    /// A key press leaves the selected row in place when more initial matches arrive.
    #[test]
    fn key_pressed_cancels_pending_adjustment() {
        let alpha = session("alpha");
        let rest = [alpha.clone(), session("beta")];

        let mut state = State::new(Some(alpha.clone()));
        let mut selected = None;
        adjust(&mut state, &mut selected, &rest[..1], true);

        state.key_pressed();
        adjust(&mut state, &mut selected, &rest, false);
        assert_eq!(selected.as_ref(), Some(&alpha));
    }

    /// An unfinished empty or singleton snapshot may still gain a second initial match.
    #[test]
    fn pending_adjustment_applies_once_as_matches_arrive() {
        let alpha = session("alpha");
        let beta = session("beta");

        let mut state = State::new(Some(alpha.clone()));
        let mut selected = Some(alpha.clone());

        adjust(&mut state, &mut selected, &[], true);

        // No sessions yet, so no adjustment, and the initial session is still pending.
        assert!(state.initial.is_some());

        adjust(
            &mut state,
            &mut selected,
            std::slice::from_ref(&alpha),
            true,
        );

        // There's a single session and it matches, but there's no alternative to switch to, so no
        // adjustment is made.
        assert_eq!(selected.as_ref(), Some(&alpha));
        assert!(state.initial.is_some());

        let rest = [alpha.clone(), beta.clone()];
        adjust(&mut state, &mut selected, &rest, true);

        // Now there's more than one session and the first one matches, so the adjustment is made.
        assert_eq!(selected.as_ref(), Some(&beta));
        assert!(state.initial.is_none());

        // Reset the selection to the first row -- the adjustment will not be re-applied, because it
        // was already consumed.
        state.list.select(Some(NEW_SESSION_ROWS));
        adjust(&mut state, &mut selected, &rest, false);
        assert_eq!(selected.as_ref(), Some(&alpha));
    }

    /// In each of these cases, the adjustment cannot be applied on the first sessions passed in,
    /// but the initial session is discarded rather than being saved for a time when it could be
    /// applied.
    #[test]
    fn settled_results_discard_unused_adjustment() {
        for sessions in [
            vec![],
            vec![session("alpha")],
            vec![session("beta"), session("alpha")],
        ] {
            let mut state = State::new(Some(session("alpha")));
            let mut selected = Some(session("alpha"));
            adjust(&mut state, &mut selected, &sessions, false);

            assert!(state.initial.is_none());

            state.list.select(Some(NEW_SESSION_ROWS));
            adjust(
                &mut state,
                &mut selected,
                &[session("alpha"), session("beta")],
                false,
            );

            assert_eq!(selected, Some(session("alpha")));
        }
    }
}
