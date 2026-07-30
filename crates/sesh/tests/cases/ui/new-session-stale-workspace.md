# New session with stale workspace

A stale default jj workspace should not prevent creating a new repo-backed
session. `sesh` should allow jj to update the stale working copy automatically
as part of workspace creation.

    :bins jj tmux sh sed cat

    :copy tests/fixtures/jjconfig.toml .jjconfig.toml

Create a default workspace with tracked content and a second, healthy workspace.
Rewriting the default workspace's commit from the second workspace leaves its
working copy stale.

    :t rename-session -t 0 runner
    :$ jj git init alpha
    :$ sh -c 'printf "tracked\n" > alpha/tracked'
    :$ jj workspace add -R alpha --name healthy alpha.healthy
    :$ jj restore -R alpha.healthy --into 'default@' --from 'root()'
    :t new-session -d -s ui "cd alpha && sesh; cat"
    :t resize-window -t ui:0 -x 120 -y 10
    :pane ui:0.0
    :settle -d 2s

Type a new workspace name while the stale default workspace is the current repo
context.

    :k feature
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

    :t set-hook -g client-session-changed "set-hook -gu client-session-changed; wait-for -S stale-workspace-switched"
    :k Enter
    :t wait-for stale-workspace-switched

The client should switch to the new workspace-backed session. The new workspace
is registered in the shared repository and the default working copy is fresh.

    :t display-message -p '#{client_session}'

    :t list-sessions -F '#{session_name}:#{b:@sesh.repo}'

    :$ sh -c 'jj workspace list -R alpha --ignore-working-copy --no-pager --color never --template "name ++ \"\\t\" ++ root ++ \"\\n\"" | sed "s#$PWD/##g"'

    :$ sh -c 'jj status -R alpha --config snapshot.auto-update-stale=false >/dev/null'

The update should also materialize the rewritten working-copy commit, which no
longer contains the tracked file.

    :$ sh -c 'test ! -e alpha/tracked'

---
vim: set ft=markdown:
