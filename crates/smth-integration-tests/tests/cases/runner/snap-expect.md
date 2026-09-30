# Capture expected pane content

A conditional snapshot waits past the stable old screen and captures the same
frame that satisfied every condition. Matching only `waiting` is not enough.
Create a pane that prints `waiting`, blocks on a tmux signal, then prints
`snapshot-ready` after asynchronous release. The snapshot should include both
lines, not just the stable screen that existed before release.

    :b sleep
    :t new-window -d -n expected 'echo waiting; tmux wait-for -S snap-old-ready; tmux wait-for snap-release; echo snapshot-ready; sleep 10'
    :p 0:expected.0
    :t resize-window -x 40 -y 3 -t 0:expected
    :t wait-for snap-old-ready
    :settle -c 1 -e waiting

    :t run-shell -b 'sleep 0.3; tmux wait-for -S snap-release'
    :snap -d 2s -c 1 -e waiting -e snapshot-ready

The short alias supports expectations too, evaluated after replacement filters.

    :s -c 2 -e waiting -e snapshot-XXXXX /ready/X

Matching only one expectation reports all required conditions instead of emitting
a successful terminal snapshot.

    :snap -d 50ms -c 1 -e missing -e snapshot-ready

---
vim: set ft=markdown:
