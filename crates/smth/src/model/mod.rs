// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Application model for discovered sessions and derived session candidates.

pub(crate) mod agent;
pub(crate) mod picker;
pub(crate) mod serialize;
pub(crate) mod session;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use anyhow::bail;
use anyhow::ensure;
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt as _;
use nucleo::Snapshot;
use nucleo::Status;

use crate::cmd::jj;
use crate::cmd::tmux;
use crate::model::agent::AgentState;
use crate::model::picker::Picker;
use crate::model::serialize::SerializedSession;
use crate::model::session::Base;
use crate::model::session::LiveKind;
use crate::model::session::NewKind;
use crate::model::session::Repo;
use crate::model::session::RepoKind;
use crate::model::session::Session;

/// Application-level model for the session picker.
///
/// This owns the discovered session rows, the collision sets used while deriving candidate rows,
/// and the mapping from repository paths to workspace metadata needed by workspace-aware session
/// construction.
///
/// The default model is empty and does not discover sessions.
#[derive(Default)]
pub struct Model {
    /// Fuzzy finder state for discovered sessions.
    picker: Picker<Session>,

    /// The sessions to fuzzy find over.
    sessions: Vec<Session>,

    /// Index of the unattached session that was most recently attached to.
    recently_attached: Option<usize>,

    /// Names of live tmux sessions, used to disambiguate candidate session names.
    seen_tmux_names: BTreeSet<String>,

    /// Workspaces found, identified by their root (default) path and workspace name. Used to
    /// disambiguate the creation of new workspaces.
    seen_workspaces: BTreeMap<PathBuf, BTreeSet<String>>,

    /// Mapping from a repository path to optional workspace metadata.
    ///
    /// A present `None` value means workspace discovery succeeded for this path, but no workspace
    /// root was recorded for it.
    workspaces: BTreeMap<PathBuf, Option<Workspace>>,
}

/// Workspace metadata for a discovered repository.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Workspace {
    /// Workspace name reported by `jj workspace list` for this repository root.
    ///
    /// `None` represents the default workspace.
    name: Option<String>,
    /// Root path for the default workspace in the same jj repository, when available.
    default: Option<PathBuf>,
}

impl Model {
    /// Construct a model with discovered sessions and a seeded fuzzy query.
    ///
    /// `globs` is a list of glob patterns to search for repositories in. `current` is an
    /// optional current repository path. The model will discover workspace information for all the
    /// repositories found between the two. `query` seeds the model's fuzzy picker.
    pub async fn new(
        globs: &[String],
        current: Option<&Path>,
        query: String,
    ) -> anyhow::Result<Self> {
        let mut model = Self {
            picker: Picker::new(query),
            sessions: Vec::new(),
            recently_attached: None,
            seen_tmux_names: BTreeSet::new(),
            seen_workspaces: BTreeMap::new(),
            workspaces: BTreeMap::new(),
        };

        model.discover(globs, current).await?;
        Ok(model)
    }

    /// Return all matched sessions after the matcher has finished processing pending updates.
    pub fn matches(&mut self) -> Vec<Session> {
        loop {
            let (status, snapshot, _) = self.picker.refresh();
            if !status.running {
                return snapshot
                    .matched_items(..)
                    .map(|item| item.data.clone())
                    .collect();
            }
        }
    }

    /// Return serialized records for all sessions matched by the seeded query.
    pub fn matches_json(&mut self) -> Vec<SerializedSession> {
        let sessions = self.matches();
        sessions
            .iter()
            .map(|session| self.serialize_session(session))
            .collect()
    }

    /// Return the discovered session matching an optional repository base and name.
    pub fn session(&self, base: Option<&Path>, name: Option<&str>) -> Option<&Session> {
        match base {
            Some(base) => self.session_by_workspace(base, name),
            None => name.and_then(|name| self.session_by_name(name)),
        }
    }

    /// Return the existing or prospective session represented by an explicit request.
    ///
    /// When `repo_root` is supplied, return a fresh repository candidate below it rather than
    /// reusing an existing session. This requires an empty repository base and a valid, explicit
    /// directory name whose destination is unoccupied.
    pub fn session_for_request(
        &self,
        repo_root: Option<&Path>,
        base: Option<&Path>,
        name: Option<&str>,
        onto: &str,
    ) -> anyhow::Result<Session> {
        if let Some(root) = repo_root {
            ensure!(base.is_none(), "repo creation requires no base");
            let name = name.context("a repository name is required")?;
            return self.new_repo_session(name, root);
        }

        // If a session already exists for this base and name, return it.
        if let Some(session) = self.session(base, name) {
            return Ok(session.clone());
        }

        let Some(base) = base else {
            let name = name.context("a session name is required without a repository base")?;
            return Ok(self.new_session(name, Base::Cwd(None)));
        };

        let name = name.or_else(|| self.workspace_name(base));
        let base = self.repo_context(base.to_owned()).path().to_owned();

        let Some(name) = name else {
            return Ok(self.repo_session(None, base.clone(), base));
        };

        if let Some(checkout) = self.workspace_path(&base, name) {
            return Ok(self.repo_session(Some(name), base, checkout.to_owned()));
        }

        let repo = Repo::new(base).with_revision(onto.to_owned());
        Ok(self.new_session(name, Base::Repo(repo)))
    }

    /// Return lifecycle state counts for agents across all discovered sessions.
    pub(crate) fn agent_summary(&self) -> BTreeMap<AgentState, usize> {
        let mut summary = BTreeMap::new();
        for agents in self.sessions.iter().filter_map(Session::agents) {
            for (&state, &count) in agents {
                *summary.entry(state).or_default() += count;
            }
        }
        summary
    }

    /// Clear the active query string.
    pub(crate) fn clear_query(&mut self) {
        self.picker.clear();
    }

    /// Discover sessions while preserving the current fuzzy query.
    pub(crate) async fn discover(
        &mut self,
        globs: &[String],
        current: Option<&Path>,
    ) -> anyhow::Result<()> {
        self.sessions.clear();
        self.recently_attached = None;
        self.seen_tmux_names.clear();
        self.seen_workspaces.clear();
        self.workspaces.clear();

        let mut tmux_repos = BTreeSet::new();

        let tmux_sessions = tmux::sessions().await?;
        for (name, info) in &tmux_sessions {
            self.seen_tmux_names.insert(name.clone());
            tmux_repos.extend(info.repo.clone());
        }

        let globbed = jj::repos(globs)?;
        let repos = globbed
            .iter()
            .chain(&tmux_repos)
            .map(PathBuf::as_path)
            .chain(current);

        // Discover workspace names and locations for the repositories found -- this is used to
        // construct workspace-aware session names.
        self.workspaces = workspaces(repos).await;

        // Order live sessions by attached status (attached first), then last attached time (most
        // recent first), the name (lexicographically).
        let mut tmux_sessions: Vec<_> = tmux_sessions.into_iter().collect();
        tmux_sessions.sort_by(|(lname, left), (rname, right)| {
            let l = (left.attached, left.last_attached, lname);
            let r = (right.attached, right.last_attached, rname);
            r.cmp(&l)
        });

        // Find the unattached session that was most recently attached.
        if self.picker.query().is_empty()
            && let Some(index) = tmux_sessions.iter().position(|(_, info)| !info.attached)
        {
            self.recently_attached = Some(index);
        }

        for (name, info) in tmux_sessions {
            let workspace = info
                .repo
                .as_ref()
                .and_then(|repo| self.workspace_name(repo))
                .map(str::to_owned);

            let session = LiveKind::new(
                name,
                info.repo,
                workspace,
                info.agents,
                info.alerts,
                info.flagged,
            );
            self.sessions.push(session.into());
        }

        // Add an entry for every repo found, as long as it's not already associated with a
        // live tmux session.
        for repo in globbed {
            if tmux_repos.contains(&repo) {
                continue;
            }

            let session = if let Some(Some(workspace)) = self.workspaces.get(&repo) {
                self.repo_session(
                    workspace.name.as_deref(),
                    existing_default(workspace).unwrap_or(&repo).to_owned(),
                    repo.to_owned(),
                )
            } else {
                self.repo_session(None, repo.to_owned(), repo.to_owned())
            };

            self.sessions.push(session);
        }

        // Attach information about workspaces seen, to help disambiguate future new workspaces
        // created by the tool.
        for (root, workspace) in &self.workspaces {
            let Some(workspace) = workspace else {
                continue;
            };

            let root = existing_default(workspace).unwrap_or(root);
            let name = workspace.name.as_deref().unwrap_or(jj::DEFAULT_WORKSPACE);

            self.seen_workspaces
                .entry(root.to_owned())
                .or_default()
                .insert(name.to_owned());
        }

        self.picker.reset();
        self.picker.inject(self.sessions.clone());

        Ok(())
    }

    /// Index of the unattached live session that was most recently attached to.
    pub(crate) fn recently_attached(&self) -> Option<usize> {
        self.recently_attached
    }

    /// Remove the trailing character from the active query string.
    pub(crate) fn pop_query(&mut self) {
        self.picker.pop();
    }

    /// Append one character to the active query string.
    ///
    /// Returns whether this character started a new query.
    pub(crate) fn push_query(&mut self, ch: char) -> bool {
        let started = self.picker.query().is_empty();
        self.picker.push(ch);
        started
    }

    /// Refresh fuzzy matches and return the currently visible rows.
    pub(crate) fn refresh(&mut self) -> (Status, &Snapshot<Session>, &str) {
        self.picker.refresh()
    }

    /// Construct a repository context, normalizing it to an existing default workspace when known.
    pub(crate) fn repo_context(&self, path: PathBuf) -> Repo {
        let path = self
            .workspaces
            .get(&path)
            .and_then(Option::as_ref)
            .and_then(existing_default)
            .map(Path::to_owned)
            .unwrap_or(path);

        Repo::new(path)
    }

    /// Return the discovered sessions.
    pub(crate) fn sessions(&self) -> &[Session] {
        &self.sessions
    }

    /// Return prospective sessions for a nonempty query, with the default choice last.
    ///
    /// Without repository context, offer a fresh repository below `repo_root` followed by a plain
    /// session. With context, offer only the usual workspace or checkout-backed candidate.
    pub(crate) fn sessions_for_query(&self, repo: Option<&Repo>, repo_root: &Path) -> Vec<Session> {
        let query = self.picker.query();
        let mut sessions = Vec::new();

        if query.is_empty() {
            return sessions;
        }

        if let Some(repo) = repo {
            let base = if self.workspaces.contains_key(repo.path()) {
                Base::Repo(repo.clone())
            } else {
                Base::Cwd(Some(repo.path().to_owned()))
            };

            sessions.push(self.new_session(query, base))
        } else {
            if let Ok(session) = self.new_repo_session(query, repo_root) {
                sessions.push(session);
            }
            sessions.push(self.new_session(query, Base::Cwd(None)));
        }

        sessions
    }

    /// Return the exact jj workspace name for `repo`, if it is a named workspace.
    pub(crate) fn workspace_name(&self, repo: &Path) -> Option<&str> {
        self.workspace_info(repo).and_then(|w| w.name.as_deref())
    }

    /// Construct a fresh repository candidate without changing its requested directory name.
    ///
    /// Reject empty names, `.` and `..`, path separators, NUL, and occupied destinations.
    fn new_repo_session(&self, name: &str, root: &Path) -> anyhow::Result<Session> {
        let mut parts = Path::new(name).components();

        match parts.next() {
            Some(Component::Normal(_)) => {
                ensure!(!name.contains('\0'), "repo names must not contain NUL");
            }

            None => bail!("repo names must be non-empty"),

            Some(Component::Prefix(_) | Component::RootDir) => {
                bail!("repo names must not be absolute paths",)
            }

            Some(Component::CurDir | Component::ParentDir) => {
                bail!("repo names must not be '.' or '..'",)
            }
        }

        if parts.next().is_some() {
            bail!("repository names must not contain path separators");
        }

        let path = root.join(name);
        ensure!(!path.exists(), "repo '{}' already exists", path.display());
        Ok(self.new_session(name, Base::NewRepo(root.to_owned())))
    }

    /// Construct a prospective session from a name and base.
    fn new_session(&self, name: &str, base: Base) -> Session {
        let empty = BTreeSet::new();
        let siblings = match &base {
            Base::Repo(base) => self.seen_workspaces.get(base.path()).unwrap_or(&empty),
            Base::NewRepo(_) | Base::Cwd(_) => &empty,
        };

        let mut session = NewKind::new(name, base);
        session.disambiguate(&self.seen_tmux_names, siblings);
        session.into()
    }

    /// Construct a session for an existing repository checkout.
    fn repo_session(&self, workspace: Option<&str>, default: PathBuf, path: PathBuf) -> Session {
        let mut session = RepoKind::new(workspace, default, path);
        session.disambiguate(&self.seen_tmux_names);
        session.into()
    }

    /// Convert one picker session into its stable serialized schema.
    fn serialize_session(&self, session: &Session) -> SerializedSession {
        let path = session.repo();
        let (base, name) = if let Some(path) = path.as_deref() {
            if let Some(workspace) = self.workspace_info(path) {
                let base = existing_default(workspace).unwrap_or(path).to_owned();
                (Some(base), workspace.name.clone())
            } else {
                (Some(path.to_owned()), None)
            }
        } else {
            (None, session.is_live().then(|| session.name()))
        };

        let agents: BTreeMap<_, _> = session
            .agents()
            .into_iter()
            .flatten()
            .map(|(state, count)| (state.value(), *count))
            .collect();

        SerializedSession {
            base,
            name,
            path,
            tmux: session.name(),
            live: session.is_live(),
            deletable: session.can_delete(),
            flagged: session.flag(),
            attention: session.attention_windows().cloned().unwrap_or_default(),
            agents,
        }
    }

    /// Return the plain session matching `name`.
    fn session_by_name(&self, name: &str) -> Option<&Session> {
        self.sessions
            .iter()
            .find(|session| session.repo().is_none() && session.name() == name)
    }

    /// Return the session matching a repository family and workspace name.
    fn session_by_workspace(&self, base: &Path, name: Option<&str>) -> Option<&Session> {
        let selected = self.workspace_info(base)?;
        let name = name.or(selected.name.as_deref());
        let base = existing_default(selected).unwrap_or(base);

        self.sessions.iter().find(|session| {
            let Some(repo) = session.repo() else {
                return false;
            };

            let Some(workspace) = self.workspace_info(&repo) else {
                return false;
            };

            workspace.name.as_deref() == name
                && existing_default(workspace).unwrap_or(&repo) == base
        })
    }

    /// Return the checkout path for a named workspace in a repository family.
    fn workspace_path(&self, base: &Path, name: &str) -> Option<&Path> {
        self.workspaces.iter().find_map(|(path, workspace)| {
            let workspace = workspace.as_ref()?;
            (workspace.name.as_deref() == Some(name)
                && existing_default(workspace).unwrap_or(path) == base)
                .then_some(path.as_path())
        })
    }

    /// Return workspace metadata for `repo`.
    fn workspace_info(&self, repo: &Path) -> Option<&Workspace> {
        self.workspaces.get(repo).and_then(Option::as_ref)
    }
}

/// Return the recorded default workspace root when it still exists.
fn existing_default(workspace: &Workspace) -> Option<&Path> {
    workspace.default.as_deref().filter(|root| root.exists())
}

/// Discover workspace metadata for every workspace associated with each repository.
async fn workspaces<'a>(
    repos: impl IntoIterator<Item = &'a Path>,
) -> BTreeMap<PathBuf, Option<Workspace>> {
    let mut tasks: FuturesUnordered<_> = repos
        .into_iter()
        .map(|repo| async move { (repo.to_owned(), jj::workspaces(repo).await) })
        .collect();

    let mut info = BTreeMap::new();
    while let Some((repo, Ok(ws))) = tasks.next().await {
        let default = ws.get(&None).cloned().flatten();
        let mut found = false;
        for (name, root) in ws {
            let Some(root) = root else {
                continue;
            };

            found |= root == repo;
            let default = default.clone();
            info.insert(root, Some(Workspace { name, default }));
        }

        if !found {
            info.insert(repo, None);
        }
    }

    info
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn default_model_is_empty() {
        let mut model = Model::default();

        assert!(model.picker.query().is_empty());
        assert!(model.matches().is_empty());
        assert!(model.sessions_for_query(None, Path::new(".")).is_empty());
    }

    fn model_with_workspace(workspace: &Path, default: PathBuf) -> Model {
        let session = RepoKind::new(Some("feature"), default.clone(), workspace.to_owned()).into();

        let mut workspaces = BTreeMap::from([(
            workspace.to_owned(),
            Some(Workspace {
                name: Some("feature".to_owned()),
                default: Some(default.clone()),
            }),
        )]);

        if default.exists() {
            workspaces.insert(
                default.clone(),
                Some(Workspace {
                    name: None,
                    default: Some(default),
                }),
            );
        }

        Model {
            sessions: vec![session],
            workspaces,
            ..Model::default()
        }
    }

    #[test]
    fn new_repo_accepts_single_normal_components() {
        let temp = tempdir().unwrap();
        for (name, expected) in [
            ("foo.bar", "foo-bar"),
            ("project one", "project-one"),
            ("foo/", "foo"),
            ("foo//", "foo"),
            ("foo/.", "foo"),
            ("a\\b", "a-b"),
            ("...", "1"),
        ] {
            let mut model = Model::default();
            let session = model
                .session_for_request(Some(temp.path()), None, Some(name), jj::DEFAULT_BASE_REVSET)
                .unwrap();

            assert_eq!(session.name(), expected, "{name:?}");
            assert_eq!(session.repo(), Some(temp.path().join(name)), "{name:?}");
            assert!(!session.is_live() && !session.can_delete(), "{name:?}");

            for ch in name.chars() {
                model.push_query(ch);
            }

            let candidates = model.sessions_for_query(None, temp.path());
            assert_eq!(candidates.len(), 2);
            assert_eq!(candidates[0], session);
        }
    }

    #[test]
    fn new_repo_rejects_invalid_names() {
        let temp = tempdir().unwrap();
        for name in [
            None,
            Some(""),
            Some("."),
            Some(".."),
            Some("a/b"),
            Some("a/../b"),
            Some("./a"),
            Some("/absolute"),
            Some("nul\0"),
        ] {
            let mut model = Model::default();
            assert!(
                model
                    .session_for_request(Some(temp.path()), None, name, jj::DEFAULT_BASE_REVSET)
                    .is_err(),
                "accepted invalid repository name {name:?}"
            );
            for ch in name.unwrap_or_default().chars() {
                model.push_query(ch);
            }
            assert!(
                model
                    .sessions_for_query(None, temp.path())
                    .iter()
                    .all(|s| s.repo().is_none())
            );
        }
    }

    #[test]
    fn new_repo_session_disambiguates_existing_session() {
        let temp = tempdir().unwrap();
        let mut model = Model::default();
        let existing: Session = LiveKind::new(
            "project".to_owned(),
            None,
            None,
            BTreeMap::new(),
            BTreeSet::new(),
            false,
        )
        .into();
        model.sessions.push(existing.clone());
        model.seen_tmux_names.insert("project".to_owned());

        let reused = model
            .session_for_request(None, None, Some("project"), jj::DEFAULT_BASE_REVSET)
            .unwrap();
        assert_eq!(reused, existing);

        let fresh = model
            .session_for_request(
                Some(temp.path()),
                None,
                Some("project"),
                jj::DEFAULT_BASE_REVSET,
            )
            .unwrap();
        assert_eq!(fresh.name(), "project~1");
        assert_eq!(fresh.repo(), Some(temp.path().join("project")));
        assert!(!fresh.is_live());
    }

    #[test]
    fn new_repo_session_rejects_path_collision() {
        let temp = tempdir().unwrap();
        fs::create_dir(temp.path().join("project")).unwrap();

        for name in ["project", "project/", "project/."] {
            let mut model = Model::default();
            model.seen_tmux_names.insert("project~1".to_owned());

            let error = model
                .session_for_request(Some(temp.path()), None, Some(name), jj::DEFAULT_BASE_REVSET)
                .unwrap_err();

            assert_eq!(
                error.to_string(),
                format!("repo '{}' already exists", temp.path().join(name).display())
            );

            for ch in name.chars() {
                model.push_query(ch);
            }

            let candidates = model.sessions_for_query(None, temp.path());
            assert_eq!(candidates.len(), 1, "{name:?}");
            assert_eq!(candidates[0].repo(), None, "{name:?}");
        }
    }

    #[test]
    fn new_repo_session_requires_empty_context() {
        let temp = tempdir().unwrap();
        let model = Model::default();

        let error = model
            .session_for_request(
                Some(temp.path()),
                Some(temp.path()),
                Some("project"),
                jj::DEFAULT_BASE_REVSET,
            )
            .unwrap_err();

        assert_eq!(error.to_string(), "repo creation requires no base");
    }

    #[test]
    fn query_candidates_disambiguate_only_session_names() {
        let temp = tempdir().unwrap();
        for (name, sessions, occupied, expected) in [
            ("...", vec![], "1", "1"),
            ("...", vec!["1", "2"], "1", "3"),
            ("...", vec!["1", "3"], "2", "2"),
            (
                "project",
                vec!["project", "project~2"],
                "project~1",
                "project~1",
            ),
        ] {
            let existing = temp.path().join("existing");
            fs::create_dir_all(existing.join(occupied)).unwrap();

            for root in [existing, temp.path().join("missing")] {
                let mut model = Model {
                    seen_tmux_names: sessions.iter().map(|s| (*s).to_owned()).collect(),
                    ..Model::default()
                };

                let requested = model
                    .session_for_request(Some(&root), None, Some(name), jj::DEFAULT_BASE_REVSET)
                    .unwrap();

                for ch in name.chars() {
                    model.push_query(ch);
                }

                let candidates = model.sessions_for_query(None, &root);
                assert_eq!(candidates.len(), 2, "{name:?}, {sessions:?}");
                assert_eq!(candidates[0], requested);
                assert_eq!(candidates[0].name(), expected);
                assert_eq!(candidates[0].repo(), Some(root.join(name)));
                assert_eq!(candidates[1].name(), expected);
                assert_eq!(candidates[1].repo(), None);
                assert!(candidates.iter().all(|s| !s.is_live() && !s.can_delete()));
                assert!(!root.join(name).exists());
            }
        }
    }

    #[test]
    fn query_candidates_preserve_repository_context() {
        let temp = tempdir().unwrap();
        let base = temp.path().join("repo");
        let repo = Repo::new(base.clone());
        let mut model = Model::default();
        assert!(
            model
                .sessions_for_query(Some(&repo), temp.path())
                .is_empty()
        );
        for ch in "feature".chars() {
            model.push_query(ch);
        }

        let candidates = model.sessions_for_query(Some(&repo), temp.path());
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].name(), "feature");
        assert_eq!(candidates[0].repo(), None);

        model.workspaces.insert(base, None);
        let candidates = model.sessions_for_query(Some(&repo), temp.path());
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].name(), "repo/feature");
        assert_eq!(candidates[0].repo(), Some(temp.path().join("repo.feature")));
    }

    #[test]
    fn repo_context_keeps_path_when_default_workspace_is_missing() {
        let temp = tempdir().unwrap();
        let workspace = temp.path().join("repo.feature");
        let default = temp.path().join("repo");
        fs::create_dir(&workspace).unwrap();
        let model = model_with_workspace(&workspace, default);

        assert_eq!(model.repo_context(workspace.clone()).path(), workspace);
    }

    #[test]
    fn repo_context_normalizes_to_existing_default_workspace() {
        let temp = tempdir().unwrap();
        let workspace = temp.path().join("repo.feature");
        let default = temp.path().join("repo");
        fs::create_dir(&workspace).unwrap();
        fs::create_dir(&default).unwrap();
        let model = model_with_workspace(&workspace, default.clone());

        assert_eq!(model.repo_context(workspace).path(), default);
    }

    #[test]
    fn session_finds_non_live_workspace() {
        let temp = tempdir().unwrap();
        let workspace = temp.path().join("repo.feature");
        let default = temp.path().join("repo");
        fs::create_dir(&workspace).unwrap();
        fs::create_dir(&default).unwrap();
        let model = model_with_workspace(&workspace, default.clone());

        let session = model.session(Some(&default), Some("feature")).unwrap();
        assert_eq!(session.repo().as_deref(), Some(workspace.as_path()));
        assert!(!session.is_live());
    }

    #[test]
    fn session_finds_plain_session_by_name() {
        let temp = tempdir().unwrap();
        let workspace = temp.path().join("repo.feature");
        let default = temp.path().join("repo");
        let mut model = model_with_workspace(&workspace, default);
        let repo = LiveKind::new(
            "scratch".to_owned(),
            Some(workspace),
            Some("feature".to_owned()),
            BTreeMap::new(),
            BTreeSet::new(),
            false,
        );
        let plain = LiveKind::new(
            "scratch".to_owned(),
            None,
            None,
            BTreeMap::new(),
            BTreeSet::new(),
            false,
        );
        model.sessions = vec![repo.into(), plain.into()];

        let session = model.session(None, Some("scratch")).unwrap();
        assert!(session.repo().is_none());
    }
}
