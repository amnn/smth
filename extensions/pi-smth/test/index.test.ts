// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

import assert from "node:assert/strict";
import test from "node:test";

import type {
  AgentEndEvent,
  ExtensionAPI,
  ExtensionContext,
} from "@earendil-works/pi-coding-agent";

import extension, { outcome } from "../index.ts";

interface TestEvent {
  messages?: AgentEndEvent["messages"];
}

type EventHandler = (
  event: TestEvent,
  ctx: ExtensionContext,
) => Promise<void> | void;

interface HarnessOptions {
  code?: number;
  sessionName?: string;
  stderr?: string;
  stdout?: string;
  throws?: Error;
}

function assistant(
  stopReason: "stop" | "length" | "toolUse" | "error" | "aborted",
  text?: string,
): AgentEndEvent["messages"][number] {
  return {
    role: "assistant",
    stopReason,
    content: text === undefined ? [] : [{ type: "text", text }],
  } as AgentEndEvent["messages"][number];
}

function createHarness(options: HarnessOptions = {}) {
  const calls: string[][] = [];
  const handlers = new Map<string, EventHandler>();
  const notifications: string[] = [];
  let sessionName = options.sessionName;

  const pi = {
    exec: async (command: string, args: string[]) => {
      assert.equal(command, "smth");
      assert.equal(args[0], "agent");
      calls.push(args);
      if (options.throws) {
        throw options.throws;
      }

      return {
        code: options.code ?? 0,
        killed: false,
        stderr: options.stderr ?? "",
        stdout: options.stdout ?? "",
      };
    },

    getSessionName: () => sessionName,

    on: (event: string, handler: EventHandler) => {
      handlers.set(event, handler);
    },
  } as unknown as ExtensionAPI;

  const ctx = {
    hasUI: true,
    ui: { notify: (message: string) => notifications.push(message) },
  } as unknown as ExtensionContext;

  extension(pi);

  return {
    calls,
    notifications,

    hasHandler: (event: string) => handlers.has(event),

    setSessionName: (name: string | undefined) => {
      sessionName = name;
    },

    async emit(
      event: string,
      data: TestEvent | AgentEndEvent["messages"] = {},
    ) {
      const handler = handlers.get(event);
      assert.ok(handler, `expected a handler for ${event}`);
      await handler(Array.isArray(data) ? { messages: data } : data, ctx);
    },
  };
}

async function insideTmux(run: () => Promise<void>): Promise<void> {
  const pane = process.env.TMUX_PANE;
  process.env.TMUX_PANE = "%test";
  try {
    await run();
  } finally {
    if (pane === undefined) {
      delete process.env.TMUX_PANE;
    } else {
      process.env.TMUX_PANE = pane;
    }
  }
}

test("publishes the Pi lifecycle and waits for the final settled outcome", async () => {
  await insideTmux(async () => {
    const harness = createHarness();

    await harness.emit("session_start");
    await harness.emit("agent_start");
    await harness.emit("agent_end", [assistant("error", "retry")]);
    await harness.emit("agent_start");
    await harness.emit("agent_end", [
      assistant("stop", "  Finished\n successfully  "),
    ]);

    assert.deepEqual(harness.calls, [
      ["agent", "idle", "--title", "pi"],
      ["agent", "running", "--title", "pi"],
      ["agent", "running", "--title", "pi"],
    ]);

    await harness.emit("agent_settled");
    await harness.emit("session_shutdown");

    assert.deepEqual(harness.calls, [
      ["agent", "idle", "--title", "pi"],
      ["agent", "running", "--title", "pi"],
      ["agent", "running", "--title", "pi"],
      [
        "agent",
        "succeeded",
        "--title",
        "pi",
        "--summary",
        "Finished\n successfully",
      ],
      ["agent", "exit", "--title", "pi"],
    ]);
  });
});

test("forwards smth output to the Pi terminal", async (t) => {
  await insideTmux(async () => {
    const writes: string[] = [];
    t.mock.method(process.stdout, "write", ((chunk: string | Uint8Array) => {
      writes.push(chunk.toString());
      return true;
    }) as typeof process.stdout.write);

    const harness = createHarness({ stdout: "\x07" });
    await harness.emit("session_start");

    assert.deepEqual(writes, ["\x07"]);
  });
});

test("uses the current Pi session name without owning session naming", async () => {
  await insideTmux(async () => {
    const harness = createHarness({ sessionName: "Initial explicit name" });

    assert.equal(harness.hasHandler("before_agent_start"), false);

    await harness.emit("session_start");
    harness.setSessionName("Updated explicit name");
    await harness.emit("agent_end", [assistant("stop", "Done")]);
    await harness.emit("agent_settled");

    assert.deepEqual(harness.calls[0], [
      "agent",
      "idle",
      "--title",
      "pi · Initial explicit name",
    ]);
    assert.deepEqual(harness.calls.at(-1), [
      "agent",
      "succeeded",
      "--title",
      "pi · Updated explicit name",
      "--summary",
      "Done",
    ]);
  });
});

test("classifies incomplete and unsuccessful assistant responses as failures", () => {
  for (const reason of ["length", "error", "aborted"] as const) {
    assert.deepEqual(outcome([assistant(reason)]), {
      state: "failed",
    });
  }

  assert.deepEqual(outcome([]), { state: "failed" });
});

test("accepts normal and terminating-tool responses as successes", () => {
  assert.deepEqual(outcome([assistant("stop")]), {
    state: "succeeded",
  });
  assert.deepEqual(outcome([assistant("toolUse")]), {
    state: "succeeded",
  });
});

test("passes the settled summary through without normalization or truncation", () => {
  const summary = `  done\n\u001b now ${"🦀".repeat(200)}  `;

  assert.deepEqual(outcome([assistant("stop", summary)]), {
    state: "succeeded",
    summary: summary.trim(),
  });
});

test("does not register lifecycle hooks outside tmux", () => {
  const previousPane = process.env.TMUX_PANE;
  delete process.env.TMUX_PANE;
  try {
    const harness = createHarness();
    assert.equal(harness.hasHandler("session_start"), false);
  } finally {
    if (previousPane !== undefined) {
      process.env.TMUX_PANE = previousPane;
    }
  }
});

test("warns only once when smth cannot publish state", async () => {
  await insideTmux(async () => {
    const harness = createHarness({
      code: 1,
      stderr: "tmux unavailable\nmore",
    });

    await harness.emit("session_start");
    await harness.emit("agent_start");

    assert.deepEqual(harness.notifications, [
      "Could not notify smth: tmux unavailable",
    ]);
  });
});
