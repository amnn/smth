# smth: switch to something else

[![CI][badge]][ci]

A **tmux**-native session switcher for navigating between sessions and opening
new ones backed by **jujutsu** (jj) repositories and workspaces.

![Opening, filtering, previewing, and switching sessions][demo]

[badge]: https://github.com/amnn/smth/actions/workflows/ci.yml/badge.svg
[ci]: https://github.com/amnn/smth/actions/workflows/ci.yml
[demo]: docs/assets/session-switching.gif

## Features

- **Tmux-native navigation.** Fuzzy-find sessions in a popup, inspect a live
  preview, and jump back to the previous session using [recency-aware
  ordering][ord].
- **First-class jj workflows.** Discover repositories and workspaces, open an
  existing checkout, or create a new workspace at `trunk()` or a chosen commit.
- **Keyboard-first session management.** Create sessions in the background,
  flag them, close them, or delete their associated workspace without leaving
  the picker. See the complete [key bindings][keys].
- **Agent attention at a glance.** [Pi][pi] is currently the only supported
  agent integration. Track its lifecycle state per pane, surface sessions that
  need attention, and optionally send terminal or desktop notifications.
- **Flexible configuration.** Add repository globs, customize new tmux
  sessions, and change the live-session sigil.
- **Scriptable workflows.** Seed queries, filter candidates, or switch
  immediately using [fzf-style startup flags][cli].

[cli]: docs/scripting.md
[keys]: #key-bindings
[ord]: #session-ordering
[pi]: docs/agent-integration.md#pi-extension

## Installation

`smth` expects `tmux` and `jj` to be available on `$PATH`.

Install the latest version from this repository with Cargo:

```sh
cargo install --locked --git https://github.com/amnn/smth --package smth
```

Make sure Cargo's binary directory is on your `$PATH` so tmux can find the
installed `smth` binary.

## Setup

Add to `~/.tmux.conf`:

```tmux
bind s display-popup -E -w 80% -h 80% -T smth -d "#{pane_current_path}" "smth"
bind S choose-tree -s
```

Then reload the tmux configuration:

```sh
tmux source-file ~/.tmux.conf
```

Next:

- [Configure `smth`][cfg], including [repository discovery][repo].
- [Connect agents][agent] so `smth` can track their statuses.
- [Configure notifications][note] to signal when sessions need attention.

[agent]: docs/agent-integration.md
[cfg]: docs/configuration.md
[note]: docs/notifications.md
[repo]: docs/configuration.md#repository-discovery

## Session ordering

Live tmux sessions are ordered by when they were most recently attached to a
tmux client, newest first. Once at least two live sessions have attachment
history, the picker initially selects the second newest so pressing `enter`
returns to the previous session.

`smth` reads tmux's built-in `session_last_attached` value, so switches made
outside `smth` also affect the order. Sessions that have never been attached
follow sessions with attachment history in name order. Inspect the values with:

```sh
tmux list-sessions -F '#{session_name}:#{session_last_attached}'
```

## Key bindings

`smth -h` prints brief CLI help. `smth --help` prints complete help, including
all picker key bindings:

| Key | Action |
| --- | --- |
| `C-d` | Delete the repository and close the session. |
| `C-f` | Flag or unflag a live session. |
| `C-n` | Create the session if necessary without switching to it. |
| `C-o` | Open or cancel the onto revision picker. |
| `C-p` | Toggle the preview pane outside onto mode. |
| `C-r`, `M-r` | Set or reset the current repo. |
| `C-u` | Clear the filter. |
| `C-x` | Close a live session. |
| `C-y` | Confirm a pending deletion. |
| `up`, `down`, `C-k`, `C-j` | Move selection by one row. |
| `M-up`, `M-down`, `M-k`, `M-j` | Move selection to the first or last row. |
| `S-up`, `S-down` | Scroll the preview pane up or down. |
| `tab`, `S-tab` | Jump between fuzzy matches in onto mode. |
| `enter` | Accept the onto revision, or switch to the session, creating it if necessary. |
| `esc`, `C-g`, `C-c` | Cancel onto mode, or close the UI. |

![Flagging a live workspace session][flag]

[flag]: docs/assets/session-flagging.gif

## Troubleshooting

If repository detection, session metadata, flags, or secondary jj workspaces do
not behave as expected, start with the [troubleshooting guide][help].

[help]: docs/troubleshooting.md

## Alternatives

Choose `smth` when you want to keep tmux as the foundation, use jj repositories
and workspaces as the session model, and add pane-scoped agent attention without
adopting a larger terminal or task-orchestration environment.

- Unlike agent-focused terminal environments such as [cmux][cmux],
  [Herdr][herdr], and [Orca][orca], `smth` preserves your existing terminal and
  tmux setup.
- Compared with general tmux session tools such as [sesh][sesh],
  [Tmux Sessionizer][ts], and [Tmuxinator][tmuxi], `smth` adds first-class jj
  workspace creation, revision selection, and repository metadata.
- Compared with [workmux][wm], `smth` uses jj workspaces rather than Git
  worktrees and leaves integration workflows to jj.

[cmux]: https://github.com/manaflow-ai/cmux
[herdr]: https://github.com/herdrdev/herdr
[orca]: https://github.com/stablyai/orca
[sesh]: https://github.com/joshmedeski/sesh
[tmuxi]: https://github.com/tmuxinator/tmuxinator
[ts]: https://github.com/ThePrimeagen/tmux-sessionizer
[wm]: https://github.com/raine/workmux

## Contributing

New contributors should start with a [human-written issue][issue] that explains
the problem or proposed change. Please discuss and agree on an approach there
before opening a pull request.

[issue]: https://github.com/amnn/smth/issues/new

## License

`smth` is licensed under the [Apache License 2.0][lic].

[lic]: LICENSE.md
