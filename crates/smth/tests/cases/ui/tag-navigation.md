# Tag navigation

Create two members of tag a and one member of b. Use alphabetical ordering so
navigation and wraparound have predictable destinations.

    :b jj cat
    :t rename-session -t 0 runner
    :t new-session -d -s alpha cat
    :t new-session -d -s beta cat
    :t new-session -d -s gamma cat
    :$ -q smth --no-base --tag a alpha
    :$ -q smth --no-base --tag b beta
    :$ -q smth --no-base --tag a gamma
    :$ -q smth --no-base --flag alpha
    :$ -q smth --no-base --flag beta
    :t new-session -d -s ui 'smth --order +name'
    :t resize-window -t ui:0 -x 80 -y 12
    :p ui:0.0
    :settle -d 2s

Hide the preview. C-j enters next-key mode without changing the selection;
then a skips beta and selects gamma. The tag key does not edit the query.

    :k C-p C-j
    :snap

    :k a
    :snap

Next wraps to alpha, and previous wraps back to gamma. A tag with no members
leaves gamma selected. An invalid key cancels without editing the query.

    :k C-j a
    :snap

    :k C-k a
    :snap

    :k C-j z
    :snap

    :k C-j A
    :snap

Attention is independent of tags: next from gamma wraps to flagged alpha,
next selects flagged beta, and previous returns to alpha.

    :k C-j -
    :snap

    :k C-j -
    :snap

    :k C-k -
    :snap

Filtering to alpha makes it the sole member. Both directions stay on alpha.

    :k alpha
    :settle
    :k C-j a
    :snap

    :k C-k a
    :snap

Attention navigation also respects the filter and stays on alpha.

    :k C-j -
    :snap

    :k C-k -
    :snap

---
vim: set ft=markdown:
