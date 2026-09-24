# Long-running operation spinner

Creating a session in the background should keep the picker responsive and show
delayed, operation-specific progress over the last session-list row until it finishes.

    :bins jj tmux cat sleep

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
    :$ sleep 0.6

The query should be cleared when creation is dispatched. Once the display delay
has elapsed, the last session-list row should show a right-aligned green
`creating` label and trailing spinner. The header and footer remain untouched.
Normalize the spinner for the snapshot.

    :snap -d 2s "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

On a narrow terminal, progress stays above the shortcuts. One blank column
separates the trailing spinner from the right edge of the session list.

    :t resize-window -t ui:0 -x 30 -y 14
    :snap -d 2s "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

With only one session-list row, progress overwrites the row rather than reserving
space for itself. Colour coverage verifies the overlay clears underlying styles.

    :t resize-window -t ui:0 -x 30 -y 4
    :$ sleep 0.6
    :snap --color -d 2s "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

The label truncates before losing the spinner or its trailing blank column.
When only two session-list columns remain, show just the spinner and padding.
With only one column, leave the underlying row untouched.

    :t resize-window -t ui:0 -x 8 -y 4
    :snap -d 2s "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

    :t resize-window -t ui:0 -x 3 -y 4
    :snap -d 2s "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

    :t resize-window -t ui:0 -x 2 -y 4
    :snap -d 2s

    :t resize-window -t ui:0 -x 120 -y 14
    :$ sleep 0.6

Query editing and navigation should remain available, while another create
request should be ignored until the active operation completes.

    :k omega C-n
    :snap -d 2s "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

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
