# Session setup config

A custom session setup script runs in the new session's tmux context and repo
working directory, so setup commands can rely on default tmux targets and cwd.

Create a discoverable `beta` repository without a live session. Configure setup
to write files, rename the initial window, and open a second window; those side
effects identify both the target session and the directory used by setup.

    :b jj tmux cat test
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q jj git init beta
    :w .config/smth/smth.toml

```toml
[tmux]
setup = '''
: > setup-touched
tmux rename-window configured
tmux new-window -n shell ': > new-window-touched; cat'
'''
```

Launch the picker and select the discovered repo entry. The snapshot should
show `beta` without a live-session marker before accepting it triggers setup.

    :t new-session -d -s ui "smth -r beta"
    :t resize-window -t ui:0 -x 120 -y 10
    :p ui:0.0
    :settle
    :k beta
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

    :k enter
    :$ sh -c 'until test -f beta/new-window-touched; do :; done'

The setup script should have renamed the initial window, created another window,
and run commands from the repo directory.

    :t list-windows -t beta -F '#W'

    :$ test -f beta/setup-touched
    :$ test -f beta/new-window-touched

---
vim: set ft=markdown:
