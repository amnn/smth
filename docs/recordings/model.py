# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

"""Serve scripted OpenAI-compatible responses for the isolated Pi recording.

Write models.json to the supplied Pi agent directory once the loopback listener
is ready. The first review reads a real fixture, then fails; the retry runs jj
and succeeds. Delays leave time to inspect smth while Pi is running. Background
prompts stay in flight so several real agents can populate the summary scene.
"""

import json
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


MODEL_ID = "scripted"
READING_PAUSE = 8
BACKGROUND_PROMPT = "Keep working on this workspace."


class Model(BaseHTTPRequestHandler):
    """Handle concurrent chat completion requests from the isolated demo Pi agents."""

    def do_POST(self):
        """Stream a tool call or outcome based on the visible conversation."""
        request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        messages = request["messages"]
        prompts = [message for message in messages if message["role"] == "user"]
        after_tool = messages[-1]["role"] == "tool"
        background = any(
            BACKGROUND_PROMPT in json.dumps(prompt["content"]) for prompt in prompts
        )

        if len(prompts) == 1 and after_tool and not background:
            time.sleep(READING_PAUSE)
            error = {"message": "Demo connection interrupted. Please retry."}
            body = json.dumps({"error": error}).encode()
            self.send_response(400)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()
        self.chunk({"role": "assistant"})
        if after_tool and background:
            # These real Pi requests stay in flight throughout the summary scene.
            # SSE comments keep the connection alive without filling the transcript.
            try:
                while True:
                    self.wfile.write(b": working\n\n")
                    self.wfile.flush()
                    time.sleep(2)
            except (BrokenPipeError, ConnectionResetError):
                return
        elif after_tool:
            time.sleep(READING_PAUSE)
            text = "The session workflow is documented and the checkout is clean. Ready for review."
            for word in text.split():
                self.chunk({"content": word + " "})
                time.sleep(0.08)
            self.chunk({}, "stop")
        else:
            first = len(prompts) == 1
            text = (
                "I'll read the session guide."
                if first
                else "I'll check the workspace before summarizing."
            )
            arguments = (
                {"path": "docs/sessions.md"} if first else {"command": "jj status"}
            )
            self.chunk({"content": text})
            time.sleep(1)
            self.chunk(
                {
                    "tool_calls": [
                        {
                            "index": 0,
                            "id": f"demo-{len(prompts)}",
                            "type": "function",
                            "function": {
                                "name": "read" if first else "bash",
                                "arguments": json.dumps(arguments),
                            },
                        }
                    ]
                },
                "tool_calls",
            )
        self.wfile.write(b"data: [DONE]\n\n")
        self.wfile.flush()
        self.close_connection = True

    def chunk(self, delta, finish_reason=None):
        """Emit one SSE chat-completion chunk and make it visible immediately."""
        data = {
            "id": "demo",
            "object": "chat.completion.chunk",
            "created": 0,
            "model": MODEL_ID,
            "choices": [{"index": 0, "delta": delta, "finish_reason": finish_reason}],
        }
        self.wfile.write(f"data: {json.dumps(data)}\n\n".encode())
        self.wfile.flush()


def main():
    """Bind an ephemeral local port and publish Pi's model configuration."""
    with ThreadingHTTPServer(("127.0.0.1", 0), Model) as server:
        provider = {
            "baseUrl": f"http://{server.server_address[0]}:{server.server_port}/v1",
            "api": "openai-completions",
            "apiKey": "demo-only",
            "models": [
                {
                    "id": MODEL_ID,
                    "name": "Scripted demo model",
                    "contextWindow": 32768,
                    "maxTokens": 1024,
                }
            ],
        }
        agent_dir = Path(sys.argv[1])
        pending = agent_dir / "models.json.tmp"
        pending.write_text(json.dumps({"providers": {"demo": provider}}))
        pending.replace(agent_dir / "models.json")
        server.serve_forever()


if __name__ == "__main__":
    main()
