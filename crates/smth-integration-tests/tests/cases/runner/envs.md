# Environment bindings and argument expansion

Assignments evaluate left to right; values keep whitespace and equal signs.

    :b printf

    :envs FIRST=one SECOND=${FIRST}-two WORDS='two words' EQUAL=a=b EMPTY=
    :e COMMAND=printf
    :$ $COMMAND '<%s>\n' $FIRST ${SECOND} $WORDS $EQUAL $EMPTY $MISSING

    :$ sh -c 'printf "<%s>\n" "$$SECOND" "$$WORDS"'

Escaped dollars belong to the child shell, including positional parameters.
Expansion is single-pass, without command substitution or word splitting.

    :envs LITERAL='$$FIRST' CODE='$(echo not-executed)'
    :$ printf '<%s>\n' $LITERAL $CODE $$FIRST end$

    :$ sh -c 'printf "%s\n" "$$1"' sh 'positional value'

Sandbox defaults are visible without inheriting arbitrary host environment.
Overriding HOME changes the child environment, not the sandbox working directory.

    :$ sh -c 'test "$HOME" = "$$HOME" && test "$PATH" = "$$PATH" && test "$ENV" = "$$ENV" && test "$SHELL" = "$$SHELL" && test "$LC_CTYPE" = "$$LC_CTYPE" && test "$TMUX" = "$$TMUX" && test "$TMUX_PANE" = "$$TMUX_PANE" && echo defaults-match'

    :envs HOME=custom TMUX=custom-socket TMUX_PANE=custom-pane
    :$ sh -c 'printf "%s\n" "$$HOME" "$$TMUX" "$$TMUX_PANE"'

    :$ printf '%s\n' $HOME $TMUX $TMUX_PANE

    :e -u HOME TMUX TMUX_PANE SECOND
    :$ sh -c 'printf "<%s>\n" "$${HOME-unset}" "$${TMUX-unset}" "$${TMUX_PANE-unset}" "$${SECOND-unset}"'

A failed pane selection preserves an override. A successful selection replaces
it, or restores an unset value. Other removed defaults stay absent, and ordinary
tmux commands do not restore the pane binding.

    :e TMUX_PANE=override
    :p no-such-pane

    :$ printf '%s\n' $TMUX_PANE

    :t split-window -d
    :p 0.1
    :$ sh -c 'test "$TMUX_PANE" = "$$TMUX_PANE" && printf "%s\n" "$$TMUX_PANE" "$${TMUX-unset}" "$${HOME-unset}"'

    :e -u TMUX_PANE
    :t display-message -p constant

    :$ sh -c 'printf "%s\n" "$${TMUX_PANE-unset}"'

    :p 0.0
    :$ sh -c 'test "$TMUX_PANE" = "$$TMUX_PANE" && printf "%s\n" "$$TMUX_PANE"'

Exports affect subsequent host commands, not the running tmux server.

    :envs ONLY_CHILD=present
    :t show-environment -g ONLY_CHILD

    :$ sh -c 'printf "%s\n" "$$ONLY_CHILD"'

Malformed assignments warn independently; valid later assignments still apply.

    :envs INVALID =missing GOOD=kept BROKEN='${OPEN' AFTER=$GOOD

    :$ printf '%s\n' $AFTER

    :envs

    :$ printf '%s\n' '${OPEN'

---
vim: set ft=markdown:
