# pi-sesh

Pi agent lifecycle integration for the
[`sesh`](https://github.com/amnn/sesh) tmux session switcher.

The extension publishes Pi's lifecycle state to `sesh agent`, including a
session title and a summary when a run settles. It activates only when Pi is
running inside tmux and requires the `sesh` binary to be available on `PATH`.

## Installation

Install the extension directly from the repository:

```sh
pi install git:github.com/amnn/sesh
```

See the repository's
[agent lifecycle documentation](https://github.com/amnn/sesh#agent-lifecycle-state)
for configuration and development instructions.
