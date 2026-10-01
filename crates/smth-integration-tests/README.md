# smth integration tests

These tests run markdown directives in a headless tmux server and snapshot pane output with
`tmux capture-pane`. `:snap --color` also writes linked light and dark SVG snapshots for visual
style regression coverage.

## Test case syntax

Test cases live in `tests/cases/**/*.md`.

- Lines starting with four spaces or a tab followed by `:` are directives.
- Other markdown lines are copied verbatim into the snapshot transcript.

Supported directives:

- `:b` / `:bins <binary...>`
  - Make host binaries available in the sandboxed PATH.
  - Success appends `(available)` to the directive; failures produce warning
    callouts for unavailable binaries, without success callouts.
- `:$` / `:shell [-q|--quiet] <cmd...>`
  - Run a host command via Rust `Command`. `:sh` is not supported.
  - Arguments are parsed with `shlex`. Directive flags go before the
    executable; flags after it remain executable arguments.
  - The directive and exit annotation always appear. By default, stdout is
    rendered when present, and stderr is rendered on failure.
  - `-q` / `--quiet` suppresses successful output, not execution or internal
    capture of stdout/stderr. For example, `:$ -q smth --base alpha --create
    feature` runs setup without showing its successful output.
  - Non-zero exits still render captured stdout/stderr; spawn errors remain
    visible.
- `:=` / `:bind [-x|--export] <NAME>`
  - Bind the immediately preceding standalone `:$`/`:shell` command's raw stdout
    locally, or export it with `-x`. Names are literal and obey the assignment
    name restrictions. Values cannot contain NUL, even for local binds.
    Consecutive binds may read the same output.
  - Decode strict UTF-8 and remove exactly one trailing LF or CRLF. Preserve all
    other whitespace, ANSI escapes, and dollar signs. Empty stdout is valid;
    stderr is never included. Quiet successful commands still capture stdout.
    Completed nonzero/killed commands can bind; spawn failures cannot.
  - A blank/text line, another directive, or a parser error ends the bind chain.
    Orphans, invalid UTF-8, and invalid names/exports warn without changing the
    destination. Runtime bind failures keep stdout available to the next bind.
  - Only raw extraction is supported: no JSON/regex extraction or pipelines.
- `:v` / `:vars <NAME=VALUE ...>`
  - Set runner-local bindings in source order. Values expand against earlier
    bindings; names remain literal. Invalid assignments warn and leave that
    binding unchanged, without preventing later assignments. Names must be
    nonempty and contain neither `=` nor NUL; values cannot contain NUL.
  - Locals never enter the child environment and shadow exported bindings only
    during expansion, including sandbox defaults. Empty locals still shadow.
- `:e` / `:envs [-u|--unset] <NAME=VALUE ...>`
  - Set exported bindings for subsequent host commands, or remove named
    bindings with `--unset NAME ...`, including defaults. Assignments run left
    to right; values expand against bindings already applied. Names remain
    literal.
  - Invalid assignments warn and leave that binding unchanged; later
    assignments still run. Names must be nonempty and contain neither `=` nor
    NUL; exported values cannot contain NUL.
- `:t` / `:tmux <args...>`
  - Run a tmux command on the test socket.
  - Wait for the command queue to resume before continuing, including
    foreground `run-shell` jobs and `wait-for`. Background jobs still need
    explicit synchronization.
- `:p` / `:pane <target>`
  - Set current pane target (initially the default tmux session's pane).
  - Use this instead of `:tmux switch-client ...` when later `:keys`, `:shell`,
    or `:snap` directives should operate on the new pane; `:pane` waits for the
    control-mode pane notification to settle before the next directive runs.
- `:k` / `:keys <tokens...>`
  - Send key presses to the current pane.
  - Key names are lowercase only: `enter`, `up`, `down`, `left`, `right`,
    `backspace`, `btab`, `esc`, `tab`, `space`.
  - Modifiers are canonical uppercase only: `C-`, `M-`, `S-`.
  - `S-` only applies to arrow keys.
  - Anything that doesn't match the above is sent literally with `tmux
    send-keys -l`, including tmux names such as `Enter` and option-like text
    such as `-l`. Use lowercase `enter` for the actual key press.
- `:settle [-e <regex>]... [-c <count>] [-d <duration>] [dregexdgrapheme ...]`
  - Wait for the current pane to settle without appending a snapshot.
  - Accepts the same settle options and filters as `:snap`, but does not accept
    `--color`.
- `:s` / `:snap [--color] [-e <regex>]... [-c <count>] [-d <duration>]
  [dregexdgrapheme ...]`
  - Capture current pane and append it in a fenced `terminal` code block.
  - `-e` / `--expect` can be repeated. Every regex must match the same filtered
    pane text throughout the consecutive matching captures. If any regex does
    not match, the streak resets. A timeout reports all required regexes. With
    no expectations, only stability is checked. The snapshot renders the exact
    frame that satisfied every condition, including when `--color` is used.
  - `--color` additionally writes linked light and dark SVG snapshots.
  - `-c` / `--count` sets the required consecutive matching captures and
    defaults to `5`.
  - `-d` / `--duration` sets the maximum settle time and defaults to `1s`.
  - Durations use human-readable values such as `100ms` or `5s`.
  - Optional replacement rules are `dregexdgrapheme`, separated by
    whitespace.
  - Replacements are global and applied in order, painting over matches with
    the replacement grapheme cluster.
  - If the regex has capture groups, only those groups' contents are painted.
  - Filters match against the plaintext transcript, then the corresponding
    styled cells are painted before SVG rendering so the original cell styles
    are preserved.

## Variable expansion

Directives are shlex-tokenized and parsed once, without variable expansion. At
execution time, operands expand individually using `$NAME` or `${NAME}`.
Unbraced names consume ASCII letters, digits, and underscores (including
positional-looking `$1`). Braces allow other literal name characters. Locals
take precedence over environment bindings. Missing bindings expand to empty
strings; unterminated braces warn and skip the directive. `$$` produces one
literal dollar. A dollar not followed by a name or `{` is preserved.

Expansion does not run a shell, split words, or recursively expand substituted
values. Quotes group arguments but do not disable expansion. Escape shell-owned
dollars: `$$PWD`, `$$status`, `$$1`, `$${name:-default}`, and `$$$$`
for shell PID. Use `sh -c` explicitly for shell syntax. Fenced `:write`
contents and ordinary markdown are not expanded. Binary names, executable
names, command arguments, copy/write paths, pane targets, literal key text, and
assignment values expand at their point of use.

Expanded values are never reinterpreted as directive syntax: an executable
expanding to `-q` is an executable name, not `:shell`'s quiet flag, and key
text expanding to `enter` is typed literally rather than becoming the Enter
key. Directive flags, counts, durations, snapshot/settle regexes and filters
are parsed statically and do not expand. Regex dollar anchors therefore need no
runner escaping.

Sandbox `HOME`, `PATH`, `ENV`, `SHELL`, and `LC_CTYPE` defaults are
ordinary environment bindings, available for expansion and initialized once.
`TMUX` and `TMUX_PANE` initially identify the runner socket and pane.
`:envs` can replace or remove any binding. A successful `:pane` selection
sets `TMUX_PANE` to the selected pane ID, replacing any explicit override or
restoring an unset value; a failed selection leaves it unchanged. Other
bindings are not reset. Use `:pane` to update this environment context after
switching panes. The host process environment is not used for expansion.
Changing `HOME` does not move the sandbox working directory or file-operation
root. Exports apply to subsequent host commands; they do **not** update the
already-running tmux server or its panes. Use tmux's own environment options
when that is required.

Assignment names are literal in both namespaces; only values expand. Raw binds
preserve ANSI bytes and remove exactly one trailing line ending. Use
`-x/--export` to export a binding.

For example, capture the sandbox's physical root path and reuse it in a command:

```text
:$ -q pwd -P
:= ROOT
:$ printf '%s\n' "${ROOT}"
```

Declare `pwd` and `printf` with `:b` before running this example.

## Synchronizing asynchronous actions

`:settle` and `:snap` detect unchanged pane text, not completion of background work.
Before sending keys to a fresh picker, use `:settle -d 2s`. After starting an
asynchronous action, wait for an observable side effect before settling or
inspecting state. For example, install a one-shot hook before a session switch:

```text
:t set-hook -g client-session-changed "set-hook -gu client-session-changed; wait-for -S switched"
:k enter
:t wait-for switched
:settle -d 2s
```

For creation without switching, wait for the expected session or file instead.

Side effects can precede rediscovery and rendering. Before taking a UI snapshot or
sending an action that depends on refreshed selection, also wait for the expected
screen state, for example:

```text
:k C-d
:snap -d 2s -e '2 sessions' -e '1 hidden'
```

Use `:settle --expect` instead when waiting before a subsequent action rather than
capturing a snapshot.

Use a condition that distinguishes the new state from the old one. This also
applies to query changes before acting on a selected match. `--expect` is opt-in;
plain `:settle` still checks only stability, and increasing its timeout does not
make it wait for an operation to complete.

## Run tests

Run only this crate's integration test harness:

```bash
cargo nextest run -p smth-integration-tests --test test
```

Run the full workspace test suite:

```bash
cargo nextest run --workspace
```
