# Create repositories from the picker

The picker should offer a fresh-repository candidate above the ordinary plain
candidate. `C-n` creates the selected candidate while retaining the picker.

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
    :t resize-window -t runner:0 -x 120 -y 14
    :t respawn-pane -k -t runner:0.0 'smth --no-base; cat'
    :pane runner:0.0
    :settle -d 2s

With an empty query, navigation skips both reserved rows and stays on a live
session.

    :k M-up up up
    :snap

Type a repository name. With no discovered matches, the plain candidate remains
the default even though the fresh-repository candidate appears first.

    :k foo.bar
    :snap

Select the repository candidate explicitly. Its destination is visible before
creation, and `C-n` creates that exact candidate.

    :k up
    :snap

    :k C-n
    :$ sh -c 'until test -f repos/foo.bar/.smth-ready; do :; done'

    :settle -d 2s

The picker should remain open with its query cleared, and the control-mode
client should remain attached to the runner session.

    :snap

    :t display-message -p '#{client_session}'

The repository should be colocated, use the configured root, and be attached to
its live tmux session.

    :$ sh -c 'test -d repos/foo.bar/.jj && test -d repos/foo.bar/.git'

    :$ sh -c 'tmux show-options -qv -t "=foo-bar:" @smth.repo | sed "s#$PWD#<ROOT>#g"'

Occupied paths and invalid directory names should not offer repository candidates.

    :k foo.bar
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

    :k C-u ../escape
    :snap

    :k C-u

A query that sanitizes to an empty name should use the first available numeric
session name while preserving the directory name.

    :k ... up
    :snap

    :k C-n
    :$ sh -c 'until test -f repos/.../.smth-ready; do :; done'

    :settle -d 2s
    :$ sh -c 'test -d repos/.../.jj && test -d repos/.../.git'

    :$ sh -c 'tmux show-options -qv -t "=1:" @smth.repo | sed "s#$PWD#<ROOT>#g"'

With a repository context, only the workspace candidate is offered. Navigation
must skip the empty row above it, and `C-n` creates the workspace normally.

    :t respawn-pane -k -t runner:0.0 'smth --base repos/foo.bar; cat'
    :settle -d 2s
    :k blocked M-up up up
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

    :k C-n
    :$ sh -c 'until test -f repos/foo.bar.blocked/.smth-ready; do :; done'

    :settle -d 2s
    :snap

    :t display-message -p '#{client_session}'

    :$ sh -c 'test -d repos/foo.bar.blocked/.jj && test ! -e repos/blocked'

    :$ sh -c 'tmux show-options -qv -t "=foo-bar/blocked:" @smth.repo | sed "s#$PWD#<ROOT>#g"'

---
vim: set ft=markdown:
