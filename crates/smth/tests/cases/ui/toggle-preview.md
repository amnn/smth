# Toggle preview

This scenario verifies that `C-p` toggles the preview pane.

Create a discoverable `alpha` repository with a described commit and open the
picker at a width where hiding the preview visibly expands the session list.

    :b jj cat
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ jj git init alpha
    :$ jj describe -R alpha -m "alpha commit"
    :t new-session -d -s ui "smth -r alpha"
    :t resize-window -t ui:0 -x 100 -y 12
    :p ui:0.0

The preview should be visible initially.

    :snap --color

Pressing `C-p` should hide the preview and allow the list to use the full height.

    :k C-p
    :snap

Pressing `C-p` again should restore the preview.

    :k C-p
    :snap --color

---
vim: set ft=markdown:
