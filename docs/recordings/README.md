# Documentation recordings

## Prerequisites

The following installation commands target macOS. Keep the installed tools on
`PATH` in the terminal where you run the recordings.

- **Homebrew:** follow the [Homebrew installation instructions][brew],
  including the printed `brew shellenv` setup for your shell.

- **Rust and Cargo:** install via [rustup], then load its environment. Cargo
  uses this repository's `rust-toolchain.toml` to select the required
  toolchain.

  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  . "$HOME/.cargo/env"
  ```

- **Recording tools:** install `jj`, tmux (3.5 or newer), `ttyd`, FFmpeg,
  Python 3, and Fontconfig. These provide the demo repositories, terminal
  sessions, video encoding, orchestration, and caption font discovery.

  ```sh
  brew install jj tmux ttyd ffmpeg python fontconfig
  ```

- **Pillow:** create and activate a Python environment for rendering captions.
  Activate this environment again in any new terminal before recording.

  ```sh
  python3 -m venv /tmp/smth-recording-tools
  . /tmp/smth-recording-tools/bin/activate
  python3 -m pip install Pillow
  ```

- **VHS:** install [VHS] with Homebrew and check that it is 0.12.1 or newer.

  ```sh
  brew install vhs
  vhs --version
  ```

  If an older version is already installed, run `brew upgrade vhs`. VHS may
  download a browser on first use.

- **Pi:** install Node.js (22.19 or newer), then a recent [Pi] release.

  ```sh
  brew install node
  npm install -g --ignore-scripts @earendil-works/pi-coding-agent
  pi --version
  ```

- **Iosevka Term SS15:** install the [Iosevka] SS15 font collection.

  ```sh
  brew install --cask font-iosevka-ss15
  ```

## Recording

From the repository root, run:

```sh
python3 docs/recordings/record.py
```

The script builds the current `smth` and regenerates the documentation GIFs
in `docs/assets/`. Each animation gets its own temporary HOME, demo
repositories, and private tmux server; the agent demo also starts a local
scripted model. No animation depends on another having run first. Everything is
cleaned up on exit. No model credentials or external AI service are needed.
Your sessions and configuration are not used. Run the script rather than
invoking the tapes directly so this isolation and setup are in place.

Edit the `.tape` files to change the demonstrations; their comments describe
each scenario. Shared terminal font, colours, and dimensions live in
`theme.tape`, using Iosevka Term SS15 and VHS's bundled Raycast Light theme.
The switching tape opens an existing checkout and creates a new workspace; the
agent tape follows a real Pi failure/retry before cutting to several Pi agents
with different states. The [session-management
guide](../session-management.md) has targeted animations alongside each command
and each part of Open, including repository context, fresh repositories,
workspaces, and plain tmux sessions.

## Caption cues

Place a caption immediately after the screen condition that introduces it:

```text
Wait+Screen /Work on review/
# caption: Fuzzy-find sessions as you type.
Sleep 3s
```

A caption lasts until the next cue. Use `# caption: off` for an empty caption
area. Leave at least one recorded frame between cues; place cues outside `Hide`
blocks. No caption appears before the first cue.

The runner inserts VHS `Screenshot` commands into temporary tape copies. Its
private FFmpeg wrapper records the actual frame numbers used for those
screenshots, then passes every command through unchanged. This accounts for
`Wait` durations and hidden setup without estimating wall-clock time or
matching identical images. The current tapes use normal playback speed and no
loop offset; those settings must remain unchanged for the frame-index mapping.

After recording, `captions.py` appends an 84-pixel area below the intact
terminal and tmux bar. Captions use the theme's font in bold at 32 pixels, on
the terminal's background colour. Keep labels short enough to fit a single
line; the renderer rejects overflowing text. All captioned GIFs must render
before any checked-in asset is replaced.

With the same prerequisites installed, check Python and tape syntax with:

```sh
python3 -m py_compile docs/recordings/*.py
vhs validate 'docs/recordings/*.tape'
```

Then regenerate and inspect the GIFs, including caption transitions,
staged-deletion counts, and agent summaries.

[brew]: https://docs.brew.sh/Installation
[rustup]: https://rustup.rs/
[VHS]: https://github.com/charmbracelet/vhs
[Iosevka]: https://github.com/be5invis/Iosevka
[Pi]: https://pi.dev
