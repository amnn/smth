# Delete uses discovered sessions and closes aliases concurrently

Deletion must use the live sessions discovered by the model, without listing
sessions again. All matching session closures run concurrently and are awaited
even when one fails.

Create one named workspace and attach two differently named sessions to its
checkout. These are the initial aliases that discovery must retain as targets
before the wrappers introduce a failure and a late alias.

    :b jj tmux cat sh test chmod sed
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ -q jj git init alpha
    :$ smth --base alpha --create feature

Rename the CLI-created session and add the second alias manually: another CLI
create would reuse the first session rather than create a second deletion target.

    :t rename-session -t alpha/feature first
    :t new-session -d -s second -c alpha.feature "cat"
    :t set-option -F -t '=second:' @smth.repo '#{pane_start_path}'

Log session discovery and hold each kill request behind its own barrier. The
first kill fails; the second should still complete.

    :w wrappers/tmux

```sh
#!/bin/sh
case "$1" in
    list-sessions)
        echo discovery >> "$HOME/discoveries"
        ;;
    kill-session)
        name=${3#=}
        printf '' > "$HOME/$name-ready"
        "$HOME/../bin/tmux" wait-for "$name-release"
        if test "$name" = first; then
            printf '' > "$HOME/first-failed"
            printf 'injected kill failure' >&2
            exit 1
        fi
        ;;
esac
exec "$HOME/../bin/tmux" "$@"
```

Create another alias during workspace removal, after discovery has finished.
It must not become a deletion target merely because it shares the checkout.

    :w wrappers/jj

```sh
#!/bin/sh
if test "$1 $2" = 'workspace forget'; then
    "$HOME/../bin/tmux" new-session -d -s late -c "$HOME/alpha.feature" cat
    "$HOME/../bin/tmux" set-option -F -t '=late:' @smth.repo '#{pane_start_path}'
fi
exec "$HOME/../bin/jj" "$@"
```

    :$ chmod +x wrappers/tmux wrappers/jj
    :t new-session -d -s worker 'PATH="$$HOME/wrappers:$$PATH" smth --base alpha --delete feature 2>errors; echo $? > finished; cat'

Both kill requests must arrive before either is released. The checkout must
already be removed, but the deletion command must still be running.

    :$ sh -c 'until test -f first-ready && test -f second-ready; do :; done'
    :$ test ! -e alpha.feature
    :$ test ! -f finished

    :t wait-for -S first-release
    :$ sh -c 'until test -f first-failed; do :; done'
    :$ test ! -f finished

    :t wait-for -S second-release
    :$ sh -c 'until test -f finished; do :; done'

The first session remains after its injected failure, while the second closes.
The late alias stays alive because it was not part of discovery.

    :t has-session -t '=first'
    :t has-session -t '=second'

    :t has-session -t '=late'
    :$ cat discoveries

    :$ cat finished

Report the failed session only after all kill requests finish. Normalize the
temporary home path in the diagnostic.

    :$ sh -c 'sed "s|$(pwd -P)/|~/|g" errors'

---
vim: set ft=markdown:
