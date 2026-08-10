// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

import assert from "node:assert/strict";
import test from "node:test";

import type {
  AgentEndEvent,
  ExtensionAPI,
  ExtensionContext,
  SessionEntry,
} from "@earendil-works/pi-coding-agent";

import extension, { outcome } from "../index.ts";
import { deserializeTitle } from "../src/title.ts";

interface TestEvent {
  messages?: AgentEndEvent["messages"];
  prompt?: string;
}

type EventHandler = (
  event: TestEvent,
  ctx: ExtensionContext,
) => Promise<void> | void;

interface HarnessOptions {
  code?: number;
  deferTitle?: boolean;
  entries?: readonly SessionEntry[];
  generatedTitle?: string;
  sessionName?: string;
  stderr?: string;
  throws?: Error;
  titleThrows?: Error;
}

interface AppendedEntry {
  customType: string;
  data: unknown;
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

function title(title: string, id = "title"): SessionEntry {
  return {
    type: "custom",
    customType: "sesh.notification-title",
    data: { version: 1, title },
    id,
    parentId: null,
    timestamp: "2026-01-01T00:00:00.000Z",
  };
}

function user(text: string, id = "user"): SessionEntry {
  return {
    type: "message",
    id,
    parentId: null,
    timestamp: "2026-01-01T00:00:00.000Z",
    message: {
      role: "user",
      content: [{ type: "text", text }],
      timestamp: 0,
    },
  } as SessionEntry;
}

function createHarness(options: HarnessOptions = {}) {
  const appendedEntries: AppendedEntry[] = [];
  const calls: string[][] = [];
  const entries = [...(options.entries ?? [])];
  const handlers = new Map<string, EventHandler>();
  const notifications: string[] = [];
  const titlePrompts: string[] = [];

  let guard: (() => void) | undefined;
  const barrier = options.deferTitle
    ? new Promise<void>((resolve) => {
        guard = resolve;
      })
    : undefined;

  let sessionName = options.sessionName;

  const pi = {
    appendEntry: (customType: string, data: unknown) => {
      appendedEntries.push({ customType, data });
      entries.push({
        type: "custom",
        customType,
        data,
        id: `custom-${appendedEntries.length}`,
        parentId: null,
        timestamp: "2026-01-01T00:00:00.000Z",
      });
    },

    exec: async (command: string, args: string[]) => {
      assert.equal(command, "sesh");
      assert.equal(args[0], "agent");
      calls.push(args);
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

    getSessionName: () => sessionName,

    on: (event: string, handler: EventHandler) => {
      handlers.set(event, handler);
    },
  } as unknown as ExtensionAPI;

  const ctx = {
    hasUI: true,
    model: { id: "test-model" },
    modelRegistry: {
      complete: async (
        _model: unknown,
        context: {
          messages: Array<{
            content: string | Array<{ type: string; text?: string }>;
          }>;
        },
      ) => {
        const content = context.messages[0]?.content;

        titlePrompts.push(
          typeof content === "string"
            ? content
            : (content ?? [])
                .filter((block) => block.type === "text")
                .map((block) => block.text ?? "")
                .join("\n"),
        );

        await barrier;

        if (options.titleThrows) {
          throw options.titleThrows;
        }

        return {
          content: [
            {
              type: "text",
              text: options.generatedTitle ?? "Generated session title",
            },
          ],
          stopReason: "stop",
        };
      },
    },

    sessionManager: { getBranch: () => entries },

    signal: undefined,

    ui: { notify: (message: string) => notifications.push(message) },
  } as unknown as ExtensionContext;

  extension(pi);

  return {
    appendedEntries,
    calls,
    notifications,
    titlePrompts,

    async release() {
      const release = guard;
      assert.ok(release, "expected deferred title generation");
      guard = undefined;
      release();
      await new Promise<void>((resolve) => setImmediate(resolve));
    },

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
    await harness.emit("before_agent_start", {
      prompt: "Implement the requested change",
    });
    await harness.emit("agent_start");
    await harness.emit("agent_end", [assistant("error", "retry")]);
    await harness.emit("agent_start");
    await harness.emit("agent_end", [
      assistant("stop", "  Finished\n successfully  "),
    ]);

    assert.deepEqual(harness.calls, [
      ["agent", "idle", "--title", "pi"],
      ["agent", "running", "--title", "pi"],
      ["agent", "running", "--title", "pi · Generated session title"],
    ]);

    await harness.emit("agent_settled");
    await harness.emit("session_shutdown");

    assert.deepEqual(harness.calls, [
      ["agent", "idle", "--title", "pi"],
      ["agent", "running", "--title", "pi"],
      ["agent", "running", "--title", "pi · Generated session title"],
      [
        "agent",
        "succeeded",
        "--title",
        "pi · Generated session title",
        "--summary",
        "Finished\n successfully",
      ],
      ["agent", "exit", "--title", "pi · Generated session title"],
    ]);

    assert.deepEqual(harness.titlePrompts, ["Implement the requested change"]);

    assert.deepEqual(harness.appendedEntries, [
      {
        customType: "sesh.notification-title",
        data: { version: 1, title: "Generated session title" },
      },
    ]);
  });
});

test("uses an explicit session name without suppressing title generation", async () => {
  await insideTmux(async () => {
    const harness = createHarness({
      generatedTitle: "Generated fallback",
      sessionName: "Initial explicit name",
    });

    await harness.emit("session_start");
    await harness.emit("before_agent_start", { prompt: "Fix session naming" });
    harness.setSessionName("Updated explicit name");
    await harness.emit("agent_end", [assistant("stop", "Done")]);
    await harness.emit("agent_settled");

    assert.deepEqual(harness.titlePrompts, ["Fix session naming"]);

    assert.deepEqual(harness.appendedEntries, [
      {
        customType: "sesh.notification-title",
        data: { version: 1, title: "Generated fallback" },
      },
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

test("restores the earliest generated title without regenerating it", async () => {
  await insideTmux(async () => {
    const harness = createHarness({
      entries: [
        title("Stable title", "first-title"),
        title("Duplicate title", "second-title"),
      ],
    });

    await harness.emit("session_start");
    await harness.emit("before_agent_start", { prompt: "A later prompt" });
    await harness.emit("agent_end", [assistant("stop")]);
    await harness.emit("agent_settled");

    assert.deepEqual(harness.titlePrompts, []);
    assert.deepEqual(harness.appendedEntries, []);
    assert.deepEqual(harness.calls.at(-1), [
      "agent",
      "succeeded",
      "--title",
      "pi · Stable title",
    ]);
  });
});

test("deserializes the first title without consuming later entries", () => {
  let consumedLaterEntry = false;

  function* entries(): Generator<SessionEntry> {
    yield title("Stable title", "first-title");
    consumedLaterEntry = true;
    yield title("Duplicate title", "second-title");
  }

  assert.equal(deserializeTitle(entries()), "Stable title");
  assert.equal(consumedLaterEntry, false);
});

test("generates once from the initial stored user message", async () => {
  await insideTmux(async () => {
    const harness = createHarness({
      entries: [user("The original task")],
    });

    await harness.emit("session_start");
    await harness.emit("before_agent_start", { prompt: "A follow-up task" });
    await harness.emit("before_agent_start", { prompt: "Another follow-up" });

    assert.deepEqual(harness.titlePrompts, ["The original task"]);
  });
});

test("prefixes generated titles without further normalization", async () => {
  await insideTmux(async () => {
    const title = "  Raw\n title  ";
    const harness = createHarness({ generatedTitle: title });

    await harness.emit("session_start");
    await harness.emit("before_agent_start", { prompt: "A task" });
    await harness.emit("agent_end", [assistant("stop")]);
    await harness.emit("agent_settled");

    assert.deepEqual(harness.calls.at(-1), [
      "agent",
      "succeeded",
      "--title",
      `pi · ${title.trim()}`,
    ]);
  });
});

test("does not wait for title generation before notifying", async () => {
  await insideTmux(async () => {
    const harness = createHarness({
      deferTitle: true,
      generatedTitle: "Late title",
    });

    await harness.emit("session_start");
    await harness.emit("before_agent_start", { prompt: "Initial task" });
    await harness.emit("agent_end", [assistant("stop")]);
    await harness.emit("agent_settled");

    assert.deepEqual(harness.titlePrompts, ["Initial task"]);
    assert.deepEqual(harness.appendedEntries, []);
    assert.deepEqual(harness.calls.at(-1), [
      "agent",
      "succeeded",
      "--title",
      "pi",
    ]);

    await harness.emit("before_agent_start", { prompt: "Follow-up task" });
    assert.deepEqual(harness.titlePrompts, ["Initial task"]);

    await harness.release();
    assert.deepEqual(harness.appendedEntries, [
      {
        customType: "sesh.notification-title",
        data: { version: 1, title: "Late title" },
      },
    ]);

    await harness.emit("agent_end", [assistant("stop")]);
    await harness.emit("agent_settled");
    assert.deepEqual(harness.calls.at(-1), [
      "agent",
      "succeeded",
      "--title",
      "pi · Late title",
    ]);
  });
});

test("discards a generated title that finishes after shutdown", async () => {
  await insideTmux(async () => {
    const harness = createHarness({
      deferTitle: true,
      generatedTitle: "Stale title",
    });

    await harness.emit("session_start");
    await harness.emit("before_agent_start", { prompt: "Initial task" });
    await harness.emit("session_shutdown");
    await harness.release();

    assert.deepEqual(harness.appendedEntries, []);
  });
});

test("keeps lifecycle updates working when title generation fails", async () => {
  await insideTmux(async () => {
    const harness = createHarness({
      entries: [user("A task")],
      titleThrows: new Error("model unavailable"),
    });

    await harness.emit("session_start");
    await harness.emit("before_agent_start", { prompt: "A task" });
    await harness.emit("before_agent_start", { prompt: "A follow-up task" });
    await harness.emit("agent_end", [assistant("stop", "Done")]);
    await harness.emit("agent_settled");

    assert.deepEqual(harness.titlePrompts, ["A task"]);
    assert.deepEqual(harness.appendedEntries, []);
    assert.deepEqual(harness.calls.at(-1), [
      "agent",
      "succeeded",
      "--title",
      "pi",
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
