# Wait for expected pane content

A stable old screen must not satisfy an expectation for the next state, even
with a single required capture. All expectations must match the same frame;
matching only `waiting` is not enough. Release the waiting pane asynchronously.

    :bins echo sleep

    :t new-window -d -n expected 'echo waiting; tmux wait-for -S old-ready; tmux wait-for release; echo ready; sleep 10'
    :pane 0:expected.0
    :t resize-window -x 40 -y 3 -t 0:expected
    :t wait-for old-ready
    :settle -c 1 -e waiting

    :t run-shell -b 'sleep 0.3; tmux wait-for -S release'
    :settle -d 2s -c 1 -e waiting -e ready

    :snap -c 1

Expectations apply after replacement filters, just like stability checks.

    :settle -c 2 -e waiting -e XXXXX /ready/X

Matching only one expectation must time out and report all required conditions.

    :settle -d 50ms -c 1 -e missing -e ready

---
vim: set ft=markdown:
