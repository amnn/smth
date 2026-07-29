# Rename session

Under tmux 3.4, `C-e` should open a rename prompt for the selected live session
with its current name available for editing.

    :bins jj tmux cat

    :t rename-session -t 0 runner
    :t new-session -d -s alpha "cat"
    :t new-session -d -s ui "sesh; cat"
    :t resize-window -t ui:0 -x 100 -y 10
    :pane ui:0.0
    :settle -d 2s

    :k alpha
    :k C-e
    :snap

Accepting the edited name should keep the picker open and refresh its session
list with the new name.

    :t set-hook -g session-renamed "set-hook -gu session-renamed; wait-for -S renamed-session"
    :k C-u renamed Enter
    :t wait-for renamed-session
    :settle -d 2s
    :snap

    :t has-session -t renamed

---
vim: set ft=markdown:
