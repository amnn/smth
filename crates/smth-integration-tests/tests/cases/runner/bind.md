# Standalone stdout bindings

Both aliases bind stdout locally. Quiet success suppresses rendering, not
capture. Consecutive binds read the same stdout, and `-x` exports explicitly.

    :b printf python3 pwd

    :$ -q printf 'two words\n'
    := LOCAL
    :bind --export EXPORTED

    :$ sh -c 'printf "<%s>\n" "$LOCAL" "$${LOCAL-missing}" "$$EXPORTED"'

Without quiet, both bind results should precede the command's stdout block.
The command below prints `visible`; expect its exit annotation, two successful
bind annotations, then `visible` once under stdout. Binding must neither consume
nor change the bytes displayed.

    :$ printf 'visible\n'
    := VISIBLE
    := -x VISIBLE_ENV

The local and exported values should both be `visible`, with the trailing newline
removed. Expect `visible/visible`: the runner expands `$VISIBLE`, and the shell
reads the exported value through the escaped `$$VISIBLE_ENV`.

    :$ sh -c 'printf "%s/%s\n" "$VISIBLE" "$$VISIBLE_ENV"'

An invalid bind must not prevent a later valid bind in the same group. The empty
name should warn, `REUSABLE` should bind successfully, and only then should the
command's `reusable` stdout appear. These bind results must not suppress output
from a successful non-quiet command.

    :$ printf 'reusable\n'
    := ''
    := REUSABLE

Printing `REUSABLE` should produce `reusable` again, confirming that the failed
bind neither consumed the stdout nor interrupted the following bind.

    :$ printf '%s\n' $REUSABLE

Literal bind names do not expand. Existing locals still shadow exported binds.

    :v NAME=EXPANDED EXPORTED=shadow

    :$ -q printf 'new value\n'
    := $NAME
    := -x EXPORTED

    :$ sh -c 'printf "<%s>\n" "${$NAME}" "$EXPANDED" "$EXPORTED" "$$EXPORTED"'

Bind a physical root without exposing it in the transcript. The value can be
passed directly to a later standalone command; no pipeline is involved.

    :$ -q pwd -P
    := ROOT

    :$ sh -c 'test "$ROOT" = "$(pwd -P)" && echo physical-root-matches'

Exactly one LF or CRLF is removed. Spaces, internal newlines, a lone CR, ANSI
escapes, and dollar signs remain untouched; bound values are not re-expanded.
Empty stdout is a valid binding. Python repr makes these boundaries visible.

    :$ -q printf ' first\nsecond \r\n'
    := MULTILINE

    :$ -q printf 'two\n\n'
    := TWO

    :$ -q printf 'lone\r'
    := CR

    :$ -q printf '\033[31m$$LOCAL\033[0m'
    := ANSI

    :$ -q printf ''
    := EMPTY

    :$ python3 -c 'import sys; print([repr(v) for v in sys.argv[1:]])' $MULTILINE $TWO $CR $ANSI $EMPTY

Nonzero exits still expose their stdout for binding, but stderr is never bound.
Quiet failure diagnostics remain visible. Expect the command's exit-7 annotation
and successful `PARTIAL` bind first, then `partial` under stdout and `error` under
stderr. The following `printf` should print `partial` again.

    :$ -q sh -c 'printf "partial\n"; printf "error\n" >&2; exit 7'
    := PARTIAL

    :$ printf '%s\n' $PARTIAL

A command terminated by a signal still supplies any stdout collected before it
exited. The shell prints `before-signal`, then sends SIGTERM to itself. Expect
`exit: killed`, a successful bind, and visible `before-signal` stdout despite
quiet, because the command did not succeed.

    :$ -q sh -c 'printf "before-signal\n"; kill -TERM $$$$'
    := SIGNALED

Printing `SIGNALED` should produce `before-signal` again, confirming the signal
status did not discard captured output.

    :$ printf '%s\n' $SIGNALED

A spawn failure marks the command `failed` and skips its associated
bind rather than attempting an assignment. Expect `KEEP (skipped)` before the
command-not-found warning, and `original` when its unchanged value is
printed. Orphan binds still warn after blank lines, text, parser errors, or
non-shell directives rather than reusing stale stdout.

    :v KEEP=original

    :$ no-such-bind-command
    := KEEP

    :$ printf '%s\n' $KEEP

    := ORPHAN

    :$ -q printf ignored
Text interrupts the bind chain.
    := ORPHAN

    :$ -q printf ignored
    :t display-message -p constant

    := ORPHAN

    :$ -q printf ignored
    :unknown

    := ORPHAN

Malformed bind syntax, unsupported extraction flags, and invalid UTF-8 are
reported. Failed local and exported NUL binds leave both old values intact.

    :bind

    :$ -q printf ignored
    := ''

    :$ -q printf ignored
    := TOO MANY

    :$ -q printf ignored
    := VALUE --json

    :$ -q python3 -c 'import sys; sys.stdout.buffer.write(bytes([255]))'
    := KEEP

    :$ printf '%s\n' $KEEP

    :e KEEP=environment

    :$ -q printf '\000'
    := KEEP

    :$ -q printf '\000'
    := -x KEEP

    :$ sh -c 'printf "%s\n" "$KEEP" "$$KEEP"'

---
vim: set ft=markdown:
