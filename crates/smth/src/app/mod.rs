// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Picker UI state, rendering, and input handling.

mod actions;
mod agent;
mod component;
mod delete;
mod footer;
mod header;
mod highlight;
mod layout;
mod onto;
mod sessions;
mod sort;
mod span;
mod tag;

use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

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

use crate::app::actions::AvailableActions;
use crate::app::component::activity;
use crate::app::component::block::Block;
use crate::app::component::prompt;
use crate::app::component::spinner;
use crate::app::component::spinner::Spinner;
use crate::app::footer::Footer;
use crate::app::header::Header;
use crate::app::sessions::Sessions;
use crate::app::sessions::preview;
use crate::app::sessions::preview::Preview;
use crate::config::Alphabet;
use crate::model::Model;
use crate::model::session::Repo;
use crate::model::session::Session;
use crate::model::tag::Tags;
use crate::terminal::AlternateScreenGuard;

/// Timeout for waiting for a key event.
const POLL_TIMEOUT: Duration = Duration::from_millis(16);

/// Session picker state, caches, and UI behavior.
pub struct App {
    /// Active background activity, including a completed result awaiting handling.
    bg: Option<activity::State<bool>>,

    actions: AvailableActions,
    mode: Mode,
    repo: Option<Repo>,
    spinner: spinner::State,
    model: Model,
    preview: preview::State,
    sessions: sessions::State,
}

/// Runtime inputs used by the interactive picker but not owned by its UI state.
pub struct Context<'a> {
    /// Repository globs to discover alongside existing tmux sessions.
    pub globs: &'a [String],

    /// Parent directory for newly created repositories.
    pub repo_root: &'a Path,

    /// Shell setup to run when creating a tmux session.
    pub setup: &'a str,

    /// Character used to mark live tmux sessions in the picker.
    pub sigil: char,

    /// Display-only alphabet for session tags.
    pub tag_alphabet: &'a Alphabet,
}

/// Completed action chosen from the picker.
enum Action {
    /// Do nothing and exit the picker.
    Cancel,

    /// Close the selected tmux session without deleting any attached workspace.
    Close(Session),

    /// Stage, confirm, or cancel workspace deletions.
    Delete(delete::Action),

    /// Create the selected session without switching to it.
    Create(Session),

    /// Switch to the selected session, creating it first if needed.
    Switch(Session),

    /// Toggle the selected live session's manual flag.
    ToggleFlag(Session),

    /// Apply pending tag changes to the selected session.
    Tag(Tags),
}

/// Mutually exclusive picker editors; staged deletions remain in the model.
enum Mode {
    Normal,
    TagNext,
    TagPrev,
    Onto(Box<onto::State>),
    Sort(sort::State),
    Tag(tag::State),
}

impl App {
    /// Create a new application.
    ///
    /// `repo` is the initial base repository, `revision` optionally overrides its workspace base
    /// revision, and `model` contains the underlying data to drive the interface. The model's
    /// selection should point to the current session, if there is one, or be `None` otherwise.
    ///
    /// Clears that selection and prepares a one-time skip of the current session when it is the
    /// first of multiple matches, making the next best candidate session the default destination.
    pub fn new(repo: Option<PathBuf>, revision: Option<String>, mut model: Model) -> Self {
        // Use the existing selection only for the startup adjustment; let the first draw choose
        // the default row instead of re-anchoring.
        let sessions = sessions::State::new(model.selected.take());
        let mut preview = preview::State::new();
        preview.feed(model.sessions());
        let repo = repo.map(|repo| {
            let repo = model.repo_context(repo);
            if let Some(revision) = revision {
                repo.with_revision(revision)
            } else {
                repo
            }
        });

        Self {
            bg: None,
            actions: AvailableActions::default(),
            mode: Mode::Normal,
            repo,
            spinner: spinner::State::new(),
            model,
            preview,
            sessions,
        }
    }

    /// Run the interactive picker for discovered sessions.
    pub async fn run(mut self, cwd: &Path, ctx: Context<'_>) -> anyhow::Result<()> {
        let _guard = AlternateScreenGuard::new()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

        loop {
            terminal.draw(|frame| self.draw(frame, ctx.sigil, ctx.tag_alphabet, ctx.repo_root))?;

            match self.poll_bg() {
                Some(Err(err)) => return Err(err),
                Some(Ok(true)) => return Ok(()),
                Some(Ok(false)) => {
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

                Some(Action::Create(session)) => {
                    self.model.clear_query();
                    self.sessions.select_first();

                    let cwd = cwd.to_owned();
                    let setup = ctx.setup.to_owned();

                    self.bg = Some(activity::State::new(
                        Span::raw("creating").reset().green(),
                        async move {
                            session.create(&cwd, &setup).await?;
                            Ok(false)
                        },
                    ));
                }

                Some(Action::Delete(delete::Action::Accept)) => {
                    if let Some(current) = &self.repo
                        && self.model.is_deleting(current.path())
                    {
                        self.repo = None;
                    }

                    let delete = self.model.delete();
                    self.bg = Some(activity::State::new(
                        Span::raw("deleting").reset().light_red(),
                        async move {
                            delete.await?;
                            Ok(false)
                        },
                    ));
                }

                Some(Action::Delete(delete::Action::Cancel)) => {
                    self.model.reset_delete().await?;
                    self.discover(ctx.globs).await?;
                }

                Some(Action::Delete(delete::Action::Toggle(path))) => {
                    self.model.toggle_delete(&path).await?;
                    self.discover(ctx.globs).await?;
                }

                Some(Action::Switch(session)) => {
                    let cwd = cwd.to_owned();
                    let setup = ctx.setup.to_owned();

                    self.bg = Some(activity::State::new(
                        Span::raw("switching").reset().yellow(),
                        async move {
                            session.switch(&cwd, &setup).await?;
                            Ok(true)
                        },
                    ));
                }

                Some(Action::Tag(tags)) => {
                    let session = self
                        .model
                        .selected
                        .as_ref()
                        .expect("tag editing requires a selection");
                    let result = session.set_tags(tags).await;
                    self.discover(ctx.globs).await?;
                    result?;
                }

                Some(Action::ToggleFlag(session)) => {
                    session.toggle_flag().await?;
                    self.discover(ctx.globs).await?;
                }
            }
        }
    }

    /// Accept the selected onto-picker commit as the base for new workspaces.
    ///
    /// Reloads the picker when the selected revision can no longer be resolved.
    async fn accept_onto(&mut self) {
        let (Mode::Onto(onto), Some(repo)) =
            (std::mem::replace(&mut self.mode, Mode::Normal), &self.repo)
        else {
            return;
        };

        if let Ok(revision) = onto.accept().await {
            self.repo = Some(repo.with_revision(revision));
        } else {
            self.mode = Mode::Onto(Box::new(onto::State::new(repo.path().to_owned())));
        }
    }

    /// Discover sessions and inject them into the picker.
    async fn discover(&mut self, globs: &[String]) -> anyhow::Result<()> {
        let repo = self.repo.as_ref().map(|r| r.path());
        self.model.discover(globs, repo).await?;
        self.preview.feed(self.model.sessions());
        Ok(())
    }

    /// Draw the UI into the provided frame based on the current application state.
    ///
    /// The frame is split up into regions, each with its own widget. The `preview` region and its
    /// scroll bar are only visible when the preview is toggled on (defaults to visible).
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        sigil: char,
        alphabet: &Alphabet,
        repo_root: &Path,
    ) {
        let onto = matches!(self.mode, Mode::Onto(_));
        let l = layout::Layout::new(f.area(), self.preview.visible() || onto);

        let new_sessions = self.model.sessions_for_query(self.repo.as_ref(), repo_root);

        // Refresh once and borrow the picker, sort, and deletion data for this render pass.
        let snapshot = self.model.refresh();
        if !snapshot.query.is_empty() {
            // A supplied startup query expresses the same intent as typing it.
            self.sessions.key_pressed();
        }

        let order = if let Mode::Sort(state) = &self.mode {
            &state.pending
        } else {
            snapshot.order
        };

        let items = order.matches(snapshot.matches);

        let (label, query) = if let Mode::Onto(onto) = &self.mode {
            ("onto", onto.query())
        } else {
            ("session", snapshot.query)
        };

        // (1) Render picker state.
        f.render_widget(prompt::widget(label, query), l.prompt);
        f.render_stateful_widget(
            Spinner::new(snapshot.status.running),
            l.loading,
            &mut self.spinner,
        );

        let sessions = Sessions::new(
            sigil,
            alphabet,
            &new_sessions,
            &items,
            order,
            snapshot.matches.pattern().column_pattern(0),
            snapshot.delete,
        );

        // (2) Render session list, updating its row index and the model's shared selection
        // from the same matcher snapshot.
        sessions.draw(
            f,
            l.sessions,
            l.scroll,
            &mut self.sessions,
            snapshot.selected,
            snapshot.status.running,
        );

        if let Some(state) = &mut self.bg {
            f.render_stateful_widget(activity::Activity::new(), l.sessions, state);
        }

        // (2.a) Ensure the currently selected session is fed into the preview cache. Most sessions
        // have already been fed to preview during discovery and this will do nothing, but if the
        // selected row corresponds to the new session, then its repo may not have been fed to
        // preview yet.
        self.preview.feed(snapshot.selected.as_ref());

        let header = Header::new(
            snapshot.agents,
            items.len(),
            self.repo.as_ref(),
            snapshot.matches.item_count() as usize,
            order,
            if let Mode::Sort(state) = &self.mode {
                Some(state.selected)
            } else {
                None
            },
        );

        let footer = Footer::new(
            snapshot.delete,
            &items,
            snapshot.selected.as_ref(),
            &self.mode,
            alphabet,
        );

        self.actions = AvailableActions::new(
            snapshot.selected.as_ref(),
            self.bg.as_ref().is_some_and(activity::State::is_loading),
            &self.mode,
            snapshot.delete,
        );

        // (3) Render context and actions after the session list updates the selected session.
        header.draw(f, l.header);
        footer.draw(f, l.footer, &self.actions);

        let Some(l_preview) = l.preview else {
            return;
        };

        if let Some(separator) = l.separator {
            f.render_widget(Block::new('─'), separator);
        }

        // (4) Render the selected session preview or current-repo onto-picker surface, if it is
        // toggled on.
        if let Mode::Onto(onto) = &mut self.mode {
            onto.draw(f, l_preview);
        } else {
            let preview = Preview::new(snapshot.selected.as_ref());
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

        self.sessions.key_pressed();

        let alt = key.modifiers.contains(ALT);
        let ctrl = key.modifiers.contains(CTRL);
        let shift = key.modifiers.contains(SHIFT);

        // App exit takes precedence over internal modal state, but not an active mutation.
        match key.code {
            KC::Esc if self.actions.exit => return Some(Action::Cancel),
            KC::Char('c') if ctrl && self.actions.exit => return Some(Action::Cancel),
            _ => {}
        }

        match &mut self.mode {
            Mode::Normal => {}

            mode @ (Mode::TagNext | Mode::TagPrev) => {
                if let KC::Char(tag @ ('a'..='z' | '-')) = key.code
                    && key.modifiers.is_empty()
                {
                    let target = tag.to_string().parse().expect("lowercase tag or dash");
                    let destination = if matches!(mode, Mode::TagPrev) {
                        self.model.prev(target)
                    } else {
                        self.model.next(target)
                    };

                    if let Some(destination) = destination {
                        self.model.selected = Some(destination.clone());
                        self.sessions.reanchor();
                    }
                }

                self.mode = Mode::Normal;
                return None;
            }

            Mode::Onto(state) => {
                match state.handle_key(key) {
                    Some(onto::Action::Accept) => self.accept_onto().await,
                    Some(onto::Action::Cancel) => self.mode = Mode::Normal,
                    None => {}
                }

                return None;
            }

            Mode::Sort(state) => {
                match state.handle_key(key) {
                    Some(sort::Action::Accept) => {
                        self.model.order = state.pending.clone();
                        self.mode = Mode::Normal;
                    }

                    Some(sort::Action::Cancel) => self.mode = Mode::Normal,
                    None => {}
                }

                return None;
            }

            Mode::Tag(state) => {
                match state.handle_key(key) {
                    Some(tag::Action::Accept) => {
                        let tags = state.pending;
                        self.mode = Mode::Normal;
                        return Some(Action::Tag(tags));
                    }

                    Some(tag::Action::Cancel) => self.mode = Mode::Normal,
                    None => {}
                }

                return None;
            }
        }

        if self.actions.deleting
            && let Some(action) = delete::handle_key(key)
        {
            return Some(Action::Delete(action));
        }

        match key.code {
            // Accept the selected row, switching to it.
            KC::Enter if self.actions.switch => {
                return self.model.selected.take().map(Action::Switch);
            }

            // Create the selected row without switching.
            KC::Char('n') if ctrl && self.actions.create => {
                return self.model.selected.take().map(Action::Create);
            }

            // Session actions
            KC::Char('x') if ctrl && self.actions.close => {
                return self.model.selected.take().map(Action::Close);
            }

            KC::Char('d') if ctrl && self.actions.delete.is_some() => {
                return self
                    .model
                    .selected
                    .take()
                    .and_then(|session| session.repo())
                    .map(|path| Action::Delete(delete::Action::Toggle(path)));
            }

            KC::Char('f') if ctrl && self.actions.flag.is_some() => {
                return self.model.selected.take().map(Action::ToggleFlag);
            }

            KC::Char('t') if ctrl && self.actions.tag => {
                if let Some(tags) = self.model.selected.as_ref().and_then(Session::tags) {
                    self.mode = Mode::Tag(tag::State::new(tags));
                }
            }

            // Scroll preview
            KC::Up if shift => self.preview.scroll_up(),
            KC::Down if shift => self.preview.scroll_down(),

            // Session list selection
            KC::Up | KC::Char('k') if alt => {
                self.sessions.select_first();
            }

            KC::Down | KC::Char('j') if alt => {
                self.sessions.select_last();
            }

            KC::Up => {
                self.sessions.select_previous();
            }

            KC::Char('k') if ctrl => {
                self.mode = Mode::TagPrev;
            }

            KC::Down => {
                self.sessions.select_next();
            }

            KC::Char('j') if ctrl => {
                self.mode = Mode::TagNext;
            }

            // App state
            KC::Char('o') if ctrl => {
                if let Some(repo) = &self.repo {
                    self.mode = Mode::Onto(Box::new(onto::State::new(repo.path().to_owned())));
                }
            }

            KC::Char('s') if ctrl => {
                self.mode = Mode::Sort(sort::State::new(&self.model.order));
            }

            KC::Char('r') if alt => self.reset_current_repo(),
            KC::Char('r') if ctrl => self.set_current_repo(),

            // View state
            KC::Char('p') if ctrl => self.preview.toggle(),

            // Edit query
            KC::Backspace => self.model.pop_query(),
            KC::Char('u') if ctrl => self.model.clear_query(),
            KC::Char(c) if key.modifiers.is_empty() => self.push_query(c),
            KC::Char(c) if shift => self.push_query(c),

            _ => {}
        };

        None
    }

    /// Take a completed background result, where `true` indicates that the picker should exit.
    fn poll_bg(&mut self) -> Option<anyhow::Result<bool>> {
        let result = self.bg.as_mut()?.take()?;
        self.bg = None;
        Some(result)
    }

    /// Append a character, resetting selection when it starts a new query.
    fn push_query(&mut self, ch: char) {
        if self.model.push_query(ch) {
            self.sessions.reset_selection(&mut self.model.selected);
        }
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
        let repo = self.model.selected.as_ref().and_then(Session::repo);
        self.repo = repo.map(|repo| self.model.repo_context(repo));
    }
}
