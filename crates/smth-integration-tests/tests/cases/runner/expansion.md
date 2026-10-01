# Parse once, expand operands at use

Executable names and arguments expand separately. Expansion does not turn an
executable name into a directive flag: `-q` must fail to execute, rather than
making the following printf command quiet. Reassignment affects later commands
without reparsing their directive syntax.

    :b printf
    :e COMMAND=printf VALUE='first value'
    :$ $COMMAND '<%s>\n' $VALUE

    :e VALUE=second
    :$ $COMMAND '<%s>\n' $VALUE

    :e OPTION=-q
    :$ $OPTION printf should-not-run

Typed directive options are parsed before any assignments run. A variable
reference is not a valid duration, even if its value would be one.

    :e DURATION=2s
    :settle -d $DURATION

Paths and binary requirements expand when used. Copied files keep their original
contents, and fenced write contents do not undergo expansion.

    :e BINARY=cat SOURCE=tests/cases/runner/text.md COPY='copied text.md' FILE='written text.txt'
    :b $BINARY
    :cp $SOURCE $COPY
    :w $FILE

```text
$VALUE stays literal
```

    :$ $BINARY $FILE

    :$ sh -c 'test -f "$$1" && echo copied-file-exists' sh $COPY

Tmux commands and pane targets expand as operands. The pane prints `ready` and
stays alive with cat. Expanded key text that spells `enter` must be typed, not
interpreted as a key name. The snapshot should contain `ready` and `enter`;
its regex expectation is static, including the dollar anchor.

    :e SESSION=expanded-pane TARGET=expanded-pane:0.0 KEY=enter TMUX_COMMAND=display-message
    :t new-session -d -s $SESSION 'printf "ready\n"; cat'
    :t resize-window -t $SESSION:0 -x 40 -y 4
    :p $TARGET
    :settle -d 2s
    :k $KEY
    :snap -d 2s -e '(?m)^enter$'

    :t $TMUX_COMMAND -p $VALUE

Malformed references report errors at their use sites, without executing the
operation. In particular, the invalid shell argument must not create a file,
and invalid key text must not send the preceding valid text either.

    :$ sh -c 'printf executed > marker' '${OPEN'

    :$ sh -c 'test ! -e marker && echo not-executed'

    :b '${OPEN'

    :cp $SOURCE '${OPEN'

    :w '${OPEN'

```text
not written
```

    :p '${OPEN'

    :t display-message -p '${OPEN'

    :k should-not-be-typed '${OPEN'

    :snap -d 2s -e '(?m)^enter$'

---
vim: set ft=markdown:
