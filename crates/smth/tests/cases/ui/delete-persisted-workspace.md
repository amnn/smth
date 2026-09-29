# Pending deletion belongs to the checkout

Two live sessions sharing a checkout share one persisted deletion selection.

    :b jj tmux cat sh test sleep
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ jj git init alpha
    :$ jj workspace add -R alpha --name feature alpha.feature
    :t new-session -d -s alpha/first
    :t new-session -d -s alpha/alias
    :$ sh -c 'cd alpha.feature && tmux set-option -t alpha/first @smth.repo "$(pwd -P)" && tmux set-option -t alpha/alias @smth.repo "$(pwd -P)"'
    :t new-session -d -s ui "smth -r 'alpha*'; cat"
    :t resize-window -t ui:0 -x 120 -y 12
    :p ui:0.0
    :settle -d 2s
    :k C-p first
    :settle -d 2s -e '1/5' -e alpha/first -e 'C-d. delete'
    :k C-d
    :settle -d 2s -e '2 sessions' -e '1 hidden'
    :snap

    :$ test -f alpha.feature/.jj/.smth-pending-delete

Both rows are staged, and the footer counts two sessions, with no hidden target.

    :k C-u
    :settle -d 2s -e '5/5'
    :snap

Kill and restart the picker without cancelling its selection. Discovery restores
the staged state from the checkout, even when filtering to its other session.

    :t switch-client -t runner
    :t kill-session -t ui
    :t new-session -d -s ui "smth -r 'alpha*' --query alias; cat"
    :t resize-window -t ui:0 -x 120 -y 12
    :p ui:0.0
    :settle -d 2s
    :k C-p
    :settle -d 2s -e alpha/alias -e '2 sessions' -e '1 hidden' -e 'C-d. unstage'
    :snap

Toggling through the alias removes the same marker. Cancellation removes it too.

    :k C-d
    :settle -d 2s -e alpha/alias -e 'C-d. delete'
    :snap

    :$ test ! -f alpha.feature/.jj/.smth-pending-delete
    :k C-d
    :settle -d 2s -e '2 sessions' -e '1 hidden'
    :k C-g
    :settle -d 2s -e alpha/alias -e 'C-d. delete'
    :snap

    :$ test ! -f alpha.feature/.jj/.smth-pending-delete

Confirming deletes the checkout once and closes both live sessions.

    :k C-d
    :settle -d 2s -e '2 sessions' -e '1 hidden'
    :k C-y
    :$ sh -c 'i=0; while test -d alpha.feature || tmux has-session -t alpha/first 2>/dev/null || tmux has-session -t alpha/alias 2>/dev/null; do i=$((i+1)); test "$i" -lt 100 || exit 1; sleep 0.05; done'
    :settle -d 2s -e '0/3'

    :k C-u
    :settle -d 2s -e '3/3'
    :snap

    :$ jj workspace list -R alpha --ignore-working-copy --no-pager --color never --template 'name ++ "\n"'

    :t has-session -t alpha/first

    :t has-session -t alpha/alias

    :t has-session -t ui

---
vim: set ft=markdown:
