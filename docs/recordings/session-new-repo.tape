# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

# Clear context and choose the repository candidate rather than the default plain
# session. The demo config places fresh repositories beside the existing checkouts.
# Run through record.py for isolated fixtures, frame-synced captions, and cleanup.
Output docs/assets/session-new-repo.gif
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
Alt+r
Wait+Screen /repo: none/
# caption: Clear context with M-r to start a fresh repo.
Sleep 2s
Type "atlas"
Wait+Screen /atlas.*repo/
Alt+k
# caption: Select the repo row, not the tmux row.
Sleep 3s
Enter
Wait+Screen /atlas +0:/
Sleep 500ms
Type "jj status"
Enter
Wait+Screen /The working copy has no changes/
# caption: Enter opens the new repository's session.
Sleep 3s
