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

Enter the editor with live selected. Full names appear, with disabled attention
dimmed but its initial still interactive. Select recency without changing it;
then select name, move it to the front, and reverse it. Alpha remains selected
as the list reorders.

    :k M-s
    :snap --color

    :k r
    :snap --color

    :k n left left n
    :snap --color

Tab and Shift-Tab wrap through all four comparators. Moving left from the first
position wraps the selected name comparator to the end; right brings it back.
The outward triangles occupy reserved space and never shift the labels.

    :k btab
    :snap --color

    :k tab left
    :snap --color

    :k right
    :snap --color

Selecting attention once only selects it. Repeating enables ascending, repeating
again reverses, and repeating again disables it without losing its position.

    :k a a
    :snap --color

    :k a
    :snap

    :k a
    :snap --color

Query typing, deletion, flagging, closing, creation, Enter, and onto commands
are inert in the editor. Cancel restores the default ordering and alpha's row.

    :k xyz C-d C-f C-x C-n enter C-o C-g
    :snap

Accept a name-first descending stack. A subsequent edit can be cancelled without
losing that accepted stack. Launching again uses the built-in default, not the
accepted edit.

    :k M-s n left left n M-s
    :snap

    :k M-s n C-g
    :snap

    :t respawn-pane -k -t runner:0 'smth; cat'
    :settle -d 2s
    :k C-p
    :snap

An explicitly empty stack shows just `order`, and selects the first discovery
result rather than preferring recency.

    :t respawn-pane -k -t runner:0 'smth --order ""; cat'
    :settle -d 2s
    :k C-p
    :snap --color

All-disabled entry selects live; enabling it does not revive any other key.
Escape exits the application rather than accepting pending edits.

    :k M-s l
    :snap --color

    :k escape

---
vim: set ft=markdown:
