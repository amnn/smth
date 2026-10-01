# Delete repo session

`C-d` should be available for a repo-backed named workspace entry even when
there is no live tmux session to close. Confirming should forget the workspace
and delete the workspace checkout.

Create `beta` and its named `feature` workspace with distinct descriptions, but
no attached tmux sessions. The repository glob supplies both picker entries.

    :b jj tmux cat sh sleep
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q jj git init beta
    :$ -q jj describe -R beta -m "beta commit"
    :$ -q jj workspace add -R beta --name feature beta.feature
    :$ -q jj describe -R beta.feature -m "feature commit"
    :t new-session -d -s ui "smth -r 'beta*'; cat"
    :t resize-window -t ui:0 -x 120 -y 12
    :p ui:0.0
    :settle -d 2s

Filter to the repo-only named workspace entry. The footer should offer deletion
even though the selected row has no live tmux sigil.

    :k feature
    :snap -d 2s -e '1/4' -e beta/feature -e 'C-d. delete' "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Pressing `C-d` should mark the repo entry for deletion.

    :k C-d
    :snap -d 2s -e '1 session' --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Confirming should leave the picker alive, forget the workspace, and remove the
workspace checkout.

    :k C-y
    :$ sh -c 'i=0; while test -d beta.feature; do i=$((i+1)); test "$$i" -lt 100 || exit 1; sleep 0.05; done'
    :snap -d 2s -e '0/3' "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

    :$ sh -c 'test ! -e beta.feature'
    :$ jj workspace list -R beta --ignore-working-copy --no-pager --color never --template 'name ++ "\n"'

    :t has-session -t ui

---
vim: set ft=markdown:
