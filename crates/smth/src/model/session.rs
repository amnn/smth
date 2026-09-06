// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Session domain model.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use anyhow::ensure;

use crate::cmd::jj;
use crate::cmd::tmux;
use crate::model::agent::AgentState;
use crate::model::picker::Pickable;
use crate::path::TruncatedExt as _;

pub(crate) const DELIM_SUFFIX: &str = "~";
pub(crate) const NAME_WIDTH: usize = 40;

const DELIM_WORKSPACE: &str = "/";
const TMUX_REPO_OPTION: &str = "@smth.repo";

/// A tmux session or potential session.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Session(Kind);

/// The base used when creating a new session.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Base {
    /// Create a jj workspace from this repository information.
    Repo(Repo),
    /// Create a new repository below this parent directory.
    NewRepo(PathBuf),
    /// Create a tmux session at this working directory, or the process cwd if absent.
    Cwd(Option<PathBuf>),
}

/// A live tmux session with optional verified jj workspace identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct LiveKind {
    name: String,
    repo: Option<PathBuf>,
    workspace: Option<String>,
    agents: BTreeMap<AgentState, usize>,
    alerts: BTreeSet<String>,
    flagged: bool,
}

/// A new session, optionally backed by a jj workspace to create from a repository base.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct NewKind {
    name: String,
    base: Base,
    suffix: Option<String>,
}

/// Repository context used while constructing workspace-backed sessions.
///
/// `path` is normalized to an existing default workspace when available. `revision` is the jj
/// revset used as the new workspace base.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Repo {
    /// Checkout used as the repository context and base for sibling workspaces.
    path: PathBuf,
    /// jj revset used as the base revision for new workspaces.
    revision: String,
}

/// Session for a repository or workspace checkout that already exists.
///
/// The exact jj workspace name is preserved; sanitization applies only to the derived tmux name.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct RepoKind {
    workspace: Option<String>,
    default: PathBuf,
    path: PathBuf,
    suffix: Option<String>,
}

/// Backing kind for a picker session.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Kind {
    Live(LiveKind),
    New(NewKind),
    Repo(RepoKind),
}

impl Session {
    /// Return whether this entry can be deleted.
    pub fn can_delete(&self) -> bool {
        self.workspace().is_some()
    }

    /// Close this session without deleting any attached workspace.
    pub async fn close(&self) -> anyhow::Result<()> {
        match &self.0 {
            Kind::Live(kind) => kind.close().await,
            Kind::New(_) | Kind::Repo(_) => Ok(()),
        }
    }

    /// Create this session if needed without switching the current tmux client.
    pub async fn create(&self, cwd: &Path, setup: &str) -> anyhow::Result<()> {
        self.ensure_tmux(cwd, setup).await
    }

    /// Delete this session's named jj workspace and close it if live.
    ///
    /// A session without a verified named workspace is a no-op. Errors may be returned after the
    /// workspace has been forgotten if checkout removal or session closure fails.
    pub async fn delete(&self) -> anyhow::Result<()> {
        let Some((repo, workspace)) = self.workspace() else {
            return Ok(());
        };

        jj::forget_workspace(repo, workspace).await?;
        match tokio::fs::remove_dir_all(repo).await {
            Ok(()) => {}
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("failed to remove repository '{}'", repo.display()));
            }
        }

        self.close().await
    }

    /// Return this session's manual flag state, if this entry can be flagged.
    pub fn flag(&self) -> Option<bool> {
        match &self.0 {
            Kind::Live(kind) => Some(kind.flagged),
            Kind::New(_) | Kind::Repo(_) => None,
        }
    }

    /// Return whether this entry represents a currently live tmux session.
    pub fn is_live(&self) -> bool {
        matches!(&self.0, Kind::Live(_))
    }

    /// Return the session name.
    pub fn name(&self) -> String {
        match &self.0 {
            Kind::Live(kind) => kind.name(),
            Kind::New(kind) => kind.name(),
            Kind::Repo(kind) => kind.name(),
        }
    }

    /// Return the repository attached to this session, if any.
    pub fn repo(&self) -> Option<PathBuf> {
        match &self.0 {
            Kind::Live(kind) => kind.repo(),
            Kind::New(kind) => kind.repo(),
            Kind::Repo(kind) => kind.repo(),
        }
    }

    /// Set this live session's persisted manual flag idempotently.
    pub async fn set_flag(&self, flagged: bool) -> anyhow::Result<()> {
        match &self.0 {
            Kind::Live(kind) => kind.set_flag(flagged).await,
            Kind::New(_) | Kind::Repo(_) => Ok(()),
        }
    }

    /// Switch the current tmux client to this session, creating the session first if needed.
    pub async fn switch(&self, cwd: &Path, setup: &str) -> anyhow::Result<()> {
        self.create(cwd, setup).await?;
        tmux::switch_client(&self.switch_target()).await
    }

    /// Toggle this session's persisted manual flag.
    pub async fn toggle_flag(&self) -> anyhow::Result<()> {
        match &self.0 {
            Kind::Live(kind) => kind.toggle_flag().await,
            Kind::New(_) | Kind::Repo(_) => Ok(()),
        }
    }

    /// Return lifecycle state counts for agents in this live session.
    pub(crate) fn agents(&self) -> Option<&BTreeMap<AgentState, usize>> {
        match &self.0 {
            Kind::Live(kind) => Some(&kind.agents),
            Kind::New(_) | Kind::Repo(_) => None,
        }
    }

    /// Return windows with a bell or agent alert in this live session.
    pub(crate) fn attention_windows(&self) -> Option<&BTreeSet<String>> {
        match &self.0 {
            Kind::Live(kind) => Some(&kind.alerts),
            Kind::New(_) | Kind::Repo(_) => None,
        }
    }

    /// Return the live tmux alert windows for this session, if any.
    pub(crate) fn has_alerts(&self) -> bool {
        matches!(&self.0, Kind::Live(kind) if !kind.alerts.is_empty())
    }

    /// Convert a prospective plain session into a fresh repository candidate when possible.
    ///
    /// Recheck its name against `sessions` and existing paths below `root`. Return sessions with a
    /// repository context, live sessions, and existing checkouts unchanged.
    pub(crate) fn try_into_new_repo(mut self, root: &Path, sessions: &BTreeSet<String>) -> Self {
        let Kind::New(kind) = &mut self.0 else {
            return self;
        };

        if !matches!(kind.base, Base::Cwd(None)) {
            return self;
        }

        kind.base = Base::NewRepo(root.to_owned());
        kind.disambiguate(sessions, &BTreeSet::new());
        self
    }

    /// Return the repository whose log should be shown in the preview pane.
    pub(crate) fn preview_repo(&self) -> Option<PathBuf> {
        match &self.0 {
            Kind::Live(kind) => kind.repo(),
            Kind::New(kind) => kind.preview_repo(),
            Kind::Repo(kind) => kind.repo(),
        }
    }

    /// Ensure the tmux session we are switching to is ready.
    async fn ensure_tmux(&self, cwd: &Path, setup: &str) -> anyhow::Result<()> {
        match &self.0 {
            Kind::Live(_) => Ok(()),
            Kind::New(kind) => kind.ensure_tmux(cwd, setup).await,
            Kind::Repo(kind) => kind.ensure_tmux(setup).await,
        }
    }

    /// Return the tmux target for switching to this session.
    fn switch_target(&self) -> String {
        let session = self.name();
        let Kind::Live(LiveKind { alerts, .. }) = &self.0 else {
            return session;
        };

        if let Some(window) = alerts.first() {
            format!("{session}:{window}")
        } else {
            session
        }
    }

    /// Return the checkout and exact jj workspace name deleted by this session.
    fn workspace(&self) -> Option<(&Path, &str)> {
        match &self.0 {
            Kind::Live(kind) => Some((kind.repo.as_deref()?, kind.workspace.as_deref()?)),
            Kind::New(_) => None,
            Kind::Repo(kind) => Some((&kind.path, kind.workspace.as_deref()?)),
        }
    }
}

impl LiveKind {
    /// Construct a potential session from information extracted from `tmux`.
    ///
    /// `name` is a tmux session name, `repo` is an optional checkout attached as a user-option on
    /// the tmux session, `workspace` is its exact named jj workspace when verified, `agents`
    /// contains lifecycle state counts published by agent harnesses, `alerts` is a set of windows
    /// with a bell or agent alert, and `flagged` indicates whether the session was manually flagged.
    pub(crate) fn new(
        name: String,
        repo: Option<PathBuf>,
        workspace: Option<String>,
        agents: BTreeMap<AgentState, usize>,
        alerts: BTreeSet<String>,
        flagged: bool,
    ) -> Self {
        Self {
            name,
            repo,
            workspace,
            agents,
            alerts,
            flagged,
        }
    }

    /// Close the live tmux session without deleting any attached workspace.
    async fn close(&self) -> anyhow::Result<()> {
        tmux::kill_session(&self.name).await
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn repo(&self) -> Option<PathBuf> {
        self.repo.clone()
    }

    /// Set the persistent tmux user option that stores this session's manual flag.
    async fn set_flag(&self, flagged: bool) -> anyhow::Result<()> {
        tmux::set_flag(&self.name, flagged).await
    }

    /// Toggle the persistent tmux user option that stores this session's manual flag.
    async fn toggle_flag(&self) -> anyhow::Result<()> {
        self.set_flag(!self.flagged).await
    }
}

impl NewKind {
    /// Construct a new potential session from a query and session base.
    pub(crate) fn new(name: &str, base: Base) -> Self {
        Self {
            name: sanitize(name),
            base,
            suffix: None,
        }
    }

    /// Tweak session's suffix until its tmux name is non-empty and its tmux name, workspace name,
    /// and repo path are all unique.
    ///
    /// `sessions` is the list of all tmux sessions found on startup, and `siblings` is the set of
    /// other workspaces associated with the same default repo as this session.
    pub(crate) fn disambiguate(
        &mut self,
        sessions: &BTreeSet<String>,
        workspaces: &BTreeSet<String>,
    ) {
        let mut i = 1;
        while self.name().is_empty()
            || sessions.contains(&self.name())
            || self.workspace().is_some_and(|w| workspaces.contains(&w.1))
            || self.repo().is_some_and(|r| r.exists())
        {
            self.suffix = Some(i.to_string());
            i += 1;
        }
    }

    /// Ensure the checkout backing this new session exists, when it is repo-backed.
    async fn ensure_checkout(&self) -> anyhow::Result<()> {
        let Some(dest) = self.repo() else {
            return Ok(());
        };

        if let Base::NewRepo(root) = &self.base {
            ensure!(!dest.exists(), "repo '{}' already exists", dest.display());
            tokio::fs::create_dir_all(root).await.with_context(|| {
                format!("failed to create repository root '{}'", root.display())
            })?;

            jj::git_init(&dest).await?;
        } else if let Some((default, workspace, revision)) = self.workspace() {
            jj::add_workspace(default, &dest, &workspace, revision).await?;
        };

        Ok(())
    }

    /// Ensure the tmux session for this new session exists.
    async fn ensure_tmux(&self, cwd: &Path, setup: &str) -> anyhow::Result<()> {
        let target = self.name();
        let repo = self.repo();
        let cwd = match &self.base {
            Base::Repo(_) | Base::NewRepo(_) => repo.clone().context("missing repo")?,
            Base::Cwd(Some(cwd)) => cwd.clone(),
            Base::Cwd(None) => cwd.to_owned(),
        };

        self.ensure_checkout().await?;
        tmux::new_session(&target, &cwd).await?;

        if let Some(repo) = self.repo() {
            tmux::set_option(&target, TMUX_REPO_OPTION, &repo).await?;
        }

        tmux::run_shell(&format!("{target}:0"), &cwd, setup).await
    }

    /// The tmux session name for the new session.
    fn name(&self) -> String {
        let base = match &self.base {
            Base::Repo(base) => Some(base.path()),
            Base::NewRepo(_) | Base::Cwd(_) => None,
        };

        workspace_session_name(base, Some(&self.name), self.suffix.as_deref())
    }

    /// The repository whose log should be shown before this session's workspace exists.
    fn preview_repo(&self) -> Option<PathBuf> {
        match &self.base {
            Base::Repo(base) => Some(base.path().to_owned()),
            Base::NewRepo(_) | Base::Cwd(_) => None,
        }
    }

    /// The repository associated with this session. Disambiguation ensures this path does not
    /// collide with an existing repo.
    fn repo(&self) -> Option<PathBuf> {
        match &self.base {
            Base::Repo(_) => {
                let (default, workspace, _) = self.workspace()?;
                Some(default.with_added_extension(&workspace))
            }
            Base::NewRepo(root) => Some(root.join(self.name())),
            Base::Cwd(_) => None,
        }
    }

    /// This session's workspace name. Disambiguation ensures this name does not collide with an
    /// existing workspace name.
    fn workspace(&self) -> Option<(&Path, String, &str)> {
        let Base::Repo(base) = &self.base else {
            return None;
        };

        let mut workspace = self.name.clone();
        if let Some(suffix) = &self.suffix {
            workspace.push_str(DELIM_SUFFIX);
            workspace.push_str(suffix);
        }

        Some((base.path(), workspace, base.revision()))
    }
}

impl Repo {
    /// Package repository information with the default base revision.
    pub(crate) fn new(path: PathBuf) -> Self {
        Self {
            path,
            revision: jj::DEFAULT_BASE_REVSET.to_owned(),
        }
    }

    /// Return the repository context path.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Return the selected base revision expression.
    pub(crate) fn revision(&self) -> &str {
        &self.revision
    }

    /// Return a copy of this repo with the base revision overridden.
    pub(crate) fn with_revision(&self, revision: String) -> Self {
        Self {
            path: self.path.clone(),
            revision,
        }
    }
}

impl RepoKind {
    /// Construct a potential session from an existing repository or workspace checkout.
    pub(crate) fn new(workspace: Option<&str>, default: PathBuf, path: PathBuf) -> Self {
        Self {
            workspace: workspace.map(str::to_owned),
            default,
            path,
            suffix: None,
        }
    }

    /// Tweak the session's `suffix` until `session.name()` does not collide with any live tmux
    /// session names already seen.
    pub(crate) fn disambiguate(&mut self, sessions: &BTreeSet<String>) {
        let mut i = 1;
        while sessions.contains(&self.name()) {
            self.suffix = Some(i.to_string());
            i += 1;
        }
    }

    /// Ensure the tmux session for this repository checkout exists.
    async fn ensure_tmux(&self, setup: &str) -> anyhow::Result<()> {
        let target = self.name();
        tmux::new_session(&target, &self.path).await?;
        tmux::set_option(&target, TMUX_REPO_OPTION, &self.path).await?;
        tmux::run_shell(&format!("{target}:0"), &self.path, setup).await
    }

    /// The tmux session name for a session attached to this existing repo/workspace.
    fn name(&self) -> String {
        workspace_session_name(
            Some(&self.default),
            self.workspace.as_deref().map(sanitize).as_deref(),
            self.suffix.as_deref(),
        )
    }

    /// The repository associated with this session.
    fn repo(&self) -> Option<PathBuf> {
        Some(self.path.clone())
    }
}

impl From<LiveKind> for Session {
    fn from(kind: LiveKind) -> Self {
        Self(Kind::Live(kind))
    }
}

impl From<NewKind> for Session {
    fn from(kind: NewKind) -> Self {
        Self(Kind::New(kind))
    }
}

impl From<RepoKind> for Session {
    fn from(kind: RepoKind) -> Self {
        Self(Kind::Repo(kind))
    }
}

impl Pickable for Session {
    fn text(&self) -> String {
        let Some(repo) = self.repo() else {
            return self.name();
        };

        format!(
            "{:<NAME_WIDTH$} {}",
            self.name(),
            repo.truncated().display()
        )
    }
}

/// Make the name safe for use as a tmux session name and a workspace name.
pub(crate) fn sanitize(name: &str) -> String {
    let strip = |c: char| c.is_control() || [' ', ':', '.', '/', '\\', '-'].contains(&c);
    let mut cs = name.trim_matches(strip).chars().peekable();

    let mut sanitized = String::new();
    while let Some(c) = cs.peek() {
        if !strip(*c) {
            sanitized.push(cs.next().unwrap());
            continue;
        }

        while cs.peek().is_some_and(|c| strip(*c)) {
            cs.next();
        }

        sanitized.push('-')
    }

    sanitized
}

/// Derive a workspace-aware tmux session name.
///
/// Each component is optional, but one of `base` or `workspace` is expected to be present. The
/// resulting name takes the form `{base}/{workspace}~{suffix}`. Each part's prefix is omitted if
/// the part itself is omitted or it is the first part.
fn workspace_session_name(
    base: Option<&Path>,
    workspace: Option<&str>,
    suffix: Option<&str>,
) -> String {
    let mut name = String::new();
    if let Some(base) = base {
        let base = base.file_name().expect("non-canonical");
        let base = sanitize(&base.to_string_lossy());
        name.push_str(&base);
    }

    if let Some(workspace) = workspace {
        if !name.is_empty() {
            name.push_str(DELIM_WORKSPACE);
        }

        name.push_str(workspace);
    }

    if let Some(suffix) = suffix {
        if !name.is_empty() {
            name.push_str(DELIM_SUFFIX);
        }

        name.push_str(suffix);
    }

    name
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn new_plain_sessions_do_not_create_checkouts() {
        let temp = tempdir().unwrap();
        for base in [Base::Cwd(None), Base::Cwd(Some(temp.path().to_owned()))] {
            let session = NewKind::new("plain", base);
            session.ensure_checkout().await.unwrap();
        }

        assert!(fs::read_dir(temp.path()).unwrap().next().is_none());
    }

    #[tokio::test]
    async fn new_repo_checkout_initializes_colocated_repo() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("missing").join("repos");
        let session = NewKind::new("project", Base::NewRepo(root.clone()));

        session.ensure_checkout().await.unwrap();

        assert!(root.join("project").join(".jj").is_dir());
        assert!(root.join("project").join(".git").is_dir());
    }

    #[tokio::test]
    async fn new_repo_checkout_rejects_existing_destination() {
        let temp = tempdir().unwrap();
        let destination = temp.path().join("project");
        fs::create_dir(&destination).unwrap();
        let session = NewKind::new("project", Base::NewRepo(temp.path().to_owned()));

        let error = session.ensure_checkout().await.unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("repo '{}' already exists", destination.display())
        );
        assert!(!destination.join(".jj").exists());
    }

    #[tokio::test]
    async fn new_workspace_checkout_creates_workspace() {
        let temp = tempdir().unwrap();
        let default = temp.path().join("repo");
        let workspace = temp.path().join("repo.feature");
        jj::git_init(&default).await.unwrap();

        let repo = Repo::new(default.clone()).with_revision("@".to_owned());
        let session = NewKind::new("feature", Base::Repo(repo));
        session.ensure_checkout().await.unwrap();

        assert!(workspace.join(".jj").is_dir());
        let workspaces = jj::workspaces(&default).await.unwrap();
        assert!(workspaces.contains_key(&Some("feature".to_owned())));
    }

    #[test]
    fn new_workspace_sessions_derive_names_and_paths() {
        let temp = tempdir().unwrap();
        let default = temp.path().join("repo");
        let session = NewKind::new("feature", Base::Repo(Repo::new(default)));

        assert_eq!(session.name(), "repo/feature");
        assert_eq!(session.repo(), Some(temp.path().join("repo.feature")));
    }

    #[test]
    fn only_plain_candidates_convert_to_new_repos() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let names = BTreeSet::new();
        let plain: Session = NewKind::new("project one", Base::Cwd(None)).into();
        let converted = plain.try_into_new_repo(root, &names);

        assert_eq!(converted.name(), "project-one");
        assert_eq!(converted.repo(), Some(root.join("project-one")));
        assert!(!converted.is_live());
        assert!(!converted.can_delete());

        let ineligible: [Session; 5] = [
            NewKind::new("project", Base::Cwd(Some(root.to_owned()))).into(),
            NewKind::new("project", Base::NewRepo(root.to_owned())).into(),
            NewKind::new("project", Base::Repo(Repo::new(root.to_owned()))).into(),
            RepoKind::new(None, root.to_owned(), root.to_owned()).into(),
            LiveKind::new(
                "project".to_owned(),
                None,
                None,
                BTreeMap::new(),
                BTreeSet::new(),
                false,
            )
            .into(),
        ];

        for session in ineligible {
            assert_eq!(session.clone().try_into_new_repo(root, &names), session);
        }
    }

    #[test]
    fn plain_candidate_conversion_disambiguates_empty_name() {
        let temp = tempdir().unwrap();
        fs::create_dir(temp.path().join("2")).unwrap();
        let names = BTreeSet::from(["1".to_owned(), "3".to_owned()]);
        let mut candidate = NewKind::new("...", Base::Cwd(None));
        candidate.disambiguate(&names, &BTreeSet::new());
        assert_eq!(candidate.name(), "2");

        let session = Session::from(candidate).try_into_new_repo(temp.path(), &names);

        assert_eq!(session.name(), "4");
        assert_eq!(session.repo(), Some(temp.path().join("4")));
    }

    #[test]
    fn plain_candidate_conversion_disambiguates_names_and_paths() {
        let temp = tempdir().unwrap();
        fs::create_dir(temp.path().join("project~1")).unwrap();
        let names = BTreeSet::from(["project".to_owned(), "project~2".to_owned()]);
        let mut candidate = NewKind::new("project", Base::Cwd(None));
        candidate.disambiguate(&names, &BTreeSet::new());
        assert_eq!(candidate.name(), "project~1");

        let session = Session::from(candidate).try_into_new_repo(temp.path(), &names);

        assert_eq!(session.name(), "project~3");
        assert_eq!(session.repo(), Some(temp.path().join("project~3")));
    }

    #[test]
    fn repo_sessions_preserve_exact_workspace_names() {
        let temp = tempdir().unwrap();
        let default = temp.path().join("repo");
        let workspace = temp.path().join("repo.feature-one");
        let session: Session =
            RepoKind::new(Some("feature.one"), default, workspace.clone()).into();

        assert_eq!(
            session.workspace(),
            Some((workspace.as_path(), "feature.one"))
        );
        assert_eq!(session.name(), "repo/feature-one");
    }

    #[test]
    fn workspace_session_names_are_sanitized() {
        let session = NewKind::new(
            "feature: one.two/path\\name\n",
            Base::Repo(Repo::new(PathBuf::from("repo.default"))),
        );

        assert_eq!(session.name(), "repo-default/feature-one-two-path-name");
    }
}
