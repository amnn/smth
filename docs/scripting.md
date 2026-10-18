# Scripting

`smth` accepts fzf-style startup flags for scripted bindings. For interactive
workflows and common lifecycle examples, see the [session-management
guide](session-management.md).

Repository context is selected independently from discovery:

- `-b`, `--base REPO` uses a repository or workspace as the base context. A
  named workspace uses its repository's default checkout when available,
  following the same resolution as current-directory inference.
- `-B`, `--no-base` suppresses repository inference from the current working
  directory.
- With neither option, the nearest repository containing the current working
  directory is used when available.
- `-o`, `--onto REV` sets the revision used as the base of newly created
  workspaces. It defaults to `trunk()` and requires a repository base when
  supplied explicitly.
- `-r`, `--repo GLOB` adds repositories to discovery; it does not select the
  base context.
- `--repo-root PATH` selects the parent for fresh repository creation. It
  overrides `[repo].root`; without either setting, the process working
  directory is used. Relative paths are resolved from that directory, and a
  leading `~` path component expands to the user's home directory. This option
  does not add repositories to discovery.

The picker and filtering modes support these startup options:

- `--order ORDER` overrides `[ui].order` for the picker, `--filter`, `--json`,
  `--next`, and `--prev`. Quote the signed stack, for example `--order "-live -attn +name"`.
  Omitted keys are disabled; `--order ""` disables all comparators. The default
  is `-live -recency +name`. Without a query, complete ties keep discovery order.
  With a query, fuzzy score takes precedence and the stack breaks score ties.
  See [ordering][ordering].
- `-q`, `--query STR` seeds the interactive query or limits CLI candidates.
- `-1`, `--select-1` switches immediately when the initial query has one
  match.
- `-0`, `--exit-0` exits instead of opening the UI when the initial query has
  no matches.
- `-f`, `--filter` skips the UI and prints matches for the query from `--query`;
  combine it with `-1` to switch when there is exactly one match.
- `--json` skips the UI and emits structured records for live sessions and
  repository candidates. It cannot be combined with `--filter` or `--select-1`.
  `--query` narrows the output with the picker's fuzzy matcher. `--exit-0` is
  ignored so that an empty result is always emitted as `[]`.

[ordering]: session-management.md#session-ordering-and-switching

## Structured inspection

`smth --json` emits a JSON array. Every record includes its resolved tmux
name and live and deletion state. Repo-backed records include the normalized
default workspace path to pass to `--base`; plain sessions omit `base`. The
`name` field contains a named-workspace or plain-session operand and is
omitted for a default checkout. `path` is omitted when no checkout exists,
`flagged` appears only for live sessions, and `attention` and `agents`
are omitted when empty. `tags` is an alphabetically sorted array of lowercase
letters for live sessions (`[]` when untagged), and is omitted for non-live
candidates. Configured display glyphs never change JSON or query matching.
`pending_deletion` reports the checkout's persisted staging marker,
independently of whether the session is live or manually flagged.

Use the `base` and `name` fields together when constructing a lifecycle
command. Do not substitute a discovery glob or derive a target from the tmux
name: collision suffixes and repository metadata are resolved independently.

## Navigation

Use `smth --next TAG` or `smth --prev TAG` with a lowercase tag letter or `-`
to switch to the next or previous matching live session. Omitting the operand
is an error.

`-` is a special tag for sessions needing attention: manual flags, bell
alerts, or agents waiting, failed, or succeeded. Tags are otherwise independent
of attention signals.

'Next' and 'previous' are relative to the current tmux session and the
configured session order, with any `--query` filters applied. Navigation uses
the same ordering and filtering as the picker. `--order ORDER` overrides
`[ui].order`; use `--order +name` for alphabetical ordering unaffected by
recency changes.

Traversal starts after the current session in the chosen direction, even if it
does not carry the tag, and wraps around at either end. If the current session
is unavailable or filtered out, traversal starts between the first and last
sessions in the list: next starts at the first and previous at the last. If
there is no matching session other than the current one, the command succeeds
without switching or printing anything.

After a successful switch, stdout contains the destination session name.
Switching selects the first window needing attention, as with a normal session
switch.

Navigation is mutually exclusive with other root actions and rejects
`--select-1` and `--exit-0`.

```sh
smth --next a
smth --prev a --order +name --query project
smth --next -
smth --prev -
```

For example, tmux bindings can run `smth --next a` and `smth --prev a` using
`run-shell` without opening the picker.

## Lifecycle commands

Lifecycle commands are mutually exclusive with each other, `--filter`, and
`--json`. They also reject `--query`, `--select-1`, and `--exit-0`, and never
fall back to the interactive picker.

- `--flag [SESSION]` marks a matching live session as flagged.
- `--unflag [SESSION]` clears that state.
- `--tag TAG [SESSION]` adds one lowercase tag a–z to a matching live session.
- `--untag TAG [SESSION]` removes only that membership.
- `-c`, `--create [SESSION]` ensures the target exists without switching the
  current tmux client and prints its actual tmux name.
- `-s`, `--switch [SESSION]` performs the same ensure operation, then switches
  the current tmux client and prints its actual tmux name.
- `-x`, `--close [SESSION]` kills a matching live tmux session without removing
  its checkout or workspace registration.
- `-d`, `--delete SESSION` forgets and removes a matching discovered named
  workspace checkout, then closes its discovered live sessions.
- `--stage-delete SESSION` persistently stages a named workspace, idempotently.
- `--unstage-delete SESSION` removes its marker, also idempotently.
- `--delete-staged` deletes every discovered staged checkout and closes its
  discovered live sessions. It rejects `--base` and `--no-base` and never infers
  a base from cwd. Discovery still uses configured globs, `--repo`, and live
  sessions; undiscovered markers are unaffected. An empty selection succeeds.

Staging is shared with the picker and survives exit. CLI deletion never prompts;
inspect the selection with `--json` using the same discovery settings first.
See [staged workspace deletion](session-management.md#delete)
for the TUI flow, persistence, and partial-failure behavior.

Create and switch print the actual tmux session name on stdout after success,
whether creating or reusing the session. A failed operation does not print a
session name.

Flag and tag operations are idempotent and emit no output on success. Tags are
independent many-to-many memberships: adding the same tag to another session
does not remove any existing membership. Removing an absent tag is a no-op.
They are separate from attention flags and last only as long as the tmux session.

An explicit session operand overrides a named
workspace inferred from `--base`; without a repository base, a plain session
name is required. Repo-backed operations verify both the normalized repository
family and workspace checkout metadata before changing the tmux option.

For example:

```sh
smth --base ~/Code/project --tag a feature
smth --base ~/Code/project --tag a other
smth --base ~/Code/project --tag z feature
smth --base ~/Code/project --untag a feature
```

After these commands, `other` still carries a and `feature` still carries z.
For plain sessions, use `--no-base --tag a SESSION`. For a default checkout,
use `--base REPO --tag a` without a session operand. Invalid tag letters,
missing targets, and non-live targets fail rather than opening the picker.

Create starts a tmux session for an existing default or named checkout. If a
named workspace does not exist, create adds it beside the default checkout at
`--onto` (or `trunk()`), then starts tmux. A plain session starts in the process
working directory. Newly created tmux sessions run `tmux.setup`; existing live
sessions are left unchanged.

Names are sanitized and disambiguated against existing tmux sessions, sibling
workspaces, and checkout paths. Collisions use the first available `~N` suffix
(`~1`, `~2`, and so on). If the resulting tmux name would be empty, it instead
uses the first available numeric name (`1`, `2`, and so on). Callers should use
the printed name.

Switching to an existing live session prefers its first window with a bell or
agent attention, matching interactive picker behavior.

Add the no-argument `--create-repo` modifier to `--create [NAME]` or
`--switch [NAME]` to initialize a fresh repository before creating its session.
A non-empty directory name is required; path separators, `.` and `..` are
rejected. The resolved repository context must be empty. Use `--no-base` when
current-directory inference would otherwise select a repository; passing
`--base` is invalid, and `--onto` does not apply. The destination is
`<repo-root>/<NAME>` with the directory name preserved exactly; only the tmux
name is sanitized and disambiguated. Occupied paths are rejected, including on
repeated requests; existing directories are never initialized. It creates a
missing repository root, runs `jj git init --colocate` for the destination,
starts tmux in that checkout, records the checkout as repository metadata, and
runs `tmux.setup`.

Delete requires both a repository base and an explicit named workspace session.
It rejects plain sessions and the default workspace. Use `--repo` to surface a
non-live checkout that is not otherwise discovered. The command itself is the
confirmation: it forgets the workspace from jj, removes the checkout, and only
then closes the session when live.
