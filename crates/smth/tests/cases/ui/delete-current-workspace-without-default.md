# Delete current workspace without a default

When jj cannot resolve a default workspace, deleting the checkout used as the
current repository context should clear that context rather than retain the
deleted path.

    :b jj tmux cat sh
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q jj git init beta
    :$ -q jj describe -R beta -m "beta commit"

Create a named workspace and its live session through the CLI before removing
the default registration below.

    :$ smth --base beta --create zeta

Forget the default workspace registration while leaving its checkout and the
repository store in place. Workspace discovery can still identify `zeta`, but
cannot normalize it to a default workspace.

    :$ -q jj workspace forget -R beta.zeta --ignore-working-copy -- default
    :t new-session -d -s ui -c beta.zeta "smth; cat"
    :t resize-window -t ui:0 -x 120 -y 12
    :p ui:0.0
    :settle -d 2s

Before deletion, the header should use `beta.zeta` as its fallback context and
the selected live workspace should offer deletion despite the missing default.

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
