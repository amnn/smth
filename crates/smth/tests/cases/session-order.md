# Configured session ordering

Create live sessions whose discovery order differs from both descending names
and attachment recency. Without a query, listing uses the comparator stack or
plain discovery order when the stack is empty.

    :b jj tmux cat
    :t rename-session -t 0 runner
    :t new-session -d -s alpha cat
    :t new-session -d -s beta cat
    :$ smth --filter --order '-name'

    :$ smth --filter --order ''

With a query, fuzzy score takes precedence: alpha's prefix match beats beta's
suffix match even with descending names. Disabling comparators still uses fuzzy
ranking. JSON shares the same result order.

    :$ smth --filter --order '-name' --query a

    :$ smth --filter --order '' --query a

    :$ smth --json --order '-name' --query a

Configuration supplies the initial stack. The CLI overrides it, including an
explicit empty string. No comparator uses a legacy alias or a disabled sigil.

    :w order.toml

```toml
[ui]
order = "-name"
```

    :$ smth --config order.toml --filter

    :$ smth --config order.toml --filter --order '+name'

    :$ smth --config order.toml --filter --order ''

    :$ smth --filter --order '-last'

    :$ smth --filter --order '!name'

    :$ smth --filter --order '+name -name'

Two equal-length names with the same matching prefix have equal fuzzy scores.
The explicit name comparator breaks that tie in either direction.

    :t new-session -d -s tiez cat
    :t new-session -d -s tiea cat
    :$ smth --filter --order '-name' --query tie

    :$ smth --filter --order '+name' --query tie

---
vim: set ft=markdown:
