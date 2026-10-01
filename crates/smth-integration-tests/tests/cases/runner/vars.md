# Local variables

Both aliases assign in source order. Locals expand as individual arguments but
never enter the child environment. Local values shadow exported values only for
expansion, including sandbox defaults.

    :vars BIN=printf
    :b $BIN cat
    :e SHADOW=environment
    :v FIRST=one SECOND=$FIRST-two LOCAL='two words' SHADOW=local HOME=local-home
    :$ $BIN '<%s>\n' ${SECOND} $LOCAL $SHADOW $HOME

    :$ sh -c 'printf "<%s>\n" "$${LOCAL-missing}" "$$SHADOW"; test "$$HOME" != "$HOME" && echo home-not-exported'

    :e SHADOW=changed DERIVED=$SHADOW
    :$ sh -c 'printf "%s\n" "$SHADOW" "$$SHADOW" "$$DERIVED"'

    :e -u SHADOW
    :$ sh -c 'printf "%s\n" "$SHADOW" "$${SHADOW-missing}"'

Names stay literal, including dollars. Values expand only once; reassignments
can refer to the old value. An empty local still shadows an exported value.

    :v NAME=EXPANDED $NAME=literal VALUE='$$FIRST' FIRST=$FIRST-again
    :$ printf '%s\n' '${$NAME}' $EXPANDED $VALUE $FIRST

    :e EMPTY=environment
    :v EMPTY=
    :$ printf '<%s>\n' $EMPTY

Paths and tmux arguments share expansion. Fenced file contents are not expanded.

    :v SOURCE=tests/cases/runner/text.md COPY='copy file.md' FILE='local file.txt'
    :cp $SOURCE $COPY
    :w $FILE

```text
$FIRST stays literal
```

    :$ cat $FILE

Create a pane that prints `ready` and stays alive with `cat`. Expand its session
and pane targets, then send the local text. Durations and expectations remain
static. The snapshot should show `ready` followed by `variable text`.

    :v SESSION=local-session TARGET=local-session:0.0 TEXT='variable text'
    :t rename-session -t 0 runner
    :t new-session -d -s $SESSION 'printf "ready\n"; cat'
    :t resize-window -t $SESSION:0 -x 40 -y 4
    :p $TARGET
    :settle -d 2s
    :k $TEXT
    :snap -d 2s -e '(?m)variable text$'

The pane binding tracks the new selection; locals do not change child tmux
context. Successful pane selection replaces an explicit environment override.

    :$ sh -c 'test "$TMUX_PANE" = "$$TMUX_PANE" && echo pane-default-matches'

    :e TMUX_PANE=override
    :p runner:0.0
    :$ sh -c 'test "$TMUX_PANE" = "$$TMUX_PANE" && test "$$TMUX_PANE" != override && echo pane-selection-replaces-override'

Malformed assignments preserve earlier values and do not prevent later updates.

    :v INVALID =missing FIRST='${OPEN' LAST=$FIRST

    :$ printf '%s\n' $LAST

    :vars

---
vim: set ft=markdown:
