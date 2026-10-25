# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

# C-n creates a new workspace session while leaving both the picker and the
# original session open. Run through record.py for isolated fixtures and captions.
Output docs/assets/session-create.gif
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
Type "scratch"
Wait+Screen /smth\/scratch.*workspace/
# caption: C-n creates without switching sessions.
Sleep 2s
Ctrl+N
# Creation clears the query and brings discovery to five sessions. The new row
# may be below the viewport, so filter again once that discovery has completed.
Wait+Screen /5\/5.*C-s/
Type "scratch"
Wait+Screen /■.*smth\/scratch/
# caption: The new session is live; the picker stays open.
Sleep 3s
Escape
Wait+Screen /smth \$/
# caption: You're still in your original session.
Sleep 3s
