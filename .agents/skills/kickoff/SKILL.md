---
name: kickoff
description: Review GitHub issues and pick the next best task to start now
---

## Goal

Start execution quickly by choosing the next most appropriate GitHub issue.

## Steps

1. List open GitHub issues with `gh issue list --repo amnn/smth`.
2. Read candidate issues and their dependencies, assignees, and progress notes.
   Exclude blocked work and avoid taking over work already in progress.
3. Choose one concrete next task using these tie-breakers in order:
   - Prefer tasks that unblock other listed work.
   - Prefer tasks with clear acceptance criteria over vague investigations.
   - Prefer smaller, high-leverage tasks when multiple options are equal.
   - If a section is already active, continue within that section for momentum.
4. Propose a concrete plan first.
5. Ask for explicit approval before starting implementation.
6. If information is missing, ask targeted questions as needed, with at most
   one question per area (for example scope, risk, environment, acceptance).
   For each question, include a recommended default and explain what changes
   based on the answer.

## Output

- State the selected issue title and link.
- Explain briefly why it was selected.
- Provide a short plan in the same response.
- End by asking for explicit go-ahead before implementation begins.

## Constraints

- Track follow-up work in GitHub issues, not a local TODO file.
- Do not close issues unless the implementation is actually done.
- Do not rewrite or reprioritize the whole issue backlog unless explicitly asked.
- Do not start implementation until the user explicitly approves the plan.
