# Agent attention

Agent harnesses should render right-aligned lifecycle summaries on their
session rows and in aggregate on the header. Waiting, failed, and succeeded
harnesses should use the same attention pip styling as a tmux bell; running and
idle harnesses should not. A manual flag on a session that also has agent
attention should retain the existing attention-over-flag precedence.

    :bins jj cat

    :copy tests/fixtures/jjconfig.toml .jjconfig.toml

    :t rename-session -t 0 runner
    :t new-session -d -s waiting "cat"
    :t set-option -p -t waiting:0.0 @smth.agent.state waiting
    :t split-window -d -t waiting:0 "cat"
    :t set-option -p -t waiting:0.1 @smth.agent.state running
    :t new-window -d -t waiting: -n worker "cat"
    :t set-option -p -t waiting:1.0 @smth.agent.state running
    :t new-session -d -s failed "cat"
    :t set-option -p -t failed:0.0 @smth.agent.state failed
    :t new-session -d -s succeeded "cat"
    :t set-option -p -t succeeded:0.0 @smth.agent.state succeeded
    :t split-window -d -t succeeded:0 "cat"
    :t set-option -p -t succeeded:0.1 @smth.agent.state succeeded
    :t new-window -d -t succeeded: -n ready "cat"
    :t set-option -p -t succeeded:1.0 @smth.agent.state idle
    :t set-option -t succeeded @smth.flag 1
    :t new-session -d -s running "cat"
    :t set-option -p -t running:0.0 @smth.agent.state running
    :t split-window -d -t running:0 "cat"
    :t set-option -p -t running:0.1 @smth.agent.state running
    :t new-session -d -s idle "cat"
    :t set-option -p -t idle:0.0 @smth.agent.state idle
    :t new-session -d -s ui "smth; cat"
    :t resize-window -t ui:0 -x 100 -y 12
    :pane ui:0.0
    :settle -d 2s

Hide the preview so every session is visible in one snapshot. The linked colour
snapshots distinguish lifecycle states and attention, flag, and ordinary
live-session pips. Status indicators and counts should remain undimmed.

    :k C-p
    :settle
    :snap --color

Filter to a session with successful agent responses. The header summary should
continue to aggregate agents across every session while the selected row shows
only that session's agents. In the linked colour snapshots, the success glyph
should remain bright green without creating a green background cell.

    :k succeeded
    :snap --color

---
vim: set ft=markdown:
