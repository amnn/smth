// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Agent lifecycle state shared by metadata discovery and publishing.

use std::fmt;

use clap::ValueEnum as _;

/// `tmux` pane option used to publish agent lifecycle state.
pub const STATE_OPTION: &str = "@sesh.agent.state";

/// Lifecycle state published by an agent harness.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd, clap::ValueEnum)]
pub enum AgentState {
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

impl AgentState {
    /// Parse a lifecycle state stored in a tmux pane option.
    pub fn parse(value: &str) -> Option<Self> {
        Self::from_str(value, false).ok()
    }

    /// Whether this state indicates that the agent needs attending to by the user to continue
    /// making progress.
    pub fn needs_attention(self) -> bool {
        matches!(self, Self::Waiting | Self::Succeeded | Self::Failed)
    }

    /// Return the stable value stored in the tmux pane option.
    pub fn value(self) -> String {
        self.to_possible_value()
            .expect("agent state variants should have clap values")
            .get_name()
            .to_owned()
    }
}

impl fmt::Display for AgentState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let summary = match self {
            Self::Idle => "Agent is idle",
            Self::Running => "Agent is running",
            Self::Waiting => "Agent is waiting for input",
            Self::Succeeded => "Agent completed successfully",
            Self::Failed => "Agent failed",
        };
        formatter.write_str(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_returns_default_summary() {
        assert_eq!(AgentState::Idle.to_string(), "Agent is idle");
        assert_eq!(AgentState::Running.to_string(), "Agent is running");
        assert_eq!(
            AgentState::Waiting.to_string(),
            "Agent is waiting for input"
        );
        assert_eq!(
            AgentState::Succeeded.to_string(),
            "Agent completed successfully"
        );
        assert_eq!(AgentState::Failed.to_string(), "Agent failed");
    }

    #[test]
    fn option_values_round_trip() {
        for state in AgentState::value_variants() {
            assert_eq!(AgentState::parse(&state.value()), Some(*state));
        }
    }

    #[test]
    fn unknown_option_value_is_ignored() {
        assert_eq!(AgentState::parse("unknown"), None);
    }
}
