# Agent lifecycle state

The agent subcommand should publish each supported lifecycle state to the
invoking tmux pane's `@sesh.agent.state` user option.

    :bins env

    :$ sesh agent idle
    :t show-options -pqv @sesh.agent.state

    :$ sesh agent running
    :t show-options -pqv @sesh.agent.state

    :$ sesh agent waiting
    :t show-options -pqv @sesh.agent.state

    :$ sesh agent succeeded
    :t show-options -pqv @sesh.agent.state

    :$ sesh agent failed
    :t show-options -pqv @sesh.agent.state

The command should target `$TMUX_PANE`, even when it identifies a pane other
than tmux's active pane.

    :t split-window -d -t 0:0
    :$ env TMUX_PANE=0:0.1 sesh agent waiting
    :t show-options -pqv -t 0:0.1 @sesh.agent.state

    :t show-options -pqv -t 0:0.0 @sesh.agent.state

Exiting agent tracking should remove the pane option.

    :$ sesh agent exit
    :t show-options -pqv @sesh.agent.state

Unsupported actions should be rejected before tmux metadata is changed.

    :$ sesh agent running
    :$ sesh agent start

    :t show-options -pqv @sesh.agent.state

    :$ sesh agent exit
    :t show-options -pqv @sesh.agent.state

---
vim: set ft=markdown:
