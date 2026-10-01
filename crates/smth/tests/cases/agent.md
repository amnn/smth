# Agent lifecycle state

The agent subcommand should publish each supported lifecycle state to the
invoking tmux pane's `@smth.agent.state` user option.

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
than tmux's active pane. Save the runner's default locally, override the child
environment for this command, then restore it for later commands.

    :t split-window -d -t 0:0
    :v DEFAULT_PANE=$TMUX_PANE
    :e TMUX_PANE=0:0.1
    :$ smth agent waiting
    :e TMUX_PANE=$DEFAULT_PANE
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
