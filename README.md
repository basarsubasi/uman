# `uman` — Universal Man Pages

`uman` read man pages from any operating system on any unix machine, natively.

```bash
uman install linux-upstream
uman linux execve
```

## Dependencies

`uman` delegates rendering to your system's man page renderer. Make sure you have either `man-db` or `mandoc` on your system.

`git` is required for cloning backends. `curl` is needed for HTTP-backed backends.



## Installation


### From source

```bash
git clone https://github.com/your-org/uman.git
cd uman
cargo install --path .
```


## Configuration

`uman` stores its data in two locations:

| Path | Purpose |
|------|---------|
| `~/.config/uman/config.json` | Backend registry and settings |
| `~/.uman/` | Backend data and SQLite index |

The config file is created automatically on first run with default backends. You can edit it to add custom backends:

To print the config path:

```bash
uman config
```

```json
{
  "backends": {
    "linux-upstream": {
      "name": "linux",
      "source": "https://github.com/mkerrisk/man-pages",
      "format": "roff",
      "fetching": "git",
      "aliases": ["linux"]
    },
    "freebsd": {
      "name": "freebsd",
      "source": "https://gitlab.freebsd.org/freebsd/doc-manual.git",
      "format": "roff",
      "fetching": "curl",
      "aliases": ["bsd"]
    }
  },
  "default_backend": "linux-upstream"
}
```

### Backend fields

| Field | Description |
|-------|-------------|
| `name` | Identifier used in commands |
| `source` | URL to clone (`git`) or download (`curl`) |
| `format` | Man page format (`roff`) |
| `fetching` | Download method: `git` (recommended) or `curl` |
| `aliases` | Short names that resolve to this backend (e.g. `linux` → `linux-upstream`) |

### Default backend

The first backend you install becomes the default automatically. You can read man pages without specifying a backend:

```bash
uman execve              # uses default backend
uman 2 execve            # section + topic with default backend
```

Change or view the default:

```bash
uman default              # show current default
uman default linux        # set by alias
uman default linux-upstream  # set by name
```

### Storage layout

```
~/.config/uman/
  config.json 

~/.uman/
  backends/
    linux-upstream/    # raw man pages
    freebsd/
  index/
    uman.db            # SQLite db
```

## Usage

### Reading man pages

```bash
uman <backend> [<section>] <topic>     # explicit backend
uman <topic>                            # default backend
uman <section> <topic>                  # default backend with default section
```

```bash
uman linux-upstream 2 execve           # full form
uman linux execve                      # alias, section auto-resolved
uman execve                            # default backend, default section
uman 2 execve                          # default backend, explicit section
```


### Installing backends

```bash
uman install <backend>
```

```bash
uman install linux-upstream
uman install freebsd
```

The first installed backend is automatically set as the default.

### Listing backends

```bash
uman list
```

Output:

```
NAME                 DEFAULT    STATUS     FORMAT SOURCE
linux-upstream       *          installed  roff   https://github.com/mkerrisk/man-pages
freebsd                         available  roff   https://gitlab.freebsd.org/freebsd/doc-manual.git
```

### Listing topics in a backend

```bash
uman list <backend>
```

```bash
uman list linux-upstream
uman list linux          # alias works too
```

Lists every man page topic indexed for that backend, sorted by section then name:

```
SEC    NAME                                     DESCRIPTION
1      bash                                     GNU Bourne-Again SHell
1      cp                                       copy files and directories
2      execve                                   execute program
2      open                                     open and possibly create a file
3      printf                                   formatted output conversion
...

4821 topic(s) in backend 'linux-upstream'.
```


### Searching

```bash
uman search <topic>         # filename search (default)
uman search -k <keyword>    # keyword search (name + description)
```

```bash
uman search execve
```

Output:

```
BACKEND              SECTION    NAME
linux-upstream       2          execve
linux-upstream       2          execveat
linux-upstream       3          fexecve
```

```bash
uman search -k execute
```

Output:

```
BACKEND              SECTION    NAME                             DESCRIPTION
linux-upstream       2          execve                           execute program
linux-upstream       2          execveat                         execute program relative to directory
```

### Shell completions

#### bash

Add to `~/.bashrc`:

```bash
eval "$(uman completion bash)"
```

#### zsh

Add to `~/.zshrc`:

```zsh
eval "$(uman completion zsh)"
```

Then reload your shell: `exec $SHELL` or open a new terminal.
