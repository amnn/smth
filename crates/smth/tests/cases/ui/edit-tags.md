# Editing session tags

Create a live plain session. Keep the picker pane alive after exit so persisted
memberships remain inspectable. Hide the preview and filter to the target.

    :b jj cat tmux
    :t rename-session -t 0 runner
    :t new-session -d -s target "cat"
    :t new-session -d -s ui "smth --no-base; cat"
    :t resize-window -t ui:0 -x 100 -y 10
    :p ui:0.0
    :settle -d 2s
    :k C-p target C-t

The first-input footer shows the tags mode and cancellation, like other modes.
No tags exist yet.

    :snap

A lowercase first input saves a immediately and leaves editing. The option
query prints 1 because a is now persisted.

    :k a
    :snap -e 'tags: a'

    :t show-options -qv -t '=target:' @smth.tag.a

Uppercase B starts batch editing. Lowercase c then stages another tag without
committing. The row still shows saved tag a; the footer shows abc with pending
additions in the bold blue flagged style and unchanged a plain. Neither b nor c
exists in tmux yet.

    :k C-t B c
    :snap -e 'tags: abc \[C-t\] accept \[C-g\] cancel'

    :t show-options -qv -t '=target:' @smth.tag.b
    :t show-options -qv -t '=target:' @smth.tag.c

The modal footer puts memberships and accept/cancel controls together on the
right, leaving the left empty. Narrowing does not add footer rows.

    :t resize-window -t ui:0 -x 55 -y 10
    :snap --color -e 'tags: abc \[C-t\] accept \[C-g\] cancel'

Toggle a off: the footer retains it dimmed, while the row still
shows saved a. Toggle it back to restore its unchanged style.

    :k a
    :snap --color -e 'tags: abc.*accept'

    :k a
    :t resize-window -t ui:0 -x 100 -y 10

After the wider frame has rendered, digits and navigation are ignored in batch
editing: neither the target nor the query changes. C-g then discards b and c,
leaving only a.

    :k 1 down
    :snap -e '^session: target {80,}┃'

    :k C-g
    :snap -e 'tags: a'

Before staging, saved memberships appear on the right with only cancellation.
Removing a retains its glyph as a pending removal. Toggling it back leaves the
batch open with accept still available. Cancellation leaves saved a unchanged.

    :k C-t
    :snap -e 'tags: a \[C-g\] cancel'

    :k A
    :snap -e 'tags: a \[C-t\] accept \[C-g\] cancel'

    :k a
    :snap -e 'tags: a \[C-t\] accept \[C-g\] cancel'

    :k C-g

Repeating C-t confirms a batch. Toggling B twice cancels its pending change;
c remains the only addition. The option query for c prints 1.

    :k C-t B B c C-t
    :snap -e 'tags: ac'

    :t show-options -qv -t '=target:' @smth.tag.c

An invalid first character cancels and is consumed, not inserted into the query.

    :k C-t 1
    :snap

Flags remain independent. Adding the flag leaves ac intact and updates only
the flag control. A new non-live candidate cannot enter tag editing.

    :k C-f
    :snap -e unflag

    :k C-u missing C-t
    :snap -e 'session: missing'

Esc remains an overall exit, discarding pending tags. The wrapper signals
completion before keeping the pane alive. Waiting for that signal synchronizes
picker completion before querying c (still 1) and z (absent).

    :t new-session -d -s exit-ui "smth --no-base; tmux wait-for -S tags-exited; cat"
    :t resize-window -t exit-ui:0 -x 100 -y 10
    :p exit-ui:0.0
    :settle -d 2s
    :k target C-t Z esc
    :t wait-for tags-exited
    :t show-options -qv -t '=target:' @smth.tag.c

    :t show-options -qv -t '=target:' @smth.tag.z

A captured target disappearing before confirmation must report failure, not
redirect the edit to another session. The wrapper signals the failed picker's
exit; its captured error explains that previously applied changes may persist.
Before closing the target, the snapshot shows z staged and the apply control
visible because uppercase Z starts batch editing without committing.

    :t new-session -d -s failure-ui "smth --no-base; tmux wait-for -S tags-failed; cat"
    :t resize-window -t failure-ui:0 -x 100 -y 10
    :p failure-ui:0.0
    :settle -d 2s
    :k target C-t Z
    :snap -e 'tags: acz \[C-t\] accept'

    :$ smth --no-base --close target
    :k C-t
    :t wait-for tags-failed
    :snap -e 'failed to apply tag z'

---
vim: set ft=markdown:
