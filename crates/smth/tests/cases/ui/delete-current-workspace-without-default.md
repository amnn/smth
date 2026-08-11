# Delete current workspace without a default

When jj cannot resolve a default workspace, deleting the checkout used as the
current repository context should clear that context rather than retain the
deleted path.

    :bins jj tmux cat test sh

    :copy tests/fixtures/jjconfig.toml .jjconfig.toml

    :t rename-session -t 0 runner
    :$ jj git init beta
    :$ jj describe -R beta -m "beta commit"

Create a named workspace and a live tmux session carrying the same repository
metadata that `smth` records.

    :$ jj workspace add -R beta --name zeta beta.zeta
    :t new-session -d -s beta/zeta -c beta.zeta "cat"
    :t set-option -F -t '=beta/zeta:' @smth.repo '#{pane_current_path}'

Forget the default workspace registration while leaving its checkout and the
repository store in place. Workspace discovery can still identify `zeta`, but
cannot normalize it to a default workspace.

    :$ jj workspace forget -R beta.zeta --ignore-working-copy -- default

    :t new-session -d -s ui -c beta.zeta "smth; cat"
    :t resize-window -t ui:0 -x 120 -y 12
    :pane ui:0.0
    :settle -d 2s

    :k zeta
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Delete the selected workspace. Closing its live session happens after the
checkout is removed, so the one-shot tmux hook is the completion signal.

    :k C-d
    :t set-hook -g session-closed "set-hook -gu session-closed; wait-for -S deleted-current-workspace"
    :k C-y
    :t wait-for deleted-current-workspace
    :settle -d 2s

The picker should remain usable without advertising the deleted repository as
its current context.

    :snap

    :t has-session -t beta/zeta

    :$ sh -c 'test ! -e beta.zeta'

---
vim: set ft=markdown:
