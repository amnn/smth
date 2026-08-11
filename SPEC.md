# sesh: tmux session switcher

A tmux-native session switcher, for navigating between and opening new sessions
based on jujutsu (jj) repositories and workspaces. The session switcher
supports opening new sessions:

- based on a jj repository
  - ...and existing workspace
  - ...and an `onto` revision (to create a new workspace)
  - ...on its own.
- based on a custom name (and no repository), to create a simple tmux session.

## Configuration
The switcher is configured via a configuration file at
`~/.config/sesh/sesh.toml`, containing the following properties:

- `notification.bell`: Whether to emit a terminal bell in the agent pane.
  Defaults to false.
- `notification.command`: An optional root argument array executed directly.
  An empty or omitted array disables command delivery. Nested arrays are
  recursively evaluated depth-first and POSIX-shell-joined into one parent
  argument. Command strings interpolate `{message}`, `{pane}`, `{socket}`,
  `{state}`, `{title}`, and `{tty}`.
- `repo.globs`: A list of glob patterns to locate jj repositories. These stack
  with repository globs supplied on the command line. A leading `~` path
  component expands to the user's home directory.
- `ui.sigil`: A character used to indicate a live tmux session.
- `workspace.template`: A template for naming new workspaces. This can be
  a relative path that ends in a directory name that contains the `{repo}`
  and `{name}` placeholders. `{repo}` is the repo basename, and `{name}`
  must not contain `/`.
  - **Default**: `../{repo}.{name}`
- `session.template`: A template for configuring how to set up a new tmux
  session.
- `session.name`: A template for configuring the name of the tmux session.

## State
The session switcher maintains the following state:

- When the switcher is open, it remembers:
  - The currently selected repository (if any). Defaults to the closest
    containing jj repo root to the current working directory of the active tmux
    pane, normalized to the recorded default workspace when that checkout
    exists.
  - The currently selected `onto` revision, if there is a selected repository.
    Defaults to `trunk()`, and resets if the selected repository changes. The
    revision picker lists bookmarks (including `origin/*`) and a `trunk()` entry.
- On each open session, tmux metadata is added to indicate whether the session
  corresponds to a repository/workspace, or is plain.
  - This is used by the session switcher to advertise metadata about
    existing sessions.
- Agent harnesses can publish idle, running, waiting, succeeded, or failed
  lifecycle state on their tmux pane. Multiple panes in one session contribute
  independently to that session's state. A state update may include a one-shot
  notification title and summary, which are not persisted in tmux metadata.

## UX
The session switcher opens a tmux pop-over when a tmux kebinding is pressed.
The popover includes a header, a fuzzy finder and a pane previewing the
selected session.

### Header
The fuzzy finder includes a header with the following information:

- Currently selected repo (if any).
- Currently selected `onto` revision (if there is a repo).
- Hints for keybindings:
  - `C-r` to change repo (next to the current repo).
  - `C-o` to change the `onto` revision (next to the current revision).
  - `C-n` to create a new session from the current query.
  - `C-x` to close a session and refresh the session list.

### Candidate Sessions
The fuzzy finder constructs a list of candidate sessions from the following
sources, in the following order:

- Existing tmux sessions.
- Repositories and workspaces found under `repo.globs` and command-line repo
  globs, in alphabetical order.

When reconciling existing sessions with candidate sessions, a name is generated
for each candidate session. If it matches the name of an existing session, the
candidate is discarded (existing sessions take precedence).

### Agent Lifecycle Indicators

Live session rows show right-aligned counts of pane-scoped agent lifecycle
state. The session-list header uses the same presentation aggregated across all
live sessions, independently of the active fuzzy filter. States appear in the
following order:

- `⏸`: waiting.
- `×`: failed.
- `✔`: succeeded.
- `▶`: running.
- `○`: idle.

A state with one agent shows only its indicator; larger counts use an indicator,
a space, and the count. Indicators and counts remain undimmed, while a non-empty
summary uses dim ` · ` separators and one dim outer space. It overdraws the
underlying row or header content at its right edge rather than reserving layout
width.

A live session's existing attention pip is active when a window has a bell
alert or an agent is waiting, failed, or succeeded. This attention styling
retains the existing precedence over manual flag styling.

### Agent Notifications

When a notification channel is enabled, an agent notifies only when crossing
from a non-attention state into waiting, succeeded, or failed. Repeated attention
states and transitions between attention states do not notify. State metadata
is published before notification discovery or delivery, and all notification
failures are ignored.

A client is categorically unfocused only when its terminal advertises focus
reporting, tmux focus events are enabled, and the client lacks tmux's `focused`
flag. Otherwise, its displayed pane conservatively counts as focused.
Control-mode, suspended, and tty-less clients are ignored. If delivery proceeds,
`sesh` selects the most recently active client from the first available group:
clients displaying the agent pane, clients not categorically unfocused, then all
eligible clients. Delivery may proceed without a client.

Bell delivery writes an ASCII BEL to `sesh agent`'s stdout. Harnesses that
capture stdout must forward it to the pane terminal so tmux can apply its normal
audible or visual bell handling. The command root runs directly as an argument
vector. Each nested array is evaluated recursively and shell-joined as one
argument at its parent depth. Interpolation occurs once before shell joining.
Titles and summaries collapse Unicode whitespace, NUL, and ESC into space
separators, preserve other non-whitespace control characters, and are truncated
safely. Bell and configured command delivery run concurrently and are
independently best-effort; configured command execution is bounded.

### Session Names and Metadata
The switcher represents sessions by their name and metadata.

- For a session without an associate repository, the name is simply the
  supplied session name, and there is no extra metadata.
- For a session with an associated repository or workspace, the name is the
  name of the directory containing the workspace root for that
  repository/workspace (derived from `workspace.template`).

### Preview Pane
The preview pane shows a live preview of the selected session, in a similar
style to tmux's `C-b s` session switcher, assuming the session already exists.

### Picking a Session
When picking a session from the fuzzy finder, all its parts are ensured to exist:

- If there is a workspace, it is set-up -- it is the CWD for the session.
- The session is created and added to tmux.

Then the pop-over switches to the session and closes itself. For an existing
session, the first window with either a bell or agent attention is the preferred
target. If no window needs attention, `sesh` uses the session's ordinary target.

### Actions
- `C-r` opens a sub-fuzzy-finder to select a different repository,
  populated by enumerating valid `jj` repositories found by evaluating
  `repo.globs` and command-line repo globs.
- `C-o` opens a sub-fuzzy-finder to select a different `onto` revision. This
  will only be enabled if a repository is selected. The fuzzy finder is
  populated with bookmarks (including `*@origin`), with `trunk()` included as
  a pseudo-entry at the top.
- `C-n` will create a new session from the current query. This will first
  check that a session with this name doesn't already exist, and if so,
  follows the "picking a session" flow above.
- `C-x` will close the selected existing tmux session, then refresh discovered
  sessions while preserving the current query.
- `C-d` will delete an existing session and/or workspace. If there is a
  session for this selection, it is closed in tmux. If the selection is tied
  to a real repository/workspace candidate, the workspace is forgotten in `jj`
  and deleted from disk; otherwise only the tmux session is closed.

## Tech recommendations
This tool will be built using Rust, taking advantage of `skim` for fuzzy
finding, and shelling out to `jj` and `tmux` for everything else.
