# Tag navigation

Create three sessions carrying tag a and an untagged session, middle. Three
members distinguish forward from backward traversal. Each loop starts at alpha
and follows the destination after every switch, as successive invocations from
the active session would in practice. Configure alphabetical ordering to keep
the loops stable across switches; navigation otherwise inherits ui.order.

Each successful navigation prints its destination. Shell directives inherit
the runner's TMUX_PANE, which does not automatically follow client switches.
Use :p to move the runner to the printed destination and update TMUX_PANE for
the next invocation. The later anchor checks explicitly reset to middle rather
than continuing a loop.

    :w .config/smth/smth.toml

```toml
[ui]
order = "+name"
```

    :b jj cat
    :t rename-session -t 0 middle
    :t new-session -d -s alpha cat
    :t new-session -d -s beta cat
    :t new-session -d -s zebra cat
    :$ -q smth --no-base --tag a alpha
    :$ -q smth --no-base --tag a beta
    :$ -q smth --no-base --tag a zebra

Configured alphabetical navigation completes the forward loop:
alpha -> beta -> zebra -> alpha, wrapping past the untagged middle session.

    :p alpha:0.0
    :$ smth --next a

    :p beta:0.0
    :$ smth --next a

    :p zebra:0.0
    :$ smth --next a

Backward navigation completes the opposite loop:
alpha -> zebra -> beta -> alpha.

    :p alpha:0.0
    :$ smth --prev a

    :p zebra:0.0
    :$ smth --prev a

    :p beta:0.0
    :$ smth --prev a

Reversing the configured order makes next follow the backward loop without
an explicit --order: alpha -> zebra -> beta -> alpha. Previous from alpha
then selects beta, confirming that both directions honor the configuration.

    :w .config/smth/smth.toml

```toml
[ui]
order = "-name"
```

    :p alpha:0.0
    :$ smth --next a

    :p zebra:0.0
    :$ smth --next a

    :p beta:0.0
    :$ smth --next a

    :p alpha:0.0
    :$ smth --prev a

An explicit --order +name overrides the reverse configuration: next from alpha
selects beta, and previous from alpha selects zebra.

    :p alpha:0.0
    :$ smth --next a --order +name

    :p alpha:0.0
    :$ smth --prev a --order +name

Restore alphabetical configuration and reset to untagged middle for each
anchor check. Next selects zebra; previous selects beta. Neither command
requires the invoking session to carry tag a.

    :w .config/smth/smth.toml

```toml
[ui]
order = "+name"
```

    :p middle:0.0
    :$ smth --next a

    :p middle:0.0
    :$ smth --prev a

A query excludes middle, which has no fuzzy score and therefore follows all
matches. Next wraps to alpha, the only candidate. From alpha, both an empty
ring and a ring containing only alpha are successful no-ops with no output.

    :p middle:0.0
    :$ smth --next a --query alpha

    :p alpha:0.0
    :$ smth --next z

    :$ smth --prev z

    :$ smth --next a --query alpha

    :$ smth --prev a --query alpha

Query navigation uses the same fuzzy-ranked results as the picker, not a
separately sorted list. Create a distinct b ring whose names all match q.
The filter shows qa -> q -> zqz -> aq, despite the reverse-name tie-breaker.
Starting at q, next selects zqz and then aq; previous returns to zqz.

    :t new-session -d -s q cat
    :t new-session -d -s qa cat
    :t new-session -d -s zqz cat
    :t new-session -d -s aq cat
    :$ -q smth --no-base --tag b aq
    :$ -q smth --no-base --tag b q
    :$ -q smth --no-base --tag b qa
    :$ -q smth --no-base --tag b zqz
    :$ smth --filter --query q --order -name

    :p q:0.0
    :$ smth --next b --query q --order -name

    :p zqz:0.0
    :$ smth --next b --query q --order -name

    :p aq:0.0
    :$ smth --prev b --query q --order -name

When the query excludes the invoking middle session, its missing fuzzy score
places it after every match. Next starts at qa and previous starts at aq,
regardless of middle's position in the reverse-name ordering.

    :p middle:0.0
    :$ smth --next b --query q --order -name

    :p middle:0.0
    :$ smth --prev b --query q --order -name

Missing and invalid tag operands exit with argument-error status 2. A tag is
required, and uppercase A is not a lowercase a-z tag.

    :$ smth --next

    :$ smth --prev A

---
vim: set ft=markdown:
