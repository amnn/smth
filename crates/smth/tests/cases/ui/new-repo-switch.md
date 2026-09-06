# Switch to a repository created from the picker

`M-enter` should initialize a fresh repository for the ephemeral plain-session
candidate, create its session, and switch the invoking client.

    :bins jj tmux cat sh sed mkdir

    :copy tests/fixtures/jjconfig.toml .jjconfig.toml
    :w .config/smth/smth.toml

```toml
[repo]
root = "repos"

[tmux]
setup = ": > .smth-ready"
```

Reserve a destination and live names so the selected plain candidate must be
disambiguated again after it becomes repo-backed.

    :$ mkdir -p repos/switched~1
    :t new-session -d -s switched "cat"
    :t new-session -d -s switched~2 "cat"
    :t rename-session -t 0 runner
    :t resize-window -t runner:0 -x 120 -y 10
    :t respawn-pane -k -t runner:0.0 'smth --no-base; cat'
    :pane runner:0.0
    :settle -d 2s

Select the ephemeral plain candidate, whose name already avoids the live
sessions but does not yet account for repository paths.

    :k switched M-up
    :snap

Synchronize on a persistent option written by a one-shot tmux hook after the
client changes session.

    :t set-hook -g client-session-changed "set-hook -gu client-session-changed; set-option -g @smth.test-switched yes"
    :k M-enter
    :$ sh -c 'until test -f repos/switched~3/.smth-ready; do :; done'
    :$ sh -c 'until test "$(tmux show-options -gqv @smth.test-switched)" = yes; do :; done'

The client should show the new session after repository initialization and tmux
setup complete.

    :t display-message -p '#{client_session}'

    :$ sh -c 'test -d repos/switched~3/.jj && test -d repos/switched~3/.git && test ! -e repos/switched~1/.jj'
    :$ sh -c 'tmux show-options -qv -t "=switched~3:" @smth.repo | sed "s#$PWD#<ROOT>#g"'

    :t has-session -t '=switched'
    :t has-session -t '=switched~2'
    :t display-message -p -t runner:0.0 '#{pane_dead}'

An existing live session cannot become a fresh repository. `M-enter` should
still switch to that selected session without initializing a checkout.

    :t switch-client -t runner
    :t new-session -d -s fallback -c '#{pane_start_path}' 'smth --no-base --query switched~2; cat'
    :t resize-window -t fallback:0 -x 120 -y 10
    :pane fallback:0.0
    :settle -d 2s
    :snap

    :t set-hook -g client-session-changed "set-hook -gu client-session-changed; wait-for -S switched-existing"
    :k M-enter
    :t wait-for switched-existing
    :t display-message -p '#{client_session}'

    :$ sh -c 'test ! -e repos/switched~2 && test ! -e repos/switched~2~1'

---
vim: set ft=markdown:
