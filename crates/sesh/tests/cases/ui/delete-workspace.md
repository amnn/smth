# Delete workspace

Deleting a live session that is attached to a named jj workspace should kill the
tmux session, forget the workspace, and remove the workspace directory.

    :bins jj tmux cat test sh

    :copy tests/fixtures/jjconfig.toml .jjconfig.toml

    :t rename-session -t 0 runner
    :$ jj git init alpha
    :$ jj describe -R alpha -m "alpha commit"

First create a workspace-backed session through `sesh`, so the session has the
same metadata as normal user-created workspace sessions.

    :t new-session -d -s create "sesh -r alpha; cat"
    :t resize-window -t create:0 -x 120 -y 12
    :pane create:0.0
    :settle -d 2s
    :k alpha C-r C-u feature Enter
    :settle -d 2s

Launch a fresh picker and select the workspace-backed session.

    :t new-session -d -s ui "sesh -r \"$HOME/alpha*\"; cat"
    :t resize-window -t ui:0 -x 120 -y 12
    :pane ui:0.0
    :settle -d 2s

    :k feature
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Requesting deletion should mark the selected session and show the confirm
shortcut.

    :k C-d
    :settle
    :snap --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Before confirming, add a sibling workspace so the refreshed picker can verify
that the repository family remains usable after deletion.

    :$ jj workspace add -R alpha --name sibling alpha.sibling

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
    :settle -d 2s
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Cancel the onto picker and switch to the surviving sibling workspace.

    :k C-g sibling
    :settle
    :t set-hook -g client-session-changed "set-hook -gu client-session-changed; wait-for -S switched-sibling"
    :k Enter
    :t wait-for switched-sibling

    :t display-message -p '#{client_session}'

---
vim: set ft=markdown:
