# Startup selection

Open the picker inside alpha, the only session. It should select alpha and keep
it selected when beta is discovered later. Use alphabetical ordering so session
attachment history does not affect the test.

    :b jj cat
    :t rename-session -t 0 alpha
    :t resize-window -t alpha:0 -x 80 -y 12
    :t respawn-pane -k -t alpha:0.0 'smth --order +name'
    :settle -d 2s
    :k C-p
    :snap

Create beta outside the picker, then flag alpha so the picker discovers beta.
Alpha should remain selected rather than being skipped in favor of the new
session. Its footer shortcut changes from flag to unflag.

    :t new-session -d -s beta cat
    :k C-f
    :snap

---
vim: set ft=markdown:
