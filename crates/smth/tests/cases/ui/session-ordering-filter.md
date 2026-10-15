# Ordering while filtering

Create two equal-score prefix matches, ab and ac, and a weaker interior match,
zaz. With query a and descending names, ac and ab must precede zaz despite its
name. The explicit query selects ac, the first result, rather than skipping it.
Hide the preview so every discovered match fits on screen.

    :b jj tmux cat
    :t rename-session -t 0 runner
    :t new-session -d -s ab cat
    :t new-session -d -s ac cat
    :t new-session -d -s zaz cat
    :t resize-window -t runner:0 -x 100 -y 14
    :t respawn-pane -k -t runner:0 'smth --order "-name" --query a; cat'
    :p runner:0.0
    :settle -d 2s
    :k C-p
    :snap

Switch to ascending names in the editor. Only the equal-score pair changes
order: ab precedes ac, with ac still selected. The weaker zaz remains last.

    :k C-s down
    :snap

Repeating Down disables every comparator. Fuzzy ranking remains active, with
ab before ac in nucleo's tied discovery order. Cancel restores descending names
and moves the selected ac back above ab, without moving zaz above either.

    :k down
    :snap

    :k C-g
    :snap

Accept ascending names, then clear the query. The full list is alphabetic,
including runner; fuzzy score no longer takes precedence. Reversing names with
no query moves zaz to the top, unlike reversing names while filtering.

    :k C-s down C-s
    :snap

    :k C-u
    :snap

    :k C-s up C-s
    :snap

With descending ordering committed, move Down from ac to ab. The ordering has
not changed, so normal navigation must not restore the previous selection.

    :k down
    :snap -e '▌.* ab '

Start a new query matching both ab and ac. Query editing resets selection to the
first result, ac, rather than preserving ab merely because it still matches.

    :k a
    :snap -e '^session: a' -e '▌.* ac '

---
vim: set ft=markdown:
