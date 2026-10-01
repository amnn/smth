# Retained picker action availability

Footer hints and input eligibility follow the same rendered selection and modal
state, including while session creation is blocked in the background. Create a
flagged live session named `first` so close, unflag, and deletion actions are
initially available. Give the repository's working-copy commit a `base` bookmark
so accepting it later produces a recognizable onto revision in the header.

    :b jj tmux cat test
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q jj git init alpha
    :$ -q jj describe -R alpha -m 'onto target'
    :$ -q jj bookmark create -R alpha -r @ base
    :$ smth --base alpha --create first

    :$ smth --base alpha --flag first

Future new sessions will wait to be released. Being able to control when they
finish allows us to test the effect of a creation in progress on action
availability.

    :w .config/smth/smth.toml

```toml
[tmux]
setup = '''
tmux wait-for -S ready
tmux wait-for release
tmux wait-for -S finished
'''
```

Open the picker in a separate tmux session and wait for it to settle.

    :t new-session -d -s ui "smth --base alpha -r 'alpha*'; cat"
    :t resize-window -t ui:0 -x 120 -y 14
    :p ui:0.0
    :settle -d 2s

Hide the preview with `C-p` and type `first` to filter the list. The existing
`alpha/first` session should become the selected row.

    :k C-p first
    :settle -d 2s -e alpha/first -e unflag

Select that existing session for deletion with `C-d`.

    :k C-d

The snapshot should show `first` staged for deletion. The footer should offer
unstage (`C-d`), close (`C-x`), and unflag (`C-f`), alongside confirmation
(`C-y`) and cancellation (`C-g`) of the one-session deletion batch.

    :snap -d 2s -e unstage -e '1 session'

Create a new session: clear the query with `C-u`, type `second`, and press `C-n`
to create it without switching.

    :k C-u second C-n

Wait until setup is blocked and the `creating` indicator appears. While creation
is in progress, both session actions and deletion confirmation should be hidden.

    :t wait-for ready
    :snap -d 2s -e creating "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

Creation clears the query. Type `first` to verify query editing still works.
The snapshot should show the filtered list with `alpha/first` selected.

    :k first
    :snap -d 2s -e '1/4' "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

Move Up to the new-session row. Wait for the selection marker to move to
`alpha/first~1`, so the next create attempt targets a new session rather than
the existing live session.

    :k up
    :settle -d 2s -e '▌.*alpha/first~1 ' "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

Try creating another session with `C-n`. It must be ignored while the first
creation is in progress.

    :k C-n

Move Down to the existing `alpha/first` session. The snapshot should show the
selection marker on that staged session, proving navigation remains available
during creation. The footer should still be empty.

    :k down
    :snap -d 2s -e '▌× alpha/first ' "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

Attempts to close (`C-x`), unstage deletion (`C-d`), unflag (`C-f`), confirm
the deletion batch (`C-y`), cancel the batch (`C-g`), switch sessions (Enter),
or exit (Escape and `C-c`) must be ignored. The snapshot should still show
`first` selected and staged, creation in progress, and no footer actions.

    :k C-x C-d C-f C-y C-g enter esc C-c
    :snap -d 2s "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

Open the onto picker with `C-o`. Onto mode takes precedence over the staged
deletion, even during creation, and should offer cancellation with `C-g`.

    :k C-o
    :settle -d 2s -e '^onto:' -e 'onto.*cancel' "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

Type `root` to edit the onto query while session creation is in progress.

    :k root

Use Down and Up to navigate revisions. The onto query and cancellation hint
should remain visible after navigation.

    :k down up
    :settle -d 2s -e '^onto: root' -e 'onto.*cancel' "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

Cancel onto selection with `C-g`. The session query should return to `first`,
with the loading indicator still visible and the footer still empty.

    :k C-g
    :snap -d 2s -e '^session: first' "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

Reopen the onto picker with `C-o` and wait for the working-copy commit to load.

    :k C-o
    :settle -d 2s -e '^onto:' -e 'onto target' "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

Accept it with Enter: this should return to session mode and change the header's
onto revision to `base`, even though creation is still in progress. The loading
indicator should remain visible and the footer empty. Unlike creating or
deleting a session, this only changes the base for future workspace creation.

    :k enter
    :snap -d 2s -e '^session: first' -e 'onto: base' -e creating "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

Release session setup and wait for it to finish.

    :t wait-for -S release
    :t wait-for finished

The selected live session should regain its unstage (`C-d`), close (`C-x`), and
unflag (`C-f`) actions. The staged batch should again offer confirmation (`C-y`)
and cancellation (`C-g`), with its one-session count unchanged.

    :snap -d 2s -e unstage -e '1 session'

Cancel the deletion batch with `C-g`, which should now work. The footer should
change from unstage to delete (`C-d`) and no longer show batch controls.

    :k C-g
    :snap -d 2s -e 'C-d. delete' -e unflag

Verify that both sessions exist: the close attempt was ignored, and the
original creation completed.

    :t has-session -t alpha/first
    :t has-session -t alpha/second

Verify that `first` remains flagged: the unflag attempt was ignored.

    :t show-option -t alpha/first -v @smth.flag

Verify that its workspace still exists: the deletion attempt was ignored.

    :$ test -d alpha.first

---
vim: set ft=markdown:
