# Agent notifications

Notifications should run a recursively nested configured command only when an
unfocused agent newly enters an attention-worthy state. Publishing idle, running,
or exit should clear that pane's pending notification after updating its metadata.
Interpolated values must remain data through every shell level.

    :bins sh cat

    :t set-window-option -g monitor-bell on

    :w .config/smth/smth.toml

```toml
[notification]
clear = [
  "sh",
  "-c",
  "printf '%s [%s]\\n' \"$1\" \"$(tmux show-options -pqv -t \"$1\" @smth.agent.state)\" >> cleared",
  "clear",
  "{pane}",
]
notify = [
  "sh",
  "-c",
  [
    "sh",
    "-c",
    "printf '# %s\\ntitle: %s\\nmessage: %s\\npane: %s\\nsocket: %s\\ntty: %s\\n\\n' \"$1\" \"$2\" \"$3\" \"$4\" \"${5:+socket-set}\" \"$6\" >> notifications",
    "notify",
    "{state}",
    "{title}",
    "{message}",
    "{pane}",
    "{socket}",
    "{tty}",
  ],
]
```

A running state is not attention-worthy, but should clear the notification for
its pane. The first settled transition should append one notification containing
the supplied title and summary without recursively interpolating the placeholders
they contain. This headless test has no eligible interactive client, so the
optional client TTY is empty.

    :$ smth agent running
    :$ cat cleared

    :$ smth agent succeeded --title "Fix {state}" --summary "it's {pane}; safe"
    :$ cat notifications

Terminal bells default to off even when command notifications are enabled.

    :t display-message -p '#{window_bell_flag}'

Repeating or changing between attention-worthy states should neither append
another notification nor invoke the clear command.

    :$ smth agent succeeded
    :$ smth agent failed --summary second
    :$ smth agent waiting
    :$ smth agent failed
    :$ cat notifications

    :$ cat cleared

After returning to a non-attention state, entering an attention state should
notify again. Every running update should also invoke the clear command.

    :$ smth agent running
    :$ smth agent running
    :$ cat cleared

    :$ smth agent failed --summary second
    :$ cat notifications

Idle should clear a pending notification after publishing its state, without
sending a notification. A subsequent waiting update should notify again, but a
repeated waiting update should not.

    :$ sh -c ': > cleared; : > notifications'
    :$ smth agent idle
    :$ cat cleared

    :$ cat notifications

    :$ smth agent waiting
    :$ smth agent waiting
    :$ cat notifications

    :$ cat cleared

Exit should clear a pending notification after removing the pane's state,
without sending a notification. A subsequent attention state should notify again
even without an intervening idle or running update.

    :$ sh -c ': > cleared; : > notifications'
    :$ smth agent exit
    :$ cat cleared

    :$ cat notifications

    :$ smth agent succeeded
    :$ cat notifications

Clearing should work without enabling notification delivery. An exit followed by
idle, as during a Pi session reset, should invoke the clear command for both
updates. Repeated idle, running, and exit updates should still clear with this
clear-only config.

    :w .config/smth/smth.toml

```toml
[notification]
clear = [
  "sh",
  "-c",
  "printf '%s [%s]\\n' \"$1\" \"$(tmux show-options -pqv -t \"$1\" @smth.agent.state)\" >> cleared",
  "clear",
  "{pane}",
]
```

    :$ sh -c ': > cleared; : > notifications'
    :$ smth agent exit
    :$ smth agent exit
    :$ smth agent idle
    :$ smth agent idle
    :$ smth agent running
    :$ smth agent running
    :$ cat cleared

    :$ smth agent waiting
    :$ cat notifications

    :t display-message -p '#{window_bell_flag}'

A bell-only configuration should enable notifications and emit a terminal bell
in a background agent window.

    :w .config/smth/smth.toml

```toml
[notification]
bell = true
```

The pane runs `smth agent` itself so its stdout reaches the pane terminal. Wait
for an explicit tmux signal before checking the asynchronous result.

    :t new-window -d -t 0:1 "smth agent running; smth agent waiting; tmux wait-for -S bell-ready; cat"
    :t wait-for bell-ready
    :t display-message -p -t 0:1.0 '#{window_bell_flag}'

Notification delivery and clearing are best-effort. Missing configured
executables must not make successful state publication fail.

    :w .config/smth/smth.toml

```toml
[notification]
clear = ["missing-clear-command"]
notify = ["missing-notification-command"]
```

    :$ smth agent running
    :t show-options -pqv @smth.agent.state

    :$ smth agent waiting
    :t show-options -pqv @smth.agent.state

    :$ smth agent idle
    :t show-options -pqv @smth.agent.state

    :$ smth agent exit
    :t show-options -pqv @smth.agent.state

A clear command that runs but exits unsuccessfully must also leave successful
metadata updates intact. Log each attempt before failing to prove it was invoked
after publishing or removing the state.

    :w .config/smth/smth.toml

```toml
[notification]
clear = [
  "sh",
  "-c",
  "printf '%s [%s]\\n' \"$1\" \"$(tmux show-options -pqv -t \"$1\" @smth.agent.state)\" >> cleared; exit 1",
  "clear",
  "{pane}",
]
```

    :$ sh -c ': > cleared'
    :$ smth agent running
    :t show-options -pqv @smth.agent.state

    :$ smth agent idle
    :t show-options -pqv @smth.agent.state

    :$ smth agent exit
    :t show-options -pqv @smth.agent.state

    :$ cat cleared

---
vim: set ft=markdown:
