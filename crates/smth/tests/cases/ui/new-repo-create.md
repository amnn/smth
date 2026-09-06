# Create repositories from the picker

`M-n` should initialize a fresh repository only for the ephemeral plain-session
candidate, then create its session while retaining the picker.

    :bins jj tmux cat sh sed

    :copy tests/fixtures/jjconfig.toml .jjconfig.toml
    :w .config/smth/smth.toml

```toml
[repo]
root = "repos"

[tmux]
setup = ": > .smth-ready"
```

    :t rename-session -t 0 runner
    :t resize-window -t runner:0 -x 120 -y 10
    :t respawn-pane -k -t runner:0.0 'smth --no-base; cat'
    :pane runner:0.0
    :settle -d 2s

Type a repository name and wait for the ephemeral candidate to become the
retained selection.

    :k detached
    :snap

    :k M-n
    :$ sh -c 'until test -f repos/detached/.smth-ready; do :; done'
    :settle -d 2s

The picker should remain open with its query cleared, and the control-mode
client should remain attached to the runner session.

    :snap

    :t display-message -p '#{client_session}'

The repository should be colocated, use the configured root, and be attached to
its live tmux session.

    :$ sh -c 'test -d repos/detached/.jj && test -d repos/detached/.git'
    :$ sh -c 'tmux show-options -qv -t "=detached:" @smth.repo | sed "s#$PWD#<ROOT>#g"'

A query that sanitizes to an empty name should use the first available numeric
suffix for both the candidate and the new repository session.

    :k ...
    :snap

    :k M-n
    :$ sh -c 'until test -f repos/1/.smth-ready; do :; done'
    :settle -d 2s
    :$ sh -c 'test -d repos/1/.jj && test -d repos/1/.git'
    :$ sh -c 'tmux show-options -qv -t "=1:" @smth.repo | sed "s#$PWD#<ROOT>#g"'

With a repository context, `M-n` should create the selected workspace normally
instead of initializing a fresh repository.

    :t respawn-pane -k -t runner:0.0 'smth --base repos/detached; cat'
    :settle -d 2s
    :k blocked
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

    :k M-n
    :$ sh -c 'until test -f repos/detached.blocked/.smth-ready; do :; done'
    :settle -d 2s
    :snap

    :t display-message -p '#{client_session}'

    :$ sh -c 'test -d repos/detached.blocked/.jj && test ! -e repos/blocked'
    :$ sh -c 'tmux show-options -qv -t "=detached/blocked:" @smth.repo | sed "s#$PWD#<ROOT>#g"'

---
vim: set ft=markdown:
