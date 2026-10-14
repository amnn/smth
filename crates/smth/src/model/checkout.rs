// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Discovery-time checkout aggregates used to count and resolve deletions.

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use crate::model::session::Session;

/// An immutable lookup built from one discovery's session rows.
///
/// Counts include every alias and repo-only row; closure targets include only live sessions.
/// Owned aggregates avoid retaining indices into the model's separately stored session list.
#[derive(Default)]
pub(crate) struct Index {
    checkouts: BTreeMap<PathBuf, Checkout>,
}

/// Session rows and live tmux names associated with a single checkout.
#[derive(Default)]
struct Checkout {
    count: usize,
    live: Vec<String>,
}

impl Index {
    /// Build a checkout lookup from discovered sessions, ignoring rows without a repository.
    pub(crate) fn new(sessions: &[Session]) -> Self {
        let mut index = Self::default();

        for session in sessions {
            let Some(path) = session.repo() else {
                continue;
            };

            let checkout = index.checkouts.entry(path).or_default();
            checkout.count += 1;

            if session.is_live() {
                checkout.live.push(session.name());
            }
        }

        index
    }

    /// Number of discovered rows associated with a checkout, including aliases.
    pub(crate) fn count(&self, path: &Path) -> usize {
        self.checkouts.get(path).map_or(0, |c| c.count)
    }

    /// Live session names to close after deleting a checkout, in discovery order.
    pub(crate) fn live(&self, path: &Path) -> &[String] {
        self.checkouts.get(path).map_or(&[], |c| &c.live)
    }
}

#[cfg(test)]
mod tests {
    use crate::cmd::tmux;
    use crate::model::session::LiveKind;
    use crate::model::session::RepoKind;

    use super::*;

    /// Construct a discovered live row without invoking tmux.
    fn live(name: &str, repo: Option<PathBuf>) -> Session {
        LiveKind::new(
            name.to_owned(),
            None,
            tmux::SessionInfo {
                repo,
                ..Default::default()
            },
        )
        .into()
    }

    /// Aliases count separately, while only live rows become session closure targets.
    #[test]
    fn counts_rows_and_preserves_live_targets() {
        let base = PathBuf::from("repo");
        let feat = PathBuf::from("repo.feature");
        let other = PathBuf::from("repo.other");
        let missing = PathBuf::from("missing");

        let sessions = vec![
            live("first", Some(feat.clone())),
            live("alias", Some(feat.clone())),
            RepoKind::new(Some("other"), base, other.clone()).into(),
            live("plain", None),
        ];

        let index = Index::new(&sessions);
        assert_eq!(index.checkouts.len(), 2);

        assert_eq!(index.count(&feat), 2);
        assert_eq!(index.live(&feat), ["first", "alias"]);

        assert_eq!(index.count(&other), 1);
        assert!(index.live(&other).is_empty());

        assert_eq!(index.count(&missing), 0);
        assert!(index.live(&missing).is_empty());

        let index = Index::new(&[live("replacement", Some(other.clone()))]);
        assert_eq!(index.checkouts.len(), 1);

        assert_eq!(index.count(&feat), 0);
        assert!(index.live(&feat).is_empty());

        assert_eq!(index.count(&other), 1);
        assert_eq!(index.live(&other), ["replacement"]);

        let index = Index::new(&[]);
        assert!(index.checkouts.is_empty());
    }
}
