# Scripting

`smth` accepts fzf-style startup flags for scripted bindings:

- `-q`, `--query STR` seeds the interactive query.
- `-1`, `--select-1` switches immediately when the initial query has one
  match.
- `-0`, `--exit-0` exits instead of opening the UI when the initial query has
  no matches.
- `-f`, `--filter` skips the UI and prints matches for the query from `--query`;
  combine it with `-1` to switch when there is exactly one match.
