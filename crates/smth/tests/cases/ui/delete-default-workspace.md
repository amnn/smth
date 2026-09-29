# Delete default workspace

Default jj workspace checkouts are not deletable from `smth`; only named
workspaces can be deleted.

Initialize only the default `beta` checkout, with no live session attached, and
make it discoverable through the picker's repository glob.

    :b jj tmux cat test
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ jj git init beta
    :$ jj describe -R beta -m "beta commit"
    :t new-session -d -s ui "smth -r beta; cat"
    :t resize-window -t ui:0 -x 120 -y 12
    :p ui:0.0
    :settle -d 2s

Filter to the default workspace repo-only entry. The footer should not offer
`C-d` delete.

    :k beta
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Pressing `C-d` should do nothing.

    :k C-d
    :snap --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

The default checkout and picker session should both remain.

    :$ test -e beta
    :t has-session -t ui

---
vim: set ft=markdown:
