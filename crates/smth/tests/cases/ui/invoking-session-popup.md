# Initial selection from a tmux popup

Tmux popups omit `TMUX_PANE` but retain their invoking session ID in `TMUX`.
Create two detached sessions, visit beta, and return to runner. Default recency
ordering puts the attached runner first, visited beta second, and never-attached
alpha last, without depending on configurable ordering.

    :b jj tmux cat
    :t rename-session -t 0 runner
    :t new-session -d -s alpha cat
    :t new-session -d -s beta cat
    :t switch-client -t beta
    :t switch-client -t runner
    :t resize-window -t runner:0 -x 100 -y 18

Simulate the popup environment by clearing `TMUX_PANE`. Hide the preview so all
rows are visible. With no query, runner remains first but beta is selected: the
picker recognizes and skips the invoking session through the `TMUX` fallback.

    :t respawn-pane -k -t runner:0 'unset TMUX_PANE; smth; cat'
    :p runner:0.0
    :settle -d 2s
    :k C-p
    :snap

With an explicit query, the invoking runner must be selected rather than skipped.
Keep the simulated popup environment for this launch too.

    :t respawn-pane -k -t runner:0 'unset TMUX_PANE; smth --query runner; cat'
    :settle -d 2s
    :k C-p
    :snap

---
vim: set ft=markdown:
