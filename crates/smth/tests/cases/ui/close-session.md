# Close session

`C-x` closes the selected live tmux session without deleting any attached
workspace, then refreshes the session list without closing the app or resetting
the query.

Create two plain sessions, `alpha` and `alpine`, so the same query has a surviving
match after the first close and no existing match after the second.

    :b jj tmux cat
    :t rename-session -t 0 runner
    :t new-session -d -s alpha "cat"
    :t new-session -d -s alpine "cat"
    :t new-session -d -s ui "smth; cat"
    :t resize-window -t ui:0 -x 120 -y 14
    :p ui:0.0
    :settle

Filter to the matching sessions. The first match, `alpha`, is selected.

    :k alp
    :snap

Pressing `C-x` should kill `alpha`, keep `smth` running, preserve the `alp`
query, and show the refreshed list with only `alpine` remaining.

    :k C-x
    :snap

    :t has-session -t alpha

    :t has-session -t alpine
    :t has-session -t ui

Closing the last matching live session should select the prospective row once
rediscovery has completed, rather than retaining a removed session forever.

    :t set-hook -g session-closed "set-hook -gu session-closed; wait-for -S closed-last-match"
    :k C-x
    :t wait-for closed-last-match
    :snap

    :t has-session -t alpine

    :t has-session -t ui

---
vim: set ft=markdown:
