// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Runtime for parsed markdown integration scripts.

use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::num::NonZeroUsize;
use std::path::Path;
use std::path::PathBuf;
use std::slice;
use std::time::Duration;

use anyhow::Context as _;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::ensure;
use futures::future;
use nonempty::NonEmpty;
use regex::Regex;
use textwrap::Options;
use tokio::time;
use tracing::instrument;

use crate::env::Env;
use crate::parser;
use crate::parser::Key;
use crate::parser::KeyKind;
use crate::parser::Line;
use crate::parser::LineKind;
use crate::pipeline;
use crate::pipeline::Stage;
use crate::svg::Frame;
use crate::svg::Theme;
use crate::tmux::Tmux;

/// Integration-test runner state.
pub struct Runner {
    /// Client for managing the `tmux` server. Ordered before `env` so that it is
    /// dropped first and cleans up the server before the environment is cleaned
    /// up, deleting its socket file.
    tmux: Tmux,

    /// Filesystem and process environment used for each test run.
    env: Env,

    /// One-based index of the next `:snap` directive, used to name SVG artifacts.
    snap_ix: usize,

    /// Path to the markdown transcript snapshot. SVG snapshots are written alongside it.
    snapshot_path: PathBuf,
}

/// A bind directive grouped with its preceding shell command or pipeline.
struct Bind<'s> {
    raw: &'s str,
    export: bool,
    name: &'s str,
}

impl Runner {
    /// Construct a runner with an isolated environment and tmux server.
    pub async fn new(
        manifest_dir: impl AsRef<Path>,
        snapshot_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let mut env = Env::new(manifest_dir.as_ref().to_path_buf()).await?;
        let tmux = Tmux::new(&env).await?;
        env.set_env("TMUX".to_owned(), format!("{},,0", tmux.socket().display()));
        env.set_env("TMUX_PANE".to_owned(), tmux.pane());

        Ok(Self {
            tmux,
            env,
            snap_ix: 0,
            snapshot_path: snapshot_path.as_ref().to_path_buf(),
        })
    }

    /// Add a binary to the runner environment's `$PATH`.
    pub async fn bin(&self, bin: impl AsRef<OsStr>) -> anyhow::Result<()> {
        self.env.bin(bin).await?;
        Ok(())
    }

    /// Evaluate a parsed script, grouping each shell command with its pipes and trailing binds.
    pub async fn run(
        &mut self,
        w: &mut impl fmt::Write,
        script: &parser::Script<'_>,
    ) -> fmt::Result {
        let mut lines = script.lines.iter();
        while let Some(line) = lines.next() {
            self.eval_lines(w, line, &mut lines).await?;
        }

        Ok(())
    }

    /// Gracefully shutdown the runner, waiting for all its components to exit.
    pub async fn shutdown(self) -> anyhow::Result<()> {
        self.tmux.shutdown().await?;
        Ok(())
    }

    /// Capture the active pane as styled cells and normalize dynamic spans with `filters`.
    async fn capture_frame(&self, filters: &[parser::Filter]) -> anyhow::Result<Frame> {
        let size = self
            .tmux
            .command("display-message")
            .args(["-p", "#{pane_height} #{pane_width}"])
            .status()
            .await?;

        let size = String::from_utf8_lossy(&size);
        let (rows, cols) = size
            .trim()
            .split_once(' ')
            .context("tmux did not report pane dimensions")?;

        let rows = rows.parse().context("invalid tmux pane height")?;
        let cols = cols.parse().context("invalid tmux pane width")?;

        let output = self
            .tmux
            .capture_pane()
            .await
            .context("failed to capture pane")?;

        Ok(Frame::parse(&output, rows, cols, filters))
    }

    /// Apply grouped binds to the same raw stdout before rendering command output.
    /// Missing output means execution failed, so binds are skipped. Otherwise, individual bind
    /// failures do not stop later binds.
    fn eval_binds(
        &mut self,
        w: &mut impl fmt::Write,
        binds: &[Bind<'_>],
        stdout: Option<&[u8]>,
    ) -> fmt::Result {
        for bind in binds {
            match stdout {
                Some(stdout) => self.eval_bind(w, bind, stdout)?,
                None => writeln!(w, "{} (skipped)", bind.raw)?,
            }
        }
        Ok(())
    }

    /// Resolve requested binaries into the runner environment and report gaps.
    async fn eval_bins(&self, w: &mut impl fmt::Write, raw: &str, args: &[String]) -> fmt::Result {
        write!(w, "{raw}")?;
        let futures = args.iter().map(|arg| self.env.bin(arg));
        let results = future::join_all(futures).await;
        let failures: Vec<_> = args
            .iter()
            .zip(results)
            .filter_map(|(arg, result)| result.err().map(|error| (arg, error)))
            .collect();

        if failures.is_empty() {
            return writeln!(w, " (available)");
        }

        writeln!(w)?;
        for (bin, error) in failures {
            writeln!(w)?;
            let line = format!("'{bin}' is unavailable: {error:#}");
            write_callout(w, "WARNING", &[&line])?;
        }
        Ok(())
    }

    /// Copy a fixture file into the sandboxed home directory.
    async fn eval_copy(
        &self,
        w: &mut impl fmt::Write,
        raw: &str,
        source: &Path,
        path: &Path,
    ) -> fmt::Result {
        write!(w, "{raw}")?;

        if let Err(error) = self.env.copy_file(source, path).await {
            writeln!(w, "\n")?;
            let msg = format!("failed to copy test file: {error:#}");
            write_callout(w, "WARNING", &[&msg])?;
        } else {
            writeln!(w, " (copied)")?;
        }

        Ok(())
    }

    /// Expand text operands and send keys without reinterpreting text as named keys.
    async fn eval_keys(&self, w: &mut impl fmt::Write, raw: &str, keys: &[Key]) -> fmt::Result {
        writeln!(w, "{raw}")?;

        let mut keys = keys.to_vec();
        for key in &mut keys {
            if let KeyKind::Text(text) = &mut key.kind {
                match self.env.expand_arg(&*text) {
                    Ok(expanded) => *text = expanded.to_string_lossy().into_owned(),
                    Err(error) => {
                        let msg = format!("failed to expand key text: {error:#}");

                        writeln!(w)?;
                        return write_callout(w, "WARNING", &[&msg]);
                    }
                }
            }
        }

        // Keep literal text out of tmux's key-name lookup, and preserve ordering when text and
        // named keys are interleaved. Group adjacent keys to avoid a command for every character.
        for keys in keys.chunk_by(|a, b| a.is_literal() == b.is_literal()) {
            let mut command = self.tmux.command("send-keys");
            if keys[0].is_literal() {
                command = command.arg("-l");
            }
            let command = command
                .arg("--")
                .args(keys.iter().map(|k| k.code().into_owned()));

            if let Err(e) = command.status().await {
                let stderr = e.to_string();
                let stderr = stderr.trim();
                let msg = if !stderr.is_empty() {
                    format!("failed to send keys: {stderr}")
                } else {
                    "failed to send keys".to_owned()
                };

                writeln!(w)?;
                return write_callout(w, "WARNING", &[&msg]);
            }
        }

        Ok(())
    }

    /// Evaluate `head`, consuming adjacent pipes and binds from `rest` for shell commands.
    /// Write directives inspect a clone of the remaining lines so their fenced text still renders.
    #[instrument(level = "trace", skip(self, w, head, rest), fields(raw = head.raw))]
    async fn eval_lines(
        &mut self,
        w: &mut impl fmt::Write,
        head: &Line<'_>,
        rest: &mut slice::Iter<'_, Line<'_>>,
    ) -> fmt::Result {
        match &head.kind {
            LineKind::Bind { .. } => {
                writeln!(w, "{} (failed)\n", head.raw)?;
                let msg = "':bind' must immediately follow a completed ':shell', ':pipe', or ':bind' directive";
                write_callout(w, "WARNING", &[msg])?;
            }

            LineKind::Text => {
                writeln!(w, "{}", head.raw)?;
            }

            LineKind::Error { message } => {
                writeln!(w, "{}", head.raw)?;
                writeln!(w)?;
                write_callout(w, "WARNING", &[&format!("Parser error: {message}")])?;
            }

            LineKind::Vars { args } => {
                self.eval_assignments(w, head.raw, false, false, args)?;
            }

            LineKind::Envs { unset, args } => {
                self.eval_assignments(w, head.raw, true, *unset, args)?;
            }

            LineKind::Bins { args } => {
                self.eval_bins(w, head.raw, args).await?;
            }

            LineKind::Pipe { .. } => {
                writeln!(w, "{}\n", head.raw)?;
                let msg = "':pipe' must immediately follow a ':shell' or ':pipe' directive";
                write_callout(w, "WARNING", &[msg])?;
            }

            LineKind::Shell { quiet, args } => {
                let mut stages = vec![Stage {
                    raw: head.raw,
                    args,
                }];

                while let Some(Line { kind, raw }) = peek(rest)
                    && let LineKind::Pipe { args } = kind
                {
                    stages.push(Stage { raw, args });
                    rest.next();
                }

                let mut binds = vec![];
                while let Some(Line { kind, raw }) = peek(rest)
                    && let LineKind::Bind { export, name } = kind
                {
                    let export = *export;
                    binds.push(Bind { raw, export, name });
                    rest.next();
                }

                self.eval_pipeline(w, *quiet, &stages, &binds).await?;
            }

            LineKind::Write { path } => {
                self.eval_write(w, head.raw, path, rest.clone()).await?;
            }

            LineKind::Copy { source, path } => {
                self.eval_copy(w, head.raw, source, path).await?;
            }

            LineKind::Tmux { args } => {
                self.eval_tmux(w, head.raw, args).await?;
            }

            LineKind::Pane { target } => {
                writeln!(w, "{}", head.raw)?;
                self.eval_pane(w, target).await?;
            }

            LineKind::Keys { keys } => {
                self.eval_keys(w, head.raw, keys).await?;
            }

            LineKind::Settle {
                count,
                duration,
                expect,
                filters,
            } => {
                self.eval_settle(w, head.raw, *count, *duration, expect, filters)
                    .await?;
            }

            LineKind::Snap {
                count,
                duration,
                color,
                expect,
                filters,
            } => {
                writeln!(w, "{}", head.raw)?;
                writeln!(w)?;
                self.eval_snap(w, *count, *duration, *color, expect, filters)
                    .await?;
            }
        }

        Ok(())
    }

    /// Apply assignments one at a time so later values see earlier updates.
    fn eval_assignments(
        &mut self,
        w: &mut impl fmt::Write,
        raw: &str,
        export: bool,
        unset: bool,
        args: &[String],
    ) -> fmt::Result {
        writeln!(w, "{raw}")?;
        for arg in args {
            if let Err(error) = self.eval_assignment(export, unset, arg) {
                let msg = format!("invalid assignment '{arg}': {error:#}");

                writeln!(w)?;
                write_callout(w, "WARNING", &[&msg])?;
            }
        }

        Ok(())
    }

    /// Apply one local or exported assignment, or remove an exported binding when `unset`.
    ///
    /// Names remain literal; values expand against current bindings. Invalid names, malformed
    /// assignments or expansions, and values containing NUL fail without changing bindings.
    fn eval_assignment(&mut self, export: bool, unset: bool, arg: &str) -> anyhow::Result<()> {
        let (name, value) = if unset {
            (arg, None)
        } else if let Some((name, value)) = arg.split_once('=') {
            (name, Some(self.env.expand_arg(value)?))
        } else {
            bail!("expected NAME=VALUE");
        };

        self.set_binding(export, name, value)
    }

    /// Bind captured stdout without changing it, allowing consecutive binds of the same output.
    ///
    /// Values containing NUL are rejected for both local and exported bindings, leaving the
    /// destination unchanged.
    fn eval_bind(
        &mut self,
        w: &mut impl fmt::Write,
        bind: &Bind<'_>,
        stdout: &[u8],
    ) -> fmt::Result {
        write!(w, "{}", bind.raw)?;

        let stdout = match std::str::from_utf8(stdout) {
            Ok(stdout) => stdout,
            Err(error) => {
                writeln!(w, " (failed)\n")?;
                let msg = format!("stdout is not valid UTF-8: {error}");
                return write_callout(w, "WARNING", &[&msg]);
            }
        };

        let value = stdout
            .strip_suffix("\r\n")
            .or_else(|| stdout.strip_suffix('\n'))
            .unwrap_or(stdout);

        if let Err(error) = self.set_binding(bind.export, bind.name, Some(value.into())) {
            writeln!(w, " (failed)\n")?;
            return write_callout(w, "WARNING", &[&format!("{error:#}")]);
        }

        writeln!(w, " (bound)")
    }

    /// Switch the runner to a different active pane when the target exists.
    ///
    /// Confirmed selection replaces or restores the `TMUX_PANE` environment binding.
    /// Failed selection leaves that binding unchanged.
    async fn eval_pane(&mut self, w: &mut impl fmt::Write, target: &str) -> fmt::Result {
        let pane = match async { self.target_to_pane_id(&self.env.expand_arg(target)?).await }.await
        {
            Ok(pane) => pane,
            Err(e) => {
                writeln!(w)?;
                let message = format!("failed to validate pane target '{target}': {e}");
                write_callout(w, "WARNING", &[&message])?;
                return Ok(());
            }
        };

        let Some(pane) = pane else {
            writeln!(w)?;
            write_callout(w, "WARNING", &["No such pane."])?;
            return Ok(());
        };

        if let Err(error) = self
            .tmux
            .command("switch-client")
            .args(["-t", &pane])
            .status()
            .await
        {
            writeln!(w)?;
            let message = format!("failed to switch client to pane '{target}': {error}");
            write_callout(w, "WARNING", &[&message])?;
            return Ok(());
        }

        // tmux control-mode subscriptions are throttled, so confirm the switch with an explicit
        // query instead of waiting for the next subscription notification.
        match self.tmux.refresh_pane().await {
            Ok(current) if current == pane => {
                self.env.set_env("TMUX_PANE".to_owned(), current);
            }

            Ok(_) => {
                writeln!(w)?;
                let msg = format!("failed to observe pane target '{target}'");
                write_callout(w, "WARNING", &[&msg])?;
            }

            Err(error) => {
                writeln!(w)?;
                let message = format!("failed to observe pane target '{target}': {error}");
                write_callout(w, "WARNING", &[&message])?;
            }
        }

        Ok(())
    }

    /// Run a shell command and any contiguous pipes, applying binds before rendering output.
    async fn eval_pipeline(
        &mut self,
        w: &mut impl fmt::Write,
        quiet: bool,
        stages: &[Stage<'_>],
        binds: &[Bind<'_>],
    ) -> fmt::Result {
        let outputs = match pipeline::run(&self.env, stages).await {
            Ok(outputs) => outputs,
            Err(error) => {
                for stage in stages {
                    writeln!(w, "{} (failed)", stage.raw)?;
                }

                self.eval_binds(w, binds, None)?;
                writeln!(w)?;
                write_callout(w, "WARNING", &[&format!("{error:#}")])?;

                return Ok(());
            }
        };

        for (stage, output) in stages.iter().zip(&outputs) {
            if let Some(code) = output.status.code() {
                writeln!(w, "{} (exit: {code})", stage.raw)?;
            } else {
                writeln!(w, "{} (exit: killed)", stage.raw)?;
            }
        }

        let stdout = &outputs.last().unwrap().stdout;
        self.eval_binds(w, binds, Some(stdout))?;

        if quiet && outputs.iter().all(|output| output.status.success()) {
            return Ok(());
        }

        if !stdout.is_empty() {
            writeln!(w)?;
            write_fenced_block(w, "stdout", &String::from_utf8_lossy(stdout))?;
        }

        for (index, output) in outputs.iter().enumerate() {
            if output.stderr.is_empty() {
                continue;
            }

            let label = if index == 0 {
                "stderr".to_owned()
            } else {
                format!("stderr-{index}")
            };

            writeln!(w)?;
            write_fenced_block(w, &label, &String::from_utf8_lossy(&output.stderr))?;
        }

        Ok(())
    }

    /// Wait for the pane to reach a settled state within `deadline`. Repeatedly takes snapshots
    /// until `count` consecutive snapshots match and satisfy every screen condition.
    async fn eval_settle(
        &self,
        w: &mut impl fmt::Write,
        raw: &str,
        count: NonZeroUsize,
        duration: Duration,
        expect: &[Regex],
        filters: &[parser::Filter],
    ) -> fmt::Result {
        write!(w, "{raw}")?;
        match self.settle(count, duration, expect, filters).await {
            Ok(_) => writeln!(w, " (settled)"),
            Err(e) => {
                writeln!(w, "\n")?;
                write_callout(w, "WARNING", &[&format!("{e:#}")])
            }
        }
    }

    /// Capture the current pane in a settled state within `deadline`. Repeatedly takes snapshots
    /// until `count` consecutive snapshots match and satisfy every screen condition.
    async fn eval_snap(
        &mut self,
        w: &mut impl fmt::Write,
        count: NonZeroUsize,
        duration: Duration,
        color: bool,
        expect: &[Regex],
        filters: &[parser::Filter],
    ) -> fmt::Result {
        self.snap_ix += 1;
        match self.settle(count, duration, expect, filters).await {
            Ok(frame) => {
                write_fenced_block(w, "terminal", frame.text())?;
                if color {
                    writeln!(w)?;
                    write_svg(w, &self.snapshot_path, self.snap_ix, &frame, Theme::Light)?;
                    write_svg(w, &self.snapshot_path, self.snap_ix, &frame, Theme::Dark)?;
                }
                Ok(())
            }
            Err(e) => write_callout(w, "WARNING", &[&format!("{e:#}")]),
        }
    }

    /// Run a tmux command against the test server and render its outcome.
    async fn eval_tmux(
        &self,
        w: &mut impl fmt::Write,
        raw: &str,
        args: &NonEmpty<String>,
    ) -> fmt::Result {
        write!(w, "{raw}")?;

        let result: anyhow::Result<_> = async {
            let output = self
                .tmux
                .command(self.env.expand_arg(&args.head)?)
                .args(self.env.expand_args(&args.tail)?)
                .status()
                .await?;

            // `%end` is emitted even when a command returns CMD_RETURN_WAIT. Queue a query behind
            // it so host directives cannot run ahead of wait-for or foreground run-shell jobs.
            // This also refreshes the pane context after commands that switch the client.
            self.tmux
                .refresh_pane()
                .await
                .context("failed to synchronize tmux command")?;

            Ok(output)
        }
        .await;

        match result {
            Ok(output) => {
                writeln!(w, " (success)")?;

                let output = String::from_utf8_lossy(&output);
                let output = output.trim();
                if !output.is_empty() {
                    writeln!(w)?;
                    write_fenced_block(w, "", output)?;
                }
            }

            Err(e) => {
                writeln!(w, " (failure)")?;

                let output = e.to_string();
                let output = output.trim();
                if !output.is_empty() {
                    writeln!(w)?;
                    write_fenced_block(w, "", output)?;
                }
            }
        }

        Ok(())
    }

    /// Write a fenced block into the sandboxed home directory.
    async fn eval_write(
        &self,
        w: &mut impl fmt::Write,
        raw: &str,
        path: &Path,
        mut lines: slice::Iter<'_, Line<'_>>,
    ) -> fmt::Result {
        write!(w, "{raw}")?;

        for line in &mut lines {
            if let LineKind::Text = line.kind {
                if line.raw.trim().is_empty() {
                    continue;
                }

                if line.raw.starts_with("```") {
                    break;
                }
            }

            let msg = "Parser error: ':write' expects to be followed by a fenced code block";
            writeln!(w, "\n")?;
            return write_callout(w, "WARNING", &[msg]);
        }

        let mut contents = String::new();
        loop {
            let Some(line) = lines.next() else {
                writeln!(w, "\n")?;
                let msg = "Parser error: unexpected end of input in ':write' block";
                return write_callout(w, "WARNING", &[msg]);
            };

            let LineKind::Text = line.kind else {
                writeln!(w, "\n")?;
                let msg = "Parser error: ':write' expects a fenced code block with text content";
                return write_callout(w, "WARNING", &[msg]);
            };

            if line.raw.starts_with("```") {
                break;
            }

            contents.push_str(line.raw);
            contents.push('\n');
        }

        if let Err(error) = self.env.write_file(path, &contents).await {
            writeln!(w, "\n")?;
            let msg = format!("failed to write file: {error:#}");
            write_callout(w, "WARNING", &[&msg])?;
            return Ok(());
        }

        writeln!(w, " (written)")?;
        Ok(())
    }

    /// Set a local or exported binding, or remove an exported binding when `value` is `None`.
    ///
    /// Names must be nonempty and contain neither `=` nor NUL; values cannot contain NUL.
    /// Values are stored literally, without expansion. Validation failures leave bindings unchanged.
    fn set_binding(
        &mut self,
        export: bool,
        name: &str,
        value: Option<OsString>,
    ) -> anyhow::Result<()> {
        ensure!(
            !name.is_empty() && !name.contains(['=', '\0']),
            "invalid variable name '{name}'"
        );

        if let Some(value) = value {
            ensure!(
                !value.as_encoded_bytes().contains(&0),
                "variable value contains NUL"
            );

            if export {
                self.env.set_env(name.to_owned(), value);
            } else {
                self.env.set_local(name.to_owned(), value);
            }
        } else {
            self.env.unset_env(name);
        }

        Ok(())
    }

    /// Capture the current pane in a settled state within `duration`.
    ///
    /// A settled state implies that a streak of `count` snapshots all observed the same state,
    /// after filters have been applied, each satisfying every expected regex.
    async fn settle(
        &self,
        count: NonZeroUsize,
        duration: Duration,
        expect: &[Regex],
        filters: &[parser::Filter],
    ) -> anyhow::Result<Frame> {
        const INTERVAL: Duration = Duration::from_millis(25);

        let deadline = time::Instant::now() + duration;
        let mut capture = None;
        let mut streak = 0;
        let target = count.get();

        loop {
            let frame = self
                .capture_frame(filters)
                .await
                .context("failed to capture pane")?;
            let pane = frame.text().to_owned();

            match &mut capture {
                _ if expect.iter().any(|pattern| !pattern.is_match(&pane)) => {
                    capture = None;
                    streak = 0;
                }

                _ if pane.trim().is_empty() => {
                    // Ignore empty captures, they usually indicate that tmux hasn't initialized
                    // the pane yet.
                }

                Some(prev) if prev == &pane => {
                    streak += 1;
                }

                Some(prev) => {
                    *prev = pane;
                    streak = 1;
                }

                None => {
                    capture = Some(pane);
                    streak = 1;
                }
            }

            if streak >= target {
                return Ok(frame);
            }

            time::sleep(INTERVAL).await;
            if time::Instant::now() > deadline {
                let duration = duration.as_millis();
                if expect.is_empty() {
                    bail!("pane did not stabilize in {duration}ms");
                } else {
                    let patterns: Vec<_> = expect.iter().map(Regex::as_str).collect();
                    bail!("pane did not stabilize matching all of {patterns:?} in {duration}ms");
                }
            }
        }
    }

    /// Resolve a user-facing pane target into a concrete tmux pane id.
    async fn target_to_pane_id(&self, target: &OsStr) -> anyhow::Result<Option<String>> {
        let output = self
            .tmux
            .command("display-message")
            .args(["-p", "-t"])
            .arg(target)
            .arg("#{pane_id}\t#{session_name}:#{window_index}.#{pane_index}")
            .status()
            .await?;

        let output = String::from_utf8_lossy(&output);
        let candidates: Vec<_> = output
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .collect();

        if let [(pane, _)] = &candidates[..]
            && pane.starts_with('%')
        {
            Ok(Some((*pane).to_owned()))
        } else {
            Ok(None)
        }
    }
}

/// Inspect the next parsed line without advancing the iterator.
fn peek<'a, 's>(lines: &slice::Iter<'a, Line<'s>>) -> Option<&'a Line<'s>> {
    lines.as_slice().first()
}

/// Write a GitHub-style markdown callout, wrapping content to the repo line
/// width.
fn write_callout<S: AsRef<str>>(w: &mut impl fmt::Write, kind: &str, lines: &[S]) -> fmt::Result {
    writeln!(w, "> [!{kind}]")?;

    for line in lines {
        for line in line.as_ref().split('\n') {
            if line.is_empty() {
                writeln!(w, ">")?;
                continue;
            }

            let opts = Options::new(100).break_words(false);
            let wrapped = if let Some(rest) = line.strip_prefix("- ") {
                textwrap::wrap(rest, opts.initial_indent("> - ").subsequent_indent(">   "))
            } else {
                textwrap::wrap(line, opts.initial_indent("> ").subsequent_indent("> "))
            };

            for line in wrapped {
                writeln!(w, "{line}")?;
            }
        }
    }

    Ok(())
}

/// Write a fenced code block, ensuring the block always ends with a trailing
/// newline.
fn write_fenced_block(w: &mut impl fmt::Write, label: &str, text: &str) -> fmt::Result {
    writeln!(w, "```{label}")?;
    write!(w, "{text}")?;

    if !text.ends_with('\n') {
        writeln!(w)?;
    }

    writeln!(w, "```")?;
    Ok(())
}

/// Writes one SVG rendering for a pane snapshot, and embeds it in the transcript.
fn write_svg(
    w: &mut impl fmt::Write,
    snapshot_path: &Path,
    snap_ix: usize,
    frame: &Frame,
    theme: Theme,
) -> fmt::Result {
    let theme_name = match theme {
        Theme::Light => "light",
        Theme::Dark => "dark",
    };

    let mut path = snapshot_path.to_owned();
    path.add_extension(format!("{snap_ix}.{theme_name}.svg"));
    if let Err(e) = fs::write(&path, frame.render_svg(theme)) {
        writeln!(w)?;
        let msg = format!("{:#}", anyhow!(e).context("failed to write SVG snapshot"));
        write_callout(w, "WARNING", &[&msg])?;
    } else {
        let name = path.file_name().expect("SVG path must have a file name");
        writeln!(w, "![{}]({})", theme_name, name.display())?;
    }

    Ok(())
}
