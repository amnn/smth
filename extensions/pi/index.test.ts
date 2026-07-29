// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

import assert from "node:assert/strict";
import test from "node:test";

import type {
  AgentEndEvent,
  ExtensionAPI,
  ExtensionContext,
} from "@earendil-works/pi-coding-agent";

import extension, { settledState } from "./index.ts";

type EventHandler = (
  event: { messages?: AgentEndEvent["messages"] },
  ctx: ExtensionContext,
) => Promise<void> | void;

interface HarnessOptions {
  code?: number;
  stderr?: string;
  throws?: Error;
}

function assistantMessage(
  stopReason: "stop" | "length" | "toolUse" | "error" | "aborted",
): AgentEndEvent["messages"][number] {
  return { role: "assistant", stopReason } as AgentEndEvent["messages"][number];
}

function createHarness(options: HarnessOptions = {}) {
  const calls: string[] = [];
  const handlers = new Map<string, EventHandler>();
  const notifications: string[] = [];
  const pi = {
    exec: async (command: string, args: string[]) => {
      assert.equal(command, "sesh");
      assert.equal(args[0], "agent");
      calls.push(args.at(-1) ?? "");
      if (options.throws) {
        throw options.throws;
      }

      return {
        code: options.code ?? 0,
        killed: false,
        stderr: options.stderr ?? "",
        stdout: "",
      };
    },
    on: (event: string, handler: EventHandler) => {
      handlers.set(event, handler);
    },
  } as unknown as ExtensionAPI;
  const ctx = {
    hasUI: true,
    ui: {
      notify: (message: string) => notifications.push(message),
    },
  } as unknown as ExtensionContext;

  extension(pi);

  return {
    calls,
    notifications,
    hasHandler: (event: string) => handlers.has(event),
    async emit(event: string, messages: AgentEndEvent["messages"] = []) {
      const handler = handlers.get(event);
      assert.ok(handler, `expected a handler for ${event}`);
      await handler({ messages }, ctx);
    },
  };
}

async function insideTmux(run: () => Promise<void>): Promise<void> {
  const previousPane = process.env.TMUX_PANE;
  process.env.TMUX_PANE = "%test";
  try {
    await run();
  } finally {
    if (previousPane === undefined) {
      delete process.env.TMUX_PANE;
    } else {
      process.env.TMUX_PANE = previousPane;
    }
  }
}

test("publishes the Pi lifecycle and waits for the final settled outcome", async () => {
  await insideTmux(async () => {
    const harness = createHarness();

    await harness.emit("session_start");
    await harness.emit("agent_start");
    await harness.emit("agent_end", [assistantMessage("error")]);
    await harness.emit("agent_start");
    await harness.emit("agent_end", [assistantMessage("stop")]);
    assert.deepEqual(harness.calls, ["idle", "running", "running"]);

    await harness.emit("agent_settled");
    await harness.emit("session_shutdown");
    assert.deepEqual(harness.calls, [
      "idle",
      "running",
      "running",
      "succeeded",
      "exit",
    ]);
  });
});

test("classifies incomplete and unsuccessful assistant responses as failures", () => {
  for (const reason of ["length", "error", "aborted"] as const) {
    assert.equal(settledState([assistantMessage(reason)]), "failed");
  }

  assert.equal(settledState([]), "failed");
});

test("accepts normal and terminating-tool responses as successes", () => {
  assert.equal(settledState([assistantMessage("stop")]), "succeeded");
  assert.equal(settledState([assistantMessage("toolUse")]), "succeeded");
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

test("warns only once when sesh cannot publish state", async () => {
  await insideTmux(async () => {
    const harness = createHarness({
      code: 1,
      stderr: "tmux unavailable\nmore",
    });

    await harness.emit("session_start");
    await harness.emit("agent_start");

    assert.deepEqual(harness.notifications, [
      "Could not notify sesh: tmux unavailable",
    ]);
  });
});
