# Configuration

`smth` reads TOML configuration from `$XDG_CONFIG_HOME/smth/smth.toml`, or
`~/.config/smth/smth.toml` when `$XDG_CONFIG_HOME` is unset. Pass
`--config PATH` to use an explicit file instead.

The config file is optional. When the default path does not exist, `smth` uses
its built-in defaults. A path passed with `--config` must exist.

## Options

| Setting               | Default | Description                                           |
| --------------------- | ------- | ----------------------------------------------------- |
| `notification.bell`   | `false` | Emit a terminal bell for agent attention transitions. |
| `notification.clear`  | `[]`    | Clear a pane's notification when its agent runs.      |
| `notification.notify` | `[]`    | Run a custom command for agent attention transitions. |
| `repo.globs`          | `[]`    | Discover jj repositories from glob patterns.          |
| `tmux.setup`          | `""`    | Run a shell script after creating a tmux session.     |
| `ui.sigil`            | `"⬤"`   | Mark live tmux sessions with this character.          |

### Notifications

Use `[notification].bell` to enable terminal bells,
`[notification].notify` to configure desktop notifications, and
`[notification].clear` to remove a pane's notification when its agent starts
running. All are disabled by default. See [Notifications][note] for transition
behavior, focus detection, command interpolation, and desktop notification
examples.

[note]: notifications.md

### Repository discovery

Use `[repo].globs` to surface jj repositories alongside existing tmux sessions:

```toml
[repo]
globs = [
  "~/Code/*",
  "~/.bootstrap",
  "~/.config/nvim"
]
```

These patterns stack with any `--repo` or `-r` globs supplied on the command
line. A leading `~` path component expands to your home directory.

When invoking `smth` from a tmux popup, use `-d "#{pane_current_path}"` so its
working directory comes from the active pane:

```tmux
bind s display-popup -E -w 80% -h 80% -T smth -d "#{pane_current_path}" "smth"
```

This lets the picker derive its current repository context from that pane. For
a named jj workspace, `smth` uses the recorded default workspace as the
repository context while its checkout still exists.

### Tmux session setup

Use `[tmux].setup` to run a shell script after `smth` creates a detached tmux
session. The script runs in the new session's tmux context and working
directory, so commands can use default tmux targets and relative paths.

```toml
[tmux]
setup = '''
tmux rename-window shell
tmux new-window -n editor 'nvim .'
'''
```

### Picker sigil

Use `[ui].sigil` to choose the single character that marks live tmux sessions
in the picker:

```toml
[ui]
sigil = "●"
```

## Complete example

The following example uses every available setting:

```toml
[notification]
bell = true
clear = [
  "terminal-notifier", "-remove", "smth:{pane}",
]
notify = [
  "terminal-notifier",
  "-title", "{title}",
  "-message", "{message}",
  "-group", "smth:{pane}",
]

[repo]
globs = [
  "~/Code/*",
  "~/.config/nvim"
]

[tmux]
setup = '''
tmux rename-window shell
tmux new-window -n editor 'nvim .'
'''

[ui]
sigil = "●"
```
