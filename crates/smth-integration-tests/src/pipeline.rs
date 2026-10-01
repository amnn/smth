// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Concurrent shell stages joined by OS pipes.

use std::io;
use std::io::PipeReader;
use std::process::Output;
use std::process::Stdio;

use anyhow::Context as _;
use futures::future;
use nonempty::NonEmpty;
use tokio::process::Child;

use crate::env::Env;

/// A source directive and its parsed, unexpanded command arguments.
pub(crate) struct Stage<'a> {
    pub(crate) raw: &'a str,
    pub(crate) args: &'a NonEmpty<String>,
}

/// Run every stage concurrently with null stdin for the root and piped stdout between stages.
///
/// Expand and spawn each stage in source order. Return each exit status and stderr in source
/// order, with stdout only on the final output.
/// On setup failure, kill and reap all previously started children before reporting the error.
/// Dropping this future also requests child termination through `kill_on_drop`.
pub(crate) async fn run(env: &Env, stages: &[Stage<'_>]) -> anyhow::Result<Vec<Output>> {
    let mut stdin = None;
    let mut children = Vec::with_capacity(stages.len());
    if let Err(error) = setup(env, stages, &mut stdin, &mut children) {
        drop(stdin);
        for child in &mut children {
            let _ = child.start_kill();
        }

        future::join_all(children.iter_mut().map(|child| child.wait())).await;
        return Err(error);
    }

    // Drain every stderr and the final stdout while all processes are running. Waiting on one
    // child first can deadlock when another fills its stderr pipe or stops reading its stdin.
    future::join_all(children.into_iter().map(|child| child.wait_with_output()))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .context("failed to collect pipeline output")
}

/// Expand and spawn stages in order, retaining started children for cleanup if a later stage fails.
fn setup(
    env: &Env,
    stages: &[Stage<'_>],
    stdin: &mut Option<PipeReader>,
    children: &mut Vec<Child>,
) -> anyhow::Result<()> {
    for (i, stage) in stages.iter().enumerate() {
        let mut command = env.command(&stage.args.head)?;
        command
            .args(env.expand_args(&stage.args.tail)?)
            .stdin(stdin.take().map_or_else(Stdio::null, Stdio::from))
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        if i + 1 == stages.len() {
            command.stdout(Stdio::piped());
        } else {
            let (reader, writer) = io::pipe().context("failed to create pipeline pipe")?;
            *stdin = Some(reader);
            command.stdout(Stdio::from(writer));
        }

        children.push(command.spawn().with_context(|| {
            if i == 0 {
                "failed to execute command".to_owned()
            } else {
                format!("failed to execute pipeline {i}")
            }
        })?);
    }

    Ok(())
}
