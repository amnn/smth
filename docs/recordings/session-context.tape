# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

# Context starts at the current repository. Clear it, restore it from a selected
# checkout, and show that the restored context offers a new workspace candidate.
# Run through record.py for isolated fixtures, frame-synced captions, and cleanup.
Output docs/assets/session-context.gif
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
Wait+Screen /repo:.*smth.*onto:/
# caption: The current directory sets repository context.
Sleep 3s
Alt+r
Wait+Screen /repo: none/
# caption: M-r clears repository context.
Sleep 3s
Type "review"
Wait+Screen /Work on review/
Ctrl+R
Wait+Screen /repo:.*smth.*onto:/
# caption: C-r uses the selected checkout's repository.
Sleep 3s
Ctrl+U
Type "scratch"
Wait+Screen /smth\/scratch.*workspace/
# caption: New workspaces use that repository.
Sleep 3s
