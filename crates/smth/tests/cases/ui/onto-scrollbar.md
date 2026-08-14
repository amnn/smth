# Onto scrollbar

This scenario verifies that the onto picker selects and inverts the working-copy
commit, navigates commits independently from fuzzy matching, jumps between
matches, and scrolls an overflowing current-repo log.

    :bins jj cat python3 sleep

    :copy tests/fixtures/jjconfig.toml .jjconfig.toml

    :w scripts/mklog.py

```python
from subprocess import run
from sys import argv

repo, prefix, count = argv[1:]
count = int(count)
for i in range(1, count + 1):
    run(["jj", "describe", "-R", repo, "-m", f"{prefix} {i:02d}"], check=True)
    if i != count:
        run(["jj", "new", "-R", repo], check=True)
```

    :t rename-session -t 0 runner
    :$ jj git init long
    :$ python3 scripts/mklog.py long line 6
    :$ jj new -R long
    :$ jj describe -R long -m child
    :$ jj edit -R long @-
    :t new-session -d -s plain "cat"
    :t new-session -d -s ui "cd long && smth -r ../long"
    :t resize-window -t ui:0 -x 90 -y 10
    :pane ui:0.0

Press `C-o` to render the current repo log in the onto picker. The child commit
appears first, but the working-copy commit marked `@` should start selected and
inverted. The scrollbar thumb should remain visible at the top edge.

    :settle -d 2s
    :k C-o
    :snap --color -d 2s "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Typing substring atoms for the stable `(empty) line 06` description should
update the fuzzy model used by rendering. Each leading apostrophe selects
substring matching, and all three atoms must match the same candidate line.
Matching characters should be reversed against their surrounding row; inside
the selected reversed `line 06` row, they should be switched back for contrast.

    :k "'(empty)" space "'line" space "'06"
    :snap --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

`Down` should move selection to `line 05` even though it does not match the
query. The matching characters stay reversed on `line 06` while the full-row
inversion moves down.

    :k Down
    :snap --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

`Up` should move selection back to the working-copy commit.

    :k Up
    :snap --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Pressing `Up` twice should move to the child commit, then remain there because it
is the first commit in the view.

    :k Up Up
    :snap --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Search for every numbered `line` commit. `Tab` should skip the selected,
non-matching child and select the next matching commit in rendered order. A
transient padded, inverted widget immediately left of the scrollbar shows that
this is the first of six matching commits.

    :k C-u line
    :settle

    :k Tab
    :snap --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

`S-tab` should wrap to the final matching commit before the root and update the
widget.

    :k BTab
    :snap --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

`Tab` should wrap back to the first matching commit.

    :k Tab
    :snap --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Moving manually should hide the match-position widget immediately, before its
timeout.

    :k Down
    :snap --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Jump to another match, then wait beyond the one-second timeout. The selection
should remain while the widget disappears.

    :k Tab
    :$ sleep 1.1

    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

Repeatedly pressing `Down` past the other end should leave the root commit
selected and scroll it into view.

    :k Down Down Down Down Down Down Down Down Down Down Down Down
    :snap --color "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

After jumping to a match, editing the query should immediately hide the match
counter without moving the selection.

    :k Tab
    :k backspace
    :snap "/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{1,2}/t" "/(?:@|○|◆)\s+([a-z]{8})/w" "/\b([0-9a-f]{8})\b/h"

---
vim: set ft=markdown:
