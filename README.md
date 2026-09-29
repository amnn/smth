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
  existing checkout, create a new workspace at `trunk()` or a chosen commit, or
  initialize a fresh colocated Git-backed jj repository.
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
cargo install --locked --git https://github.com/amnn/smth
```

Make sure Cargo's binary directory is on your `$PATH` so tmux can find the
installed `smth` binary.

## Setup

Add to `~/.tmux.conf`:

```tmux
set -g detach-on-destroy off
bind s display-popup -E -w 80% -h 80% -T smth -d "#{pane_current_path}" smth --
bind S choose-tree -s
```

Then reload the tmux configuration:

```sh
tmux source-file ~/.tmux.conf
```

### Next Steps

- [Configure `smth`][cfg], including [repository discovery][repo].
- [Connect agents][agent] so `smth` can track their statuses.
- [Configure notifications][note] to signal when sessions need attention.

[agent]: docs/agent-integration.md
[cfg]: docs/configuration.md
[note]: docs/notifications.md
[repo]: docs/configuration.md#repository-discovery

## Session ordering

If the query is non-empty, the picker offers new session candidates above
discovered sessions. This can be to create a new workspace in the current repo,
or to create a new repo or plain tmux session, based on context. A new repo is
only offered for a valid, non-empty directory name with an unoccupied path.
Directory names are preserved exactly; only tmux names are sanitized and
disambiguated. `--create-repo` applies the same rules to explicit creation,
rejecting missing names, path separators, `.` and `..`, and occupied paths.

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

## Staged workspace deletion

Deletion staging is shared by the CLI and picker. It is stored in each named
workspace's `.jj/.smth-pending-delete` file, so it survives picker exit and is
shared by all sessions attached to the same checkout. Default workspaces and
plain tmux sessions cannot be staged.

In the picker, use `C-d` to stage or unstage the selected checkout, navigate or
filter to select more, then press `C-y` to delete the entire staged selection,
including hidden matches. Counts describe affected session rows; each checkout
is deleted only once. `C-g` cancels onto mode first, then clears staged markers.
Esc and `C-c` exit without clearing them. Mutating actions are disabled while a
background operation runs, and the footer hides their shortcuts.

From the CLI:

```sh
smth --base /path/to/repo --stage-delete feature
smth --base /path/to/repo --unstage-delete feature
smth --repo '/path/to/checkouts/*' --json
smth --repo '/path/to/checkouts/*' --delete-staged
```

Stage and unstage are idempotent. JSON records expose `pending_deletion`, which
lets scripts inspect the selection before acting. `--delete-staged` has no base
context: it rejects both `--base` and `--no-base`, does not infer a base from the
current directory, and executes all staged checkouts found through configured
repository globs, `--repo` globs, and live tmux sessions. Markers in undiscovered
checkouts remain untouched. Keep discovery settings consistent between inspection
and execution; use globs to include checkouts whose sessions have been closed.

Execution forgets each jj workspace, removes its checkout, and closes its
associated discovered live sessions. Workspaces and session closures run
concurrently, and failures are collected after all tasks finish. A failed
operation may already have removed its checkout or closed some sessions; inspect
the reported targets before retrying. With no staged checkouts, execution succeeds
without doing anything. **CLI deletion does not prompt and removes checkout
contents.** `--delete SESSION` still immediately deletes only that explicit target,
without consuming an unrelated staged selection.

## Key bindings

`smth -h` prints brief CLI help. `smth --help` prints complete help, including
all picker key bindings:

| Key | Action |
| --- | --- |
| `C-d` | Toggle the workspace's persisted pending-deletion marker. |
| `C-f` | Flag or unflag a live session. |
| `C-g` | Cancel onto mode, or clear staged deletions. |
| `C-n` | Create the session if necessary without switching to it. |
| `C-o` | Open or cancel the onto revision picker. |
| `C-p` | Toggle the preview pane outside onto mode. |
| `C-r`, `M-r` | Set or reset the current repo. |
| `C-u` | Clear the filter. |
| `C-x` | Close a live session. |
| `C-y` | Delete all staged workspaces, including hidden ones, and close their sessions. |
| `up`, `down`, `C-k`, `C-j` | Move selection by one row. |
| `M-up`, `M-down`, `M-k`, `M-j` | Move selection to the first or last row. |
| `S-up`, `S-down` | Scroll the preview pane up or down. |
| `tab`, `S-tab` | Jump between fuzzy matches in onto mode. |
| `enter` | Accept the onto revision, or switch to the session, creating it if necessary. |
| `esc`, `C-c` | Close the UI without clearing staged deletions. |

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

## Acknowledgements

Credit to [@giacgiuliari][giac] for the name, [`smth`][why-smth].

[giac]: https://github.com/giacgiuliari
[why-smth]: docs/troubleshooting.md#why-is-this-called-smth

## License

`smth` is licensed under the [Apache License 2.0][lic].

[lic]: LICENSE.md
