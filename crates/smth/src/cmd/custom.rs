// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Rendering and execution for recursive user-configured commands.

use std::collections::BTreeMap;
use std::process::Stdio;
use std::time::Duration;

use anyhow::Context as _;
use anyhow::ensure;
use serde::Deserialize;
use serde::Serialize;
use tokio::process::Command;
use tokio::time;

/// Maximum time allowed for a configured command to exit.
const TIMEOUT: Duration = Duration::from_secs(2);

/// One recursively nestable argument in a user-configured command.
///
/// Strings interpolate variables directly. Arrays render their children from the inside out,
/// shell-join the results, and become one argument at their parent level.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Cmd {
    /// One interpolated command argument.
    String(String),

    /// A command rendered as one POSIX shell string after recursively evaluating its arguments.
    Array(Vec<Cmd>),
}

impl Cmd {
    /// Render this argument using arbitrary variable names and recursively quote nested commands.
    pub fn render(&self, variables: &BTreeMap<&str, &str>) -> anyhow::Result<String> {
        match self {
            Self::String(value) => Ok(interpolate(value, variables)),
            Self::Array(nodes) => {
                let values = nodes
                    .iter()
                    .map(|node| node.render(variables))
                    .collect::<anyhow::Result<Vec<_>>>()?;

                shlex::try_join(values.iter().map(|s| s.as_str()))
                    .context("custom command contains a NUL byte")
            }
        }
    }
}

/// Run a rendered custom command directly as an argument vector.
pub async fn run(arguments: &[String]) -> anyhow::Result<()> {
    let Some((program, arguments)) = arguments.split_first() else {
        anyhow::bail!("custom command is empty");
    };

    ensure!(!program.is_empty(), "custom command executable is empty");
    let mut command = Command::new(program);

    command
        .args(arguments)
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let status = time::timeout(TIMEOUT, command.status())
        .await
        .context("custom command timed out")?
        .context("failed to run custom command")?;

    ensure!(status.success(), "custom command exited with {status}");
    Ok(())
}

/// Interpolate variables recognized from the supplied map exactly once.
fn interpolate(template: &str, variables: &BTreeMap<&str, &str>) -> String {
    let mut output = String::with_capacity(template.len());
    let mut cursor = 0;

    while let Some(i) = template[cursor..].find('{') {
        let start = cursor + i;
        output.push_str(&template[cursor..start]);

        let Some(j) = template[start + 1..].find('}') else {
            output.push_str(&template[start..]);
            return output;
        };

        let end = start + j + 1;
        let name = &template[start + 1..end];

        if let Some(value) = variables.get(name) {
            output.push_str(value);
        } else {
            output.push_str(&template[start..=end]);
        }
        cursor = end + 1;
    }

    output.push_str(&template[cursor..]);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct Config {
        command: Vec<Cmd>,
    }

    #[test]
    fn deserializes_nested_commands() {
        let config: Config = toml::from_str(
            r#"
            command = [
              "program",
              "--command",
              ["outer", ["inner", "{arbitrary.name}"]],
            ]
            "#,
        )
        .unwrap();

        assert_eq!(
            config.command,
            vec![
                Cmd::String("program".to_owned()),
                Cmd::String("--command".to_owned()),
                Cmd::Array(vec![
                    Cmd::String("outer".to_owned()),
                    Cmd::Array(vec![
                        Cmd::String("inner".to_owned()),
                        Cmd::String("{arbitrary.name}".to_owned()),
                    ]),
                ]),
            ]
        );
    }

    #[test]
    fn interpolates_arbitrary_names_once_and_preserves_unknown_variables() {
        let variables = BTreeMap::from([
            ("arbitrary.name-1", "literal {other}"),
            ("other", "replacement"),
        ]);
        let command = Cmd::String("{arbitrary.name-1}|{other}|{unknown}|{unterminated".to_owned());

        assert_eq!(
            command.render(&variables).unwrap(),
            "literal {other}|replacement|{unknown}|{unterminated"
        );
    }

    #[test]
    fn quotes_nested_commands_at_every_depth() {
        let message = "it's {pane}; $(not-run)";
        let variables = BTreeMap::from([("anything", message)]);
        let command = Cmd::Array(vec![
            Cmd::String("outer".to_owned()),
            Cmd::Array(vec![
                Cmd::String("middle".to_owned()),
                Cmd::Array(vec![
                    Cmd::String("inner".to_owned()),
                    Cmd::String("{anything}".to_owned()),
                ]),
            ]),
        ]);

        let rendered = command.render(&variables).unwrap();
        let outer = shlex::split(&rendered).unwrap();
        assert_eq!(outer[0], "outer");

        let middle = shlex::split(&outer[1]).unwrap();
        assert_eq!(middle[0], "middle");

        let inner = shlex::split(&middle[1]).unwrap();
        assert_eq!(inner, ["inner", message]);
    }

    #[test]
    fn rejects_non_command_values() {
        assert!(toml::from_str::<Config>("command = [42]").is_err());
    }

    #[test]
    fn rejects_nul_bytes_during_nested_quoting() {
        let command = Cmd::Array(vec![Cmd::String("contains\0nul".to_owned())]);

        let error = command.render(&BTreeMap::new()).unwrap_err();
        assert_eq!(error.to_string(), "custom command contains a NUL byte");
    }
}
