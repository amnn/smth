# Troubleshooting

## Why is this called `smth`?

It's a **s**ession **m**anager for **t**mux **h**opping. Or something.

## Tmux closes when the current session is closed

Tmux's `detach-on-destroy` option defaults to `on`, so it detaches the client
when `smth` closes (`C-x`) or deletes (`C-d`) the session the client is attached
to. Add this to `~/.tmux.conf`:

```tmux
set -g detach-on-destroy off
```

With this option disabled, tmux switches the client to the most recently active
remaining session instead. Reload the tmux configuration to apply it:

```sh
tmux source-file ~/.tmux.conf
```

## `smth` does not detect the repository from the current directory

`smth` detects the default repository context from the directory it starts in.
If the picker header does not show the expected `repo: ...` value, check the
path tmux uses when launching `smth`:

```tmux
bind s display-popup -E -w 80% -h 80% -T smth -d "#{pane_current_path}" "smth"
```

The `-d "#{pane_current_path}"` option should be present so tmux starts `smth`
from the pane that was active when you opened the picker.

To confirm the directory is inside a jj workspace, run the same jj check from
that pane:

```sh
cd /path/to/repo/or/subdirectory
jj workspace root
```

If `jj workspace root` fails, switch to a directory inside the jj checkout or
fix the tmux binding so `smth` starts from the active pane's path.

## `smth` does not detect the repository for an existing session

Live tmux sessions only have repository metadata when they were opened by
`smth`, or when you set the `@smth.repo` user option yourself. Plain tmux
sessions still appear in the picker, but `smth` cannot associate them with a jj
repository.

Check the session's repo metadata with:

```sh
tmux show-options -t SESSION -qv @smth.repo
```

The command should print the repository path for repo-backed sessions. Empty
output means `smth` will treat the tmux session as a plain live session.

To fix this, create or open the session through `smth`. If you know the correct
repository path and want to attach it manually, set the user option yourself:

```sh
tmux set-option -t SESSION @smth.repo /path/to/repo
```

## Session flags do not appear as expected

`smth` stores manual flags in the `@smth.flag` tmux user option for each live
session. You can inspect or repair the flag outside the picker with:

```sh
tmux show-options -t SESSION -qv @smth.flag
tmux set-option -t SESSION @smth.flag 1
tmux set-option -t SESSION @smth.flag ""
```

## Git tools do not detect a secondary jj workspace

Released versions of jj can create additional workspaces for a colocated Git
repository, but those workspaces do not automatically become Git worktrees. Git
commands and tools that require repository discovery may fail with `fatal: not
a git repository`, or may behave unsafely if the workspace has only a direct
`.git` pointer to the default workspace's `.git` directory.

Check the current workspace with:

```sh
cd /path/to/jj/workspace
/path/to/smth/scripts/jj-workspace-colocate.py doctor
```

You can also inspect the Git metadata directly:

```sh
git rev-parse --show-toplevel
git rev-parse --path-format=absolute --git-dir
git rev-parse --path-format=absolute --git-common-dir
jj log --ignore-working-copy -r @- --no-graph -T 'commit_id ++ "\n"'
```

A healthy secondary workspace should have `--show-toplevel` equal to the jj
workspace root, `--git-dir` different from `--git-common-dir` (meaning Git is
using a linked worktree with its own HEAD and index), and `git rev-parse HEAD`
matching jj `@-` printed by the `jj log` command above.

The repository includes [`scripts/jj-workspace-colocate.py`][colocate] to attach
linked Git worktree metadata to an existing jj workspace and to keep that
metadata aligned later.

> [!WARNING]
> This is a workaround for jj releases that do not yet support colocated
> secondary workspaces. It writes Git worktree metadata under the default
> workspace's `.git/worktrees/` directory and writes `.jj/.gitignore` in the
> workspace. Inspect the script first and use it only when you are comfortable
> repairing Git worktree metadata directly.

To create missing Git worktree metadata or refresh existing metadata, run:

```sh
cd /path/to/jj/workspace
/path/to/smth/scripts/jj-workspace-colocate.py sync
```

If you previously used the unsafe one-line hack where `.git` points directly at
the default workspace's `.git` directory, replace that file with a real Git
worktree pointer:

```sh
cd /path/to/jj/workspace
/path/to/smth/scripts/jj-workspace-colocate.py sync --replace-existing
```

`sync` creates linked Git worktree metadata when it is missing. It also sets
Git's HEAD and index to jj `@-`, so workspace file changes appear to Git as
ordinary worktree modifications. If you later run jj commands that change `@-`,
such as `jj new`, `jj edit`, or a rebase from another workspace, run `sync`
again.

`sync` uses `git reset --mixed`, so it will unstage any changes that Git tools
staged. It does not update the working tree files.

[colocate]: ../scripts/jj-workspace-colocate.py

## `smth` does not associate a workspace with its default checkout

If `smth` can find a jj checkout but new workspace names or paths are based on
the current workspace instead of the default checkout, the jj workspace path
index may be stale or missing. Check whether jj can report workspace paths:

```sh
cd /path/to/repo/or/subdirectory
jj workspace root
jj workspace list --template 'name ++ "\t" ++ root ++ "\n"'
```

If `jj workspace root` points at the right checkout but `jj workspace list`
prints `<Error: ... workspace_store/index ...>` or
`<Error: Workspace has no recorded path: ...>`, the repository was likely
created before jj recorded workspace paths. `smth` can still find the `.jj`
directory, but it cannot tell which checkout is the default workspace.

The repository includes [`scripts/fix-jj-workspace-index.py`][index] to recreate
the missing workspace path index. It has been tested with `jj 0.40.0`.

> [!WARNING]
> This script writes jj repository metadata. Use it at your own risk, inspect
> the script first, and only run it when you are comfortable repairing the
> `.jj/repo/workspace_store/index` file directly. The script backs up the
> existing index file next to the original before replacing it.

Run it from the default workspace root with one `name=/path/to/checkout`
argument for each workspace. Include the default workspace too:

```sh
cd /path/to/default/repo
/path/to/smth/scripts/fix-jj-workspace-index.py \
  default=/path/to/default/repo \
  feature=/path/to/feature/repo
jj workspace list --template 'name ++ "\t" ++ root ++ "\n"'
```

The final command should print each workspace with its checkout path instead of
an error. After that, jj will maintain the index when you add or forget
workspaces.

[index]: ../scripts/fix-jj-workspace-index.py
