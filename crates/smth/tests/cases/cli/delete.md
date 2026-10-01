# Delete workspaces

`--delete` resolves a discovered named-workspace session, forgets and removes
its checkout, then closes every live session attached to that checkout.

Create `feature` and `other` workspaces in `alpha`; `feature` will have live
aliases, while `other` remains discoverable without a live session.

    :b jj tmux cat test
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q jj git init alpha
    :$ -q jj workspace add -R alpha --name other alpha.other

Create a plain collision and a disambiguated repo-backed feature session. Strict
metadata should ensure deletion closes only the latter. Pre-create `feature`
before reserving its tmux name so only the session name, not the workspace name,
needs disambiguation.

    :$ -q jj workspace add -R alpha --name feature alpha.feature
    :t new-session -d -s alpha/feature "cat"
    :$ smth --base alpha --create feature

The CLI reuses an existing session for a checkout, so add the second live alias
manually to exercise deletion of every session sharing that checkout.

    :t new-session -d -s feature-alias -c alpha.feature "cat"
    :t set-option -F -t '=feature-alias:' @smth.repo '#{pane_start_path}'
    :$ smth --no-base --create scratch

Stage the unrelated workspace. Explicit CLI deletion must replace the discovered
pending selection, not include it or clear its marker.

    :$ smth --base alpha --repo "alpha*" --stage-delete other
    :$ smth --base alpha --repo "alpha*" --delete feature
    :t has-session -t '=alpha/feature~1'

    :t has-session -t '=feature-alias'

    :t has-session -t '=alpha/feature'
    :$ test ! -e alpha.feature
    :$ test -d alpha.other
    :$ test -f alpha.other/.jj/.smth-pending-delete

A discovered workspace without a live tmux session should still be forgotten
and removed through the same model operation.

    :$ smth --base alpha --repo "alpha*" --delete other
    :$ test ! -e alpha.other

Only the default workspace should remain registered.

    :$ jj workspace list -R alpha --ignore-working-copy --no-pager --color never --template 'name ++ "\n"'

Missing sessions and the default checkout are not deletable.

    :$ smth --base alpha --delete missing

    :$ smth --base alpha --repo "alpha*" --delete default

    :$ test -d alpha

Delete rejects the plain-session namespace and never closes a same-named plain
session on an invalid request.

    :$ smth --no-base --delete scratch

    :t has-session -t '=scratch'

The operand is mandatory, picker controls are rejected, and delete is mutually
exclusive with every other lifecycle action.

    :$ smth --base alpha --delete

    :$ smth --base alpha --delete missing --query missing

    :$ smth --base alpha --delete missing --close missing

---
vim: set ft=markdown:
