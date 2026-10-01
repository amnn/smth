# Pending deletion belongs to the checkout

Two live sessions sharing a checkout share one persisted deletion selection.

Attach `alpha/first` and `alpha/alias` to the same named `feature` workspace.
Filter to `first` and stage it: the footer should count both sessions, including
the hidden alias, while only one marker file is written to their shared checkout.

    :b jj tmux cat sh test sleep
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q jj git init alpha
    :$ smth --base alpha --create feature

The CLI reuses one live session per checkout. Rename it and attach a second
alias manually to construct the shared-checkout state this case needs.

    :t rename-session -t alpha/feature alpha/first
    :t new-session -d -s alpha/alias -c alpha.feature "cat"
    :t set-option -F -t '=alpha/alias:' @smth.repo '#{pane_start_path}'
    :t new-session -d -s ui "smth -r 'alpha*'; cat"
    :t resize-window -t ui:0 -x 120 -y 12
    :p ui:0.0
    :settle -d 2s
    :k C-p first
    :settle -d 2s -e '1/5' -e alpha/first -e 'C-d. delete'
    :k C-d
    :snap -d 2s -e '2 sessions' -e '1 hidden'

    :$ test -f alpha.feature/.jj/.smth-pending-delete

Both rows are staged, and the footer counts two sessions, with no hidden target.

    :k C-u
    :snap -d 2s -e '5/5'

Kill and restart the picker without cancelling its selection. Discovery restores
the staged state from the checkout, even when filtering to its other session.

    :t switch-client -t runner
    :t kill-session -t ui
    :t new-session -d -s ui "smth -r 'alpha*' --query alias; cat"
    :t resize-window -t ui:0 -x 120 -y 12
    :p ui:0.0
    :settle -d 2s
    :k C-p
    :snap -d 2s -e alpha/alias -e '2 sessions' -e '1 hidden' -e 'C-d. unstage'

Toggling through the alias removes the same marker. Cancellation removes it too.

    :k C-d
    :snap -d 2s -e alpha/alias -e 'C-d. delete'

    :$ test ! -f alpha.feature/.jj/.smth-pending-delete

Restage through the alias, then cancel. The ordinary delete hint should return
and the shared marker should disappear, just as it did when toggling off.

    :k C-d
    :settle -d 2s -e '2 sessions' -e '1 hidden'
    :k C-g
    :snap -d 2s -e alpha/alias -e 'C-d. delete'

    :$ test ! -f alpha.feature/.jj/.smth-pending-delete

Confirming deletes the checkout once and closes both live sessions. Clearing
the query afterward should reveal only the three surviving entries, without
staged markers or a deletion footer.

    :k C-d
    :settle -d 2s -e '2 sessions' -e '1 hidden'
    :k C-y
    :$ sh -c 'i=0; while test -d alpha.feature || tmux has-session -t alpha/first 2>/dev/null || tmux has-session -t alpha/alias 2>/dev/null; do i=$((i+1)); test "$$i" -lt 100 || exit 1; sleep 0.05; done'
    :settle -d 2s -e '0/3'

    :k C-u
    :snap -d 2s -e '3/3'

    :$ jj workspace list -R alpha --ignore-working-copy --no-pager --color never --template 'name ++ "\n"'

    :t has-session -t alpha/first

    :t has-session -t alpha/alias

    :t has-session -t ui

---
vim: set ft=markdown:
