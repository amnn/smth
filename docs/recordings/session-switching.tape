# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

# We start with a live review session and a docs checkout that has no session yet.
# We'll find the review, open docs, then create a completely new workspace.
# Run through record.py for isolated sessions, frame-synced captions, and cleanup.
Output docs/assets/session-switching.gif
Require tmux
Source docs/recordings/theme.tape
Set TypingSpeed 140ms

# The main checkout's log provides context; captions begin when we open the picker.
Hide
Type@1ms "tmux attach-session -t smth"
Enter
Wait+Screen /smth \$/
Show
Sleep 1s

# The list and preview let us inspect work before switching. Squares identify live
# sessions; docs is an existing checkout without a square or a running session.
Ctrl+B
Type "s"
Wait+Screen /C-x.*close/
# caption: Browse sessions and preview their changes.
Sleep 3s

# We don't have to spell out review: non-adjacent letters are enough to find it.
Type "rvw"
Wait+Screen /Work on review/
# caption: Fuzzy-find sessions as you type.
Sleep 3s

# Accepting the match switches to review, where the diff is still on screen.
# caption: Press Enter to switch sessions.
Sleep 2s
Enter
Wait+Screen /return f/
Wait+Screen /smth\/review \$/
Sleep 2s

# Docs already exists on disk, but has no tmux session. Enter opens one in that
# checkout instead of creating another workspace; jj status confirms we're ready.
Ctrl+B
Type "s"
Wait+Screen /C-x.*close/
Type "docs"
Wait+Screen /Work on docs/
# caption: Open a session for an existing workspace.
Sleep 3s
Enter
Wait+Screen /smth\/docs +0:/
Sleep 500ms
Type "jj status"
Enter
Wait+Screen /The working copy has no changes/
Sleep 2s

# A new name offers a workspace creation row. Enter creates experiment at trunk()
# and opens its session; the workspace list now includes the new checkout.
Ctrl+B
Type "s"
Wait+Screen /C-x.*close/
Type "experiment"
Wait+Screen /smth\/experiment.*workspace/
# caption: Create a new workspace from the picker.
Sleep 3s
Enter
Wait+Screen /smth\/experiment +0:/
Sleep 500ms
Type "jj workspace list"
Enter
Wait+Screen /experiment:/
Sleep 3s
