# Prospective session labels

Only prospective rows have dim, right-aligned creation labels. Labels overwrite
an overlapping repository path, with a space on either side of the label.

Launch without repository context and use `parent` as the repository root. The
query `prospective` produces both creation candidates; at the initial width,
the colour snapshot should show their dim labels alongside the destination.

    :b cat jj
    :t rename-session -t 0 runner
    :t resize-window -t runner:0 -x 80 -y 14
    :t respawn-pane -k -t runner:0.0 'smth --no-base --repo-root parent; cat'
    :p runner:0.0
    :settle -d 2s
    :k prospective
    :snap --color

Narrow the picker and select the repository candidate. The dim label should
overwrite the end of its basename, leaving a separating space.

    :t resize-window -t runner:0 -x 65 -y 14
    :k up
    :snap --color -d 2s -e '(?m)^▌.* repo '

The label should also take precedence when the row is not selected.

    :k down
    :snap --color -d 2s -e '(?m)^▌.* tmux '

At an even narrower width, the complete label remains visible over the path.

    :t resize-window -t runner:0 -x 52 -y 14
    :snap --color

Existing sessions do not acquire a creation label.

    :t resize-window -t runner:0 -x 80 -y 14
    :k C-u runner
    :snap

---
vim: set ft=markdown:
