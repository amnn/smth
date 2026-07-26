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

use sesh::AGENT_STATE_OPTION;
use sesh::AgentState;
use sesh::cmd::tmux;

/// Arguments for updating agent metadata on the current tmux pane.
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Print help.
    #[arg(short, long, action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Agent tracking action or lifecycle state.
    #[arg(value_parser = state_parser())]
    action: Action,
}

/// Agent lifecycle state to publish, or `None` to stop tracking.
///
/// Naming the type prevents clap's derive from treating the `Option` as an optional argument and
/// unwrapping the custom parser's value type.
type Action = Option<AgentState>;

impl Args {
    /// Apply the requested tracking action or lifecycle state to the pane that invoked `sesh`.
    pub(crate) async fn run(self) -> anyhow::Result<()> {
        let pane = env::var("TMUX_PANE")
            .context("'sesh agent' must be run from inside a tmux pane ($TMUX_PANE is unset)")?;

        tmux::ensure()?;
        if let Some(state) = self.action {
            let value = state.option_value();
            tmux::set_pane_option(&pane, AGENT_STATE_OPTION, &value).await
        } else {
            tmux::unset_pane_option(&pane, AGENT_STATE_OPTION).await
        }
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
