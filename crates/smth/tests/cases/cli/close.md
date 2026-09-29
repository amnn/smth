# Close sessions

`--close` should kill only a strictly resolved live tmux session and preserve
all jj workspace state and checkout directories.

Create an `alpha` repository with `feature` and `other` workspaces so closure can
be checked separately from checkout deletion and workspace registration.

    :b jj tmux cat sh sed test
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ smth --no-base --create-repo --create alpha

    :t rename-session -t alpha alpha-live

Rename the default session above to exercise identity independent of its tmux
name. Pre-create the exact `feature` workspace before reserving its natural tmux
name: otherwise creating a missing workspace would disambiguate its checkout too.
The CLI should only disambiguate the live session attached to this workspace.

    :$ jj workspace add -R alpha --name feature alpha.feature
    :t new-session -d -s alpha/feature "cat"
    :$ smth --base alpha --create feature

    :$ smth --base alpha --create other

    :t new-session -d -s scratch "cat"

A named-workspace base should infer `feature` and close the disambiguated
repo-backed session while leaving the colliding plain session alive.

    :$ smth --base alpha.feature --close
    :t has-session -t '=alpha/feature~1'

    :t has-session -t '=alpha/feature'
    :$ test -d alpha.feature
    :$ sh -c 'jj workspace list -R alpha --ignore-working-copy --no-pager --color never --template "name ++ \"\\n\"" | sed -n "/feature/p"'

An explicit operand overrides the inferred workspace, and omitting the operand
for a default base closes the default checkout's live session.

    :$ smth --base alpha.feature --close other
    :t has-session -t '=alpha/other'

    :$ test -d alpha.other
    :$ smth --base alpha --close
    :t has-session -t '=alpha-live'

    :$ test -d alpha

The plain namespace remains independent: it can close both an ordinary plain
session and the plain collision without consulting repository metadata.

    :$ smth --no-base --close scratch
    :t has-session -t '=scratch'

    :$ smth --no-base --close alpha/feature
    :t has-session -t '=alpha/feature'

Closing a registered workspace with no live session fails safely, as does an
unspecified plain target.

    :$ smth --base alpha --repo "alpha*" --close feature

    :$ smth --no-base --close

Close rejects picker controls and every other lifecycle action.

    :$ smth --no-base --close runner --query runner

    :$ smth --no-base --close runner --create runner

---
vim: set ft=markdown:
