# Structured inspection

`--json` should describe live tmux sessions and discovered repository checkout
candidates with identities suitable for strict lifecycle commands.

Create `alpha` and its named `feature` workspace, then attach a live session only
to the default checkout. This gives inspection both live and repo-only identities.

Capture the physical sandbox root once to normalize paths in JSON output.

    :b jj sed pwd
    :$ -q pwd -P
    := ROOT
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q smth --no-base --create-repo --create alpha
    :$ -q jj workspace add -R alpha --name feature alpha.feature

Rename the default checkout's live session, flag it, and publish an agent state
that requires attention. The runner session remains a plain live
session, while the named workspace is a non-live candidate discovered by the
glob.

    :t rename-session -t alpha alpha-live
    :$ -q smth --base alpha --flag
    :t set-option -p -t alpha-live:0.0 @smth.agent.state waiting
    :$ smth --no-base --repo "alpha*" --json
    :| sed "s#${ROOT}#<ROOT>#g"

The same fuzzy matcher used by the picker should narrow structured output, and
an unmatched query should emit an empty JSON array even with `--exit-0`.

    :$ smth --no-base --repo "alpha*" --query feature --json
    :| sed "s#${ROOT}#<ROOT>#g"

    :$ smth --no-base --repo "alpha*" --query zzz --json --exit-0

JSON inspection must not combine with line-oriented filtering or automatic
session selection.

    :$ smth --json --filter

    :$ smth --json --select-1

---
vim: set ft=markdown:
