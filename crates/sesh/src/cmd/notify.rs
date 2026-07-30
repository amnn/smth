// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Configurable best-effort desktop notification delivery for agent attention transitions.

use std::collections::BTreeMap;

use futures::try_join;

use crate::cmd::custom;
use crate::cmd::tmux;
use crate::cmd::tmux::ClientSnapshot;
use crate::config::NotificationConfig;
use crate::model::agent::AgentState;

/// Maximum number of Unicode scalar values retained in notification text.
const MAX_TEXT_CHARS: usize = 160;

/// Send a configured notification for an agent pane unless that pane is focused.
///
/// Delivery is bounded and best-effort at the call site. The configured root command is executed
/// directly as an argument vector; every nested array is recursively rendered as one POSIX shell
/// command argument.
pub async fn send(
    config: &NotificationConfig,
    pane: &str,
    state: AgentState,
    title: Option<&str>,
    summary: Option<&str>,
) -> anyhow::Result<()> {
    if !config.enabled() {
        return Ok(());
    }

    let (clients, socket) = try_join!(tmux::client_snapshot(), tmux::socket_path())?;
    if !should_send(&clients, pane) {
        return Ok(());
    }

    let tty = target_tty(&clients, pane).unwrap_or_default();
    let title = sanitize_text(title.unwrap_or_default());
    let summary = sanitize_text(summary.unwrap_or_default());
    let message = if summary.is_empty() {
        state.to_string()
    } else {
        summary
    };

    let state = state.value();
    let variables = BTreeMap::from([
        ("message", message.as_str()),
        ("pane", pane),
        ("socket", socket.as_str()),
        ("state", state.as_str()),
        ("title", title.as_str()),
        ("tty", tty),
    ]);

    let arguments = config
        .command
        .iter()
        .map(|command| command.render(&variables))
        .collect::<anyhow::Result<Vec<_>>>()?;

    custom::run(&arguments).await
}

/// Normalize Unicode whitespace plus NUL and ESC separators.
///
/// Other non-whitespace control characters are preserved. Truncation retains at most
/// [`MAX_TEXT_CHARS`] Unicode scalar values, including the ellipsis.
fn sanitize_text(text: &str) -> String {
    let words: Vec<_> = text
        .split(|c: char| c.is_whitespace() || matches!(c, '\0' | '\x1b'))
        .filter(|segment| !segment.is_empty())
        .collect();

    let normalized = words.join(" ");
    if normalized.chars().count() <= MAX_TEXT_CHARS {
        return normalized;
    }

    let mut truncated: String = normalized.chars().take(MAX_TEXT_CHARS - 1).collect();
    truncated.push('…');
    truncated
}

/// Whether no possibly focused client is displaying the agent pane.
fn should_send(snap: &ClientSnapshot, pane: &str) -> bool {
    !snap.clients.iter().any(|c| c.pane == pane && !c.unfocussed)
}

/// Choose the client TTY to focus if the notification is activated.
///
/// Clients are prioritized first by whether they are displaying `pane`, then whether they are
/// possibly focussed, and finally by activity time (newer clients preferred).
fn target_tty<'a>(snap: &'a ClientSnapshot, pane: &str) -> Option<&'a str> {
    snap.clients
        .iter()
        .max_by_key(|c| (c.pane == pane, !c.unfocussed, c.activity))
        .map(|c| c.tty.as_str())
}

#[cfg(test)]
mod tests {
    use crate::cmd::tmux::ClientInfo;

    use super::*;

    fn client(activity: u64, pane: &str, tty: &str, unfocussed: bool) -> ClientInfo {
        ClientInfo {
            activity,
            pane: pane.to_owned(),
            tty: tty.to_owned(),
            unfocussed,
        }
    }

    #[test]
    fn conservatively_suppresses_when_a_displayed_client_is_not_unfocussed() {
        let snapshot = ClientSnapshot {
            clients: vec![client(1, "%7", "/dev/ttys001", false)],
        };

        assert!(!should_send(&snapshot, "%7"));
    }

    #[test]
    fn notifies_without_a_client_for_detached_sessions() {
        let snapshot = ClientSnapshot { clients: vec![] };

        assert!(should_send(&snapshot, "%7"));
        assert_eq!(target_tty(&snapshot, "%7"), None);
    }

    #[test]
    fn reliable_focus_events_allow_notifications_for_background_terminals() {
        let snapshot = ClientSnapshot {
            clients: vec![client(1, "%7", "/dev/ttys001", true)],
        };

        assert!(should_send(&snapshot, "%7"));
        assert_eq!(target_tty(&snapshot, "%7"), Some("/dev/ttys001"));
    }

    #[test]
    fn reliable_focus_or_uncertainty_suppresses_only_for_the_displayed_pane() {
        let snapshot = ClientSnapshot {
            clients: vec![client(1, "%7", "/dev/ttys001", false)],
        };

        assert!(!should_send(&snapshot, "%7"));
        assert!(should_send(&snapshot, "%8"));
        assert_eq!(target_tty(&snapshot, "%8"), Some("/dev/ttys001"));
    }

    #[test]
    fn sanitizes_whitespace_nul_escape_and_long_unicode_text() {
        let text = format!(
            "\u{2003} hello\n\0\x1b world\x07\u{2003} {}",
            "🦀".repeat(MAX_TEXT_CHARS)
        );
        let sanitized = sanitize_text(&text);

        assert!(sanitized.starts_with("hello world\x07 🦀"));
        assert!(sanitized.ends_with('…'));
        assert_eq!(sanitized.chars().count(), MAX_TEXT_CHARS);
        assert!(!sanitized.contains(['\0', '\x1b']));
    }

    #[test]
    fn selects_tty_by_pane_visibility_focus_and_activity() {
        let clients = vec![
            client(10, "%7", "/dev/displaying-old", true),
            client(20, "%7", "/dev/displaying", true),
            client(30, "%1", "/dev/possibly-focused-old", false),
            client(35, "%3", "/dev/possibly-focused", false),
            client(40, "%2", "/dev/newest", true),
        ];
        let snapshot = ClientSnapshot { clients };
        assert_eq!(target_tty(&snapshot, "%7"), Some("/dev/displaying"));

        let snapshot = ClientSnapshot {
            clients: snapshot
                .clients
                .into_iter()
                .filter(|client| client.pane != "%7")
                .collect(),
        };
        assert_eq!(target_tty(&snapshot, "%7"), Some("/dev/possibly-focused"));

        let snapshot = ClientSnapshot {
            clients: snapshot
                .clients
                .into_iter()
                .map(|mut client| {
                    client.unfocussed = true;
                    client
                })
                .collect(),
        };
        assert_eq!(target_tty(&snapshot, "%7"), Some("/dev/newest"));
    }
}
