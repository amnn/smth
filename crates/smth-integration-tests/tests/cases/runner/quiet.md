# Quiet shell directives

    :b sh cat echo

## Successful setup still runs

Both directive spellings and both quiet flags retain exit annotations but hide
successful stdout and stderr. The following read proves setup actually ran.

    :$ -q sh -c "printf 'created\n' > quiet.txt; printf 'hidden stdout\n'; printf 'hidden stderr\n' >&2"
    :shell --quiet cat quiet.txt
    :$ cat quiet.txt

    :shell -q echo hidden short flag
    :$ --quiet echo hidden long flag

Successful ANSI output is also hidden, without output-derived SVGs.

    :shell -q sh -c "printf '\033[31mhidden ansi\033[0m\n'"

## Executable flags are not directive flags

Flags after the executable remain arguments even when they look like directive
flags. Non-quiet commands keep their output.

    :shell echo -q --quiet --help

    :$ sh -c 'printf "%s\n" "$@"' args -q --quiet

    :shell -q sh -c 'printf "%s\n" "$@" > args.txt' args -q --quiet --help
    :$ cat args.txt

## Failures remain visible

Unsuccessful quiet commands show both captured streams. Signal termination also
counts as failure.

    :$ -q sh -c "printf 'failure stdout\n'; printf 'failure stderr\n' >&2; exit 7"

    :shell --quiet sh -c "printf 'killed stdout\n'; printf 'killed stderr\n' >&2; kill -TERM $$$$"

Missing executables still produce spawn diagnostics.

    :shell -q missing-quiet-command

    :$ --quiet missing-quiet-command

## Invalid directives remain visible

An executable is required, unknown directive flags are rejected, and the old
`:sh` spelling is not an alias.

    :shell -q

    :$ --unknown echo nope

    :sh echo unsupported

---
vim: set ft=markdown:
