# Long-running session switch

Switching to a session that takes time to create should show progress until the
client switches and the picker exits.

    :bins jj tmux cat

    :t rename-session -t 0 runner
    :t resize-window -t runner:0 -x 120 -y 14

Block session setup so the switch remains in flight long enough to observe its
loading state.

    :w .config/smth/smth.toml
```toml
[tmux]
setup = '''
: > switch-ready
tmux wait-for switch-release
: > switch-finished
'''
```

Keep the runner pane alive after `smth` exits so its final process can be
inspected after the client switches away.

    :$ tmux respawn-pane -k 'smth; : > picker-exited; cat'
    :settle -d 2s

Start switching to a new session and synchronize on its blocked setup.

    :t set-hook -g client-session-changed "set-hook -gu client-session-changed; wait-for -S switched-session"
    :k zeta enter
    :$ sh -c 'until test -f switch-ready; do :; done'

The query should remain visible while the bottom row of the session list is
overdrawn with a spinner and animated `switching...` label. Normalize both
animations for the snapshot.

    :snap -d 2s "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋" "/switching([.\x{a0}]{3})/."

Release setup and wait for both the client switch and picker exit.

    :t wait-for -S switch-release
    :t wait-for switched-session
    :$ sh -c 'until test -f picker-exited; do :; done'

The client should now show the new session, while the old runner pane remains
alive.

    :t display-message -p '#{client_session}'

    :t display-message -p -t runner:0.0 '#{pane_dead}'

---
vim: set ft=markdown:
