# Stage and delete multiple sessions

Deletion selections survive navigation and filtering, remain separate from
persistent flags, and include hidden sessions when confirmed.

    :bins jj tmux cat sh test sleep

    :copy tests/fixtures/jjconfig.toml .jjconfig.toml

    :t rename-session -t 0 runner
    :$ jj git init alpha
    :$ jj workspace add -R alpha --name first alpha.first
    :$ jj workspace add -R alpha --name second alpha.second
    :t new-session -d -s alpha/first
    :$ sh -c 'cd alpha.first && tmux set-option -t alpha/first @smth.repo "$(pwd -P)"'
    :t set-option -t alpha/first @smth.flag 1
    :t new-session -d -s ui "smth -r 'alpha*'; cat"
    :t resize-window -t ui:0 -x 120 -y 16
    :pane ui:0.0
    :settle -d 2s
    :k C-p first
    :settle -d 2s -e '1/5' -e alpha/first -e C-d

Stage the live session, then filter to the repo-only session and stage that too.
The footer should count both sessions and the one hidden selection.

    :k C-d C-u second
    :settle -d 2s -e '1/5' -e alpha/second -e C-d

    :k C-d
    :settle -d 2s -e '2 sessions' -e '1 hidden'

    :snap

Clearing the query reveals both deletion markers, including the row without the
cursor. Colour coverage verifies that both staged names remain distinguished.

    :k C-u
    :settle -d 2s -e '5/5'

    :snap --color

Toggling the second session off leaves only the hidden first session staged;
toggling it back on restores the batch.

    :k second
    :settle -d 2s -e '1/5' -e alpha/second -e C-d

    :k C-d
    :settle -d 2s -e '1 session' -e '1 hidden'

    :snap

    :k C-d
    :settle -d 2s -e '2 sessions' -e '1 hidden'

    :snap

C-g clears the entire selection without exiting or deleting anything, revealing
the live session's persistent flag again. A flag toggle and its discovery refresh
must not discard the staged selection.

    :k C-g
    :settle -d 2s -e '^session:' -e 'C-d. delete'

    :k C-u first
    :settle -d 2s -e '1/5' -e alpha/first -e 'C-d. delete' -e unflag

    :snap

    :k C-d C-f
    :settle -d 2s -e '1 session' -e 'C-f. flag'

    :k C-f
    :settle -d 2s -e '1 session' -e unflag

    :snap

    :k C-g
    :settle -d 2s -e alpha/first -e 'C-d. delete' -e unflag

    :snap

    :k C-d
    :settle -d 2s -e '1 session'

    :k C-g
    :settle -d 2s -e alpha/first -e 'C-d. delete' -e unflag

    :snap

    :t show-option -t alpha/first -v @smth.flag
    :$ test -d alpha.first
    :$ test -d alpha.second

Stage both again, then hide every selected session. Confirming must delete both,
not the currently selected prospective session. A second confirmation while the
batch is running must not schedule another batch.

    :k C-d C-u second
    :settle -d 2s -e '1/5' -e alpha/second -e C-d

    :k C-d C-u unmatched
    :settle -d 2s -e '2 sessions' -e '2 hidden'

    :snap

    :k C-y C-y
    :$ sh -c 'i=0; while test -d alpha.first || test -d alpha.second; do i=$((i+1)); test "$i" -lt 100 || exit 1; sleep 0.05; done'

    :settle -d 2s -e '0/3'

    :k C-u
    :settle -d 2s -e '3/3'

    :snap

    :$ jj workspace list -R alpha --ignore-working-copy --no-pager --color never --template 'name ++ "\n"'
    :t has-session -t alpha/first

    :t has-session -t ui

---
vim: set ft=markdown:
