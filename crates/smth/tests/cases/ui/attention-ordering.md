# Attention ordering

Waiting, succeeded, and failed agents have attention; idle, running, and a
manual flag alone do not. Name is an explicit secondary comparator, ordering
the alert group alphabetically rather than by lifecycle severity.

    :b jj tmux cat
    :t rename-session -t 0 runner
    :t new-session -d -s waiting cat
    :t set-option -p -t waiting:0.0 @smth.agent.state waiting
    :t new-session -d -s succeeded cat
    :t set-option -p -t succeeded:0.0 @smth.agent.state succeeded
    :t new-session -d -s failed cat
    :t set-option -p -t failed:0.0 @smth.agent.state failed
    :t new-session -d -s running cat
    :t set-option -p -t running:0.0 @smth.agent.state running
    :t new-session -d -s idle cat
    :t set-option -p -t idle:0.0 @smth.agent.state idle
    :t set-option -t idle @smth.flag 1
    :$ smth --filter --order '-attn +name'

Reverse attention to put non-alert sessions first. The explicit name comparator
continues to order equal alert states ascending.

    :$ smth --filter --order '+attn +name'

    :t resize-window -t runner:0 -x 120 -y 22
    :t respawn-pane -k -t runner:0 'smth --order "-attn +name"; cat'
    :p runner:0.0
    :settle -d 2s
    :k C-p
    :snap --color

The first enabled comparator is attention, despite live and recency normally
preceding it. Select it on entry, disable it, then cancel. Failed stays selected
while idle, runner, and running move ahead of succeeded and waiting. Cancellation
restores the alert-first order.

    :k M-s a
    :snap --color

    :k C-g
    :snap

## Header rendering priority

Shrink the terminal until the compact header and agent summary overlap. The
right-aligned summary takes precedence: all five agent-state glyphs remain
visible even though the repository text is partly overwritten.

    :t resize-window -t runner:0 -x 56 -y 14
    :snap

Open the ordering editor at the same width. The expanded controls now take
precedence over the summary: attention, name, live, and recency stay visible,
while the overlapping agent glyphs are covered by the header.

    :k M-s
    :snap

Widen the terminal while keeping the editor open. There is room for both the
expanded controls and the complete agent summary; entering the editor must not
hide the summary unconditionally.

    :t resize-window -t runner:0 -x 120 -y 14
    :snap

Cancel the editor, then shrink the terminal again. The compact header restores
summary-first priority, keeping every agent-state glyph visible on the right.

    :k C-g
    :t resize-window -t runner:0 -x 56 -y 14
    :snap

---
vim: set ft=markdown:
