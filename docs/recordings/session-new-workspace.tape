# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

# Choose a new workspace name and use the onto picker to base it on the review
# change instead of trunk(). Accepting the revision does not yet open the session.
# Run through record.py for isolated fixtures, frame-synced captions, and cleanup.
Output docs/assets/session-new-workspace.gif
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
Type "experiment"
Wait+Screen /smth\/experiment.*workspace/
Alt+k
# caption: Name a new workspace in the current repo.
Sleep 2s
Ctrl+O
Wait+Screen /onto:/
Type "Work on review"
Wait+Screen /onto: Work on review/
Tab
# caption: C-o searches revisions; Tab selects a match.
Sleep 3s
Enter
Wait+Screen /session: experiment/
Wait+Screen /onto: [a-z]{8}/
# caption: Enter accepts the base revision first.
Sleep 3s
Enter
Wait+Screen /smth\/experiment +0:/
Sleep 500ms
Type "jj log --limit 3"
Enter
Wait+Screen /Work on review/
# caption: Enter again creates and opens the workspace.
Sleep 3s
