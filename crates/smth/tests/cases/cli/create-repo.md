# Create repository sessions

`--create-repo` should modify `--create` and `--switch` to initialize a fresh
colocated repository before creating its tmux session.

    :bins jj tmux cat sh sed mkdir

    :copy tests/fixtures/jjconfig.toml .jjconfig.toml

    :w smth.toml
```toml
[repo]
root = "config-repos"

[tmux]
setup = "tmux set-option @smth.test-created yes"
```

    :t rename-session -t 0 runner

The modifier takes no argument: `--create` supplies the directory name, preserved
exactly while the tmux name is sanitized. It
should initialize below the configured root and create a detached session with
repository metadata and the usual setup script.

    :$ smth --config smth.toml --no-base --create-repo --create "project one"

    :$ sh -c 'test -d "config-repos/project one/.jj" && test -d "config-repos/project one/.git"'

    :t show-options -qv -t '=project-one:' @smth.test-created

    :$ sh -c 'tmux show-options -qv -t "=project-one:" @smth.repo | sed "s#$PWD#<ROOT>#g"'

    :$ sh -c 'tmux display-message -p -t "=project-one:0.0" "#{pane_current_path}" | sed "s#$PWD#<ROOT>#g"'

    :t display-message -p '#{client_session}'

Repeating the request should fail without choosing another directory. The
modifier can also follow the action's name.

    :$ sh -c 'smth --config smth.toml --no-base --create "project one" --create-repo 2> error; status=$?; sed "s#$PWD#<ROOT>#g" error >&2; exit "$status"'

    :$ sh -c 'test ! -e "config-repos/project one~1" && test -d "config-repos/project one/.jj"'

Dots are preserved in the path even when the sanitized tmux name collides.

    :t new-session -d -s foo-bar "cat"
    :t new-session -d -s foo-bar~1 "cat"
    :$ smth --config smth.toml --no-base --create foo.bar --create-repo

    :$ sh -c 'test -d config-repos/foo.bar/.jj && test ! -e config-repos/foo.bar~2'

    :$ sh -c 'tmux show-options -qv -t "=foo-bar~2:" @smth.repo | sed "s#$PWD#<ROOT>#g"'

Configuring a repository root must not change the existing plain-session
behavior.

    :$ smth --config smth.toml --no-base --create bare

    :t show-options -qv -t '=bare:' @smth.repo

    :$ sh -c 'test ! -e config-repos/bare'

Adding `--create-repo` must disambiguate instead of reusing or converting the
plain session.

    :$ smth --config smth.toml --no-base --create bare --create-repo

    :$ sh -c 'test -d config-repos/bare/.jj && test ! -e config-repos/bare~1'

    :t show-options -qv -t '=bare:' @smth.repo

A command-line root should override the configured root.

    :$ smth --config smth.toml --no-base --repo-root cli-repos --create override --create-repo

    :$ sh -c 'test -d cli-repos/override/.jj && test ! -e config-repos/override'

An occupied path should be rejected without modifying the existing directory.

    :$ mkdir config-repos/collision

    :t new-session -d -s collision~1 "cat"

    :$ sh -c 'smth --config smth.toml --no-base --create collision --create-repo 2> error; status=$?; sed "s#$PWD#<ROOT>#g" error >&2; exit "$status"'

    :$ sh -c 'test ! -e config-repos/collision~2 && test ! -e config-repos/collision/.jj'

`--switch` with the modifier should perform the same initialization and name
disambiguation before switching the client, leaving the existing session alone.
Its stdout should report the disambiguated name, not the requested one.

    :t new-session -d -s switched "cat"
    :t respawn-pane -k -t runner:0.0 'smth --config smth.toml --no-base --repo-root switch-repos --switch switched --create-repo > switch-name; tmux wait-for -S switched-repo; cat'
    :t wait-for switched-repo
    :$ cat switch-name

    :t display-message -p '#{client_session}'

    :$ sh -c 'test -d switch-repos/switched/.jj && test -d switch-repos/switched/.git'

    :t has-session -t '=switched'
    :t switch-client -t runner
    :pane runner:0.0

Without either root setting, creation should default to the process working
directory.

    :w no-root.toml
```toml
[repo]
globs = []
```

    :$ mkdir fallback

    :$ sh -c 'cd fallback && smth --config ../no-root.toml --no-base --create local --create-repo'

    :$ sh -c 'test -d fallback/local/.jj && test -d fallback/local/.git'

Names that sanitize to empty should use numeric suffixes, including when the
repository root does not exist yet. The root itself must not be initialized.

    :$ smth --config smth.toml --no-base --repo-root unnamed-repos --create ... --create-repo

    :$ sh -c 'test -d unnamed-repos/.../.jj && test -d unnamed-repos/.../.git && test ! -e unnamed-repos/.jj'

Empty, omitted, and invalid directory names should be rejected.

    :$ smth --config smth.toml --no-base --create "" --create-repo

    :$ smth --config smth.toml --no-base --create --create-repo

    :$ smth --config smth.toml --no-base --switch --create-repo

    :$ smth --config smth.toml --no-base --create ../escape --create-repo

    :$ smth --config smth.toml --no-base --create /absolute --create-repo

    :$ smth --config smth.toml --no-base --create . --create-repo

    :$ smth --config smth.toml --no-base --create nested/repo --create-repo

    :$ smth --config smth.toml --no-base --create .. --create-repo

    :$ smth --config smth.toml --no-base --create ./relative --create-repo

Single normal components are accepted with trailing separators. Backslashes are
ordinary filename characters on Unix.

    :$ smth --config smth.toml --no-base --create 'a\b' --create-repo

    :$ sh -c 'test -d "config-repos/a\b/.jj" && test -d "config-repos/a\b/.git"'

    :$ smth --config smth.toml --no-base --create trailing/ --create-repo

    :$ sh -c 'test -d config-repos/trailing/.jj && test -d config-repos/trailing/.git'

A trailing `/.` passes component validation, but jj cannot initialize that path.
No repository or tmux session should be created.

    :$ smth --config smth.toml --no-base --create dotted/. --create-repo

    :$ sh -c 'test ! -e config-repos/dotted/.jj && test ! -e config-repos/dotted/.git'

    :t has-session -t '=dotted'

    :t display-message -p '#{client_session}'

Repository creation requires an empty context and rejects incompatible or
competing actions.

    :$ sh -c 'cd "config-repos/project one" && smth --config ../../smth.toml --create nested --create-repo'

    :$ smth --config smth.toml --base "config-repos/project one" --create nested --create-repo

    :$ smth --config smth.toml --no-base --onto @ --create invalid --create-repo

    :$ smth --config smth.toml --no-base --create first --switch second --create-repo

The modifier requires a create or switch action. It takes no operand of its own
and cannot be used with other actions or picker filtering options.

    :$ smth --config smth.toml --no-base --create-repo

    :$ smth --config smth.toml --no-base --create-repo unexpected --create invalid

    :$ smth --config smth.toml --no-base --close bare --create-repo

    :$ smth --config smth.toml --no-base --filter --create-repo

    :$ smth --config smth.toml --no-base --create invalid --create-repo --query invalid

---
vim: set ft=markdown:
