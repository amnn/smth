// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! CLI support for publishing agent lifecycle state to tmux.

use std::env;
use std::iter;

use anyhow::Context as _;
use clap::ValueEnum as _;
use clap::builder::PossibleValue;
use clap::builder::PossibleValuesParser;
use clap::builder::TypedValueParser as _;
use tracing::debug;

use sesh::AGENT_STATE_OPTION;
use sesh::AgentState;
use sesh::cmd::notify;
use sesh::cmd::tmux;
use sesh::config::NotificationConfig;

/// Arguments for updating agent metadata on the current tmux pane.
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Print help.
    #[arg(short, long, action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Agent tracking action or lifecycle state.
    #[arg(value_parser = state_parser())]
    action: Action,

    /// Optional one-shot title for a notification triggered by this transition.
    #[arg(long, value_name = "TEXT", allow_hyphen_values = true)]
    title: Option<String>,

    /// Optional one-shot summary for a notification triggered by this transition.
    #[arg(long, value_name = "TEXT", allow_hyphen_values = true)]
    summary: Option<String>,
}

/// Agent lifecycle state to publish, or `None` to stop tracking.
///
/// Naming the type prevents clap's derive from treating the `Option` as an optional argument and
/// unwrapping the custom parser's value type.
type Action = Option<AgentState>;

impl Args {
    /// Apply the requested tracking action or lifecycle state to the pane that invoked `sesh`.
    pub(crate) async fn run(self, config: &NotificationConfig) -> anyhow::Result<()> {
        let pane = env::var("TMUX_PANE")
            .context("'sesh agent' must be run from inside a tmux pane ($TMUX_PANE is unset)")?;

        tmux::ensure()?;
        let Some(state) = self.action else {
            return tmux::unset_pane_option(&pane, AGENT_STATE_OPTION).await;
        };

        // Try to fetch the previous state to detect whether we need to send a notification
        let previous = if config.enabled() && state.needs_attention() {
            tmux::pane_option(&pane, AGENT_STATE_OPTION).await
        } else {
            Ok(None)
        };

        let value = state.value();
        tmux::set_pane_option(&pane, AGENT_STATE_OPTION, &value).await?;

        // A notification is only sent when the state transitions from a non-attention state to an
        // attention state.
        if !state.needs_attention() {
            return Ok(());
        }

        if previous
            .ok()
            .flatten()
            .as_deref()
            .and_then(AgentState::parse)
            .is_some_and(|a| a.needs_attention())
        {
            return Ok(());
        };

        let summary = self.summary.as_deref().or(match state {
            AgentState::Waiting => Some("Agent is waiting for user input"),
            AgentState::Succeeded => Some("Agent run completed successfully"),
            AgentState::Failed => Some("Agent run failed"),
            _ => None,
        });

        if let Err(err) = notify::send(config, &pane, state, self.title.as_deref(), summary).await {
            debug!(?err, pane, "failed to deliver agent notification");
        }

        Ok(())
    }
}

/// Build a parser for clearing agent tracking or publishing a lifecycle state.
fn state_parser() -> impl clap::builder::TypedValueParser<Value = Action> {
    let values = AgentState::value_variants()
        .iter()
        .filter_map(clap::ValueEnum::to_possible_value)
        .chain(iter::once(
            PossibleValue::new("exit").help("The agent has exited"),
        ));

    PossibleValuesParser::new(values)
        .map(|action| (action != "exit").then(|| AgentState::parse(&action).unwrap()))
}
