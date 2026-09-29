# Runner shell configuration

## Default pane prompt is stable

The default pane prompt should be stable across test environments.

    :b env
    :t respawn-pane -k -t 0.0 'env ENV=$HOME/.shrc PS1="sh$ " /bin/sh -i'
    :t resize-window -x 80 -y 2 -t 0
    :snap

## New windows also have stable prompt

Creating a new window without a command should still produce the same stable prompt.

    :t new-window -d -n fresh 'env ENV=$HOME/.shrc PS1="sh$ " /bin/sh -i'
    :p fresh.0
    :t resize-window -x 80 -y 2 -t fresh
    :snap

---
vim: set ft=markdown:
