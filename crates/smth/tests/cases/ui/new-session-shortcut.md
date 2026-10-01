# New session row

This scenario creates several repo-backed picker entries, with both live tmux
sessions and discoverable repos that do not have live sessions. It verifies that
the ephemeral plain-session row is selectable below the repository candidate
when the query is non-empty, with names disambiguated from live sessions as
needed.

    :b jj
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q jj git init alpha
    :$ -q jj describe -R alpha -m "alpha commit"
    :$ -q jj git init beta
    :$ -q jj describe -R beta -m "beta commit"
    :$ -q jj git init gamma
    :$ -q jj describe -R gamma -m "gamma commit"
    :$ -q jj git init delta
    :$ -q jj describe -R delta -m "delta commit"

Launch live sessions for `alpha` and `gamma`, while `beta` and `delta` remain
repo-only entries discovered through the CLI globs.

    :$ smth --base alpha --create

    :$ smth --base gamma --create

    :t new-session -d -s ui "smth -r 'alpha' -r 'beta' -r 'gamma' -r 'delta'"
    :t resize-window -t ui:0 -x 120 -y 12
    :p ui:0.0
    :settle

Initially the query is empty, so no new-session candidate is available.
Pressing `C-n` should do nothing and leave the picker open in the same state.

    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

    :k C-n
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Typing a prefix of the live `alpha` session makes the new-session row selectable,
because `alp` is not an exact live session name.

    :k alp M-up down
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Completing the live session name causes the new-session row's name to become
disambiguated.

    :k ha
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Search for the non-live `beta` repo and select the prospective plain-session
row again. Its name should have no suffix because no live session is named
`beta`; selecting the candidate keeps that assertion visible in the snapshot.

    :k C-u beta
    :settle -d 2s -e '^session: beta' -e '1/6'
    :k M-up
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

---
vim: set ft=markdown:
