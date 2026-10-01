// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! A contained environment to run tests in.
//!
//! Each environment is set-up within its own temporary directory, under the cargo target temp
//! directory, which is cleaned up when the environment is dropped. That temporary directory
//! includes a `bin` directory and a `home` directory.
//!
//! Binaries can be added to the environment, and commands can be run under a restricted env (can
//! only search for binaries in its own `bin` directory, current directory is set to `home`).
//!
//! NB. Environment isolation is a convenience to ensure tests are stable, not true isolation.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use anyhow::ensure;
use tokio::fs;
use tokio::join;
use tokio::process::Command;
use which::which;

/// Sandboxed filesystem and process environment for one integration test run.
pub(crate) struct Env {
    dir: tempfile::TempDir,
    manifest_dir: PathBuf,
    envs: BTreeMap<String, OsString>,
    vars: BTreeMap<String, OsString>,
}

impl Env {
    /// Construct a new isolated environment rooted under `tmp`.
    ///
    /// All environment artifacts live under a single temporary root outside the repo checkout, so
    /// commands and tmux panes can't inherit a repository-containing cwd from the test process.
    pub(crate) async fn new(manifest_dir: PathBuf) -> anyhow::Result<Self> {
        let dir = tempfile::tempdir().context("failed to create environment root")?;
        let tmp = |rest: &[&str]| {
            let mut path = dir.path().to_path_buf();
            path.extend(rest);
            path
        };

        let (home, path) = join!(
            fs::create_dir(tmp(&["home"])),
            fs::create_dir(tmp(&["bin"])),
        );

        home.context("failed to create $HOME")?;
        path.context("failed to create $PATH")?;

        fs::write(tmp(&["home", ".shrc"]), "PS1='sh$ '\n")
            .await
            .context("failed to write sh startup config")?;

        let envs = BTreeMap::from([
            ("HOME".into(), tmp(&["home"]).into_os_string()),
            ("PATH".into(), tmp(&["bin"]).into_os_string()),
            ("ENV".into(), tmp(&["home", ".shrc"]).into_os_string()),
            ("LC_CTYPE".into(), "en_US.UTF-8".into()),
            ("SHELL".into(), "/bin/sh".into()),
        ]);

        let env = Self {
            dir,
            manifest_dir,
            envs,
            vars: BTreeMap::new(),
        };

        env.bin("sh").await?;

        Ok(env)
    }

    /// Expand a binary reference and ensure it is available in the environment.
    ///
    /// The binary can either be specified by name (in which case it is fetched from the test's
    /// $PATH), or it can be specified by path, in which case it must exist and be executable.
    ///
    /// The binary is added to the environment's `bin` directory. On Unix systems, it is added by
    /// symlink, on Windows, it is added by hard link, and on other systems it is copied.
    ///
    /// Returns the path to the binary in the environment.
    pub(crate) async fn bin(&self, bin: impl AsRef<OsStr>) -> anyhow::Result<PathBuf> {
        let bin = bin.as_ref();
        self.bin_(&self.expand_arg(bin)?)
            .await
            .with_context(|| format!("failed to add '{}' to environment", bin.display()))
    }

    /// Expand the executable name and start a new command in this environment.
    ///
    /// Use the current environment bindings. The working directory remains the sandbox home,
    /// even if the test overrides or unsets `HOME`.
    pub(crate) fn command(&self, program: impl AsRef<OsStr>) -> anyhow::Result<Command> {
        let mut command = Command::new(self.expand_arg(program)?);

        command
            .env_clear()
            .envs(&self.envs)
            .current_dir(self.path("home"));

        Ok(command)
    }

    /// Expand one operand at its point of use, without reinterpreting directive syntax.
    /// Input and substituted values are decoded lossily; the result contains UTF-8 in an OS string.
    /// Returns an error for an unterminated braced reference.
    pub(crate) fn expand_arg(&self, arg: impl AsRef<OsStr>) -> anyhow::Result<OsString> {
        crate::expansion::expand(&arg.as_ref().to_string_lossy(), |name| {
            self.vars
                .get(name)
                .or_else(|| self.envs.get(name))
                .map(|value| value.to_string_lossy().into_owned())
        })
        .map(OsString::from)
    }

    /// Expand command arguments individually, preserving argument boundaries.
    pub(crate) fn expand_args(&self, args: &[String]) -> anyhow::Result<Vec<OsString>> {
        args.iter().map(|arg| self.expand_arg(arg)).collect()
    }

    /// Set a runner-local binding that shadows exported values only during expansion.
    pub(crate) fn set_local(&mut self, name: String, value: impl Into<OsString>) {
        self.vars.insert(name, value.into());
    }

    /// Set a binding exported to subsequent child processes.
    pub(crate) fn set_env(&mut self, name: String, value: impl Into<OsString>) {
        self.envs.insert(name, value.into());
    }

    /// Remove an exported binding, including a sandbox default.
    pub(crate) fn unset_env(&mut self, name: &str) {
        self.envs.remove(name);
    }

    /// Expand both paths and copy a manifest-relative file into the sandboxed home directory.
    pub(crate) async fn copy_file(
        &self,
        src: impl AsRef<Path>,
        dst: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let src = PathBuf::from(self.expand_arg(src.as_ref().as_os_str())?);
        let dst = PathBuf::from(self.expand_arg(dst.as_ref().as_os_str())?);

        ensure!(
            src.is_relative(),
            "source must be relative to manifest directory"
        );
        ensure!(
            dst.is_relative(),
            "destination must be relative to test's $HOME"
        );

        let src = self.manifest_dir.join(src);
        let dst = self.path("home").join(dst);
        let parent = dst.parent().context("file path must have a parent")?;

        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create '{}'", parent.display()))?;

        fs::copy(src, &dst)
            .await
            .with_context(|| format!("failed to copy '{}'", dst.display()))?;

        Ok(())
    }

    /// Relativize `path` in this environment's context.
    pub(crate) fn path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.dir.path().join(path)
    }

    /// Expand the path and write literal contents beneath the sandboxed home, creating parents.
    pub(crate) async fn write_file(
        &self,
        relative: impl AsRef<Path>,
        contents: &str,
    ) -> anyhow::Result<()> {
        let relative = PathBuf::from(self.expand_arg(relative.as_ref().as_os_str())?);
        ensure!(relative.is_relative(), "file path must be relative");

        let path = self.path("home").join(relative);
        let parent = path.parent().context("file path must have a parent")?;

        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create '{}'", parent.display()))?;

        fs::write(&path, contents)
            .await
            .with_context(|| format!("failed to write '{}'", path.display()))?;

        Ok(())
    }

    async fn bin_(&self, bin: &OsStr) -> anyhow::Result<PathBuf> {
        let source = which(bin)?;
        let name = source
            .file_name()
            .context("missing binary name")?
            .to_str()
            .context("invalid binary name")?
            .to_owned();

        let mut target = self.path("bin");
        target.extend([&name]);

        if !target.exists() {
            link(&source, &target)
                .await
                .context("failed to link binary")?;
        }

        Ok(target)
    }
}

/// Make `source` available at `target`.
async fn link(source: &Path, target: &Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    fs::symlink(source, target).await?;

    #[cfg(windows)]
    fs::hard_link(source, target).await?;

    #[cfg(not(any(unix, windows)))]
    fs::copy(source, target).await?;

    Ok(())
}
