// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Picker UI state, rendering, and input handling.

mod agent;
mod component;
mod header;
mod highlight;
mod layout;
mod onto;
mod sessions;
mod span;

use std::io;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context as _;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::style::Stylize as _;
use ratatui::text::Span;

use crate::app::component::activity;
use crate::app::component::block::Block;
use crate::app::component::prompt;
use crate::app::component::spinner;
use crate::app::component::spinner::Spinner;
use crate::app::header::Header;
use crate::app::sessions::Sessions;
use crate::app::sessions::preview;
use crate::app::sessions::preview::Preview;
use crate::cmd::jj;
use crate::cmd::tmux;
use crate::model::Model;
use crate::model::session::Repo;
use crate::model::session::Session;
use crate::terminal::AlternateScreenGuard;

/// Timeout for waiting for a key event.
const POLL_TIMEOUT: Duration = Duration::from_millis(16);

/// Session picker state, caches, and UI behavior.
pub struct App {
    /// Active background activity, including a completed result awaiting handling.
    bg: Option<activity::State<BackgroundOutcome>>,

    onto: Option<onto::State>,
    repo: Option<Repo>,
    spinner: spinner::State,
    model: Model,
    preview: preview::State,
    rename: Option<Rename>,
    sessions: sessions::State,
}

/// Runtime inputs used by the interactive picker but not owned by its UI state.
pub struct Context<'a> {
    /// Repository globs to discover alongside existing tmux sessions.
    pub globs: &'a [String],

    /// Shell setup to run when creating a tmux session.
    pub setup: &'a str,

    /// Character used to mark live tmux sessions in the picker.
    pub sigil: char,
}

/// Completed action chosen from the picker.
enum Action {
    /// Do nothing and exit the picker.
    Cancel,

    /// Close the selected tmux session without deleting any attached workspace.
    Close(Session),

    /// Create the selected session without switching to it.
    Create(Session),

    /// Delete the selected session's attached workspace checkout, closing tmux if live.
    Delete(Session),

    /// Rename the selected live tmux session.
    Rename { current: String, new: String },

    /// Switch to the selected session, creating it first if needed.
    Switch(Session),

    /// Toggle the selected live session's manual flag.
    ToggleFlag(Session),
}

/// State change produced by a completed background activity.
enum BackgroundOutcome {
    /// Refresh the picker and continue running.
    Continue,

    /// Refresh the picker after deleting a repository checkout.
    Deleted(PathBuf),

    /// Exit the picker after switching sessions.
    Exit,
}

/// In-progress rename of a live tmux session.
struct Rename {
    current: String,
    new: String,
}

impl App {
    /// Create a new application.
    ///
    /// `repo` is the initial base repository. `model` contains the underlying data to drive the
    /// interface.
    pub fn new(repo: Option<PathBuf>, model: Model) -> Self {
        let select = model.recently_attached().map(|i| i + 1);
        let mut preview = preview::State::new();
        preview.feed(model.sessions());

        Self {
            bg: None,
            onto: None,
            repo: repo.map(Repo::new),
            spinner: spinner::State::new(),
            model,
            preview,
            rename: None,
            sessions: sessions::State::new(select),
        }
    }

    /// Run the interactive picker for discovered sessions.
    pub async fn run(mut self, cwd: &Path, ctx: Context<'_>) -> anyhow::Result<()> {
        let _guard = AlternateScreenGuard::new()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

        loop {
            terminal.draw(|frame| self.draw(frame, ctx.sigil))?;

            match self.poll_bg() {
                Some(Err(err)) => return Err(err),
                Some(Ok(BackgroundOutcome::Exit)) => return Ok(()),
                Some(Ok(BackgroundOutcome::Deleted(repo))) => {
                    if self
                        .repo
                        .as_ref()
                        .is_some_and(|current| current.source() == repo)
                    {
                        self.repo = None;
                    }
                    self.discover(ctx.globs).await?;
                    continue;
                }
                Some(Ok(BackgroundOutcome::Continue)) => {
                    self.discover(ctx.globs).await?;
                    continue;
                }

                None => {}
            }

            if !event::poll(POLL_TIMEOUT)? {
                continue;
            }

            let Event::Key(key) = event::read()? else {
                continue;
            };

            if key.kind != KeyEventKind::Press {
                continue;
            }

            match self.handle_key(key).await {
                None => {}
                Some(Action::Cancel) => return Ok(()),

                Some(Action::Close(session)) => {
                    session.close().await?;
                    self.discover(ctx.globs).await?;
                }

                Some(Action::Delete(session)) => {
                    let repo = session.repo();
                    let workspace = repo
                        .as_deref()
                        .and_then(|repo| self.model.workspace_name(repo))
                        .map(str::to_owned);

                    self.bg =
                        Some(activity::State::new(
                            Span::raw("deleting").light_red(),
                            async move {
                                delete(repo.clone(), workspace).await?;
                                session.close().await?;
                                Ok(repo.map_or(
                                    BackgroundOutcome::Continue,
                                    BackgroundOutcome::Deleted,
                                ))
                            },
                        ));
                }

                Some(Action::Create(session)) => {
                    self.model.clear_query();
                    self.sessions.select_first();

                    let cwd = cwd.to_owned();
                    let setup = ctx.setup.to_owned();

                    self.bg = Some(activity::State::new(Span::raw("creating"), async move {
                        session.create(&cwd, &setup).await?;
                        Ok(BackgroundOutcome::Continue)
                    }));
                }

                Some(Action::Switch(session)) => {
                    let cwd = cwd.to_owned();
                    let setup = ctx.setup.to_owned();

                    self.bg = Some(activity::State::new(Span::raw("switching"), async move {
                        session.switch(&cwd, &setup).await?;
                        Ok(BackgroundOutcome::Exit)
                    }));
                }

                Some(Action::ToggleFlag(session)) => {
                    session.toggle_flag().await?;
                    self.discover(ctx.globs).await?;
                }

                Some(Action::Rename { current, new }) => {
                    tmux::rename_session(&current, &new).await?;
                    self.model.clear_query();
                    for ch in new.chars() {
                        self.model.push_query(ch);
                    }
                    self.discover(ctx.globs).await?;
                }
            }
        }
    }

    /// Accept the selected onto-picker commit as the base for new workspaces.
    ///
    /// Reloads the picker when the selected revision can no longer be resolved.
    async fn accept_onto(&mut self) {
        let (Some(onto), Some(repo)) = (self.onto.take(), &self.repo) else {
            return;
        };

        if let Ok(revision) = onto.accept().await {
            self.repo = Some(repo.with_revision(revision));
        } else {
            self.onto = Some(onto::State::new(repo.source().to_owned()));
        }
    }

    /// Discover sessions and inject them into the picker.
    async fn discover(&mut self, globs: &[String]) -> anyhow::Result<()> {
        let repo = self.repo.as_ref().map(|r| r.source());
        self.model.discover(globs, repo).await?;
        self.preview.feed(self.model.sessions());
        Ok(())
    }

    /// Draw the UI into the provided frame based on the current application state.
    ///
    /// The frame is split up into regions, each with its own widget. The `preview` region and its
    /// scroll bar are only visible when the preview is toggled on (defaults to visible).
    fn draw(&mut self, f: &mut ratatui::Frame<'_>, sigil: char) {
        let l = layout::Layout::new(f.area(), self.preview.visible() || self.onto.is_some());

        let new_session = self.model.new_session(self.repo.as_ref());
        let agent_summary = self.model.agent_summary();

        // Poll the picker for its latest state, and build the data model.
        let (status, snapshot, query) = self.model.refresh();
        let items: Vec<_> = snapshot.matched_items(..).collect();

        let (label, query) = if let Some(rename) = &self.rename {
            ("rename", rename.new.as_str())
        } else if let Some(onto) = &self.onto {
            ("onto", onto.query())
        } else {
            ("session", query)
        };

        // (1) Render picker state.
        f.render_widget(prompt::widget(label, query), l.prompt);
        f.render_stateful_widget(Spinner::new(status.running), l.loading, &mut self.spinner);

        let sessions = Sessions::new(
            sigil,
            new_session,
            &items,
            snapshot.pattern().column_pattern(0),
        );

        // (2) Render session list. This also updates `self.sessions`, so that the selected index
        // and session are up-to-date and valid.
        sessions.draw(f, l.sessions, l.scroll, &mut self.sessions);

        // (2.a) Ensure the currently selected session is fed into the preview cache. Most sessions
        // have already been fed to preview during discovery and this will do nothing, but if the
        // selected row corresponds to the new session, then its repo may not have been fed to
        // preview yet.
        self.preview.feed(self.sessions.selected());

        // (2.b) Render activity progress over the bottom row of the session list.
        if let (Some(activity), Some(area)) = (&mut self.bg, l.sessions.rows().next_back()) {
            f.render_stateful_widget(activity::Activity::new(), area, activity);
        }

        let header = Header::new(
            agent_summary,
            self.sessions.is_deleting(),
            items.len(),
            self.repo.as_ref(),
            self.sessions.selected(),
            snapshot.item_count() as usize,
        );

        // (3) Render the header, which depends on the currently selected session (so must happen
        // after session list rendering).
        header.draw(f, l.header);

        let Some(l_preview) = l.preview else {
            return;
        };

        if let Some(separator) = l.separator {
            f.render_widget(Block::new('─'), separator);
        }

        // (4) Render the selected session preview or current-repo onto-picker surface, if it is
        // toggled on.
        if let Some(onto) = &mut self.onto {
            onto.draw(f, l_preview);
        } else {
            let preview = Preview::new(self.sessions.preview());
            preview.draw(f, l_preview, &mut self.preview);
        }
    }

    /// Handle a single keyboard event, returning the consequent application action.
    async fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        use KeyCode as KC;
        use KeyModifiers as KM;

        const ALT: KM = KM::ALT;
        const CTRL: KM = KM::CONTROL;
        const SHIFT: KM = KM::SHIFT;

        let is_loading = self.bg.as_ref().is_some_and(activity::State::is_loading);
        let alt = key.modifiers.contains(ALT);
        let ctrl = key.modifiers.contains(CTRL);
        let shift = key.modifiers.contains(SHIFT);

        if let Some(rename) = &mut self.rename {
            match key.code {
                KC::Enter => {
                    let rename = self.rename.take()?;
                    return Some(Action::Rename {
                        current: rename.current,
                        new: rename.new,
                    });
                }

                KC::Esc => self.rename = None,
                KC::Char('c' | 'g') if ctrl => self.rename = None,
                KC::Backspace => {
                    rename.new.pop();
                }
                KC::Char('u') if ctrl => rename.new.clear(),
                KC::Char(c) if key.modifiers.is_empty() => rename.new.push(c),
                KC::Char(c) if shift => rename.new.push(c),
                _ => {}
            }

            return None;
        }

        if self.sessions.is_deleting() {
            self.sessions.reset_delete();

            match key.code {
                KC::Char('y') if ctrl => {
                    return self.sessions.take_selected().map(Action::Delete);
                }

                KC::Esc => return None,
                KC::Char('c') if ctrl => return None,

                _ => {}
            }
        }

        if let Some(onto) = &mut self.onto {
            let action = onto.handle_key(key);
            match action {
                Some(onto::Action::Accept) => self.accept_onto().await,
                Some(onto::Action::Cancel) => self.onto = None,
                None => {}
            }

            return None;
        }

        match key.code {
            // Accept the selected row.
            KC::Enter if !is_loading => return self.sessions.take_selected().map(Action::Switch),

            // Create the selected row without switching.
            KC::Char('n') if ctrl && !is_loading && !self.sessions.is_live() => {
                return self.sessions.take_selected().map(Action::Create);
            }

            // Cancel
            KC::Esc if !is_loading => return Some(Action::Cancel),
            KC::Char('c' | 'g') if ctrl && !is_loading => {
                return Some(Action::Cancel);
            }

            // Session actions
            KC::Char('x') if ctrl && !is_loading && self.sessions.is_live() => {
                return self.sessions.take_selected().map(Action::Close);
            }

            KC::Char('d') if ctrl && !is_loading && self.sessions.can_delete() => {
                self.sessions.start_delete();
            }

            KC::Char('f') if ctrl && !is_loading && self.sessions.can_flag() => {
                return self.sessions.take_selected().map(Action::ToggleFlag);
            }

            KC::Char('e') if ctrl && !is_loading && self.sessions.is_live() => {
                let current = self.sessions.selected()?.name();
                self.rename = Some(Rename {
                    new: current.clone(),
                    current,
                });
            }

            // Scroll preview
            KC::Up if shift => {
                self.preview.scroll_up();
            }

            KC::Down if shift => {
                self.preview.scroll_down();
            }

            // Session list selection
            KC::Up | KC::Char('k') if alt => {
                self.sessions.select_first();
                self.preview.first();
            }

            KC::Down | KC::Char('j') if alt => {
                self.sessions.select_last();
                self.preview.first();
            }

            KC::Up => {
                self.sessions.select_previous();
                self.preview.first();
            }

            KC::Char('k') if ctrl => {
                self.sessions.select_previous();
                self.preview.first();
            }

            KC::Down => {
                self.sessions.select_next();
                self.preview.first();
            }

            KC::Char('j') if ctrl => {
                self.sessions.select_next();
                self.preview.first();
            }

            // App state
            KC::Char('o') if ctrl => {
                if let Some(repo) = &self.repo {
                    self.onto = Some(onto::State::new(repo.source().to_owned()));
                }
            }

            KC::Char('r') if alt => self.reset_current_repo(),
            KC::Char('r') if ctrl => self.set_current_repo(),

            // View state
            KC::Char('p') if ctrl => self.preview.toggle(),

            // Edit query
            KC::Backspace => self.model.pop_query(),
            KC::Char('u') if ctrl => self.model.clear_query(),
            KC::Char(c) if key.modifiers.is_empty() => self.model.push_query(c),
            KC::Char(c) if shift => self.model.push_query(c),

            _ => {}
        };

        None
    }

    /// Take a completed background activity outcome.
    fn poll_bg(&mut self) -> Option<anyhow::Result<BackgroundOutcome>> {
        let result = self.bg.as_mut()?.take()?;
        self.bg = None;
        Some(result)
    }

    /// Clear the current repo.
    fn reset_current_repo(&mut self) {
        self.repo = None;
    }

    /// Set the current repo from the currently selected session.
    ///
    /// If there is no selection, or the selected session has no associated repo, the current repo
    /// is cleared.
    fn set_current_repo(&mut self) {
        self.repo = self
            .sessions
            .selected()
            .and_then(Session::repo)
            .map(Repo::new);
    }
}

/// Delete `repo`, first forgetting its named jj `workspace` when supplied.
///
/// A missing repository is a no-op. Returns an error if forgetting the workspace or removing the
/// checkout fails.
async fn delete(repo: Option<PathBuf>, workspace: Option<String>) -> anyhow::Result<()> {
    let Some(repo) = repo else {
        return Ok(());
    };

    if let Some(name) = workspace {
        jj::forget_workspace(&repo, &name).await?;
    }

    match tokio::fs::remove_dir_all(&repo).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => {
            Err(err).with_context(|| format!("failed to remove repository '{}'", repo.display()))
        }
    }
}
