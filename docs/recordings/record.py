#!/usr/bin/env python3
# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

"""Build smth and record the VHS tapes using disposable repositories and processes."""

import json
import os
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import textwrap
import time
from pathlib import Path

import captions
from model import BACKGROUND_PROMPT


DEMO_SHELL = "/bin/sh"
WORKSPACES = ("review", "docs", "agent-hooks")


def write(path, content):
    """Write an indented fixture without carrying Python indentation into the file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(textwrap.dedent(content).lstrip())


def bin(path, command, forward_args=False):
    """Write a quoted exec wrapper, optionally forwarding its caller's arguments."""
    suffix = ' "$@"' if forward_args else ""
    write(path, f"#!{DEMO_SHELL}\nexec {shlex.join(map(str, command))}{suffix}\n")
    path.chmod(0o755)


class Recording:
    """Own the isolated environment, private tmux server, and local model process."""

    def __init__(self, source, root):
        """Prepare private paths and environment without starting any processes."""
        self.source = source
        self.root = root
        self.home = root / "home"
        self.socket = root / "tmux.sock"
        self.agent_dir = self.home / ".pi/agent"
        self.model = None
        self.tmux = shutil.which("tmux")
        self.pi = shutil.which("pi")
        self.ffmpeg = shutil.which("ffmpeg")
        self.ttyd = shutil.which("ttyd")

        if not self.tmux or not self.pi or not self.ffmpeg or not self.ttyd:
            raise RuntimeError("tmux, pi, ffmpeg, and ttyd must be available on PATH")

        self.env = os.environ.copy()
        for key in (
            "TMUX",
            "TMUX_PANE",
            "JJR",
            "JJ_WORKSPACE",
            "PI_SESSION_ID",
            "PI_SESSION_FILE",
            "PI_MODEL",
            "PI_PROVIDER",
            "PI_REASONING_LEVEL",
            "PI_CODING_AGENT_SESSION_DIR",
            "PI_PACKAGE_DIR",
        ):
            self.env.pop(key, None)

        self.env.update(
            {
                "HOME": str(self.home),
                "XDG_CONFIG_HOME": str(self.home / ".config"),
                "JJ_CONFIG": str(self.home / "jj.toml"),
                "SHELL": DEMO_SHELL,
                "PS1": "$ ",
                "PI_CODING_AGENT_DIR": str(self.agent_dir),
                "PI_OFFLINE": "1",
                "PI_TELEMETRY": "0",
                "PATH": str(root / "bin") + os.pathsep + self.env["PATH"],
            }
        )

    def run(self, *args, cwd=None, check=True):
        """Run a command in the demo HOME unless an explicit directory is required."""
        return subprocess.run(args, cwd=cwd or self.home, env=self.env, check=check)

    def configure(self):
        """Install private configuration and pin every child tmux call to our socket."""

        write(
            self.agent_dir / "settings.json",
            json.dumps(
                {
                    "theme": "light",
                    "retry": {"enabled": False},
                    "compaction": {"enabled": False},
                }
            ),
        )

        write(
            self.home / "jj.toml",
            """
            [user]
            name = "Demo"
            email = "demo@example.com"
            [revset-aliases]
            "trunk()" = "coalesce(bookmarks(exact:main), root())"
            """,
        )

        # Discover closed checkouts too, without any user setup or notification hooks.
        write(
            self.home / ".config/smth/smth.toml",
            '[repo]\nroot = "~/Code"\nglobs = ["~/Code/*"]\n',
        )

        bin(self.root / "bin/tmux", [self.tmux, "-S", self.socket], forward_args=True)

        # Treat macOS Option as Meta so VHS Alt+r reaches the picker as M-r.
        bin(
            self.root / "bin/ttyd",
            [self.ttyd, "-t", "macOptionIsMeta=true"],
            forward_args=True,
        )

        bin(
            self.root / "bin/ffmpeg",
            [sys.executable, self.source / "docs/recordings/captions.py", self.ffmpeg],
            forward_args=True,
        )

        bin(
            self.root / "bin/demo-busy",
            [sys.executable, self.source / "docs/recordings/record.py", "--busy"],
        )

        (self.root / "bin/smth").symlink_to(self.source / "target/debug/smth")

    def start_tmux(self):
        """Start the private server with a shell, Pi-compatible keys, and picker binding."""

        self.run(
            *("tmux", "-f", "/dev/null", "new-session"),
            "-d",
            *("-s", "bootstrap"),
            *("-x", "100", "-y", "30"),
        )

        options = {
            "default-shell": DEMO_SHELL,
            "default-command": f"exec {shlex.quote(DEMO_SHELL)}",
            "extended-keys": "on",
            "extended-keys-format": "csi-u",
            "status-style": "bg=colour236,fg=white",
            "status-left": "#S  ",
            "status-left-length": "30",
            "status-right": "C-b s: sessions ",
            "automatic-rename": "off",
        }

        for name, value in options.items():
            self.run("tmux", "set-option", "-g", name, value)

        self.run(
            *("tmux", "bind-key", "s"),
            "display-popup",
            "-E",
            *("-w", "96%", "-h", "90%"),
            *("-T", "smth"),
            *("-d", "#{pane_current_path}"),
            *("smth", "--"),
        )

    def create_workspaces(self):
        """Create demo checkouts through smth and give each session visibly different work."""
        repo = self.home / "Code/smth"
        self.run("smth", "--json")

        self.run(
            "smth",
            "--no-base",
            *("--repo-root", repo.parent),
            "--create-repo",
            *("--create", "smth"),
        )

        write(
            repo / "docs/sessions.md",
            """
            # Session workflow

            Switch contexts without losing your place.

              C-b s   Open the session picker
              type    Filter repositories and workspaces
              Enter   Switch to the selected session

            Agent attention
              Running agents show a play indicator.
              Settled agents keep an attention marker until their next prompt.
              Switching to a session opens the window that needs attention.
        """,
        )

        write(
            repo / "sessions.py",
            """
            # Copyright (c) Ashok Menon
            # SPDX-License-Identifier: Apache-2.0


            def session_label(repo, workspace):
                return repo + "/" + workspace
        """,
        )

        self.run("jj", "-R", repo, "describe", "-m", "Add session picker")
        self.run("jj", "-R", repo, "bookmark", "create", "main")

        for workspace in WORKSPACES:
            self.run("smth", "--base", repo, "--create", workspace)
            self.run(
                "jj",
                *("-R", repo.with_name(f"smth.{workspace}")),
                "describe",
                *("-m", f"Work on {workspace}"),
            )

        self.run("smth", "--no-base", "--close", "bootstrap")
        self.run("smth", "--json")

        write(
            repo.with_name("smth.review") / "sessions.py",
            """
            # Copyright (c) Ashok Menon
            # SPDX-License-Identifier: Apache-2.0


            def session_label(repo, workspace):
                if workspace == "default":
                    return repo
                return f"{repo}/{workspace}"
        """,
        )

        commands = {
            "smth": "jj log --limit 5",
            "smth/review": "jj diff",
            "smth/agent-hooks": "jj status",
        }

        for session, command in commands.items():
            self.run("tmux", "rename-window", "-t", f"{session}:0", "shell")
            self.run(
                *("tmux", "send-keys"),
                *("-t", f"{session}:0.0"),
                f"PS1='{session} $ '; clear",
                "Enter",
            )
            self.run("tmux", "send-keys", "-t", f"{session}:0.0", command, "Enter")

        # Docs remains a discovered checkout, but Enter must open its first live session.
        self.run("smth", "--base", repo, "--close", "docs")

    def start_pi(self):
        """Launch only pi-smth against the loopback model once its configuration is ready."""
        # These sessions host agents in the multi-agent scene.
        for workspace in ("docs", "experiment"):
            self.run("smth", "--base", self.home / "Code/smth", "--create", workspace)

        log_path = self.root / "model.log"
        with log_path.open("w") as log:
            self.model = subprocess.Popen(
                [
                    sys.executable,
                    self.source / "docs/recordings/model.py",
                    self.agent_dir,
                ],
                cwd=self.home,
                env=self.env,
                stdout=log,
                stderr=subprocess.STDOUT,
            )

        deadline = time.monotonic() + 5
        while not (self.agent_dir / "models.json").exists():
            if self.model.poll() is not None or time.monotonic() >= deadline:
                raise RuntimeError(f"Demo model did not start:\n{log_path.read_text()}")
            time.sleep(0.1)

        bin(
            self.root / "bin/demo-pi",
            [
                self.pi,
                "--offline",
                "--no-session",
                "--no-extensions",
                "--no-skills",
                "--no-prompt-templates",
                "--no-context-files",
                "--no-themes",
                "--no-approve",
                *("--extension", self.source / "extensions/pi-smth/index.ts"),
                *("--provider", "demo"),
                *("--model", "scripted"),
                *("--thinking", "off"),
                *("--tools", "read,bash"),
                *("--name", "Session review"),
            ],
        )

        self.run(
            *("tmux", "new-window"),
            "-d",
            *("-t", "smth/agent-hooks:"),
            *("-n", "pi"),
            *("-c", self.home / "Code/smth.agent-hooks"),
        )

        self.run(
            *("tmux", "send-keys"),
            *("-t", "smth/agent-hooks:pi"),
            "PS1='agent-hooks $ '; clear; demo-pi",
            "Enter",
        )

    def record(self, name):
        """Render one tape and return its checked output, without publishing any assets."""
        directory = self.root / name
        tape, cues = captions.prepare(
            self.source / f"docs/recordings/{name}.tape", directory
        )
        raw = directory / "terminal.gif"
        self.run("vhs", "-o", raw, tape, cwd=self.source)
        if not raw.is_file() or raw.stat().st_size == 0:
            raise RuntimeError(f"No recording produced: {raw}")
        output = self.root / f"{name}.gif"
        captions.render(
            raw,
            cues,
            output,
            self.source / "docs/recordings/theme.tape",
            self.ffmpeg,
        )
        if not output.is_file() or output.stat().st_size == 0:
            raise RuntimeError(f"No captioned recording produced: {output}")
        return output

    def close(self):
        """Stop the private tmux server and model before their temporary files are removed."""
        try:
            self.run(
                self.tmux, "-S", self.socket, "kill-server", cwd=self.root, check=False
            )
        finally:
            if self.model is not None:
                self.model.terminate()
                self.model.wait()


def populate_agents():
    """Stage the busy scene using real Pi processes and wait for their published states."""
    agents = [
        ("agent-hooks", "worker-1", BACKGROUND_PROMPT, "running"),
        ("agent-hooks", "worker-2", BACKGROUND_PROMPT, "running"),
        ("review", "pi", "Review the session workflow.", "failed"),
        ("docs", "pi", None, "idle"),
        ("experiment", "pi", BACKGROUND_PROMPT, "running"),
    ]

    def tmux(*args):
        return subprocess.check_output(["tmux", *args], text=True).strip()

    def wait_state(pane, state):
        """Wait for the published pane state; raise RuntimeError after 30 seconds."""
        deadline = time.monotonic() + 30
        while (
            tmux("display-message", "-p", "-t", pane, "#{@smth.agent.state}") != state
        ):
            if time.monotonic() >= deadline:
                raise RuntimeError(f"Pi did not reach {state}: {pane}")
            time.sleep(0.1)

    for workspace, window, prompt, state in agents:
        pane = tmux(
            "new-window",
            "-d",
            "-P",
            "-F",
            "#{pane_id}",
            "-t",
            f"smth/{workspace}:",
            "-n",
            window,
            "-c",
            str(Path.home() / f"Code/smth.{workspace}"),
            "demo-pi",
        )
        wait_state(pane, "idle")
        if prompt:
            tmux("send-keys", "-t", pane, "-l", prompt)
            tmux("send-keys", "-t", pane, "Enter")
        wait_state(pane, state)
    print("Demo agents ready", flush=True)


def stop(signum, _frame):
    """Turn termination signals into an exit that still runs resource cleanup."""
    raise SystemExit(128 + signum)


def main():
    """Build the current checkout, record in isolation, and clean up on failure or exit."""
    source = Path(__file__).resolve().parents[2]
    subprocess.run(["cargo", "build", "--locked", "-p", "smth"], cwd=source, check=True)
    for signum in (signal.SIGHUP, signal.SIGTERM):
        signal.signal(signum, stop)
    with tempfile.TemporaryDirectory(prefix="smth-recording.", dir="/tmp") as temporary:
        outputs = []
        for name in (
            "session-switching",
            "agent-attention",
            "session-context",
            "session-new-repo",
            "session-new-workspace",
            "session-new-tmux",
            "session-create",
            "session-close",
            "session-delete",
        ):
            recording = Recording(source, Path(temporary).resolve() / name)
            try:
                recording.configure()
                recording.start_tmux()
                recording.create_workspaces()

                if name == "agent-attention":
                    recording.start_pi()
                outputs.append(recording.record(name))
            finally:
                recording.close()

        for output in outputs:
            shutil.copyfile(output, source / "docs/assets" / output.name)


if __name__ == "__main__":
    if sys.argv[1:] == ["--busy"]:
        populate_agents()
    else:
        main()
