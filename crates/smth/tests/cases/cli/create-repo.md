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

The modifier takes no argument: `--create` supplies the name to sanitize. It
should initialize below the configured root and create a detached session with
repository metadata and the usual setup script.

    :$ smth --config smth.toml --no-base --create-repo --create "project one"

    :$ sh -c 'test -d config-repos/project-one/.jj && test -d config-repos/project-one/.git'

    :t show-options -qv -t '=project-one:' @smth.test-created

    :$ sh -c 'tmux show-options -qv -t "=project-one:" @smth.repo | sed "s#$PWD#<ROOT>#g"'

    :$ sh -c 'tmux display-message -p -t "=project-one:0.0" "#{pane_current_path}" | sed "s#$PWD#<ROOT>#g"'

    :t display-message -p '#{client_session}'

Repeating the request should create another repository with a disambiguated
name rather than reuse the existing repository or session. The modifier can
also follow the action's name.

    :$ smth --config smth.toml --no-base --create "project one" --create-repo

    :$ sh -c 'test -d config-repos/project-one~1/.jj && test -d config-repos/project-one/.jj'

Configuring a repository root must not change the existing plain-session
behavior.

    :$ smth --config smth.toml --no-base --create bare

    :t show-options -qv -t '=bare:' @smth.repo

    :$ sh -c 'test ! -e config-repos/bare'

Adding `--create-repo` must disambiguate instead of reusing or converting the
plain session.

    :$ smth --config smth.toml --no-base --create bare --create-repo

    :$ sh -c 'test -d config-repos/bare~1/.jj && test ! -e config-repos/bare'

    :t show-options -qv -t '=bare:' @smth.repo

A command-line root should override the configured root.

    :$ smth --config smth.toml --no-base --repo-root cli-repos --create override --create-repo

    :$ sh -c 'test -d cli-repos/override/.jj && test ! -e config-repos/override'

Filesystem and tmux collisions should use the same `~N` disambiguation as other
prospective sessions without modifying the existing directory.

    :$ mkdir config-repos/collision

    :t new-session -d -s collision~1 "cat"

    :$ smth --config smth.toml --no-base --create collision --create-repo

    :$ sh -c 'test -d config-repos/collision~2/.jj && test ! -e config-repos/collision/.jj'

`--switch` with the modifier should perform the same initialization and name
disambiguation before switching the client, leaving the existing session alone.
Its stdout should report the disambiguated name, not the requested one.

    :t new-session -d -s switched "cat"
    :t respawn-pane -k -t runner:0.0 'smth --config smth.toml --no-base --repo-root switch-repos --switch switched --create-repo > switch-name; tmux wait-for -S switched-repo; cat'
    :t wait-for switched-repo
    :$ cat switch-name

    :t display-message -p '#{client_session}'

    :$ sh -c 'test -d switch-repos/switched~1/.jj && test -d switch-repos/switched~1/.git'
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

    :$ sh -c 'test -d unnamed-repos/1/.jj && test -d unnamed-repos/1/.git && test ! -e unnamed-repos/.jj'

An explicitly empty name should also go through disambiguation, skipping
existing paths and session names.

    :$ mkdir unnamed-repos/2

    :$ smth --config smth.toml --no-base --repo-root unnamed-repos --create "" --create-repo

    :$ sh -c 'test -d unnamed-repos/3/.jj && test ! -e unnamed-repos/2/.jj'

Omitting the name should use the same numeric disambiguation for detached
creation, without changing the current session.

    :$ smth --config smth.toml --no-base --repo-root unnamed-repos --create --create-repo

    :$ sh -c 'test -d unnamed-repos/4/.jj && test -d unnamed-repos/4/.git'

    :t display-message -p '#{client_session}'

Switching without a name should initialize the next available numeric
repository, switch the client to its session, and print the selected name.

    :t respawn-pane -k -t runner:0.0 'smth --config smth.toml --no-base --repo-root unnamed-repos --switch --create-repo > switch-name; tmux wait-for -S unnamed-repo; cat'
    :t wait-for unnamed-repo
    :$ cat switch-name

    :t display-message -p '#{client_session}'

    :$ sh -c 'test -d unnamed-repos/5/.jj && test -d unnamed-repos/5/.git'
    :t switch-client -t runner
    :pane runner:0.0

Repository creation requires an empty context and rejects incompatible or
competing actions.

    :$ sh -c 'cd config-repos/project-one && smth --config ../../smth.toml --create nested --create-repo'

    :$ smth --config smth.toml --base config-repos/project-one --create nested --create-repo

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
