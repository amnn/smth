// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! CLI support for publishing agent lifecycle state to tmux.

use std::env;

use anyhow::Context as _;

use sesh::cmd::tmux;

const TMUX_STATE_OPTION: &str = "@sesh.agent.state";

/// Arguments for updating agent metadata on the current tmux pane.
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Print help.
    #[arg(short, long, action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Agent tracking action or lifecycle state.
    #[arg(value_enum)]
    action: Action,
}

/// Agent tracking actions and lifecycle states that harness integrations can publish.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum Action {
    /// Exit agent tracking and clear its state.
    Exit,

    /// The harness is ready, but no agent run has started.
    Idle,

    /// The agent is actively processing a request.
    Running,

    /// The agent is blocked waiting for user input.
    Waiting,

    /// The most recent agent run completed successfully.
    Succeeded,

    /// The most recent agent run failed.
    Failed,
}

impl Args {
    /// Apply the requested tracking action or lifecycle state to the pane that invoked `sesh`.
    pub(crate) async fn run(self) -> anyhow::Result<()> {
        let pane = env::var("TMUX_PANE")
            .context("'sesh agent' must be run from inside a tmux pane ($TMUX_PANE is unset)")?;

        tmux::ensure()?;
        if let Some(state) = self.action.state() {
            tmux::set_pane_option(&pane, TMUX_STATE_OPTION, state).await
        } else {
            tmux::unset_pane_option(&pane, TMUX_STATE_OPTION).await
        }
    }
}

impl Action {
    /// Return the stable value to store in tmux, or `None` when the option should be removed.
    fn state(self) -> Option<&'static str> {
        match self {
            Self::Exit => None,
            Self::Idle => Some("idle"),
            Self::Running => Some("running"),
            Self::Waiting => Some("waiting"),
            Self::Succeeded => Some("succeeded"),
            Self::Failed => Some("failed"),
        }
    }
}
