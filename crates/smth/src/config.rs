// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! User configuration loaded from `smth.toml`.

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use serde::Deserialize;
use serde::Serialize;

use crate::cmd::custom::Cmd;

/// The relative config file path below the `smth` config root.
pub const PATH: &str = "smth.toml";

/// Configuration for desktop notification delivery.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct NotificationConfig {
    /// Whether to emit a terminal bell in the agent pane.
    pub bell: bool,

    /// Recursively evaluated command arguments, with the root executed directly without a shell.
    /// An empty command disables command delivery.
    pub command: Vec<Cmd>,
}

/// Configuration for discovering repositories.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct RepoConfig {
    /// Glob patterns to search for jj repositories, with leading `~` components expanded.
    pub globs: Vec<String>,
}

/// Top-level `smth` config file schema.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct SmthConfig {
    /// Configuration for desktop notification delivery.
    pub notification: NotificationConfig,

    /// Configuration for discovering repositories.
    pub repo: RepoConfig,

    /// Configuration for creating and initializing tmux sessions.
    pub tmux: TmuxConfig,

    /// Configuration for picker rendering.
    pub ui: UiConfig,
}

/// Configuration for creating and initializing tmux sessions.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct TmuxConfig {
    /// Shell script to run after creating a detached tmux session.
    pub setup: String,
}

/// Configuration for picker rendering.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct UiConfig {
    /// Character used to mark live tmux sessions in the picker.
    pub sigil: char,
}

impl NotificationConfig {
    /// Whether at least one notification delivery channel is enabled.
    pub fn enabled(&self) -> bool {
        self.bell || !self.command.is_empty()
    }
}

impl SmthConfig {
    /// Load config from an explicit path, or from the default XDG config location.
    ///
    /// If no explicit path is supplied and the default config file is missing, returns the built-in
    /// default config. An explicit path must exist.
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        let Some(contents) = read_to_string(path)? else {
            return Ok(Self::default());
        };

        toml::from_str(&contents).context("could not parse config")
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { sigil: '⬤' }
    }
}

/// Read config contents from an explicit path or the default config path.
fn read_to_string(path: Option<&Path>) -> anyhow::Result<Option<String>> {
    if let Some(path) = path {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("could not read '{}'", path.display()))?;
        return Ok(Some(contents));
    }

    let root = if let Some(config) = env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(config)
    } else {
        let home = env::var_os("HOME").context("could not find $HOME directory")?;
        PathBuf::from(home).join(".config")
    };

    let path = root.join("smth").join(PATH);
    match fs::read_to_string(&path) {
        Ok(contents) => Ok(Some(contents)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("could not read '{}'", path.display())),
    }
}
