// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Fuzzy matching adapter for pickable items.

use std::sync::Arc;

use nucleo::Config;
use nucleo::Nucleo;
use nucleo::Snapshot;
use nucleo::Status;
use nucleo::Utf32String;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;

const TICK_TIMEOUT_MS: u64 = 10;

/// Items that can be fuzzy matched by the picker.
pub(crate) trait Pickable {
    /// Return the text matched by the picker.
    fn text(&self) -> String;
}

/// Fuzzy matcher state for the session picker.
pub(crate) struct Picker<I: Send + Sync + 'static> {
    matcher: Nucleo<I>,
    query: String,
}

impl<I: Pickable + Send + Sync + 'static> Picker<I> {
    /// Construct an empty fuzzy matcher seeded with `query`.
    pub(crate) fn new(query: String) -> Self {
        let matcher = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        let mut picker = Self { matcher, query };
        picker.reparse(false);
        picker
    }

    /// Clear the active query string.
    pub(crate) fn clear(&mut self) {
        self.query.clear();
        self.reparse(false);
    }

    /// Inject replacement items into the matcher.
    pub(crate) fn inject(&self, items: impl IntoIterator<Item = I>) {
        let injector = self.matcher.injector();
        for item in items {
            injector.push(item, |item, columns| {
                columns[0] = Utf32String::from(item.text())
            });
        }
    }

    /// Remove the trailing character from the active query string.
    pub(crate) fn pop(&mut self) {
        self.query.pop();
        self.reparse(false);
    }

    /// Append one character to the active query string.
    pub(crate) fn push(&mut self, ch: char) {
        self.query.push(ch);
        self.reparse(true);
    }

    /// Return the active query string.
    pub(crate) fn query(&self) -> &str {
        &self.query
    }

    /// Advance fuzzy matching and return its processing status.
    pub(crate) fn refresh(&mut self) -> Status {
        self.matcher.tick(TICK_TIMEOUT_MS)
    }

    /// Borrow the last published matches without advancing the matcher.
    pub(crate) fn snapshot(&self) -> &Snapshot<I> {
        self.matcher.snapshot()
    }

    /// Reset matcher contents while preserving the query and last completed matches.
    ///
    /// Keep the snapshot until replacement matches are ready so a transient empty list cannot
    /// clamp the UI selection onto the new-session candidate during rediscovery.
    pub(crate) fn reset(&mut self) {
        self.matcher.restart(false);
    }

    /// Reparse the active query and tell nucleo whether the change appended text.
    fn reparse(&mut self, append: bool) {
        self.matcher.pattern.reparse(
            0,
            &self.query,
            CaseMatching::Smart,
            Normalization::Smart,
            append,
        );
    }
}

impl<I: Pickable + Send + Sync + 'static> Default for Picker<I> {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::time::Instant;

    use super::*;

    impl Pickable for &'static str {
        fn text(&self) -> String {
            (*self).to_owned()
        }
    }

    fn matches(picker: &Picker<&'static str>) -> Vec<&'static str> {
        picker
            .snapshot()
            .matched_items(..)
            .map(|item| *item.data)
            .collect()
    }

    /// Wait for matching to finish without hanging indefinitely on a worker failure.
    fn settle(picker: &mut Picker<&'static str>) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while picker.refresh().running {
            assert!(Instant::now() < deadline, "matcher did not finish");
        }
    }

    #[test]
    fn reads_preserve_published_matches_until_refresh() {
        let mut picker = Picker::default();
        picker.inject(["alpha", "beta"]);
        settle(&mut picker);

        picker.push('z');
        for _ in 0..3 {
            assert_eq!(picker.query(), "z");
            assert_eq!(matches(&picker), ["alpha", "beta"]);
            assert!(picker.snapshot().pattern().is_empty());
        }

        settle(&mut picker);
        assert!(matches(&picker).is_empty());
    }

    #[test]
    fn reset_retains_snapshot_until_matches_are_ready() {
        let mut picker = Picker::new("alp".to_owned());
        picker.inject(["alpha", "alpine"]);
        settle(&mut picker);
        assert_eq!(matches(&picker), ["alpha", "alpine"]);

        picker.reset();
        assert_eq!(picker.query(), "alp");
        assert_eq!(matches(&picker), ["alpha", "alpine"]);

        picker.inject(["alpine"]);
        settle(&mut picker);
        assert_eq!(matches(&picker), ["alpine"]);

        picker.reset();
        settle(&mut picker);
        assert!(matches(&picker).is_empty());
    }
}
