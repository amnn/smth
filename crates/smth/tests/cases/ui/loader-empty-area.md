# Resize a loading preview to an empty area

A short terminal leaves no rows for the preview. Its loader must not write
outside the terminal buffer. Completed content must appear when space becomes
available again.

Create a repository and wrap jj's log command so the preview stays loading
until the test releases it. Other jj operations use the real executable.

    :b jj tmux cat chmod
    :cp tests/fixtures/jjconfig.toml .jjconfig.toml
    :$ -q jj git init alpha
    :w wrappers/jj

```sh
#!/bin/sh
if test "$1" = log; then
    tmux wait-for -S log-ready
    tmux wait-for log-release
    echo 'loaded preview'
    exit 0
fi
exec "$HOME/../bin/jj" "$@"
```

    :$ chmod +x wrappers/jj
    :t new-session -d -s ui 'PATH="$$HOME/wrappers:$$PATH" smth --base alpha --query alpha; cat'
    :t resize-window -t ui:0 -x 120 -y 12
    :p ui:0.0
    :t wait-for log-ready
    :settle -d 2s -e '^session: alpha' -e 'Loading'

Shrink the window while the log is blocked. The picker must remain usable even
though there is no space to render the loading message.

    :t resize-window -t ui:0 -x 30 -y 4
    :snap -d 2s -e '^session: alpha'

Release the log and restore the window. The loaded preview must appear without
restarting the picker.

    :t wait-for -S log-release
    :t resize-window -t ui:0 -x 120 -y 12
    :snap -d 2s -e 'loaded preview'

---
vim: set ft=markdown:
