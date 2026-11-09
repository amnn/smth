# Startup input

Enter queued before startup must be discarded when the picker enters raw mode,
not interpreted as Ctrl-J and used to open the next-tag dialog.

Name the starting session runner and create beta as the switch target, keeping
its pane alive with cat. Fix the runner window size for stable snapshots.

    :b jj cat tmux stty diff sh
    :t rename-session -t 0 runner
    :t new-session -d -s beta "printf 'beta target'; cat"
    :t resize-window -t runner:0 -x 80 -y 10

Hold the pane before launching smth so Enter is queued with the shell's terminal
settings. Settling after sending the key lets its echo arrive before we release
the startup barrier. Save terminal settings before and after smth to check that
exiting restores them, and keep the runner pane alive with cat.

    :w startup.sh

```sh
#!/bin/sh

stty -g > terminal-before
printf "waiting for startup"
tmux wait-for -S startup-ready
tmux wait-for startup-release
smth --no-base --query beta
stty -g > terminal-after
tmux wait-for -S picker-exited
cat
```

    :$ tmux respawn-pane -k 'sh startup.sh'
    :t wait-for startup-ready
    :settle -d 2s
    :k enter
    :settle
    :t wait-for -S startup-release
    :snap -d 2s -e 'session: beta'

A genuine Ctrl-J after startup must still open the next-tag dialog.

    :k C-j
    :snap

Cancel tag navigation with an unrecognized key, then use a fresh Enter to switch
to beta. Wait for the session-change hook before checking that the client's
session name is beta.

    :k space
    :settle
    :t set-hook -g client-session-changed "set-hook -gu client-session-changed; wait-for -S switched"
    :k enter
    :t wait-for switched
    :t display-message -p '#{client_session}'

After the picker exits, its original terminal settings must be restored exactly.
The comparison should succeed without output.

    :t wait-for picker-exited
    :$ diff terminal-before terminal-after

---
vim: set ft=markdown:
