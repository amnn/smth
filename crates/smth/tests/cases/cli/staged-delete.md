# Persistent staged deletion from the CLI

Create named workspaces in two repository families and an unrelated workspace
that must survive the batch. Use the public CLI to create normal session state.

    :b jj tmux cat sh sed test
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ smth -B --repo-root . --create-repo -c alpha

    :$ smth -B --repo-root . --create-repo -c beta

    :$ smth -b alpha -c first

    :$ smth -b beta -c second

    :$ smth -b alpha -c kept

Staging is idempotent, does not remove the checkout, and is visible to structured
inspection. The second request must not toggle the marker off.

    :$ smth -b alpha --stage-delete first
    :$ smth -b alpha --stage-delete first
    :$ test -f alpha.first/.jj/.smth-pending-delete
    :$ sh -c 'smth -r "alpha*" -q first --json | sed "s|$(pwd -P)|<ROOT>|g"'

Unstaging is also idempotent and preserves the checkout and live session.

    :$ smth -b alpha --unstage-delete first
    :$ smth -b alpha --unstage-delete first
    :$ test ! -f alpha.first/.jj/.smth-pending-delete
    :$ test -d alpha.first
    :t has-session -t '=alpha/first'

Stage both families again, then close the first session. Repository discovery
must still find its marker when the batch is executed.

    :$ smth -b alpha --stage-delete first
    :$ smth -b beta --stage-delete second
    :$ smth -b alpha --close first

Batch execution rejects either explicit base option, with no side effects.

    :$ smth -b alpha --delete-staged

    :$ smth -B --delete-staged

    :$ test -f alpha.first/.jj/.smth-pending-delete
    :$ test -f beta.second/.jj/.smth-pending-delete

Even when invoked inside alpha, deletion has no inferred base: all discovered
staged workspaces are removed, including beta's live session. The unmarked
workspace and both default checkouts survive. An empty selection is a no-op.

    :$ sh -c 'cd alpha && smth -r "../alpha*" --delete-staged'
    :$ test ! -e alpha.first
    :$ test ! -e beta.second
    :t has-session -t '=beta/second'

    :$ test -d alpha.kept
    :t has-session -t '=alpha/kept'
    :$ test -d alpha
    :$ test -d beta
    :$ smth --delete-staged

Plain sessions, default checkouts, missing targets, and incompatible actions
must not create markers or execute any deletions.

    :$ smth -B --stage-delete runner

    :$ smth -b alpha --stage-delete default

    :$ smth -b alpha --unstage-delete missing

    :$ smth --stage-delete

    :$ smth --stage-delete kept --delete-staged

    :$ smth --delete-staged -q kept

    :$ test ! -f alpha/.jj/.smth-pending-delete
    :$ test ! -f alpha.kept/.jj/.smth-pending-delete

---
vim: set ft=markdown:
