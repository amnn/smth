# Standalone stdout bindings

Both aliases bind stdout locally. Quiet success suppresses rendering, not
capture. Consecutive binds read the same stdout, and `-x` exports explicitly.

    :b printf python3 pwd
    :$ -q printf 'two words\n'
    := LOCAL
    :bind --export EXPORTED
    :$ sh -c 'printf "<%s>\n" "$LOCAL" "$${LOCAL-missing}" "$$EXPORTED"'

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
Quiet failure diagnostics remain visible.

    :$ -q sh -c 'printf "partial\n"; printf "error\n" >&2; exit 7'
    := PARTIAL

    :$ printf '%s\n' $PARTIAL

A spawn failure clears previous output. Orphan binds also warn after blank
lines, text, parser errors, or non-shell directives rather than reusing stale
stdout. Failed binds leave existing values unchanged.

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
