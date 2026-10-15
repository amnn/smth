# Custom tag alphabets

A display alphabet changes glyphs, not tag identity. Create a live session
carrying a, c, and z, and configure Fraktur for its inline and footer display.

    :b jj cat
    :t rename-session -t 0 runner
    :t new-session -d -s target-with-limited-name-padding "cat"
    :$ -q smth --no-base --tag a target-with-limited-name-padding
    :$ -q smth --no-base --tag c target-with-limited-name-padding
    :$ -q smth --no-base --tag z target-with-limited-name-padding
    :w fraktur.toml

```toml
[ui]
tags = "𝖆𝖇𝖈𝖉𝖊𝖋𝖌𝖍𝖎𝖏𝖐𝖑𝖒𝖓𝖔𝖕𝖖𝖗𝖘𝖙𝖚𝖛𝖜𝖝𝖞𝖟"
```

    :t new-session -d -s ui "smth --no-base --config fraktur.toml; cat"
    :t resize-window -t ui:0 -x 100 -y 10
    :p ui:0.0
    :settle -d 2s
    :k C-p target

The row and footer show the configured glyphs. Batch input is still ASCII:
uppercase B stages adding Fraktur b, and lowercase c stages removing saved c.
The right-hand overlay shows both edits, while the row keeps saved a, c, z.
C-t applies both edits. JSON still emits a, b, z.

    :snap

    :k C-t B c
    :snap --color -e 'tags: 𝖆𝖇𝖈𝖟 \[C-t\] accept \[C-g\] cancel'

    :k C-t
    :snap -e 'tags: 𝖆𝖇𝖟'

    :$ smth --no-base --config fraktur.toml --json --query target

Fullwidth Latin glyphs occupy two cells each. Supply only fullwidth a to infer
the next 25 consecutive code points. Add d so the four glyphs cannot all fit
beside the name: the inline span contracts, while the footer shows all four.

    :$ -q smth --no-base --tag d target-with-limited-name-padding
    :w wide.toml

```toml
[ui]
tags = "ａ"
```

    :t new-session -d -s wide-ui "smth --no-base --config wide.toml; cat"
    :t resize-window -t wide-ui:0 -x 100 -y 10
    :p wide-ui:0.0
    :settle -d 2s
    :k C-p target
    :snap

Invalid alphabets fail configuration loading with exit 1 before the picker
opens: the first has too few characters, and the second ends in whitespace.

    :w short.toml

```toml
[ui]
tags = "abc"
```

    :$ smth --config short.toml --json

    :w invisible.toml

```toml
[ui]
tags = "abcdefghijklmnopqrstuvwxy "
```

    :$ smth --config invisible.toml --json

A single starting character whose sequence extends past Unicode's maximum
code point is also rejected with exit 1 rather than wrapping or skipping values.

    :w overflow.toml

```toml
[ui]
tags = "\U0010FFFF"
```

    :$ smth --config overflow.toml --json

---
vim: set ft=markdown:
