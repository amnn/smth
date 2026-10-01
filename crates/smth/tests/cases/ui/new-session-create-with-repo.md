# New session create with repo

Selecting the ephemeral new-session row uses the current repo context when
creating a new named session, so the new session starts in that repo and records
`@smth.repo` metadata.

    :b jj tmux
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q jj git init beta
    :$ -q jj describe -R beta -m "beta commit"
    :t new-session -d -s ui "smth -r beta"
    :t resize-window -t ui:0 -x 120 -y 10
    :p ui:0.0
    :settle -d 2s

Select the discovered repo, set it as the current repo context, then accept the
new-session row for `foo/bar`. Repository-backed creation should still normalize
the slash, producing session `beta/foo-bar` and checkout `beta.foo-bar`.

    :k beta C-r C-u foo/bar
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

    :t set-hook -g client-session-changed "set-hook -gu client-session-changed; wait-for -S created-session"
    :k enter
    :t wait-for created-session
    :settle -d 2s

The client should switch to the new session, and the session should carry the
selected repo metadata.

    :t display-message -p '#{client_session}'

    :t list-sessions -F '#{session_name}:#{b:@smth.repo}'

---
vim: set ft=markdown:
