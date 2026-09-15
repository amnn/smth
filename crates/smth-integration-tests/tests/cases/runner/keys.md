# Runner keys directive behavior

## Sends literal text and named keys to active pane

The runner should send text keys literally, then named keys like `enter`, to the current active
pane.

    :bins cat python3 scripts/tmcap

    :t new-window -d -n keys 'tmcap keys'
    :p 0:keys.0

    :t wait-for ready-keys
    :k "hello" space "world" enter C-d

    :t wait-for done-keys
    :$ cat keys.txt

## Pane switching sends keys to selected pane

Switching active pane should route keys into the newly selected pane.

    :t new-window -d -n new 'tmcap new'
    :p 0:new.0

    :t wait-for ready-new
    :k "pane-b" enter C-d

    :t wait-for done-new
    :$ cat new.txt

## Sends hello world as one literal text key

The runner should send a whole phrase as one literal text payload when quoted as a single token.

    :t new-window -d -n text 'tmcap text'
    :p 0:text.0

    :t wait-for ready-text
    :k "hello world" enter C-d

    :t wait-for done-text
    :$ cat text.txt

## Sends punctuation and capitals as literal text

Literal text keys should preserve case and punctuation.

    :t new-window -d -n pct 'tmcap pct'
    :p 0:pct.0

    :t wait-for ready-pct
    :k "Hello, world!" enter C-d

    :t wait-for done-pct
    :$ cat pct.txt

## Keeps tmux key names and options literal

Only the runner's lowercase key names are special. Other text must not be reinterpreted as
named keys or command options by tmux.

    :t new-window -d -n names 'tmcap names'
    :p 0:names.0

    :t wait-for ready-names
    :k Enter space Up space BTab space Escape space -l space -- enter C-d

    :t wait-for done-names
    :$ cat names.txt

## Sends complex modifier combinations

Complex modifier combinations should be forwarded to tmux as key codes, including explicit
`btab`.

    :t new-window -d -n mod 'tmcap mod'
    :p 0:mod.0

    :t wait-for ready-mod
    :k C-a M-a C-M-a btab S-up enter C-d

    :t wait-for done-mod
    :$ cat mod.txt

## Modified backtab depends on tmux's legacy encoding

Without extended key negotiation, tmux 3.4 drops `C-btab`, while tmux 3.7c sends an unmodified
backtab. Accept these two observed legacy encodings rather than requiring either tmux version.

    :t new-window -d -n backtab 'tmcap backtab'
    :p 0:backtab.0

    :t wait-for ready-backtab
    :k C-btab enter C-d

    :t wait-for done-backtab
    :$ python3 -c 'from pathlib import Path; value = Path("backtab.txt").read_text(); assert value in ("\n", r"\x1b[Z" + "\n"), repr(value)'

## Failed pane selection does not retarget active pane

If `:pane` fails, key input should still go to the last successfully selected pane.

    :t new-window -d -n c 'tmcap c'
    :p 0:c.0

    :pane does-not-exist

    :t wait-for ready-c
    :k "still-c" enter C-d

    :t wait-for done-c
    :$ cat c.txt

---
vim: set ft=markdown:
