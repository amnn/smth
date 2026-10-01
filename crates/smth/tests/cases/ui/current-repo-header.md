# Current repo header

This scenario launches `smth` from inside a repository directory to verify
the picker header shows the repo inferred from the current working directory.

    :b jj cat
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q jj git init alpha
    :$ -q jj describe -R alpha -m "alpha commit"
    :t new-session -d -s alpha-live "cat"
    :t new-session -d -s ui "cd alpha && smth"
    :t resize-window -t ui:0 -x 80 -y 10
    :p ui:0.0

This snapshot shows the initial picker state, including the current repo in the
header.

    :snap

---
vim: set ft=markdown:
