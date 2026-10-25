# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

# Run through record.py for isolated sessions, frame-synced captions, and cleanup.
Output docs/assets/session-ordering.gif
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
Type `s`
Wait+Screen /C-s.*↑l.*↑r.*↓n/
# caption: A priority stack: live, recency, then name.
Sleep 3s

Ctrl+S
Wait+Screen /↑live.*↑recency.*↓name.*attention/
# caption: C-s edits; bold initials select comparators.
Sleep 3s
Type "n"
Left
Left
Wait+Screen /↓name.*↑live/
# caption: Move name first. Selection follows the session.
Sleep 3s
Type "n"
Wait+Screen /↑name/
# caption: Repeat n to reverse: ↑ means Z–A.
Sleep 3s
Type "n"
Wait+Screen /◂ name/
# caption: Repeat to disable name, keeping its position.
Sleep 3s
Type "n"
Wait+Screen /↓name/
# caption: One more press enables ascending order (A–Z).
Sleep 3s
Ctrl+S
Wait+Screen /C-s.*↓n.*↑l.*↑r/
# caption: C-s accepts for this run, without saving.
Sleep 3s

Ctrl+S
Wait+Screen /↓name.*↑live/
Type "n"
Wait+Screen /↑name/
# caption: Preview reversal; keep the selected session.
Sleep 3s
Ctrl+G
Wait+Screen /C-s.*↓n.*↑l.*↑r/
# caption: C-g restores the accepted stack and selection.
Sleep 3s
Escape
Sleep 1s
