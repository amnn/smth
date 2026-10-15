# Session tags

Tags are independent many-to-many memberships, not attention flags. Create two
plain live sessions and a repository with a non-live named workspace. Capture
the physical fixture directory as ROOT to normalize JSON paths to <ROOT>.

    :b jj cat sed pwd
    :$ -q pwd -P
    := ROOT

    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :t new-session -d -s first "cat"
    :t new-session -d -s second "cat"
    :$ -q smth --no-base --create-repo --create alpha
    :$ -q jj workspace add -R alpha --name idle alpha.idle

Both plain sessions can carry a, and first can also carry z. Repeated tagging
is idempotent. Successful mutations exit 0 without output. Flagging first must
not alter either membership; JSON lists
first's tags alphabetically and reports the flag independently.

    :$ smth --no-base --tag z first
    :$ smth --no-base --tag a first
    :$ smth --no-base --tag a first
    :$ smth --no-base --tag a second
    :$ smth --no-base --flag first
    :$ smth --no-base --json --query first

Removing a twice from first must preserve z, its flag, and second's a.

    :$ smth --no-base --untag a first
    :$ smth --no-base --untag a first
    :$ smth --no-base --json --query first

    :$ smth --no-base --json --query second

A default-workspace base supplies the target when omitted. A named workspace
base does the same; an explicit operand overrides it. The tmux option queries
print 1 because each preceding command enabled that membership.

    :$ smth --base alpha --tag b
    :t show-options -qv -t '=alpha:' @smth.tag.b

    :$ -q smth --base alpha --create feature
    :$ smth --base alpha.feature --tag c
    :t show-options -qv -t '=alpha/feature:' @smth.tag.c

    :$ -q smth --base alpha --create other
    :$ smth --base alpha.feature --tag d other
    :t show-options -qv -t '=alpha/other:' @smth.tag.d

Closing and recreating first loses its tags and flag, while second remains
in a. Separate CLI invocations above already demonstrate persistence across
smth process exits.

    :$ smth --no-base --close first
    :t new-session -d -s first "cat"
    :$ smth --no-base --json --query first

    :$ smth --no-base --json --query second

A and aa violate the lowercase single-letter constraint, missing identifies no
session, and idle has no live tmux session. These operational errors exit 1.
The non-live idle candidate has no tags field in JSON; its paths are normalized
to <ROOT> using the physical directory captured during setup.

    :$ smth --no-base --tag A first

    :$ smth --no-base --tag aa first

    :$ smth --no-base --tag a missing

    :$ smth --base alpha --repo 'alpha*' --tag a idle

    :$ smth --base alpha --repo 'alpha*' --json --query idle
    :| sed "s#${ROOT}#<ROOT>#g"

Tag mutations reject picker controls and other lifecycle actions during argument
parsing, producing usage diagnostics and exit 2.

    :$ smth --no-base --tag a first --query first

    :$ smth --no-base --tag a first --json

    :$ smth --no-base --tag a first --untag a first

---
vim: set ft=markdown:
