---
name: smth
description: Inspect and safely control smth tmux sessions and jj workspaces through smth's non-interactive CLI. Use when listing, finding, creating, switching, closing, deleting, flagging, or unflagging smth sessions, creating fresh repositories, or managing repo-backed agent workspaces.
license: Apache-2.0
compatibility: Requires the smth binary, tmux, and jj on PATH; switching requires an invoking tmux client.
---

# smth session control

Use `smth` as the control plane for session, workspace, and fresh repository
lifecycle. Do not reimplement these operations with direct `tmux`,
`jj workspace`, or `jj git init` commands: `smth` verifies repository metadata,
preserves collision disambiguation, and runs configured setup hooks.

## Inspect before acting

Run structured inspection before choosing a target:

```sh
smth --json
```

Use `--query TEXT` only to narrow inspection with the picker matcher. An empty
result is `[]`. Each record includes:

- `base`: normalized default-workspace path, omitted for a plain session;
- `name`: named workspace or plain-session operand, omitted for the default
  checkout;
- `path`: actual checkout path, omitted when no checkout exists;
- `tmux`: resolved tmux name, including any collision suffix;
- `live` and `deletable` lifecycle state, plus `flagged` for live sessions;
- `attention`: window names requiring attention, omitted when empty;
- `agents`: agent lifecycle state counts, omitted when empty.

Treat `base` and `name` as the target identity. Never use `--repo` to select
a lifecycle target, infer a repository from `tmux`, or substitute the `tmux`
field for the session operand. To create a new repo-backed workspace that has
no record yet, take `base` from a record in the intended repository family and
use the user's requested workspace name as the operand.

## Build strict commands

For a repo-backed record (`base` is present):

```sh
smth --base "$base" --flag "$name"
smth --base "$base" --unflag "$name"
smth --base "$base" --create "$name"
smth --base "$base" --switch "$name"
smth --base "$base" --close "$name"
smth --base "$base" --delete "$name"
```

Omit the optional operand for the default checkout, where `name` is absent:

```sh
smth --base "$base" --create
smth --base "$base" --switch
smth --base "$base" --close
```

The default checkout cannot be deleted. For a plain record (`base` is absent),
force the plain namespace and pass its session name:

```sh
smth --no-base --create "$name"
smth --no-base --switch "$name"
smth --no-base --close "$name"
smth --no-base --flag "$name"
smth --no-base --unflag "$name"
```

Plain sessions cannot be deleted. Quote paths and names exactly as returned.
Lifecycle commands reject `--query`, `--select-1`, and `--exit-0`; they never
fall back to the TUI.

## Choose the operation

- **Flag or unflag:** require a matching live record. These operations are
  idempotent.
- **Create:** ensure the target exists without changing the current tmux
  client, or create a fresh repository with `--create-repo`. Capture its printed
  tmux name when creating a new or colliding target.
- **Switch:** perform the same creation or reuse operation, then switch to it.
  Existing attention windows are preferred automatically.
- **Close:** require a matching live record. This kills only tmux and preserves
  the checkout and jj workspace registration.
- **Delete:** require a repo-backed named workspace with `deletable: true`.
  This forgets the workspace, removes its checkout, and closes its verified live
  session.

Use `--onto REV` with create or switch only when a missing named workspace
should start at a specific revision. It has no effect on existing checkouts.
After a mutation, run `smth --json` again when subsequent work depends on the
new state.

## Create a fresh repository

Use `--create-repo` when the user wants a new Git-backed jj repository, not a
workspace in an existing repository. It is a no-argument modifier of
`--create [NAME]` or `--switch [NAME]`.

For a detached session:

```sh
smth --no-base --repo-root "$root" --create-repo --create "$name"
```

To create and switch instead:

```sh
smth --no-base --repo-root "$root" --create-repo --switch "$name"
```

Keep `--no-base` to suppress current-directory repository inference. Fresh
creation requires an empty repository context; do not combine it with `--base`
or `--onto`.

`--repo-root` selects the parent directory and overrides `[repo].root`. If
neither is set, the process working directory is used. Relative roots resolve
from that directory; a leading `~` path component expands to the user's home
directory. A missing root is created, but the root itself is never initialized.

The destination is `<repo-root>/<sanitized-name>`. Existing paths and tmux
names are preserved by choosing an available `~N` suffix. Omitted, empty, or
sanitized-empty names use the first available numeric name (`1`, `2`, and so
on). Each request creates a fresh repository rather than reusing or converting
an existing session or checkout. It uses `jj git init --colocate`, starts tmux
in the checkout, and runs `tmux.setup`.

After creation, inspect `smth --json` and use the new record's `base`, with no
`name`, for later lifecycle commands. This is a default checkout, not a
deletable named workspace. Do not repeat `--create-repo` to return to it.
Placement does not enable discovery: use `[repo].globs` or `--repo GLOB` to find
the checkout after its session is closed.

## Deletion confirmation

Deletion is irreversible and the CLI does not prompt. Ask the user for explicit
confirmation immediately before `--delete` unless their current request already
clearly asks to delete or remove that workspace. A request to close, stop, hide,
or leave a session is not permission to delete it; use `--close` instead.
