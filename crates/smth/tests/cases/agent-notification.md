# Agent notifications

Notifications should run a recursively nested configured command only when an
unfocused agent newly enters an attention-worthy state. Interpolated titles and
messages must remain data through every shell level.

    :bins sh cat

    :t set-window-option -g monitor-bell on

    :w .config/smth/smth.toml

```toml
[notification]
command = [
  "sh",
  "-c",
  [
    "sh",
    "-c",
    "printf '%s\\n' \"$1\" \"$2\" \"$3\" \"$4\" \"${5:+socket-set}\" \"$6\" >> notifications",
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

A running state is not attention-worthy. The first settled transition should
append one notification containing the supplied title and summary without
recursively interpolating the placeholders they contain. This headless test has
no eligible interactive client, so the optional client TTY is empty.

    :$ smth agent running
    :$ smth agent succeeded --title "Fix {state}" --summary "it's {pane}; safe"
    :$ cat notifications

Terminal bells default to off even when command notifications are enabled.

    :t display-message -p '#{window_bell_flag}'

Changing between attention-worthy states should not append another
notification.

    :$ smth agent failed --summary second
    :$ cat notifications

After returning to a non-attention state, entering an attention state should
notify again.

    :$ smth agent running
    :$ smth agent failed --summary second
    :$ cat notifications

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

Notification delivery is best-effort. A missing configured executable must not
make successful state publication fail.

    :w .config/smth/smth.toml

```toml
[notification]
command = ["missing-notification-command"]
```

    :$ smth agent running
    :$ smth agent waiting
    :t show-options -pqv @smth.agent.state

---
vim: set ft=markdown:
