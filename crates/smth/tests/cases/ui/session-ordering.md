# Session ordering editor

Create a current session and two other live sessions. The compact header shows
only enabled comparator initials, bold, in priority order. Up arrows put live
and recent values first; down arrows put names in A–Z order. Hide the preview
before each snapshot so the complete session order is visible.

    :b jj tmux cat
    :t rename-session -t 0 runner
    :t new-session -d -s alpha cat
    :t new-session -d -s beta cat
    :t resize-window -t runner:0 -x 100 -y 18
    :t respawn-pane -k -t runner:0 'smth; cat'
    :p runner:0.0
    :settle -d 2s
    :k C-p
    :snap --color

An explicitly empty stack shows just `order`, and selects the first discovery
result rather than preferring recency.

    :t respawn-pane -k -t runner:0 'smth --order ""; cat'
    :settle -d 2s
    :k C-p
    :snap --color

---
vim: set ft=markdown:
