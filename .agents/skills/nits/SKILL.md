---
name: nits
description: Check scoped changes against STYLE.md using cached rule packs and read-only subagents.
---

# Scoped Nits Check

Run this workflow once before final validation when the current task changed
relevant hand-authored files. The parent agent is the only writer;
`style-guide` and `check-nits` subagents only inspect and report.

Do not invoke a `nits` shell command. None exists.

## 1. Establish edit scope

Start from files you edited for the current user task. Use version-control
state, such as `jj status` or `jj diff`, only to confirm or supplement that
knowledge.

- Do not include a pre-existing dirty file solely because it appears in
  version-control status or a diff.
- Ask for explicit scope when ownership is ambiguous.
- Skip generated-only or inapplicable files with a reason.
- Do not run a whole-codebase check unless the user explicitly requested one.
- Run one scoped check phase, not a check after every edit.

## 2. Reuse or obtain the style-guide clustering

Before invoking `style-guide`, inspect the parent conversation for a previously
successful cached clustering for this repository's current `STYLE.md`.

A cached clustering is reusable only when:

- it names the current repository's style-guide path;
- it includes an RFC 3339 generation timestamp;
- its coverage says every inventoried rule was assigned exactly once; and
- the current last-modified timestamp for `STYLE.md` is not later than the
  cached generation timestamp.

If those conditions hold, reuse the cached result and do not invoke
`style-guide` again. If the cache is absent, stale, malformed, or of uncertain
provenance, call `style-guide` once with the exact style-guide path.

After a successful call, retain its complete final report in the parent context
as the **cached style-guide clustering**. This is a conversational cache: do not
write a cache file or add repository state. Reuse it in later `nits` skill
invocations in the same context until `STYLE.md` changes. If the clustering is
`BLOCKED` or has incomplete coverage, stop and report the workflow as blocked.

## 3. Build the scoped coverage plan

Read the scoped files and use the cached clustering as the source of logical
rule packs. Decide which exact rule headings apply to the current edit scope.
Preserve the clustering where practical, remove inapplicable headings from the
packs, and omit empty packs.

Write an in-context coverage ledger before delegation:

```markdown
## Nits plan

### Edit scope

- `path/to/file`

### Style-guide clustering

- Reused cached result | Refreshed with `style-guide`

### Assigned rule packs

- Layout: Dependencies, File Order, Imports
- API and design: Associated Functions, Helpers and Abstractions, Constants and Literals
- Rust idioms: Comments, Turbofish, Strings, Errors, Paths

### Omitted rules

- Tests: no test code in scope
- Integration Tests: no integration artifacts in scope
```

Every inventoried `STYLE.md` rule must appear exactly once under an assigned
pack or under omitted rules with a concrete reason. Do not create one checker
per file or one checker per heading.

## 4. Emit sibling checker calls

For every assigned pack, emit one `subagent` tool call in the same parent turn.
The agent harness manages concurrency across those sibling calls. Each call
must have:

- `agent: "check-nits"`;
- a concise tool-call label taken from the cached clustering; and
- a self-contained task containing the exact file scope, exact assigned
  headings, changed-hunk context, and the `check-nits` report contract.

Do not put multiple packs into one subagent call. Do not ask checkers to edit.

After all sibling calls settle, require every planned pack to have completed
successfully. Preserve successful reports for diagnosis, but do not write or
claim complete coverage if any call failed, aborted, or returned malformed or
incomplete output. Retry only the failed pack when safe and explicitly
justified; otherwise report `BLOCKED`.

## 5. Verify coverage

Before applying findings, check that:

- the cached style-guide inventory matches the assigned and omitted rules;
- every rule was assigned once or explicitly omitted;
- no rule was assigned to multiple checkers;
- every report names exactly its assigned rules;
- every checker inspected every applicable scoped file;
- no checker silently widened edit scope; and
- no required report failed, aborted, or omitted required sections.

Incomplete coverage means the style check is blocked, not partially complete.

## 6. Consolidate findings

Treat checker output as evidence, not as patches to apply blindly.

1. Group findings by file and affected region.
2. Deduplicate overlapping findings.
3. Identify conflicting suggestions.
4. Reject low-confidence automatic fixes.
5. Keep structural, semantic, API, and broad-reordering findings report-only
   unless the current task explicitly authorizes them.
6. Treat snapshot findings as actionable. Refresh or synchronize the scoped
   artifacts according to the repository's snapshot guidance.
7. Reject findings outside edit scope.
8. Build one accepted edit plan per file and record why findings were rejected.

## 7. Refresh and edit once

Immediately before editing, re-read every affected file and treat the current
filesystem as authoritative. Verify that each finding still matches its cited
evidence. Discard or re-evaluate stale findings; never restore text the user
intentionally changed or reverted.

Apply accepted findings in one batched edit per file where possible. If an
unexpected concurrent change makes an edit unclear, stop rather than reverting
or guessing. Do not launch another complete check cycle afterward.

## 8. Validate and report

Run only validation relevant to accepted changes, followed by the repository's
normal final validation. Do not automatically duplicate the full workspace test
suite.

Report:

- whether the style-guide clustering was reused or refreshed;
- exact files checked;
- assigned packs and omitted rules with reasons;
- findings applied;
- findings rejected and reasons;
- report-only findings;
- blocked or unresolved findings;
- files edited; and
- verification commands run.

A successful verdict means all applicable rules were checked for the supplied
scope. It does not claim whole-codebase conformance.
