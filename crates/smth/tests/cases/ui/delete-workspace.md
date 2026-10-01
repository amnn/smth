# Delete workspace

Deleting a live session that is attached to a named jj workspace should kill the
tmux session, forget the workspace, and remove the workspace directory.

    :b jj tmux cat sh
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q jj git init alpha
    :$ -q jj describe -R alpha -m "alpha commit"

Create the workspace-backed session through the CLI, giving it normal session
metadata without driving a second picker just for setup.

    :$ smth --base alpha --create feature

Launch the picker with the named checkout as its base and select its live
session. The header should normalize that context to the default workspace,
which must remain usable after the named checkout is deleted.

    :t new-session -d -s ui "smth --base alpha.feature -r \"$$HOME/alpha*\"; cat"
    :t resize-window -t ui:0 -x 120 -y 12
    :p ui:0.0
    :settle -d 2s
    :k feature
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Requesting deletion should mark the selected session and show the confirm
shortcut.

    :k C-d
    :snap --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Before confirming, add a sibling workspace so the refreshed picker can verify
that the repository family remains usable after deletion.

    :$ -q jj workspace add -R alpha --name sibling alpha.sibling

Confirming should remove the tmux session, remove the workspace from jj's
workspace list, and delete the workspace directory.

    :t set-hook -g session-closed "set-hook -gu session-closed; wait-for -S deleted-workspace-session"
    :k C-y
    :t wait-for deleted-workspace-session
    :settle -d 2s
    :t has-session -t alpha/feature

    :$ jj workspace list -R alpha --ignore-working-copy --no-pager --color never --template 'name ++ "\n"'

    :$ sh -c 'test ! -e alpha.feature'

The current repository context should have been normalized to the surviving
default workspace. Clear the preserved session query, then verify that the onto
picker still loads a valid log.

    :k C-u C-o
    :snap -d 2s "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Cancel the onto picker and switch to the surviving sibling workspace.

    :k C-g sibling
    :settle
    :t set-hook -g client-session-changed "set-hook -gu client-session-changed; wait-for -S switched-sibling"
    :k enter
    :t wait-for switched-sibling
    :t display-message -p '#{client_session}'

---
vim: set ft=markdown:
