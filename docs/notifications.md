# Notifications

Notification delivery complements [agent integration][agent] and is optional.
It is disabled unless `notification.bell` is `true`, `notification.command` is
non-empty, or both.

Enabled channels run when an agent newly enters `waiting`, `succeeded`, or
`failed` and no eligible tmux client has that pane focused. Moving between
attention-worthy states does not notify again; the agent must first return to
`idle` or `running`.

Notification delivery is best-effort and never makes a lifecycle state update
fail.

[agent]: agent-integration.md

## Terminal bells

Set `notification.bell = true` to make `smth agent` emit a terminal bell on
stdout. It defaults to `false`.

```toml
[notification]
bell = true
```

Harness integrations that capture stdout must forward it to the pane terminal;
the bundled Pi extension does this. Tmux can monitor the bell and surface it
visually with, for example:

```tmux
set -g visual-bell both
setw -g monitor-bell on
```

## Desktop notifications

Configure `notification.command` to run a desktop notification program. On
macOS, [`smth-notifier`][app] is a purpose-built bridge that sends silent
agent-state notifications. Clicking one switches the associated tmux client to
the requested pane and activates the configured terminal application. Its
README covers installation, notification authorization, and the corresponding
`notification.command` configuration.

For a generic alternative, [`terminal-notifier`][tn] can be installed with:

```sh
brew install terminal-notifier
```

Then configure `smth` to call it:

```toml
[notification]
command = [
  "terminal-notifier",
  "-title", "{title}",
  "-message", "{message}",
  "-group", "smth:{pane}",
  "-execute", [
    "/Users/me/.config/smth/focus.sh",
    "{socket}",
    "{tty}",
    "{pane}",
  ],
]
```

The `-execute` argument controls what happens when the user clicks the
notification. In this example, a `focus.sh` script receives the tmux socket,
preferred client TTY, and target pane. Grouping by pane replaces an older
notification when that pane needs attention again.

[app]: https://github.com/amnn/smth-notifier
[tn]: https://github.com/julienXX/terminal-notifier

### Command arguments

The root `command` array is executed directly as an argument vector, without a
shell. A nested array is evaluated from the inside out, POSIX-shell-joined, and
passed to its parent as one argument. This makes the nested `-execute` command
above safe to pass as a single shell command string.

Command strings can interpolate these values:

| Variable    | Value                                                                     |
| ----------- | ------------------------------------------------------------------------- |
| `{title}`   | The supplied notification title, or an empty string.                      |
| `{message}` | The supplied summary, or a default message for the state.                 |
| `{state}`   | `waiting`, `succeeded`, or `failed`.                                      |
| `{pane}`    | The tmux pane publishing the transition.                                  |
| `{socket}`  | The active tmux server socket path.                                       |
| `{tty}`     | The preferred tmux client TTY, or an empty string when none is available. |

Titles and messages have whitespace normalized and are bounded before command
interpolation.

## Focus detection

A notification is suppressed when an eligible tmux client is displaying the
agent pane and is not known to be unfocused. Tmux focus reporting is not always
available, so `smth` conservatively treats uncertain clients as focused.
Detached sessions can still notify. When a command needs `{tty}`, `smth` chooses
the most relevant and recently active eligible client; the value is empty when
there is no client.
