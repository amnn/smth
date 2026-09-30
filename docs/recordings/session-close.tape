# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

# Close the live review session, then reopen its preserved checkout with Enter.
# Run through record.py for isolated fixtures, frame-synced captions, and cleanup.
Output docs/assets/session-close.gif
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
# caption: C-x closes a live session.
Sleep 2s
Ctrl+X
# The footer loses close/flag actions, and the checkout is still selected.
Wait+Screen /C-d. delete +│/
# caption: The checkout remains, without a live session.
Sleep 3s
# caption: Enter reopens the same checkout.
Sleep 2s
Enter
Wait+Screen /smth\/review +0:/
Sleep 3s
