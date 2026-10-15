# Session management

Use `smth` inside tmux to open, create, close, and delete sessions. Follow the
[README setup](../README.md#setup) to open the picker in a popup, and see the
[README keybinding table](../README.md#key-bindings) for all shortcuts.

## Session kinds and operations

The picker contains both existing sessions and candidates for sessions that do
not exist yet. The available operations depend on the kind of row:

| Session kind         | Open | Create | Close | Delete |
| -------------------- | ---- | ------ | ----- | ------ |
| New repo             | Yes  | Yes    | No    | No     |
| New workspace        | Yes  | Yes    | No    | No     |
| New tmux             | Yes  | Yes    | No    | No     |
| Live tmux            | Yes  | No     | Yes   | No     |
| Live repo            | Yes  | No     | Yes   | No     |
| Live workspace       | Yes  | No     | Yes   | Yes    |
| Discovered repo      | Yes  | Yes    | No    | No     |
| Discovered workspace | Yes  | Yes    | No    | Yes    |

### Session kinds

- **New repo:** initializes a fresh jj repository with a colocated Git
  repository, then starts a tmux session in its default checkout.
- **New workspace:** creates a named jj workspace and starts a tmux session
  there. The workspace has its own checkout directory but shares the base
  repository's jj history.
- **New tmux:** starts a plain tmux session in the directory where the picker
  was opened, without creating a repository or workspace.
- **Live tmux:** a running tmux session with no associated repository checkout.
- **Live repo:** a running tmux session associated with a repository's default
  checkout, rather than a named workspace.
- **Live workspace:** a running tmux session associated with a named jj
  workspace checkout.
- **Discovered repo:** an existing default checkout found through [repository
  discovery](configuration.md#repository-discovery), with no live tmux session.
- **Discovered workspace:** an existing named jj workspace checkout found
  through repository discovery, with no live tmux session.

The picker marks live sessions with `■` by default. New candidates do not yet
have a checkout or session to close or delete. Opening one changes its kind:
for example, a new workspace becomes a live workspace, while a new repo becomes
a live repo.

## Session tags

Tags are user-managed working sets named `a`–`z`. A live session can carry
multiple tags, and a tag can belong to multiple sessions. Adding a to one
session does not move it away from another. Tags are separate from manual
attention flags, bells, and agent notifications; they do not affect ordering or
fuzzy matching.

Membership is stored on the tmux session. It survives picker restarts but ends
when the session is destroyed, even if its repository checkout remains.
Non-live candidates cannot carry tags.

### Display

Tags appear alphabetically, right-aligned in the padding before the repository
path. They use only the space left by the name in the fixed name column: longer
names cause tags to contract with an ellipsis, then disappear when nothing
fits. The selected session's tags appear beside the left-aligned per-session
controls as `[C-t] tags: abc`. Tags may be clipped on narrow screens.

The default configuration uses bold lowercase letters in the flag colour,
including on the selected row. See [tag alphabet
configuration](configuration.md#tag-alphabet) to choose display glyphs.

### Edit tags

Press `C-t` on a live session:

- A lowercase first letter toggles that tag, saves, and finishes.
- An uppercase first letter toggles the corresponding lowercase tag and enters
  batch editing. Thereafter, either case toggles pending membership without
  saving. Repeated toggles can undo a pending change.
- `C-t` again saves the pending changes and finishes.
- `C-g` discards pending changes and returns to the picker.
- Esc or `C-c` discards pending changes and exits the picker.
- Other characters cancel and are consumed while awaiting the first tag. Once
  batch editing starts, unrecognized input is ignored instead.

For example, `C-t a` toggles a immediately; `C-t A c C-t` applies changes to a
and c together. `C-t A c C-g` discards both pending changes. The row previews
pending membership, and the footer shows apply/discard controls. Editing stays
anchored to the selected session; query edits and navigation are inactive.
Tag editing cannot begin during a background mutation or another modal operation.

Only changed memberships are written. A failed confirmation can leave earlier
writes applied.

## Session ordering and switching

With no query, ordering is a priority stack of independently enabled comparators.
The default is `-live -recency +name`: live sessions first, then most recently
attached, then names A–Z. Equal values fall through to the next comparator. No
extra name or attachment tie-breaker is implied; complete ties keep discovery
order.
Disable every comparator to use discovery order directly.

| Comparator | Values                                                           |
| ---------- | ---------------------------------------------------------------- |
| `live`     | Whether the session is live.                                     |
| `recency`  | Current attachment, otherwise last-attachment timestamp.         |
| `name`     | Session name.                                                    |
| `attn`     | Presence of a bell or waiting, failed, or succeeded agent alert. |

Currently attached sessions are **maximally recent**, even without a historical
timestamp. Descending recency puts attached sessions before newest-to-oldest
history; ascending puts oldest-to-newest history before attached sessions.
Never-attached sessions are least recent: they come first in ascending recency
and last in descending recency. With the default stack, live never-attached
sessions follow live visited sessions but precede non-live repository candidates.
Two attached sessions tie, regardless of their historical timestamps. Running
and idle agents, and manual flags alone, do not count as attention; alert count
and severity are irrelevant.

The arrows show the direction in which values **increase on the page**:
`↓` is ascending (`+`, A–Z or oldest first), and `↑` is descending
(`-`, Z–A or newest first). Thus `↑live` and `↑attention` put true values first.
The collapsed header shows only enabled initials: `[M-s] ↑l ↑r ↓n`.

### Edit ordering

Press `M-s` to expand the header. All four comparators appear, including disabled
ones without arrows. Bold coloured initials are shortcuts; the selected label
is underlined and flanked by small `◂` and `▸` movement triangles.

- `l`, `r`, `n`, `a`: select live, recency, name, or attention. Repeat the
  selected initial to cycle **ascending → descending → disabled → ascending**.
- Up/down: set the selected comparator to `↑` (descending) or `↓` (ascending).
  If it already has that direction, disable it instead. Either key enables a
  disabled comparator or switches directly from the opposite direction.
- Left/right: move the selected comparator one position, wrapping at either end.
- Tab/Shift-Tab: select the next/previous comparator, also wrapping.
- `M-s`: accept for this run. `C-g`: cancel and restore the original stack.

Entry selects the first enabled comparator, or the first comparator if none
are enabled. Disabled positions survive during the run. The list previews each
edit immediately, preserving the selected session and scrolling it into view.
Cancellation restores its position under the original ordering. Query editing
and session actions are frozen; ordering, onto, and tag editing are mutually exclusive.
`C-g` cancels ordering without clearing staged deletions. Escape and `C-c` still
exit (unless a background operation prevents it), discarding pending edits.

![Editing ordering with live preview, acceptance, and cancellation][order-demo]

Set initial ordering with `[ui].order` in [configuration](configuration.md), or
use `smth --order "-live -attn +name"`. CLI overrides config, which overrides
the built-in default. Each whitespace-separated token needs a sign and one of
`live`, `recency`, `name`, `attn`; duplicate or unknown names are errors.
Omitted comparators are disabled and appended in that fixed order in the editor.
An empty string disables everything. Edits never write configuration or a
preference file; the next run starts from configuration again.

[order-demo]: assets/session-ordering.gif

### Initial selection and filtering

The picker selects the first ordered result. With an empty query, if that is
the invoking session and another result exists, it selects the next one instead.
Other attached sessions are not skipped. With an explicit query, it selects the
first match even if current. Editing ordering does not rerun startup selection.

With a query, **fuzzy score takes precedence** and the comparator stack breaks
equal-score ties. Complete ties retain nucleo's ranking; disabling all comparators
uses fuzzy ranking directly. Ordering edits may move only equal-score rows, or
none at all, while preserving the selected session. Clearing the query applies
the stack to the full discovered list again.

New-session candidates remain separately above discovered matches. When starting
a query, selection defaults to the first discovered match; with no matches it
falls back to the last new-session candidate. Navigate explicitly to a candidate
when you want something new. `C-u` clears the query. The same ordering applies
to `--filter` and `--json` output.

## Open

Select a row and press `enter` to open it. Live sessions are reused, preserving
running programs; smth prefers the first window with a bell or agent attention.
A discovered repo or workspace gets a new tmux session in its existing checkout.
New-session candidates create the required checkout, if any, before starting tmux.

![Filtering sessions and pressing Enter to switch, open a discovered workspace, or create a new one](assets/session-switching.gif)

### Choose repository context

The current repository context determines which new-session candidates appear.
Opening the picker inside a jj checkout selects its repository as context.
Select a repo-backed row and press `C-r` to use its repository, or press `M-r`
to clear context. `C-r` on a row without a repository also clears context.
Discovery determines which existing checkouts are listed; it does not by itself
select the context for creating workspaces.

![Clearing repository context with M-r and selecting it from a checkout with C-r](assets/session-context.gif)

### Open a new repo

Clear repository context with `M-r`, then type a directory name. The picker
offers a new-repo row above the new-tmux row when the destination is available.
Select the new-repo row explicitly, then press `enter` or `C-n`.

The new checkout goes under the configured [repository creation
root](configuration.md#repository-discovery), or the directory where the picker
was opened if no root is configured. Missing parent directories are created.
Names must be non-empty directory names, not `.` or `..` or paths
containing separators. Occupied destinations are not offered; existing
directories are never initialized. Directory names are preserved exactly, while
tmux names are sanitized and disambiguated.

![Selecting the new-repo row and opening a fresh repository](assets/session-new-repo.gif)

### Open a new workspace

With repository context selected, type a workspace name and select the
new-workspace row. New workspaces start at `trunk()` by default. To choose a
different revision, press `C-o` to open the onto picker, type to search,
navigate with arrows or `tab` / `S-tab`, and press `enter` to accept a
revision. That accepts the revision, not the session; then press `enter` or
`C-n` on the session row. `C-o` or `C-g` cancels onto mode.

A new `feature` workspace normally lives beside the default checkout at
`/path/to/repo.feature`, with tmux name `repo/feature`. New workspace names and
paths are sanitized and disambiguated; collisions add `~N` suffixes. To
reopen an existing checkout instead, select its discovered row, not a
new-workspace candidate.

![Choosing a base revision with C-o and opening a new workspace](assets/session-new-workspace.gif)

### Open a new tmux session

Clear repository context with `M-r`, type a name, and select the new-tmux
row. Press `enter` or `C-n` to start the session in the directory where the
picker was opened. No directory or repository is created. New tmux names are
sanitized and disambiguated against existing sessions.

![Selecting the new-tmux row and opening a plain session in the current directory](assets/session-new-tmux.gif)

## Create

For any non-live row, `C-n` creates the session **without switching away** and
keeps the picker open. It uses the same repository context and checkout choices
as [Open](#open). Pressing `enter` instead creates it and switches to it.
[Configured tmux setup](configuration.md#tmux-session-setup) runs only when a
new tmux session is created, with either key.

![Creating a workspace with C-n while staying in the original session](assets/session-create.gif)

## Close

Select a live session and press `C-x` to close it immediately. This kills its
tmux windows and panes, but leaves its checkout and jj workspace registration
intact. Save any work in running programs first.

![Closing a live session with C-x and reopening its preserved checkout](assets/session-close.gif)

Multiple live sessions can refer to the same checkout; these are aliases.
Closing one alias does not close the others.

To reopen a repo-backed session, select its discovered repo or workspace row
and press `enter` or `C-n`. Ensure [discovery
globs](configuration.md#repository-discovery) cover the checkout if you want it
to remain visible after its live session closes. Closing a staged workspace's
session does not clear its deletion marker.

## Delete

**Deletion removes checkout contents, not just tmux sessions.** Only verified
named jj workspaces can be staged or deleted, whether live or discovered; plain
sessions and default checkouts cannot. Save or move anything you need before
confirming.

![Staging workspaces across filters, preserving and cancelling the selection, then confirming deletion](assets/session-delete.gif)

### Stage, inspect, confirm, or cancel

1. Select a named workspace row (live or discovered) and press `C-d` to stage
   it. Press `C-d` again to unstage it.
2. Navigate or filter to stage more. Filtering does not remove staged targets:
   the footer counts affected session rows, including hidden selections.
3. Press `C-y` to confirm and execute the **entire staged selection**, including
   hidden rows, not just the highlighted row. There is no second prompt.
4. Before confirmation, `C-g` clears all discovered staged markers without
   deleting anything or exiting. Ordering mode consumes `C-g` without clearing
   staged markers. If onto mode is open, the first `C-g` only cancels onto mode;
   press it again to clear staged deletions.

`esc` and `C-c` exit the picker **without clearing** staged deletions. Staging is
stored in each named workspace's `.jj/.smth-pending-delete` file, so it survives
picker exits and closing the checkout's live sessions. Markers in checkouts
that are no longer discovered are untouched; keep discovery globs covering
closed checkouts so you can inspect, unstage, or delete them later.

Aliases share one marker: staging or unstaging through either row affects both.
Counts describe session rows, not unique checkouts; execution deletes each
checkout once and closes all its discovered live aliases.

During a background operation, mutating actions and picker exit are disabled,
and the footer hides mutation shortcuts. Query editing and navigation remain
available. `C-g` is not an abort for a deletion already running.

### Partial failures

For each checkout, execution first forgets the workspace in jj, then removes
its directory, then closes its discovered live sessions. Checkouts are processed
concurrently, as are session closures after each removal. All tasks finish
before failures are reported; one failed target does not cancel the others.

A failure can leave a workspace forgotten but its directory present, or its
checkout removed with only some sessions closed. There is no rollback. Inspect
the reported paths and session names, the remaining files, jj workspace
registrations, and live sessions before retrying. Do not assume that an error
means nothing changed or that every failed target is still discoverable as a
staged workspace.
