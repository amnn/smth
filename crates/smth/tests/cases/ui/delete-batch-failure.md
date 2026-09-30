# Concurrent deletion and partial failure

Every staged deletion should run concurrently in one pending task. A failed
entry must not cancel the remaining deletions, and its checkout path should be
reported after the whole batch finishes.

Create two named workspaces with no live sessions, so the batch exercises
concurrent checkout removal independently of session-closing behavior.

    :b jj tmux cat sh test chmod sed
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :t rename-session -t 0 runner
    :$ jj git init alpha
    :$ jj workspace add -R alpha --name first alpha.first
    :$ jj workspace add -R alpha --name second alpha.second

Wrap jj only inside the picker pane. Both forget commands announce readiness
and wait for separate release signals. The first command fails deliberately;
the second runs the real jj command.

    :w wrappers/jj

```sh
#!/bin/sh
if test "$1 $2" = 'workspace forget'; then
    for name do :; done
    printf '' > "$HOME/$name-ready"
    tmux wait-for "$name-release"
    if test "$name" = first; then
        echo 'injected deletion failure' >&2
        exit 1
    fi
fi
exec "$HOME/../bin/jj" "$@"
```

    :$ chmod +x wrappers/jj
    :t new-session -d -s ui 'PATH="$HOME/wrappers:$PATH" smth -r "alpha*" 2>errors; : > finished; cat'
    :t resize-window -t ui:0 -x 120 -y 16
    :p ui:0.0
    :settle -d 2s
    :k C-p first
    :settle -d 2s -e '1/5' -e alpha/first -e C-d
    :k C-d C-u second
    :settle -d 2s -e '1/5' -e alpha/second -e C-d
    :k C-d
    :settle -d 2s -e '2 sessions' -e '1 hidden'
    :k C-y

Both commands must start before either is released, proving concurrency.
Navigation and query editing remain available; mutation, switching, and exit
keys remain gated while the confirmed batch is running.

    :$ sh -c 'until test -f first-ready && test -f second-ready; do :; done'
    :k C-u first
    :snap -d 2s -e '1/5' -e alpha/first -e deleting "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"

    :k C-d C-y C-x C-f C-n enter C-c esc C-g
    :$ test ! -f finished
    :$ test -d alpha.first
    :$ test -d alpha.second

Release the failing command first. The picker must still wait for the other
deletion rather than exiting early and cancelling it.

    :t wait-for -S first-release
    :settle "/[⠋⠙⠹⠸⠼⠴⠦⠧]/⠋"
    :$ test ! -f finished

    :t wait-for -S second-release
    :$ sh -c 'until test -f finished; do :; done'

The error identifies the failed checkout and the partial failure count. Normalize
the temporary home path in the diagnostic. The successful checkout is removed
and the failed checkout remains registered.

    :$ sh -c 'sed "s|$(pwd -P)/|~/|g" errors'

    :$ test -d alpha.first
    :$ test ! -d alpha.second
    :$ jj workspace list -R alpha --ignore-working-copy --no-pager --color never --template 'name ++ "\n"'

---
vim: set ft=markdown:
