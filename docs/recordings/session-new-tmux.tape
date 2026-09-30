# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

# Select the plain-session candidate explicitly, above any fuzzy matches.
# Opening it creates no checkout and starts tmux in the picker's original directory.
# Run through record.py for isolated fixtures, frame-synced captions, and cleanup.
Output docs/assets/session-new-tmux.gif
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
# caption: M-r clears context for a plain tmux session.
Sleep 2s
Type "notes"
Wait+Screen /notes.*tmux/
Alt+k
Down
# caption: Select the tmux row, then press Enter.
Sleep 3s
Enter
Wait+Screen /notes +0:/
Sleep 500ms
Type "pwd"
Enter
Wait+Screen /home\/Code\/smth/
# caption: A plain session, in the original directory.
Sleep 3s
