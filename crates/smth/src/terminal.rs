// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Terminal state management helpers.

use std::fs::File;
use std::io::IsTerminal as _;
use std::os::fd::AsFd as _;
use std::os::fd::OwnedFd;

use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use rustix::termios::OptionalActions;
use rustix::termios::Termios;
use rustix::termios::tcgetattr;
use rustix::termios::tcsetattr;

/// Enters alternate screen and raw mode on construction and restores the previous state on drop.
pub struct AlternateScreenGuard {
    _raw: RawGuard,
}

/// Sets and restores raw mode, also flushing any pending input before the transition, to prevent
/// "cooked" (not raw) input from being interpreted as raw.
struct RawGuard {
    tty: OwnedFd,
    attrs: Termios,
}

impl AlternateScreenGuard {
    /// Enter the alternate screen and enable raw terminal mode.
    pub fn new() -> anyhow::Result<Self> {
        let _raw = RawGuard::new()?;
        let guard = Self { _raw };
        execute!(std::io::stdout(), EnterAlternateScreen, cursor::Hide)?;
        Ok(guard)
    }
}

impl RawGuard {
    fn new() -> anyhow::Result<Self> {
        // If `stdin` is not a tty, attempts to update its termios attributes will fail. Attempt to
        // direct the attribute changes to the controlling terminal instead. If both these
        // operations fail, then it will not be possible to start up an interactive terminal.
        //
        // This matches `crossterm`'s behaviour.
        let stdin = std::io::stdin();
        let tty = if stdin.is_terminal() {
            stdin.as_fd().try_clone_to_owned()?
        } else {
            File::options()
                .read(true)
                .write(true)
                .open("/dev/tty")?
                .into()
        };

        let attrs = tcgetattr(&tty)?;
        let mut raw = attrs.clone();
        raw.make_raw();

        let guard = Self { tty, attrs };

        // Discard unread input at the mode transition, preserving input received in raw mode.
        tcsetattr(&guard.tty, OptionalActions::Flush, &raw)?;

        // Crossterm caches whether the terminal is in raw mode when it enters this function, so we
        // need to redundantly call it to update that cache.
        terminal::enable_raw_mode()?;

        Ok(guard)
    }
}

impl Drop for AlternateScreenGuard {
    fn drop(&mut self) {
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, cursor::Show);
    }
}

impl Drop for RawGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = tcsetattr(&self.tty, OptionalActions::Now, &self.attrs);
    }
}
