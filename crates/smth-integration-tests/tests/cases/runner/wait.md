# Runner waits for asynchronous tmux commands

Tmux's control-mode `%end` acknowledges a command before asynchronous work has
necessarily finished. A following host directive must wait for the command
queue to resume, not just for that acknowledgement.

    :bins tmux sleep cat

A foreground `run-shell` must finish writing its file before the host reads it.

    :t run-shell 'sleep 0.2; printf "shell finished\n" > shell.txt'
    :$ cat shell.txt

A background job can signal `wait-for`. The following host read must wait for
the signal, even though tmux acknowledges `wait-for` immediately.

    :t run-shell -b 'sleep 0.2; printf "signal received\n" > signal.txt; tmux wait-for -S finished'
    :t wait-for finished
    :$ cat signal.txt

---
vim: set ft=markdown:
