# Agent lifecycle state

The agent subcommand should publish each supported lifecycle state to the
invoking tmux pane's `@smth.agent.state` user option.

    :bins env

    :$ smth agent idle
    :t show-options -pqv @smth.agent.state

    :$ smth agent running
    :t show-options -pqv @smth.agent.state

    :$ smth agent waiting
    :t show-options -pqv @smth.agent.state

    :$ smth agent succeeded --title "release ready" --summary "ready now"
    :t show-options -pqv @smth.agent.state

    :$ smth agent failed --title -still-titled --summary -still-safe
    :t show-options -pqv @smth.agent.state

The command should target `$TMUX_PANE`, even when it identifies a pane other
than tmux's active pane.

    :t split-window -d -t 0:0
    :$ env TMUX_PANE=0:0.1 smth agent waiting
    :t show-options -pqv -t 0:0.1 @smth.agent.state

    :t show-options -pqv -t 0:0.0 @smth.agent.state

Exiting agent tracking should remove the pane option.

    :$ smth agent exit
    :t show-options -pqv @smth.agent.state

Unsupported actions should be rejected before tmux metadata is changed.

    :$ smth agent running
    :$ smth agent start

    :t show-options -pqv @smth.agent.state

    :$ smth agent exit
    :t show-options -pqv @smth.agent.state

---
vim: set ft=markdown:
