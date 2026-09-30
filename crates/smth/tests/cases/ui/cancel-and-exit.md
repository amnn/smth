# Cancel internal state without exiting

C-g cancels the innermost mode, then pending deletions, and otherwise does
nothing. C-c and Esc exit the picker without clearing persisted deletions.

Create a discoverable `feature` workspace and launch the picker filtered to it.
Keep its pane alive after exit and signal each exit so persistence checks cannot
race the picker. Staging the workspace should show a one-session deletion footer.

    :b jj tmux cat test
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ jj git init alpha
    :$ jj workspace add -R alpha --name feature alpha.feature
    :t new-session -d -s ui "smth --base alpha -r 'alpha*' --query feature; tmux wait-for -S first-exited; cat"
    :t resize-window -t ui:0 -x 120 -y 12
    :p ui:0.0
    :settle -d 2s -e '1/4' -e alpha/feature -e 'C-d. delete'
    :k C-p C-d
    :snap -d 2s -e '1 session'

On a narrow terminal, the right-hand deletion controls take precedence over
session actions and the gap between them, even at extremely narrow widths.

    :t resize-window -t ui:0 -x 30 -y 4
    :snap

At two columns, the footer must clip safely rather than overflow the pane.

    :t resize-window -t ui:0 -x 2 -y 4
    :snap

    :t resize-window -t ui:0 -x 120 -y 12
    :settle -d 2s -e '1/4' -e alpha/feature -e '1 session'

Open onto mode over the staged deletion. C-g closes onto mode but leaves the
marker in place. Only the onto cancellation hint is shown while onto mode is
active, even with a deletion staged. A second C-g clears the marker, and a third
keeps the app open.

    :k C-o
    :snap -d 2s -e '^onto:' "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

The first cancellation returns to session mode with the deletion still staged.

    :k C-g
    :snap -d 2s -e '^session:' -e '1 session'

    :$ test -f alpha.feature/.jj/.smth-pending-delete

The second cancellation clears the deletion, restoring the ordinary delete hint.

    :k C-g
    :snap -d 2s -e alpha/feature -e 'C-d. delete'

    :$ test ! -f alpha.feature/.jj/.smth-pending-delete

With nothing left to cancel, C-g leaves the picker open and its query editable.
Deleting and retyping the final character should restore the original query.

    :k C-g backspace
    :settle -d 2s -e '^session: featur\s'
    :k e
    :snap -d 2s -e '^session: feature\s'

Stage again and exit with C-c. The exit signal synchronizes the assertion that
the marker survived, and the restarted picker shows the selection again.

    :k C-d
    :settle -d 2s -e '1 session'

    :k C-c
    :t wait-for first-exited

    :$ test -f alpha.feature/.jj/.smth-pending-delete

    :t respawn-pane -k -t ui:0.0 "smth --base alpha -r 'alpha*' --query feature; tmux wait-for -S second-exited; cat"
    :settle -d 2s -e '1/4' -e alpha/feature -e 'C-d. unstage'
    :k C-p
    :snap -d 2s -e '1 session'

Esc exits even from onto mode, leaving the marker intact.

    :k C-o
    :settle -d 2s -e '^onto:'
    :k esc
    :t wait-for second-exited
    :$ test -f alpha.feature/.jj/.smth-pending-delete

C-c also exits from onto mode, rather than cancelling it.

    :t respawn-pane -k -t ui:0.0 "smth --base alpha -r 'alpha*' --query feature; tmux wait-for -S third-exited; cat"
    :settle -d 2s -e '1/4' -e alpha/feature -e 'C-d. unstage'
    :k C-o
    :settle -d 2s -e '^onto:'
    :k C-c
    :t wait-for third-exited
    :$ test -f alpha.feature/.jj/.smth-pending-delete

Finally, Esc exits directly from session mode with the same persistence.

    :t respawn-pane -k -t ui:0.0 "smth --base alpha -r 'alpha*' --query feature; tmux wait-for -S fourth-exited; cat"
    :settle -d 2s -e '1/4' -e alpha/feature -e 'C-d. unstage'
    :k esc
    :t wait-for fourth-exited
    :$ test -f alpha.feature/.jj/.smth-pending-delete
    :$ test -d alpha.feature

---
vim: set ft=markdown:
