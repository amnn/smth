# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

# Follow one Pi review through failure and retry, then cut to several agents with
# different states. Every summary comes from real Pi processes using pi-smth.
# Run through record.py for isolated sessions, a scripted model, and caption cues.
Output docs/assets/agent-attention.gif
Require tmux
Source docs/recordings/theme.tape
Set TypingSpeed 65ms

# The agent-hooks session has a shell in window 0 and Pi in window 1. We begin
# without a caption, then open the picker to introduce the right-aligned summary.
Hide
Type@1ms "tmux select-window -t smth/agent-hooks:pi; tmux attach-session -t smth/agent-hooks"
Enter
Wait+Screen /scripted/
Show
Sleep 1s
Ctrl+B
Type "s"
Wait+Screen /○/
# caption: ○ Idle agents appear to the right of their session.
Sleep 3s
Escape
# caption: off
Sleep 500ms

# Pi reads the real session guide. While its request is in flight, the empty
# circle becomes a cyan play icon in both the session row and header.
Type "Review the session workflow."
Enter
Wait+Screen /read.*docs\/sessions.md/
Ctrl+B
Type "s"
Wait+Screen /▶/
# caption: ▶ means the agent is running.
Sleep 3s
Escape
# caption: off
Sleep 100ms

# The local model deliberately interrupts this request. A red cross identifies
# the failure, and the yellow session square calls attention to the agent.
Wait+Screen /Demo connection interrupted/
Ctrl+B
Type "s"
Wait+Screen /×/
# caption: Red × means the agent failed.
Sleep 3s
Escape
# caption: off
Sleep 500ms

# Retrying performs a real jj status check. The previous failure is cleared as
# soon as Pi starts the new request, and the play icon returns.
Type "Try again and summarize."
Enter
Wait+Screen /jj status/
Ctrl+B
Type "s"
Wait+Screen /▶/
# caption: ▶ Running again; the failure marker clears.
Sleep 3s
Escape
# caption: off
Sleep 100ms

# Give Pi's final reply a reading beat before visiting the shell. The picker then
# keeps the successful outcome visible while Pi's window is no longer on screen.
Wait+Screen /Ready for review/
Sleep 2s
Ctrl+B
Type "0"
Wait+Screen /smth\/agent-hooks \$/
Ctrl+B
Type "s"
Wait+Screen /✔/
# caption: Green ✔ means the agent finished successfully.
Sleep 3s

# The current session is above the initially selected unattached session.
# Selecting it takes us to Pi's response, not the shell that opened the picker.
Up
# caption: Jump straight to the window needing attention.
Sleep 2s
Enter
Wait+Screen /Ready for review/
Sleep 3s

# Cut away while the helper starts more real Pi processes and waits for their
# lifecycle states. Two workers join the successful agent in agent-hooks; review
# fails, docs stays idle, and experiment runs another worker. Setup isn't filmed.
Hide
Ctrl+B
Type "0"
Wait+Screen /smth\/agent-hooks \$/
Type@1ms "demo-busy"
Enter
Wait+Screen@60s /Demo agents ready/
Ctrl+B
Type "s"
Wait+Screen /▶ 3/
Show
# caption: ▶ 2 means two agents are running in this session.
Sleep 4s

# The same row also retains the completed review. Different states get separate
# symbols, while multiple agents in the same state share a count.
# caption: One session can contain several agent states.
Sleep 4s

# Other sessions have their own running, failed, or idle agents. Moving through
# the rows lets us compare their individual summaries without opening each Pi.
Down
# caption: Each session summarizes its own agents.
Sleep 2s
Down
Sleep 2s

# The header combines all those states, including the third running agent in
# experiment. It is a global summary rather than just the selected session.
# caption: The header summarizes agents across all sessions.
Sleep 4s

# Filtering the list down to agent-hooks leaves the header's full counts intact:
# the other sessions and their agents are still tracked outside this view.
Type "agent-hooks"
Wait+Screen /session: agent-hooks/
Wait+Screen /▶ 3/
# caption: Filtering leaves the global summary unchanged.
Sleep 4s
