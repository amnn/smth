# List extreme navigation

`M-up` and `M-down` jump to the first and last selectable rows in the session
list. Empty prospective rows are not selectable, while both repository and
plain-session candidates are selectable.

    :bins jj cat

    :copy tests/fixtures/jjconfig.toml .jjconfig.toml

    :t rename-session -t 0 runner
    :t new-session -d -s beta "cat"
    :t new-session -d -s delta "cat"
    :t new-session -d -s ta "cat"
    :t new-session -d -s zeta "cat"
    :t new-session -d -s ui "smth"
    :t resize-window -t ui:0 -x 100 -y 12
    :pane ui:0.0
    :settle

Hide the preview so the session list has enough room to show several entries.
With no query, both top rows are unselectable spacers; `M-down` jumps to the
last row and `M-up` jumps back to the first selectable session.

    :k C-p
    :snap

    :k M-down
    :snap

    :k M-up
    :snap

A query for `et` keeps multiple existing matches and offers both new-session
candidates. `M-up` jumps to the repository candidate, above the plain one.

    :k et
    :snap

    :k M-down
    :snap

    :k M-up
    :snap

A query for `ta` also keeps multiple matches, but it exactly names the live
`ta` session. Both prospective rows contain disambiguated names, and `M-up`
still jumps to the repository candidate.

    :k C-u ta
    :snap

    :k M-down
    :snap

    :k M-up
    :snap

---
vim: set ft=markdown:
