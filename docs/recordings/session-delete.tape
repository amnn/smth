# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

# Stage live review and discovered docs across filters. Exiting preserves the
# selection, C-g cancels it, and C-y deletes both checkouts and the live session.
# Run through record.py for isolated fixtures, frame-synced captions, and cleanup.
Output docs/assets/session-delete.gif
Require tmux
Source docs/recordings/theme.tape
Set TypingSpeed 140ms

Hide
Type@1ms "tmux attach-session -t smth"
Enter
Wait+Screen /smth \$/
Show
Sleep 1s

Ctrl+B
Type "s"
Wait+Screen /C-x.*close/
Type "review"
Wait+Screen /Work on review/
Ctrl+D
Wait+Screen /1 session/
# caption: C-d stages a workspace for deletion.
Sleep 2s

# Docs has no live session. Staging it also keeps review, hidden by filtering.
Ctrl+U
Type "docs"
Wait+Screen /Work on docs/
Ctrl+D
Wait+Screen /2 sessions.*1 hidden/
# caption: Filtering keeps hidden workspaces staged.
Sleep 3s

# Exit and reopen: the markers survive, including the hidden selection.
Escape
Wait+Screen /smth \$/
Sleep 1s
Ctrl+B
Type "s"
Wait+Screen /2 sessions/
Type "docs"
Wait+Screen /2 sessions.*1 hidden/
# caption: Staging survives closing the picker.
Sleep 3s

Ctrl+G
Wait+Screen /C-d. delete +│/
# caption: C-g cancels all staged deletions.
Sleep 3s

# Restage both checkouts, with docs hidden by the review query.
Ctrl+D
Wait+Screen /1 session/
Ctrl+U
Type "review"
Wait+Screen /1 hidden/
Ctrl+D
Wait+Screen /2 sessions.*1 hidden/
# caption: C-y deletes both and closes their sessions.
Sleep 3s
Ctrl+Y
# Deletion frees the exact name: the prospective review~1 row becomes review.
Wait+Screen /smth\/review +.*workspace/
Sleep 2s
Escape
Wait+Screen /smth \$/
Type "jj workspace list"
Enter
Wait+Screen /agent-hooks:/
# caption: Both workspaces are gone; others remain.
Sleep 3s
