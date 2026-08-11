# Long-running operation spinner

Creating a session in the background should keep the picker responsive and show
animated, operation-specific progress at the bottom until it finishes.

    :bins jj tmux cat

    :t rename-session -t 0 runner

Block the session setup script so the create operation stays in flight long
enough to observe its loading state.

    :w .config/smth/smth.toml
```toml
[tmux]
setup = '''
: > spinner-ready
tmux wait-for spinner-release
: > spinner-finished
'''
```

    :t new-session -d -s ui "smth; cat"
    :t resize-window -t ui:0 -x 120 -y 14
    :pane ui:0.0
    :settle -d 2s

Start creating a detached session, then wait until its setup script reaches the
blocking point.

    :k zeta C-n
    :$ sh -c 'until test -f spinner-ready; do :; done'

The query should be cleared when creation is dispatched, while the bottom row of
the session list is overdrawn with a spinner and animated `creating...` label.
Normalize both animations for the snapshot.

    :snap -d 2s "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋" "/creating([.\x{a0}]{3})/."

Query editing and navigation should remain available, while another create
request should be ignored until the active operation completes.

    :k omega C-n
    :snap -d 2s "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋" "/creating([.\x{a0}]{3})/."

Release the setup script and synchronize on its completion before inspecting the
picker again.

    :t wait-for -S spinner-release
    :$ sh -c 'until test -f spinner-finished; do :; done'
    :settle -d 2s

The progress line should be gone, the edited query should remain, and only the
original `zeta` create request should have run.

    :snap

    :t has-session -t zeta
    :t has-session -t omega

---
vim: set ft=markdown:
