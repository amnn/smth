## Code Style

### File Order

Each Rust module should follow the following order:

- Copyright notice
- Module doc comment
- Child modules
- Imports
- Re-exports
- Constants
- Type definitions and aliases
- `impl` blocks
- Trait `impl` blocks
- Module-level functions
- Test module

The entries in each of the above sections should be ordered by decreasing
visibility, and then lexicographically. However, this ordering rule should not
be applied inside item bodies or signatures. For example, struct fields and
function parameters should be ordered by domain logic and readability, not
alphabetically.

For `impl` and trait `impl` sections, treat member items (for example methods,
associated consts, and associated types) as entries and apply the same rule:
decreasing visibility, then lexicographic order. Make an exception for
constructors, which should be surfaced at the top of their `impl` block (but
can again be sorted by visibility and then lexicographically).

If you find that a module is getting too large, and elements pertaining to,
e.g. the same type are spread out too far, that is a sign that this module
needs to be split up.

### Imports

Imports at the top of the file are broken up into three sections, with an
empty line between each:

- `std` imports
- external crate imports
- internal crate imports

`tokio`, `tokio_util`, are examples of external imports.

In each section, there is one import per-line (no nested imports, or wildcard
imports), and the imports are sorted alphabetically. Internal crate imports all
start with `crate::`, (never `super::` or `self::`).

Exception: `use super::*` is recommended inside a nested test (`#[cfg(test)]`)
module.

If traits are imported for use (rather than definition), import them `as _`.

Re-exported items (i.e., `pub use ...`) are grouped separately at the bottom of
the import list (with an empty line separating them from the rest of the
imports). Items should only be re-exported from module `A` in another module
`B`, if other modules with access to `B` do not also have access to `A`
(to prevent introducing opportunities for the same item to be imported through
different paths in different places).

### Associated Functions

Member functions that don't accept `self` are called associated functions. Only
use these for constructors (functions that return some variant of `Self`).

Other functions that support the `impl` of a type should be kept as
module-level functions.

### Constants and Literals

Factor literals out into constants if they are used in multiple places and the
constant would have a clear name that conveys the meaning of the literal, and
not the contents.

Conversely, DO NOT factor literals out into constants if the name of the
constant would change any time the literal's value changes.

### Comments

Non-trivial modules, structs, enums, type aliases, and functions should have
doc comments, with detail appropriate to the item kind:

- Modules: what grouping of logic the module represents, scope boundaries, and
  representative examples where helpful.
- Structs/enums/type aliases: what data is modeled, key invariants, and field
  doc comments where field-level intent is non-obvious.
- Functions/methods: what they do (not how), relevant parameters/returns, and
  any meaningful invariants, preconditions, postconditions, panic conditions,
  or error conditions.

Hard wrap comments at column 100.

### Turbofish

Avoid turbofish operators where the following transformation is possible:

```rust
let x = (..1).collect::<..2>(); -> let x: ..2 = (..1).collect();
```

It's okay to use the turbofish in circumstances where the type cannot be
inferred and there is not a binding that a type annotation can be attached to.
It is also okay when collecting into a `Result`, so `?` can be applied directly
without introducing a typed intermediate binding and then shadowing it:

```rust
let values = iter.collect::<anyhow::Result<Vec<_>>>()?;
```

### Strings

Prefer using `.to_owned` instead of `.to_string` when converting from `&str` to
`String` (use of `.to_string` is fine in other circumstances).

### Errors

Use `anyhow` to construct internal errors, and `thiserror` to define structured
error types for public APIs.

- Do not import `anyhow::Result`; write `anyhow::Result` explicitly in type
  signatures.
- Make use of `bail!`, `ensure!`, when appropriate:

```rust
return Err(anyhow!(...)) -> bail!(...)
if !cond { bail!(...) }  -> ensure!(cond, ...)
```

- Make use of `anyhow`'s `Context`:

```rust
.map_err(|e| anyhow!("...: {e}")
  -> .context("...") OR .with_context(|| format!("..."))
```

### Paths

Avoid building paths using hard-coded separators (e.g., `"/"`). Use
`std::path::PathBuf` and its `Extend` implementation to add multiple components
to preserve the portability of the code.

### Markdown Snapshot Cases

Explain what each setup phase creates and why those fixtures matter to the
scenario. Before each snapshot, explain what output is expected and why it
follows from the setup and actions. This includes captured command output and
expected failures, not just pane snapshots. Do not leave readers to infer the
assertion from key presses or captured output. A shared explanation can cover a
short sequence when it clearly identifies the expected result of each step.

For command transcripts, explain where significant output values come from and
why they are expected: trace transformations, variable expansion, environment
lookups, and counts back to their inputs. Explain exit statuses, diagnostics,
and intentionally absent output in terms of the behavior being tested, rather
than merely restating what the commands do.

Prefer `smth`'s non-interactive CLI for ordinary setup when it simplifies
repository/workspace creation, tmux session creation, metadata assignment, or
staging. The goal is simpler, higher-level setup, not using `smth`
everywhere; keep direct setup when a CLI-based equivalent needs extra shell
plumbing or manual metadata manipulation without reducing complexity. For
example, use `smth --base alpha --create feature` for a live workspace-backed
session. Keep direct `jj` or `tmux` setup when the test needs repo-only
entries, custom history, aliases, stale or malformed metadata, or other state
the CLI cannot construct. Do not pre-create the state whose creation the test
asserts.

Declare only binaries the case uses, directly or through the application,
quoted shell commands, pane helpers, or fixture scripts. Remove stale entries
from `:b` when setup changes. Shell builtins do not require an external
binary, but direct execution such as `:$ test ...` does. Keep otherwise
unused entries when testing binary availability or the directive itself
explicitly.

Prefer short forms for integration-test directives and their flags, such as
`:$`, `:t`, and `:snap -d 2s -e ready`. This rule applies to the test
runner's syntax, not the binaries invoked by directives or quoted shell
commands. Keep long forms when testing those forms explicitly.

When `:settle` is immediately followed by `:snap`, combine them into one
`:snap`, preserving the required duration, count, expectations, and filters
along with snapshot options such as `--color`. Snapshots already wait for a
stable matching frame and capture that same frame. Keep a separate `:settle`
when it guards a subsequent action, or when testing settling itself.

Between consecutive directives, leave one blank line after multi-line rendered
output, including snapshots, fenced stdout or stderr, and warning callouts.
This applies to expected failures as well as successful commands; judge the
rendered transcript, not just the exit status or output stream.

Give each shell invocation followed by pipes or binds its own block, even when
the shell invocation is quiet. Separate the entire chain from surrounding setup
and assertions with blank lines, but keep its shell, pipe, and bind directives
adjacent: blank lines terminate chains. This required adjacency takes
precedence over spacing after multi-line output. Preserve deliberate
interruptions in tests of chain boundaries.

Leave one blank line between a `:w` (or `:write`) directive and the fenced
block containing the file's contents.

Keep closely related directives adjacent when they produce no multi-line
output, such as binary and fixture setup or key presses followed by settling.
A blank line may separate logical phases even when commands are silent—for
example, setup from assertions or one completed interaction from the next.
Do not collapse these useful groupings mechanically. Keep ordinary paragraph
and code-block separation.
