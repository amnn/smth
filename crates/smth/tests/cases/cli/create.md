# Create sessions

`--create` should ensure repo-backed and plain sessions exist without switching
the current tmux client, and report the actual tmux name it selected.

Configure setup to mark newly created sessions. Give `alpha` distinct base and
tip commits plus an existing `feature` workspace, distinguishing session-only
creation from workspace creation at a requested revision.

Capture the physical sandbox root once to normalize repository paths.

    :b jj tmux cat sed mkdir pwd
    :$ -q pwd -P
    := ROOT
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :w smth.toml

```toml
[tmux]
setup = "tmux set-option @smth.test-created yes"
```

    :t rename-session -t 0 runner
    :$ -q jj git init alpha
    :$ -q jj describe -R alpha -m "base commit"
    :$ -q jj new -R alpha
    :$ -q jj describe -R alpha -m "tip commit"
    :$ -q jj workspace add -R alpha --name feature alpha.feature

A default checkout needs only a tmux session. Its repo metadata and setup script
should be installed, while an explicit revision is ignored because no workspace
is being created.

    :$ smth --config smth.toml --base alpha --onto 'invalid(' --create

    :t has-session -t '=alpha'
    :t show-options -qv -t '=alpha:' @smth.test-created

    :$ tmux show-options -qv -t '=alpha:' @smth.repo
    :| sed "s#${ROOT}#<ROOT>#g"

An existing named workspace should get a session without another jj operation.
A colliding plain tmux name forces a suffix, which is stable on an idempotent
second call because strict repo metadata identifies the live target.

    :t new-session -d -s alpha/feature "cat"
    :$ smth --config smth.toml --base alpha --create feature

    :t show-options -qv -t '=alpha/feature:' @smth.repo
    :$ tmux show-options -qv -t '=alpha/feature~1:' @smth.repo
    :| sed "s#${ROOT}#<ROOT>#g"

    :t show-options -qv -t '=alpha/feature~1:' @smth.test-created

    :$ smth --config smth.toml --base alpha.feature --create

A missing named workspace should be created at the explicit revision before its
tmux session starts.

    :$ smth --config smth.toml --base alpha --onto @- --create from-base

    :$ jj log -R alpha.from-base --ignore-working-copy --no-pager --no-graph --color never -r @- --template description

    :t show-options -qv -t '=alpha/from-base:' @smth.test-created

When a named checkout has no registered default workspace, creation should use
that checkout as its degraded base just like the TUI.

    :$ -q jj git init beta
    :$ -q jj workspace add -R beta --name zeta beta.zeta
    :$ -q jj workspace forget -R beta.zeta --ignore-working-copy -- default
    :$ smth --config smth.toml --base beta.zeta --create fallback

    :$ tmux show-options -qv -t '=beta-zeta/fallback:' @smth.repo
    :| sed "s#${ROOT}#<ROOT>#g"

Workspace checkout path collisions preserve the TUI's `~N` disambiguation and
the printed tmux name exposes the suffix.

    :$ mkdir alpha.topic
    :$ smth --config smth.toml --base alpha --create topic

    :$ jj workspace list -R alpha --ignore-working-copy --no-pager --color never --template 'name ++ "\n"'
    :| sed -n '/topic/p'

A plain target is created at the process working directory with no repository
metadata. Matching is verbatim: creating from a name that needs sanitization
reports the sanitized name, recalling that exact name reuses the session, and
repeating the original request creates a freshly disambiguated session.

    :$ smth --config smth.toml --no-base --create "scratch one"

    :t show-options -qv -t '=scratch-one:' @smth.repo
    :t show-options -qv -t '=scratch-one:' @smth.test-created

    :$ tmux display-message -p -t '=scratch-one:0.0' '#{pane_current_path}'
    :| sed "s#${ROOT}#<ROOT>#g"

    :$ smth --config smth.toml --no-base --create scratch-one

    :$ smth --config smth.toml --no-base --create "scratch one"

    :t show-options -qv -t '=scratch-one~1:' @smth.test-created

A repo-backed name that sanitizes to empty is disambiguated against the existing
default checkout, producing a suffix-only workspace name.

    :$ smth --config smth.toml --base alpha --create "..."

    :t show-options -qv -t '=alpha/~1:' @smth.test-created

    :$ tmux show-options -qv -t '=alpha/~1:' @smth.repo
    :| sed "s#${ROOT}#<ROOT>#g"

None of these detached creates should switch the invoking client.

    :t display-message -p '#{client_session}'

A plain create without a name is invalid, and create remains mutually exclusive
with every other root action.

    :$ smth --no-base --create

    :$ smth --no-base --create scratch --flag scratch

---
vim: set ft=markdown:
