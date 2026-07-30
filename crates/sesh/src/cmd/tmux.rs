// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Helpers for querying and invoking tmux.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use anyhow::ensure;
use tokio::process::Command;
use tokio::try_join;
use which::which;

use crate::model::agent::AgentState;

/// One eligible interactive tmux client and the pane it currently displays.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientInfo {
    /// Tmux's last-activity timestamp for this client, not for its displayed pane.
    pub activity: u64,

    /// Pane currently displayed by the client.
    pub pane: String,

    /// TTY connecting tmux to the outer terminal.
    pub tty: String,

    /// True exactly when the terminal supports focus reporting, tmux focus events are enabled,
    /// and the client flags do not contain `focused`.
    pub unfocussed: bool,
}

/// All eligible interactive tmux clients captured from one query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientSnapshot {
    /// Eligible clients, excluding control, suspended, and tty-less clients.
    pub clients: Vec<ClientInfo>,
}

/// Metadata for a live tmux session.
#[derive(Debug)]
pub struct SessionInfo {
    /// Agent harnesses running in panes in this session.
    pub agents: BTreeMap<AgentState, usize>,

    /// Windows in the session that have an active bell alert.
    pub alerts: BTreeSet<String>,

    /// Whether a tmux client is currently attached to the session.
    pub attached: bool,

    /// Whether the session has been manually flagged by the user.
    pub flagged: bool,

    /// Time the session was most recently attached to a tmux client.
    pub last_attached: Option<u64>,

    /// Optional jj repository attached to the session.
    pub repo: Option<PathBuf>,
}

/// Query eligible tmux clients and their derived focus state.
pub async fn client_snapshot() -> anyhow::Result<ClientSnapshot> {
    let format = concat!(
        "#{client_activity}\t#{client_control_mode}\t#{client_flags}\t",
        "#{client_termfeatures}\t#{focus-events}\t#{client_tty}\t#{pane_id}",
    );

    let output = Command::new("tmux")
        .args(["list-clients", "-F", format])
        .output()
        .await
        .context("failed to discover tmux clients")?;

    ensure!(
        output.status.success(),
        "error running 'tmux list-clients': {}",
        String::from_utf8_lossy(&output.stderr),
    );

    let clients = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let fields: Vec<_> = line.splitn(7, '\t').collect();
            let [activity, control, flags, features, focus_events, tty, pane] = fields[..] else {
                return None;
            };

            let flags: BTreeSet<_> = flags.split(',').collect();
            let tty = tty.trim();
            if control.trim() == "1" || flags.contains("suspended") || tty.is_empty() {
                return None;
            }

            let supports_focus = features.split(',').any(|feature| feature == "focus");
            Some(ClientInfo {
                activity: activity.trim().parse().unwrap_or_default(),
                pane: pane.trim().to_owned(),
                tty: tty.to_owned(),
                unfocussed: supports_focus
                    && is_flag_set(focus_events)
                    && !flags.contains("focused"),
            })
        })
        .collect();

    Ok(ClientSnapshot { clients })
}

/// Validate that `tmux` is available on `$PATH`.
pub fn ensure() -> anyhow::Result<()> {
    ensure!(which("tmux").is_ok(), "'tmux' not found in PATH");
    Ok(())
}

/// Kill an existing tmux session.
pub async fn kill_session(session: &str) -> anyhow::Result<()> {
    let target = format!("={session}");
    let output = Command::new("tmux")
        .args(["kill-session", "-t", &target])
        .output()
        .await
        .context("failed to kill tmux session")?;

    ensure!(
        output.status.success(),
        "error running 'tmux kill-session': {}",
        String::from_utf8_lossy(&output.stderr),
    );

    Ok(())
}

/// Create a detached tmux session.
pub async fn new_session(session: &str, cwd: &Path) -> anyhow::Result<()> {
    let output = Command::new("tmux")
        .args(["new-session", "-d", "-s", session, "-c"])
        .arg(cwd)
        .output()
        .await
        .context("failed to create tmux session")?;

    ensure!(
        output.status.success(),
        "error running 'tmux new-session': {}",
        String::from_utf8_lossy(&output.stderr),
    );

    Ok(())
}

/// Read a tmux user option from one pane.
pub async fn pane_option(pane: &str, option: &str) -> anyhow::Result<Option<String>> {
    let output = Command::new("tmux")
        .args(["show-options", "-p", "-qv", "-t", pane, option])
        .output()
        .await
        .context("failed to read tmux pane option")?;

    ensure!(
        output.status.success(),
        "error running 'tmux show-options -p': {}",
        String::from_utf8_lossy(&output.stderr),
    );

    let value = String::from_utf8_lossy(&output.stdout);
    let value = value.trim();
    Ok((!value.is_empty()).then(|| value.to_owned()))
}

/// Run a shell script in the context of a target pane.
pub async fn run_shell(target: &str, cwd: &Path, script: &str) -> anyhow::Result<()> {
    let output = Command::new("tmux")
        .args(["run-shell", "-t", target, "-c"])
        .arg(cwd)
        .arg(script)
        .output()
        .await
        .context("failed to run tmux shell command")?;

    ensure!(
        output.status.success(),
        "error running 'tmux run-shell': {}",
        String::from_utf8_lossy(&output.stderr),
    );

    Ok(())
}

/// Query tmux for current sessions, attached sesh metadata, bell alerts, and agent state.
pub async fn sessions() -> anyhow::Result<BTreeMap<String, SessionInfo>> {
    let sessions_format = concat!(
        "#{session_name}\t#{session_attached}\t#{@sesh.flag}\t",
        "#{session_last_attached}\t#{@sesh.repo}",
    );

    let panes_format = concat!(
        "#{session_name}\t#{window_index}\t",
        "#{window_bell_flag}\t#{@sesh.agent.state}",
    );

    let (sessions_output, panes_output) = try_join!(
        Command::new("tmux")
            .args(["list-sessions", "-F", sessions_format])
            .output(),
        Command::new("tmux")
            .args(["list-panes", "-a", "-F", panes_format])
            .output(),
    )
    .context("failed to discover tmux session and pane information")?;

    ensure!(
        sessions_output.status.success(),
        "error running 'tmux list-sessions': {}",
        String::from_utf8_lossy(&sessions_output.stderr),
    );

    ensure!(
        panes_output.status.success(),
        "error running 'tmux list-panes': {}",
        String::from_utf8_lossy(&panes_output.stderr),
    );

    let mut sessions = BTreeMap::new();
    for line in String::from_utf8_lossy(&sessions_output.stdout).lines() {
        let fields: Vec<_> = line.splitn(5, '\t').collect();
        let [session, attached, flag, last_attached, repo] = fields[..] else {
            continue;
        };

        let session = session.trim();
        if session.is_empty() {
            continue;
        }

        let attached: usize = attached.trim().parse().unwrap_or_default();
        let repo = repo.trim();
        let repo = if repo.is_empty() {
            None
        } else {
            Some(PathBuf::from(repo))
        };

        sessions.insert(
            session.to_owned(),
            SessionInfo {
                agents: BTreeMap::new(),
                alerts: BTreeSet::new(),
                attached: attached != 0,
                flagged: is_flag_set(flag),
                last_attached: last_attached.trim().parse().ok(),
                repo,
            },
        );
    }

    for line in String::from_utf8_lossy(&panes_output.stdout).lines() {
        let fields: Vec<_> = line.splitn(4, '\t').collect();
        let [session, window, bell, state] = fields[..] else {
            continue;
        };

        let Some(info) = sessions.get_mut(session.trim()) else {
            continue;
        };

        let window = window.trim();
        if bell.trim() == "1" {
            info.alerts.insert(window.to_owned());
        }

        let Some(state) = AgentState::parse(state.trim()) else {
            continue;
        };

        if state.needs_attention() {
            info.alerts.insert(window.to_owned());
        }

        *info.agents.entry(state).or_default() += 1;
    }

    Ok(sessions)
}

/// Set or clear sesh's manual flag on a tmux session.
pub async fn set_flag(session: &str, flagged: bool) -> anyhow::Result<()> {
    let value = if flagged { "1" } else { "" };
    set_option(session, "@sesh.flag", value).await
}

/// Set a tmux session option.
pub async fn set_option<V: AsRef<OsStr> + ?Sized>(
    session: &str,
    option: &str,
    value: &V,
) -> anyhow::Result<()> {
    let output = Command::new("tmux")
        .args(["set-option", "-t", &format!("={session}:"), option])
        .arg(value)
        .output()
        .await
        .context("failed to set tmux session option")?;

    ensure!(
        output.status.success(),
        "error running 'tmux set-option': {}",
        String::from_utf8_lossy(&output.stderr),
    );

    Ok(())
}

/// Set a tmux pane option.
pub async fn set_pane_option<V: AsRef<OsStr> + ?Sized>(
    pane: &str,
    option: &str,
    value: &V,
) -> anyhow::Result<()> {
    let output = Command::new("tmux")
        .args(["set-option", "-p", "-t", pane, option])
        .arg(value)
        .output()
        .await
        .context("failed to set tmux pane option")?;

    ensure!(
        output.status.success(),
        "error running 'tmux set-option -p': {}",
        String::from_utf8_lossy(&output.stderr),
    );

    Ok(())
}

/// Return the active tmux server socket path.
pub async fn socket_path() -> anyhow::Result<String> {
    let output = Command::new("tmux")
        .args(["display-message", "-p", "#{socket_path}"])
        .output()
        .await
        .context("failed to discover tmux socket path")?;

    ensure!(
        output.status.success(),
        "error running 'tmux display-message': {}",
        String::from_utf8_lossy(&output.stderr),
    );

    let socket = String::from_utf8_lossy(&output.stdout);
    let socket = socket.trim();
    ensure!(!socket.is_empty(), "tmux returned an empty socket path");
    Ok(socket.to_owned())
}

/// Switch the current tmux client to an existing session.
pub async fn switch_client(session: &str) -> anyhow::Result<()> {
    let output = Command::new("tmux")
        .args(["switch-client", "-t", session])
        .output()
        .await
        .context("failed to switch tmux client")?;

    ensure!(
        output.status.success(),
        "error running 'tmux switch-client': {}",
        String::from_utf8_lossy(&output.stderr),
    );

    Ok(())
}

/// Unset a tmux pane option.
pub async fn unset_pane_option(pane: &str, option: &str) -> anyhow::Result<()> {
    let output = Command::new("tmux")
        .args(["set-option", "-p", "-u", "-t", pane, option])
        .output()
        .await
        .context("failed to unset tmux pane option")?;

    ensure!(
        output.status.success(),
        "error running 'tmux set-option -p -u': {}",
        String::from_utf8_lossy(&output.stderr),
    );

    Ok(())
}

/// Return whether a tmux user option value counts as an enabled flag.
fn is_flag_set(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && !matches!(value, "0" | "false" | "no" | "off")
}
