# Session recency

Live tmux sessions should be ordered by tmux's `session_last_attached` value.
The picker should initially select the second-most-recent live session so
pressing Enter can switch back to the previous session.

    :bins jj cat sh sleep tmux

    :copy tests/fixtures/jjconfig.toml .jjconfig.toml

    :t rename-session -t 0 runner
    :t new-session -d -s alpha "cat"
    :t new-session -d -s beta "cat"
    :t new-session -d -s gamma "cat"

Build deterministic attachment history. `alpha` remains detached, while the
other sessions are attached in ascending recency order. Sleep between switches
because tmux reports attachment times with one-second resolution.

    :$ sleep 1
    :t switch-client -t beta
    :$ sleep 1
    :t switch-client -t gamma
    :$ sleep 1
    :t switch-client -t runner
    :pane runner:0.0

The non-interactive list should put attached sessions in descending recency
order and leave the never-attached `alpha` session last.

    :$ smth --filter

Launch the picker in the runner pane with enough height to show the complete
order. The second row, `gamma`, should be selected initially.

    :t resize-window -t runner:0 -x 80 -y 20
    :$ tmux respawn-pane -k 'smth; : > picker-exited; cat'
    :settle -d 2s
    :snap

Accept the initial selection after tmux's timestamp advances, then synchronize
on both the client switch and picker exit.

    :$ sleep 1
    :t set-hook -g client-session-changed "set-hook -gu client-session-changed; wait-for -S recency-switched"
    :k enter
    :t wait-for recency-switched
    :$ sh -c 'until test -f picker-exited; do :; done'

    :t display-message -p '#{client_session}'

Tmux should update `gamma`'s attachment time itself, making it the most recent
session without `smth` writing any metadata.

    :$ smth --filter

---
vim: set ft=markdown:
