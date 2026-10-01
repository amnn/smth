# Concurrent command pipelines

Make the producer (`printf`), transforms (`python3`), and passthrough stages
(`cat`) available in the sandbox.

    :b printf python3 cat

Both aliases connect adjacent commands with OS pipes. Only final stdout is
rendered; each stage keeps its own exit annotation. Arguments expand without
word splitting, including the pipe command name and literal dollars.

The final stdout should be `gamma beta two words $SUFFIX environment`:

- `gamma beta`: the root prints `alpha beta`. The first pipe's `$FILTER` expands
  to `python3`, which reads that input and replaces `alpha` with `gamma`, leaving
  `beta` unchanged. The final stage reads this transformed output from stdin.
- `two words`: the local `$SUFFIX` expands to one argument despite containing a
  space. The final stage receives the whole value in `sys.argv[1]`.
- `$SUFFIX`: `$$SUFFIX` escapes the dollar, passing the literal text `$SUFFIX` as
  `sys.argv[2]`. It is not expanded again to `two words`.
- `environment`: `:e` exports `EXPORTED=environment`. The final stage reads it
  through `os.environ`, verifying that pipe stages inherit exported bindings.

The final stage strips the input's trailing newline and prints these four fields
separated by spaces, followed by one newline. Intermediate stdout is not rendered
separately. Both trailing binds should report success before this stdout block;
they capture the final line rather than either intermediate stage's output.

    :v FILTER=python3 SUFFIX='two words'
    :e EXPORTED=environment

    :$ printf 'alpha beta\n'
    :pipe $FILTER -c 'import sys; print(sys.stdin.read().replace("alpha", "gamma"), end="")'
    :| python3 -c 'import os, sys; print(sys.stdin.read().strip(), sys.argv[1], sys.argv[2], os.environ["EXPORTED"])' $SUFFIX $$SUFFIX
    := PIPED
    := -x PIPED_ENV

The local and exported bindings should contain the same complete line, including
the literal `$SUFFIX`. This check should exit 0 and print that line again: the
local arrives as one argument and the export is read directly from the child
environment, without shell interpretation of the captured dollar sign.

    :$ python3 -c 'import os, sys; assert sys.argv[1] == os.environ["PIPED_ENV"]; print(sys.argv[1])' $PIPED

The root writes `value` to stdout, and both `cat` stages pass it through, so
`value` should appear once in the final stdout block, not once per stage. All
three stages should report exit 0.

Each stage also writes its own name to stderr. Expect `root` under `stderr`,
`middle` under `stderr-1`, and `last` under `stderr-2`, in stage order. These
blocks should appear even though every stage succeeds: pipeline stderr is not
restricted to failures.

    :$ sh -c 'printf "root\n" >&2; printf "value\n"'
    :| sh -c 'printf "middle\n" >&2; cat'
    :| sh -c 'printf "last\n" >&2; cat'

The root emits `quiet`, which the Python stage converts to `QUIET`. Both stages
succeed, so `-q` should suppress the pipeline's stdout and its `hidden` stderr;
exit annotations and successful bind annotations should still appear.

Both binds capture the final `QUIET` rather than the root's `quiet`. The last
command should print `QUIET/QUIET`: `$RESULT` reads the local through runner
expansion, while `$$RESULT_ENV` lets the shell read the exported binding. This
also shows that quiet suppresses rendering, not capture, and consecutive binds
reuse the same output.

    :$ -q sh -c 'printf "hidden\n" >&2; printf "quiet\n"'
    :| python3 -c 'import sys; print(sys.stdin.read().upper(), end="")'
    := RESULT
    := -x RESULT_ENV

    :$ sh -c 'printf "%s/%s\n" "$RESULT" "$$RESULT_ENV"'

The root emits `partial` and `root error`, then exits 7. The downstream `cat`
still consumes the stdout and exits 0. Expect those separate exit annotations,
`partial` under stdout, and `root error` under stderr despite `-q`: one failed
stage disables quiet suppression for the entire pipeline. `PARTIAL` should bind
successfully because the pipeline completed and its final stdout is available.
Its bind annotation should precede both output blocks.

    :$ -q sh -c 'printf "root error\n" >&2; printf "partial\n"; exit 7'
    :| cat
    := PARTIAL

Printing `PARTIAL` should produce `partial` again, confirming the failed root's
exit status did not prevent binding the final stage's output.

    :$ printf '%s\n' $PARTIAL

Here the root succeeds, but the middle and last stages exit 8 and 9 after passing
`input` through. Expect exit annotations 0, 8, and 9 rather than just the last
status. Quiet should again be disabled: final stdout is `input`, with
`middle error` under `stderr-1` and `last error` under `stderr-2`. There should be
no root `stderr` block because the root writes none.

    :$ -q printf 'input\n'
    :| sh -c 'cat; printf "middle error\n" >&2; exit 8'
    :| sh -c 'cat; printf "last error\n" >&2; exit 9'

The producer writes 2,097,152 `x` bytes to stdout and 262,144 `e` bytes to stderr.
The middle stage writes another 262,144 `f` bytes to stderr and forwards stdout
unchanged. Both streams exceed typical pipe buffers, so concurrent draining must
let the pipeline finish rather than deadlock.

The final stage counts the received bytes and prints `2097152`. All stages
should exit 0; quiet hides that count and the large stderr streams, but `SIZE`
still captures the count. The separate `printf` should therefore print `2097152`,
showing that the full producer output reached the final stage.

    :$ -q python3 -c 'import sys; sys.stderr.write("e" * 262144); sys.stdout.write("x" * 2097152)'
    :| python3 -c 'import sys; sys.stderr.write("f" * 262144); sys.stdout.write(sys.stdin.read())'
    :| python3 -c 'import sys; print(len(sys.stdin.buffer.read()))'
    := SIZE

    :$ printf '%s\n' $SIZE

The producer writes continuously with the default SIGPIPE behavior restored.
The consumer reads one byte, prints `early-exit`, and exits 0, closing its input
pipe. A subsequent producer write should trigger SIGPIPE, so expect the producer
to report `exit: killed` rather than hang. Its unsuccessful status disables quiet
suppression, making the final `early-exit` stdout visible.

    :$ -q python3 -c 'import os, signal; signal.signal(signal.SIGPIPE, signal.SIG_DFL); exec("while True: os.write(1, b\"x\" * 65536)")'
    :| python3 -c 'import sys; sys.stdin.buffer.read(1); print("early-exit")'

A standalone command is a one-stage pipeline and also receives EOF on stdin.
`cat` should exit 0 without printing anything, and binding its empty stdout should
succeed. The final `printf` should print `<>` for that empty local value.

    :$ cat
    := STANDALONE_EMPTY

    :$ printf '<%s>\n' $STANDALONE_EMPTY

The first `cat` receives EOF rather than inherited stdin, so it emits nothing
and exits 0. The second `cat` also receives EOF and exits 0; no stdout block
should be rendered for this empty pipeline output. Binding it should nevertheless
succeed. The final `printf` should print `<>`, showing `EMPTY` contains an empty
value rather than a missing-output error.

    :$ cat
    :| cat
    := EMPTY

    :$ printf '<%s>\n' $EMPTY

The producer would sleep for 60 seconds, but its consumer does not exist. Expect
both directives to report `failed`, with a command-not-found warning for
`pipeline 1` (the first pipe). The runner must kill and reap the producer instead of
waiting for its sleep to finish.

Unlike a completed nonzero exit, a spawn failure leaves no stdout to bind.
The following `:= KEEP` should therefore be marked `skipped`, without a separate
bind warning or any change to the existing local value. Its annotation should
appear before the `pipeline 1` command-not-found diagnostic.

    :v KEEP=original

    :$ python3 -c 'import time; time.sleep(60)'
    :| no-such-pipeline-command
    := KEEP

Printing `KEEP` should still produce `original`, confirming the skipped bind left
its destination unchanged.

    :$ printf '%s\n' $KEEP

A missing root command should also fail the whole pipeline, this time reporting
`failed to execute command` without a pipeline number in the warning. Neither
stage has captured stdout, so there should be no stdout block.

    :$ no-such-pipeline-root
    :| cat

A spawn failure in the first stage must take precedence over an expansion error
in a later stage. Expect a root command-not-found warning, not an unterminated
variable warning: the runner must stop before expanding `${OPEN`. Both directives
should report `failed`, and the trailing bind should be marked `skipped`.

    :$ no-such-pipeline-root
    :| cat '${OPEN'
    := SKIPPED_BEFORE_EXPANSION

An empty `:|` is a parser error, not a pipeline stage. The preceding shell command
should therefore run independently, create `empty-pipe-marker`, and exit 0.
Expect the parser warning that `:pipe` needs at least one argument after that
command's exit annotation. The parser error breaks adjacency, so the subsequent
bind should fail rather than capture the earlier command's output.

    :$ sh -c 'printf started > empty-pipe-marker'
    :|
    := AFTER_PARSE_ERROR

The file check should exit 0 and print `started`, proving that the malformed pipe
did not prevent the preceding command from creating its marker.

    :$ sh -c 'test -e empty-pipe-marker && echo started'

Unmatched quoting also produces a parser error rather than a pipeline stage.
The shell command should create `quoted-pipe-marker` and print `before-error`;
the first `cat` passes that stdout through. Both valid stages should exit 0 and
render `before-error` before the `invalid shell arguments` warning appears. The
`cat` after the malformed directive is now an orphan and should warn instead of
joining the earlier pipeline.

    :$ sh -c 'printf started > quoted-pipe-marker; printf "before-error\n"'
    :| cat
    :| cat 'unterminated
    :| cat

The file check should again print `started` with exit 0, confirming that quoting
errors do not prevent the preceding command's side effect either.

    :$ sh -c 'test -e quoted-pipe-marker && echo started'

Each stage is expanded and spawned before preparing the next one. The producer
starts, but expanding the following stage's `${OPEN` fails because it lacks a
closing brace. Expect both directives to report `failed`, the bind to
be marked `skipped`, and an `unterminated variable reference` warning. Cleanup
must kill and reap the already-started producer instead of waiting 60 seconds.

    :v EXPANSION=unchanged

    :$ python3 -c 'import time; time.sleep(60)'
    :| cat '${OPEN'
    := EXPANSION

Printing `EXPANSION` should produce `unchanged`, showing that an expansion failure
skips the bind without altering its destination.

    :$ printf '%s\n' $EXPANSION

This pipe has no immediately preceding shell or pipe directive. Expect a warning
that `:pipe` must immediately follow one, not a command-not-found error: the
nonexistent executable should never be attempted.

    :| no-such-orphan-command

The quiet shell command succeeds and binds `bound` to `BEFORE` without rendering
stdout. The intervening bind ends the command chain, so the following `:| cat`
should produce the same orphan-pipe warning rather than consume that output.

    :$ -q printf bound
    := BEFORE
    :| cat

The quiet shell command below should suppress its `interrupted` stdout. The text
line then breaks adjacency, so `:pipe cat` should produce an orphan-pipe warning.
`Text ends the chain.` is copied documentation, not command output.

    :$ -q printf interrupted
Text ends the chain.
    :pipe cat

The tmux directive should succeed and render `constant`, the literal message it
was asked to display. This is tmux output, not stdout from a shell pipeline.

    :t display-message -p constant

A blank line and a non-shell directive cannot supply pipeline input. Expect the
following `:| cat` to warn that it must immediately follow a shell or pipe,
without executing `cat` or forwarding the tmux message.

    :| cat

---
vim: set ft=markdown:
