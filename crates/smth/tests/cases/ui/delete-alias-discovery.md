# Deletion counts follow rediscovered aliases

A staged checkout counts every discovered session alias, even when the query
hides those aliases. Rediscovery must replace these counts rather than accumulate
old entries.

Create a named workspace with two live sessions pointing to its checkout. The
second session needs direct tmux setup because CLI creation would reuse the
existing session. CLI creation prints `alpha/feature`, which is then renamed to
`first`. Filter to `first` so only one alias is visible among five discovered
rows: two aliases, the default repository, the runner, and the UI session.

    :b jj tmux cat
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q jj git init alpha
    :$ smth --base alpha --create feature

    :t rename-session -t alpha/feature first
    :t new-session -d -s second -c alpha.feature "cat"
    :t set-option -F -t '=second:' @smth.repo '#{pane_start_path}'
    :t new-session -d -s ui "smth -r 'alpha*'; cat"
    :t resize-window -t ui:0 -x 120 -y 14
    :p ui:0.0
    :settle -d 2s
    :k C-p first
    :settle -e '1/5' -e C-d

Stage the shared checkout. Both aliases count, but only `first` is visible.

    :k C-d
    :snap -e '2 sessions' -e '1 hidden'

Clear the query to show both aliases staged. The total stays at two, with no
hidden sessions.

    :k C-u
    :snap -e '5/5' -e '2 sessions'

Restore the query before the next flag toggle so it still targets `first`.

    :k first
    :settle -e '1/5' -e '1 hidden'

Add a third alias and toggle the selected session's flag to trigger discovery.
The count should grow to three, with two hidden aliases.

    :t new-session -d -s third -c alpha.feature "cat"
    :t set-option -F -t '=third:' @smth.repo '#{pane_start_path}'
    :k C-f
    :snap -e '3 sessions' -e '2 hidden'

Clear the query again. All three aliases should carry staging markers, and none
should be counted as hidden.

    :k C-u
    :snap -e '6/6' -e '3 sessions'

Restore the query so the next flag toggle targets the surviving `first` session.

    :k first
    :settle -e '1/6' -e '2 hidden'

Remove both hidden aliases and toggle the flag again. Rediscovery should count
only the surviving selected session and remove the hidden-count annotation.

    :t kill-session -t second
    :t kill-session -t third
    :k C-f
    :snap -e '1 session' -e '1/4'

Clear the query one last time. Only `first` should remain staged; the removed
aliases should be absent from the full session list.

    :k C-u
    :snap -e '4/4' -e '1 session'

---
vim: set ft=markdown:
