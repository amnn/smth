# Ordering direction shortcuts

Start with only ascending name enabled and alpha selected. All three discovered
sessions fit without scrolling. In ordering mode, Up and Down change the selected
comparator, not the selected session; the selected alpha follows its new row.

    :b jj tmux cat
    :t rename-session -t 0 runner
    :t new-session -d -s alpha cat
    :t new-session -d -s beta cat
    :t resize-window -t runner:0 -x 100 -y 14
    :t respawn-pane -k -t runner:0 'smth --order "+name"; cat'
    :p runner:0.0
    :settle -d 2s
    :k C-p C-s
    :snap --color

Up switches directly from ascending to descending, displaying ↑name and moving
alpha below runner and beta. Repeating Up disables name, restoring discovery
order; pressing Up again enables descending. The other comparators stay disabled.

    :k up
    :snap --color

    :k up
    :snap

    :k up
    :snap

Down switches directly from descending to ascending, displaying ↓name. Repeating
Down disables name; pressing Down again enables ascending. In all three states,
alpha remains selected and the editor remains on name.

    :k down
    :snap

    :k down
    :snap

    :k down
    :snap

Accept descending ordering set with Up. Reopen, disable it with another Up,
and cancel: the accepted descending stack and alpha's last-row position return.

    :k up C-s
    :snap

    :k C-s up
    :snap

    :k C-g
    :snap

---
vim: set ft=markdown:
