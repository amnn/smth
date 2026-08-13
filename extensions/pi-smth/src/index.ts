// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

/** Publish Pi agent lifecycle transitions through `smth agent`. */

import type {
  AgentEndEvent,
  ExtensionAPI,
  ExtensionContext,
} from "@earendil-works/pi-coding-agent";

type State = "idle" | "running" | "succeeded" | "failed" | "exit";

/** A lifecycle transition and optional notification summary sent to smth. */
interface Outcome {
  state: State;
  /** Assistant text passed to smth without normalization or truncation. */
  summary?: string;
}

/** Classify the last assistant message and collect its notification summary. */
export function outcome(messages: AgentEndEvent["messages"]): Outcome {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (message?.role !== "assistant") {
      continue;
    }

    const state =
      message.stopReason === "stop" || message.stopReason === "toolUse"
        ? "succeeded"
        : "failed";

    const summary = message.content
      .filter((block) => block.type === "text")
      .map((block) => block.text)
      .join("\n")
      .trim();

    return summary ? { state, summary } : { state };
  }

  return { state: "failed" };
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
    ctx.ui.notify(`Could not notify smth: ${detail}`, "warning");
  };

  // Cache the last outcome from an agent turn end event, to emit once the agent
  // is fully settled.
  //
  // There may be multiple agent start/end events during a single agent turn,
  // because of tool calls or retries. We only want to notify once we know the
  // agent is not going to make future tool calls.
  let settled: Outcome = { state: "failed" };

  const update = async (
    outcome: Outcome,
    ctx: ExtensionContext,
  ): Promise<void> => {
    try {
      const args = ["agent", outcome.state];
      const title = pi.getSessionName();

      args.push("--title", title ? `pi · ${title}` : "pi");
      if (outcome.summary) args.push("--summary", outcome.summary);

      const result = await pi.exec("smth", args, {
        timeout: 5_000,
      });

      if (result.stdout) process.stdout.write(result.stdout);

      if (result.code === 0) {
        return;
      }

      const stderr = result.stderr.trim().split("\n", 1)[0];
      const detail = stderr || `smth exited with status ${result.code}`;
      warn(ctx, detail);
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      warn(ctx, detail);
    }
  };

  pi.on("session_start", async (_event, ctx) => {
    settled = { state: "failed" };
    await update({ state: "idle" }, ctx);
  });

  pi.on("agent_start", async (_event, ctx) => {
    settled = { state: "failed" };
    await update({ state: "running" }, ctx);
  });

  pi.on("agent_end", (event) => {
    settled = outcome(event.messages);
  });

  pi.on("agent_settled", async (_event, ctx) => {
    await update(settled, ctx);
  });

  pi.on("session_shutdown", async (_event, ctx) => {
    await update({ state: "exit" }, ctx);
  });
}
