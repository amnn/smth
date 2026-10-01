# Stage and delete multiple sessions

Deletion selections survive navigation and filtering, remain separate from
persistent flags, and include hidden sessions when confirmed.

Create two named workspaces: `first` has a flagged live session, while `second`
is discoverable only through the repository glob. Hide the preview and filter
to `first` so later query changes can hide staged entries.

    :b jj tmux cat sh test sleep
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ jj git init alpha
    :$ smth --base alpha --create first

    :$ jj workspace add -R alpha --name second alpha.second
    :$ smth --base alpha --flag first
    :t new-session -d -s ui "smth -r 'alpha*'; cat"
    :t resize-window -t ui:0 -x 120 -y 16
    :p ui:0.0
    :settle -d 2s
    :k C-p first
    :settle -d 2s -e '1/5' -e alpha/first -e C-d

Stage the live session, then filter to the repo-only session and stage that too.
The footer should count both sessions and the one hidden selection.

    :k C-d C-u second
    :settle -d 2s -e '1/5' -e alpha/second -e C-d
    :k C-d
    :snap -d 2s -e '2 sessions' -e '1 hidden'

Clearing the query reveals both deletion markers, including the row without the
cursor. Colour coverage verifies that both staged names remain distinguished.

    :k C-u
    :snap -d 2s -e '5/5' --color

Toggling the second session off leaves only the hidden first session staged;
toggling it back on restores the batch.

    :k second
    :settle -d 2s -e '1/5' -e alpha/second -e C-d
    :k C-d
    :snap -d 2s -e '1 session' -e '1 hidden'

Restaging the visible second workspace restores the two-session count, with
only the first workspace hidden by the query.

    :k C-d
    :snap -d 2s -e '2 sessions' -e '1 hidden'

C-g clears the entire selection without exiting or deleting anything, revealing
the live session's persistent flag again. A flag toggle and its discovery refresh
must not discard the staged selection.

    :k C-g
    :settle -d 2s -e '^session:' -e 'C-d. delete'
    :k C-u first
    :snap -d 2s -e '1/5' -e alpha/first -e 'C-d. delete' -e unflag

Stage the live workspace and toggle its persistent flag off and on. The footer
should still count one deletion while offering `unflag` again.

    :k C-d C-f
    :settle -d 2s -e '1 session' -e 'C-f. flag'
    :k C-f
    :snap -d 2s -e '1 session' -e unflag

Cancelling the batch restores the ordinary delete hint without losing the flag.

    :k C-g
    :snap -d 2s -e alpha/first -e 'C-d. delete' -e unflag

Repeat staging and cancellation to verify the same state can be reached again.

    :k C-d
    :settle -d 2s -e '1 session'
    :k C-g
    :snap -d 2s -e alpha/first -e 'C-d. delete' -e unflag

    :t show-option -t alpha/first -v @smth.flag

    :$ test -d alpha.first
    :$ test -d alpha.second

Stage both again, then hide every selected session. Confirming must delete both,
not the currently selected prospective session. A second confirmation while the
batch is running must not schedule another batch.

    :k C-d C-u second
    :settle -d 2s -e '1/5' -e alpha/second -e C-d
    :k C-d C-u unmatched
    :snap -d 2s -e '2 sessions' -e '2 hidden'

After confirmation and rediscovery, clearing the query should show only the
three surviving entries, with no deletion footer or named workspaces remaining.

    :k C-y C-y
    :$ sh -c 'i=0; while test -d alpha.first || test -d alpha.second; do i=$((i+1)); test "$$i" -lt 100 || exit 1; sleep 0.05; done'
    :settle -d 2s -e '0/3'

    :k C-u
    :snap -d 2s -e '3/3'

    :$ jj workspace list -R alpha --ignore-working-copy --no-pager --color never --template 'name ++ "\n"'

    :t has-session -t alpha/first

    :t has-session -t ui

---
vim: set ft=markdown:
