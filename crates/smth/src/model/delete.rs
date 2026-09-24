// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Pending checkout deletions and affected session counts.

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::model::session::Session;

/// Workspace-local marker for persisted deletion staging.
pub(crate) const MARKER: &str = ".smth-pending-delete";

/// Selected deletions, shared by every session referencing a selected checkout.
pub(crate) struct Model {
    paths: BTreeSet<PathBuf>,
    /// All affected session rows, including aliases and filtered-out sessions.
    pending: usize,
}

impl Model {
    /// Build a deletion selection from checkout paths and their discovered session rows.
    pub(crate) fn new(paths: BTreeSet<PathBuf>, sessions: &[Session]) -> Option<Self> {
        if paths.is_empty() {
            return None;
        }

        let mut model = Self { paths, pending: 0 };
        model.pending = sessions.iter().filter(|s| model.contains(s)).count();
        Some(model)
    }

    /// Whether this session shares a selected checkout.
    pub(crate) fn contains(&self, session: &Session) -> bool {
        session.repo().is_some_and(|r| self.paths.contains(&r))
    }

    /// Number of affected sessions absent from the supplied rendered matches.
    ///
    /// The number of affected matches must not exceed the selection's pending count.
    pub(crate) fn hidden<'s>(&self, matching: impl Iterator<Item = &'s Session>) -> usize {
        self.pending - matching.filter(|session| self.contains(session)).count()
    }

    /// Checkout paths selected for deletion.
    pub(crate) fn paths(&self) -> &BTreeSet<PathBuf> {
        &self.paths
    }

    /// Number of affected sessions, including aliases and filtered-out sessions.
    pub(crate) fn pending(&self) -> usize {
        self.pending
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::model::session::Base;
    use crate::model::session::LiveKind;
    use crate::model::session::NewKind;

    use super::*;

    /// A shared checkout contributes one pending row per alias, but only one path.
    #[test]
    fn counts_aliases_and_derives_hidden_from_matches() {
        let path = PathBuf::from("repo.feature");
        let sessions: [Session; 2] = ["first", "alias"].map(|name| {
            LiveKind::new(
                name.to_owned(),
                Some(path.clone()),
                Some("feature".to_owned()),
                BTreeMap::new(),
                BTreeSet::new(),
                false,
            )
            .into()
        });
        let plain: Session = NewKind::new("plain", Base::Cwd(None)).into();
        let model = Model::new(BTreeSet::from([path.clone()]), &sessions).unwrap();

        assert_eq!(model.paths(), &BTreeSet::from([path]));
        assert_eq!(model.pending(), 2);
        assert!(sessions.iter().all(|session| model.contains(session)));
        assert!(!model.contains(&plain));
        assert_eq!(model.hidden(sessions.iter()), 0);
        assert_eq!(model.hidden([&sessions[0], &plain].into_iter()), 1);
        assert_eq!(model.hidden(std::iter::empty()), 2);
    }

    /// No pending paths means there is no deletion model to render or act on.
    #[test]
    fn empty_paths_have_no_model() {
        assert!(Model::new(BTreeSet::new(), &[]).is_none());
    }
}
