// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Session domain model.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use anyhow::ensure;

use crate::cmd::jj;
use crate::cmd::tmux;
use crate::model::agent::AgentState;
use crate::model::delete;
use crate::model::picker::Pickable;
use crate::model::tag::Tag;
use crate::model::tag::Tags;
use crate::path::TruncatedExt as _;

pub(crate) const DELIM_SUFFIX: &str = "~";
pub(crate) const NAME_WIDTH: usize = 40;

const DELIM_WORKSPACE: &str = "/";
const TMUX_REPO_OPTION: &str = "@smth.repo";

/// A tmux session or potential session.
///
/// Equality and hashing use the kind variant, name, and repository path. Other details, such as
/// agent state, flags, recency, and creation revision, do not change a session's identity.
#[derive(Clone, Debug, Eq)]
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
    tags: Tags,
    recency: Recency,
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

/// Attachment recency, with absent history older than every historical timestamp.
/// Current attachment is newer than every historical timestamp.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Recency {
    Never,
    Historical(u64),
    Attached,
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

    /// Return attachment recency, preserving current attachment even without history.
    pub(crate) fn recency(&self) -> Recency {
        match &self.0 {
            Kind::Live(kind) => kind.recency,
            Kind::New(_) | Kind::Repo(_) => Recency::Never,
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

    /// Set one live session tag idempotently, preserving other memberships.
    ///
    /// Returns an error if this session is not live or updating tmux fails.
    pub async fn set_tag(&self, tag: Tag, enabled: bool) -> anyhow::Result<()> {
        ensure!(self.is_live(), "session is not live");
        tmux::set_tag(&self.name(), tag, enabled).await
    }

    /// Switch the current tmux client to this session, creating the session first if needed.
    pub async fn switch(&self, cwd: &Path, setup: &str) -> anyhow::Result<()> {
        self.create(cwd, setup).await?;
        tmux::switch_client(&self.switch_target()).await
    }

    /// Return tag memberships for live sessions only.
    pub fn tags(&self) -> Option<Tags> {
        match &self.0 {
            Kind::Live(kind) => Some(kind.tags),
            Kind::New(_) | Kind::Repo(_) => None,
        }
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

    /// Return the creation base for a prospective session.
    pub(crate) fn new_base(&self) -> Option<&Base> {
        match &self.0 {
            Kind::New(kind) => Some(&kind.base),
            Kind::Live(_) | Kind::Repo(_) => None,
        }
    }

    /// Return the repository whose log should be shown in the preview pane.
    pub(crate) fn preview_repo(&self) -> Option<PathBuf> {
        match &self.0 {
            Kind::Live(kind) => kind.repo(),
            Kind::New(kind) => kind.preview_repo(),
            Kind::Repo(kind) => kind.repo(),
        }
    }

    /// Return the deletable checkout when its pending-deletion marker exists.
    ///
    /// Missing or inaccessible markers are treated as not pending.
    pub(crate) async fn repo_pending_deletion(&self) -> Option<&Path> {
        let (repo, _) = self.workspace()?;
        tokio::fs::try_exists(repo.join(".jj").join(delete::MARKER))
            .await
            .ok()?
            .then_some(repo)
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

    /// Return this session's verified named-workspace checkout and exact jj workspace name.
    fn workspace(&self) -> Option<(&Path, &str)> {
        match &self.0 {
            Kind::Live(kind) => Some((kind.repo.as_deref()?, kind.workspace.as_deref()?)),
            Kind::New(_) => None,
            Kind::Repo(kind) => Some((&kind.path, kind.workspace.as_deref()?)),
        }
    }
}

impl LiveKind {
    /// Construct a live session from owned tmux metadata and a verified named jj workspace.
    ///
    /// Current attachment is maximally recent, regardless of historical timestamps. Otherwise,
    /// recency uses the last attachment timestamp, or `Never` when history is absent.
    pub(crate) fn new(name: String, workspace: Option<String>, info: tmux::SessionInfo) -> Self {
        Self {
            name,
            repo: info.repo,
            workspace,
            agents: info.agents,
            alerts: info.alerts,
            flagged: info.flagged,
            tags: info.tags,
            recency: if info.attached {
                Recency::Attached
            } else if let Some(timestamp) = info.last_attached {
                Recency::Historical(timestamp)
            } else {
                Recency::Never
            },
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
            name: name.to_owned(),
            base,
            suffix: None,
        }
    }

    /// Tweak the suffix until the tmux name is non-empty and unique. For new workspaces, also
    /// avoid workspace name and checkout path collisions; fresh repository paths never change.
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
            || matches!(self.base, Base::Repo(_)) && self.repo().is_some_and(|r| r.exists())
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
        let (base, name) = match &self.base {
            Base::Cwd(_) => (None, sanitize_tmux(&self.name)),
            Base::NewRepo(_) => (None, sanitize(&self.name)),
            Base::Repo(base) => (Some(base.path()), sanitize(&self.name)),
        };

        session_name(base, Some(&name), self.suffix.as_deref())
    }

    /// The repository whose log should be shown before this session's workspace exists.
    fn preview_repo(&self) -> Option<PathBuf> {
        match &self.base {
            Base::Repo(base) => Some(base.path().to_owned()),
            Base::NewRepo(_) | Base::Cwd(_) => None,
        }
    }

    /// The repository associated with this session. Fresh repositories preserve the requested
    /// directory name; only new workspace paths are disambiguated.
    fn repo(&self) -> Option<PathBuf> {
        match &self.base {
            Base::Repo(_) => {
                let (default, workspace, _) = self.workspace()?;
                Some(default.with_added_extension(&workspace))
            }
            Base::NewRepo(root) => Some(root.join(&self.name)),
            Base::Cwd(_) => None,
        }
    }

    /// This session's workspace name. Disambiguation ensures this name does not collide with an
    /// existing workspace name.
    fn workspace(&self) -> Option<(&Path, String, &str)> {
        let Base::Repo(base) = &self.base else {
            return None;
        };

        let mut workspace = sanitize(&self.name);
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
        session_name(
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

impl Hash for Session {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(&self.0).hash(state);
        self.name().hash(state);
        self.repo().hash(state);
    }
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(&self.0) == mem::discriminant(&other.0)
            && self.name() == other.name()
            && self.repo() == other.repo()
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
    sanitize_tmux(&name.replace('/', "-"))
}

/// Sanitize a tmux-only session name without treating slashes as path separators.
fn sanitize_tmux(name: &str) -> String {
    let strip = |c: char| c.is_control() || [' ', ':', '.', '\\', '-'].contains(&c);
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

/// Format a tmux session name with an optional repository prefix and collision suffix.
///
/// Each component is optional, but one of `base` or `name` is expected to be present. The
/// resulting name takes the form `{base}/{name}~{suffix}`. The base directory name is sanitized;
/// `name` must already be sanitized for its session kind. Each part's prefix is omitted if
/// the part itself is omitted or it is the first part.
fn session_name(base: Option<&Path>, name: Option<&str>, suffix: Option<&str>) -> String {
    let mut result = String::new();
    if let Some(base) = base {
        let base = base.file_name().expect("non-canonical");
        let base = sanitize(&base.to_string_lossy());
        result.push_str(&base);
    }

    if let Some(name) = name {
        if !result.is_empty() {
            result.push_str(DELIM_WORKSPACE);
        }

        result.push_str(name);
    }

    if let Some(suffix) = suffix {
        if !result.is_empty() {
            result.push_str(DELIM_SUFFIX);
        }

        result.push_str(suffix);
    }

    result
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::RandomState;
    use std::fs;
    use std::hash::BuildHasher as _;

    use tempfile::tempdir;

    use super::*;

    /// Assert that two session representations have equal identities and matching hashes.
    macro_rules! assert_same_identity {
        ($left:expr, $right:expr) => {{
            let left: Session = ($left).into();
            let right: Session = ($right).into();
            let hasher = RandomState::new();
            assert_eq!(left, right);
            assert_eq!(hasher.hash_one(&left), hasher.hash_one(&right));
        }};
    }

    /// Construct a live session fixture with no activity or workspace metadata.
    fn live(name: &str, repo: Option<PathBuf>) -> LiveKind {
        LiveKind::new(
            name.to_owned(),
            None,
            tmux::SessionInfo {
                agents: BTreeMap::default(),
                alerts: BTreeSet::default(),
                attached: false,
                flagged: false,
                tags: Tags::default(),
                last_attached: None,
                repo,
            },
        )
    }

    #[test]
    fn equality_distinguishes_kinds() {
        let path = PathBuf::from("project");
        let sessions: [Session; 3] = [
            live("project", Some(path.clone())).into(),
            NewKind::new("project", Base::NewRepo(PathBuf::new())).into(),
            RepoKind::new(None, path.clone(), path.clone()).into(),
        ];

        for (i, left) in sessions.iter().enumerate() {
            assert_eq!(left.name(), "project");
            assert_eq!(left.repo(), Some(path.clone()));
            for (j, right) in sessions.iter().enumerate() {
                assert_eq!(left == right, i == j);
            }
        }
    }

    #[test]
    fn equality_distinguishes_names_and_repositories() {
        let path = PathBuf::from("project");
        let original: Session = live("project", Some(path.clone())).into();
        for different in [
            live("renamed", Some(path)),
            live("project", Some("other".into())),
            live("project", None),
        ] {
            assert_ne!(original, Session::from(different));
        }
    }

    #[test]
    fn equality_ignores_creation_revision() {
        let repo = Repo::new("repo".into());
        assert_same_identity!(
            NewKind::new("feature", Base::Repo(repo.clone())),
            NewKind::new(
                "feature",
                Base::Repo(repo.with_revision("root()".to_owned()))
            )
        );
    }

    #[test]
    fn equality_ignores_live_metadata() {
        let original = live("project", Some("project".into()));
        let mut updated = original.clone();
        updated.workspace = Some("feature".to_owned());
        updated.agents.insert(AgentState::Running, 2);
        updated.alerts.insert("window".to_owned());
        updated.flagged = true;
        updated.tags.set("a".parse().unwrap(), true);
        updated.recency = Recency::Attached;
        assert_same_identity!(original, updated);
    }

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

    /// Only tmux-only names preserve slashes; all bases retain collision suffixes.
    #[test]
    fn new_session_slashes_depend_on_base() {
        let temp = tempdir().unwrap();
        for (base, expected) in [
            (Base::Cwd(None), "foo/bar"),
            (Base::Cwd(Some(temp.path().to_owned())), "foo/bar"),
            (Base::NewRepo(temp.path().to_owned()), "foo-bar"),
            (
                Base::Repo(Repo::new(temp.path().join("repo"))),
                "repo/foo-bar",
            ),
        ] {
            let mut session = NewKind::new("foo/bar", base);
            assert_eq!(session.name(), expected);

            session.disambiguate(&BTreeSet::from([expected.to_owned()]), &BTreeSet::new());
            assert_eq!(session.name(), format!("{expected}~1"));
        }
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
        for (base, name, expected_name, expected_path, expected_workspace) in [
            ("repo", "feature", "repo/feature", "repo.feature", "feature"),
            (
                "repo.default",
                "feature: one.two/path\\name\n",
                "repo-default/feature-one-two-path-name",
                "repo.default.feature-one-two-path-name",
                "feature-one-two-path-name",
            ),
        ] {
            let default = temp.path().join(base);
            let session = NewKind::new(name, Base::Repo(Repo::new(default.clone())));

            assert_eq!(session.name(), expected_name, "{name:?}");
            assert_eq!(
                session.repo(),
                Some(temp.path().join(expected_path)),
                "{name:?}"
            );
            assert_eq!(
                session.workspace(),
                Some((
                    default.as_path(),
                    expected_workspace.to_owned(),
                    jj::DEFAULT_BASE_REVSET
                )),
                "{name:?}"
            );
        }
    }

    /// Only existing markers on deletable checkouts contribute pending paths.
    #[tokio::test]
    async fn repo_pending_deletion_requires_marker_and_named_workspace() {
        let temp = tempdir().unwrap();
        let path = temp.path().to_owned();
        let named: Session = RepoKind::new(Some("feature"), path.clone(), path.clone()).into();
        let default: Session = RepoKind::new(None, path.clone(), path.clone()).into();
        let prospective: Session =
            NewKind::new("feature", Base::Repo(Repo::new(path.clone()))).into();

        assert_eq!(named.repo_pending_deletion().await, None);
        fs::create_dir(path.join(".jj")).unwrap();
        fs::write(path.join(".jj").join(delete::MARKER), "").unwrap();

        assert_eq!(named.repo_pending_deletion().await, Some(path.as_path()));
        assert_eq!(default.repo_pending_deletion().await, None);
        assert_eq!(prospective.repo_pending_deletion().await, None);
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
}
