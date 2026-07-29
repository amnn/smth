// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

/** Publish Pi agent lifecycle transitions through `sesh agent`. */

import type {
  AgentEndEvent,
  ExtensionAPI,
  ExtensionContext,
} from "@earendil-works/pi-coding-agent";

type State = "idle" | "running" | "succeeded" | "failed" | "exit";
type Settled = "succeeded" | "failed";

/** Classify the final assistant response from a fully settled Pi agent run. */
export function settledState(messages: AgentEndEvent["messages"]): Settled {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (message?.role !== "assistant") {
      continue;
    }

    return message.stopReason === "stop" || message.stopReason === "toolUse"
      ? "succeeded"
      : "failed";
  }

  return "failed";
}

export default function (pi: ExtensionAPI): void {
  if (!process.env.TMUX_PANE) {
    return;
  }

  let warned = false;
  const warn = (ctx: ExtensionContext, detail: string): void => {
    if (warned || !ctx.hasUI) {
      return;
    }

    warned = true;
    ctx.ui.notify(`Could not notify sesh: ${detail}`, "warning");
  };

  // Cache the last state from an agent turn end event, to emit once the agent
  // is fully settled.
  //
  // There may be multiple agent start/end events during a single agent turn,
  // because of tool calls or retries. We only want to notify once we know the
  // agent is not going to make future tool calls.
  let settled: Settled = "failed";

  const update = async (state: State, ctx: ExtensionContext): Promise<void> => {
    try {
      const result = await pi.exec("sesh", ["agent", state], {
        timeout: 2_000,
      });

      if (result.code === 0) {
        return;
      }

      const stderr = result.stderr.trim().split("\n", 1)[0];
      const detail = stderr || `sesh exited with status ${result.code}`;
      warn(ctx, detail);
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      warn(ctx, detail);
    }
  };

  pi.on("session_start", async (_event, ctx) => {
    settled = "failed";
    await update("idle", ctx);
  });

  pi.on("agent_start", async (_event, ctx) => {
    settled = "failed";
    await update("running", ctx);
  });

  pi.on("agent_end", (event) => {
    settled = settledState(event.messages);
  });

  pi.on("agent_settled", async (_event, ctx) => {
    await update(settled, ctx);
  });

  pi.on("session_shutdown", async (_event, ctx) => {
    await update("exit", ctx);
  });
}
