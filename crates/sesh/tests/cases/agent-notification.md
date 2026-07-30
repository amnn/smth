# Agent notifications

Notifications should run a recursively nested configured command only when an
unfocused agent newly enters an attention-worthy state. Interpolated titles and
messages must remain data through every shell level.

    :bins sh cat

    :w .config/sesh/sesh.toml

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

    :$ sesh agent running
    :$ sesh agent succeeded --title "Fix {state}" --summary "it's {pane}; safe"
    :$ cat notifications

Changing between attention-worthy states should not append another
notification.

    :$ sesh agent failed --summary second
    :$ cat notifications

After returning to a non-attention state, entering an attention state should
notify again.

    :$ sesh agent running
    :$ sesh agent failed --summary second
    :$ cat notifications

Notification delivery is best-effort. A missing configured executable must not
make successful state publication fail.

    :w .config/sesh/sesh.toml

```toml
[notification]
command = ["missing-notification-command"]
```

    :$ sesh agent running
    :$ sesh agent waiting
    :t show-options -pqv @sesh.agent.state

---
vim: set ft=markdown:
