# New session stale repo context

If the current repo context comes from stale tmux metadata and is not actually a
jj repo, accepting a new session should use that path as the tmux working
directory without attempting workspace creation or attaching repo metadata.

Capture the physical sandbox root once to normalize the new session's path.

    :b jj tmux mkdir cat sed pwd
    :$ -q pwd -P
    := ROOT
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ mkdir plain
    :t new-session -d -s plain "cat"
    :t set-option -t plain @smth.repo plain
    :t new-session -d -s ui "smth"
    :t resize-window -t ui:0 -x 120 -y 10
    :p ui:0.0
    :settle -d 2s

Select the live session with stale repo metadata, set it as the current repo
context, then accept a new-session row.

    :k plain C-r C-u zeta
    :snap

    :t set-hook -g client-session-changed "set-hook -gu client-session-changed; wait-for -S created-session"
    :k enter
    :t wait-for created-session
    :settle -d 2s

The new tmux session should start in `plain`, but it should not record stale
repo metadata because no jj workspace was created.

    :t display-message -p '#{client_session}'

    :$ tmux display-message -p -t zeta:0 '#{pane_current_path}'
    :| sed "s#${ROOT}/##g"

    :t list-sessions -F '#{session_name}:#{b:@smth.repo}'

---
vim: set ft=markdown:
