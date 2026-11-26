# List navigation aliases

Down and Up, and their aliases `C-j` and `C-k`, move by one row.
`M-down` and `M-up` jump to the last and first selectable rows.

Create four plain sessions so single-row movement and jumps to either end have
different destinations in the unfiltered picker.

    :b jj cat
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :t new-session -d -s alpha "cat"
    :t new-session -d -s beta "cat"
    :t new-session -d -s gamma "cat"
    :t new-session -d -s zeta "cat"
    :t new-session -d -s ui "smth"
    :t resize-window -t ui:0 -x 100 -y 12
    :p ui:0.0
    :settle -d 2s

Hide the preview so the selected row movement is easy to inspect.

    :k C-p
    :snap

Down moves the selection down by one row.

    :k down
    :snap

Up moves the selection back up by one row.

    :k up
    :snap

`C-j` moves down one row without entering ring navigation.

    :k C-j
    :snap

`C-k` moves back up one row.

    :k C-k
    :snap

`M-down` jumps to the last selectable row.

    :k M-down
    :snap

`M-up` jumps back to the first selectable row.

    :k M-up
    :snap

---
vim: set ft=markdown:
