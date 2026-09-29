# Flag management

Flat lifecycle flags should resolve live sessions by repository family and
workspace identity rather than by a guessed tmux name.

Create default, `feature`, `idle`, and `other` checkouts in one repository.
Leave `idle` without a live session to distinguish missing live targets from
registered workspaces.

    :b jj cat
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ smth --no-base --create-repo --create alpha

    :t rename-session -t alpha alpha-live
    :$ jj workspace add -R alpha --name idle alpha.idle

Create a plain session whose name collides with the natural repo-backed name,
then attach the real feature workspace to a disambiguated tmux session. Also
create live targets for another named workspace and the plain-session namespace.
The CLI supplies normal repo metadata; renaming the default session above keeps
coverage of identity resolution independent of the natural tmux name.
Pre-create `feature` before reserving its tmux name so the CLI disambiguates
only the live session, not the workspace and checkout being flagged.

    :$ jj workspace add -R alpha --name feature alpha.feature
    :t new-session -d -s alpha/feature "cat"
    :$ smth --base alpha --create feature

    :$ smth --base alpha --create other

    :t new-session -d -s scratch "cat"

Flagging `feature` must update only the repo-backed session, not the colliding
plain session. Repeating the command is idempotent.

    :$ smth --base alpha --flag feature
    :t show-options -qv -t '=alpha/feature:' @smth.flag
    :t show-options -qv -t '=alpha/feature~1:' @smth.flag

    :$ smth --base alpha --flag feature
    :t show-options -qv -t '=alpha/feature~1:' @smth.flag

Unflagging is also idempotent. A named-workspace base supplies the operand when
it is omitted, while an explicit operand overrides that inferred workspace.

    :$ smth --base alpha.feature --unflag
    :t show-options -qv -t '=alpha/feature~1:' @smth.flag
    :$ smth --base alpha.feature --flag other
    :t show-options -qv -t '=alpha/other:' @smth.flag

Omitting the operand for a default-workspace base targets the default checkout.
Plain sessions require the empty base namespace and an explicit name.

    :$ smth --base alpha --flag
    :t show-options -qv -t '=alpha-live:' @smth.flag

    :$ smth --no-base --flag scratch
    :t show-options -qv -t '=scratch:' @smth.flag

Missing, non-live, and unspecified targets should fail instead of opening the
picker or modifying a similarly named session.

    :$ smth --base alpha --flag missing

    :$ smth --base alpha --repo "alpha*" --flag idle

    :$ smth --no-base --flag

Lifecycle mode rejects picker controls and every other non-interactive action.

    :$ smth --no-base --flag scratch --query scratch

    :$ smth --no-base --unflag scratch --select-1

    :$ smth --no-base --flag scratch --exit-0

    :$ smth --no-base --flag scratch --filter

    :$ smth --no-base --flag scratch --json

    :$ smth --no-base --flag scratch --unflag scratch

---
vim: set ft=markdown:
