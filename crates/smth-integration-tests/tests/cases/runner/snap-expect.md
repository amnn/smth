# Capture expected pane content

A conditional snapshot waits past the stable old screen and captures the same
frame that satisfied every condition. Matching only `waiting` is not enough.
Release the waiting pane asynchronously.

    :bins echo sleep

    :t new-window -d -n expected 'echo waiting; tmux wait-for -S snap-old-ready; tmux wait-for snap-release; echo snapshot-ready; sleep 10'

    :pane 0:expected.0
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
