# Onto option

An explicit `--onto` revision should seed the interactive repository context
without opening the revision picker.

Initialize `alpha` with a described working-copy commit, then launch from outside
it so repository and revision context must come from explicit CLI options.

    :b jj
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ jj git init alpha
    :$ jj describe -R alpha -m "base commit"

Launch the picker from outside the repository. The header should display the
revision supplied on the command line rather than `trunk()`.

    :t new-session -d -s ui "smth --base alpha --onto @"
    :t resize-window -t ui:0 -x 100 -y 10
    :p ui:0.0
    :snap -d 2s

An explicit revision is invalid when `--no-base` suppresses repository context
or when no repository can be inferred.

    :$ smth --no-base --onto @ --filter

    :$ smth --onto @ --filter

---
vim: set ft=markdown:
