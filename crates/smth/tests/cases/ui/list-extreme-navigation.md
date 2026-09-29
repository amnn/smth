# List extreme navigation

`M-up` and `M-down` jump to the first and last selectable rows in the session
list. Empty prospective rows are not selectable, while both repository and
plain-session candidates are selectable.

Create four plain sessions whose names give `et` several partial matches and
`ta` an exact live match. The latter forces prospective names to disambiguate.

    :b jj cat
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :t new-session -d -s beta "cat"
    :t new-session -d -s delta "cat"
    :t new-session -d -s ta "cat"
    :t new-session -d -s zeta "cat"
    :t new-session -d -s ui "smth"
    :t resize-window -t ui:0 -x 100 -y 12
    :p ui:0.0
    :settle

Hide the preview so the session list has enough room to show several entries.
With no query, both top rows are unselectable spacers; `M-down` jumps to the
last row (`beta`) and `M-up` jumps back to the first selectable session (`ui`).
The first snapshot establishes the initial selection before those jumps.

    :k C-p
    :snap

    :k M-down
    :snap

    :k M-up
    :snap

A query for `et` keeps multiple existing matches and offers both new-session
candidates. The first snapshot shows the filtered matches, `M-down` selects the
last match, and `M-up` jumps to the repository candidate above the plain one.

    :k et
    :snap

    :k M-down
    :snap

    :k M-up
    :snap

A query for `ta` also keeps multiple matches, but it exactly names the live
`ta` session. Both prospective rows contain disambiguated names, and `M-up`
still jumps to the repository candidate after `M-down` selects the last match.
The three snapshots show the filtered state, last match, and candidate in turn.

    :k C-u ta
    :snap

    :k M-down
    :snap

    :k M-up
    :snap

---
vim: set ft=markdown:
