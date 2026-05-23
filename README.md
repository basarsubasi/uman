# `uniman` — Universal Man Pages

`uniman` read man pages from any operating system on any unix machine, natively.

```bash
uniman install linux-upstream
uniman linux execve
```

## Dependencies

`uniman` delegates rendering to your system's man page renderer. Make sure you have either `man-db` or `mandoc` on your system.

`git` is required for cloning backends. `curl` is needed for HTTP-backed backends.



## Installation


### From source

```bash
git clone https://github.com/your-org/uniman.git
cd uniman
cargo install --path .
```


## Configuration

`uniman` stores its data in two locations:

| Path | Purpose |
|------|---------|
| `~/.config/uniman/config.json` | Backend registry and settings |
| `~/.uniman/` | Backend data and SQLite index |

The config file is created automatically on first run with default backends. You can edit it to add custom backends:

To print the config path:

```bash
uniman config
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
uniman execve              # uses default backend
uniman 2 execve            # section + topic with default backend
```

Change or view the default:

```bash
uniman default              # show current default
uniman default linux        # set by alias
uniman default linux-upstream  # set by name
```

### Storage layout

```
~/.config/uniman/
  config.json 

~/.uniman/
  backends/
    linux-upstream/    # raw man pages
    freebsd/
  index/
    uniman.db            # SQLite db
```

## Usage

### Reading man pages

```bash
uniman <backend> [<section>] <topic>     # explicit backend
uniman <topic>                            # default backend
uniman <section> <topic>                  # default backend with default section
```

```bash
uniman linux-upstream 2 execve           # full form
uniman linux execve                      # alias, section auto-resolved
uniman execve                            # default backend, default section
uniman 2 execve                          # default backend, explicit section
```


### Installing backends

```bash
uniman install <backend>
```

```bash
uniman install linux-upstream
uniman install freebsd
```

The first installed backend is automatically set as the default.

### Listing backends

```bash
uniman list
```

Output:

```
NAME                 DEFAULT    STATUS     FORMAT SOURCE
linux-upstream       *          installed  roff   https://github.com/mkerrisk/man-pages
freebsd                         available  roff   https://gitlab.freebsd.org/freebsd/doc-manual.git
```

### Listing topics in a backend

```bash
uniman list <backend>
```

```bash
uniman list linux-upstream
uniman list linux          # alias works too
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
uniman search <topic>         # filename search (default)
uniman search -k <keyword>    # keyword search (name + description)
```

```bash
uniman search execve
```

Output:

```
BACKEND              SECTION    NAME
linux-upstream       2          execve
linux-upstream       2          execveat
linux-upstream       3          fexecve
```

```bash
uniman search -k execute
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
eval "$(uniman completion bash)"
```

#### zsh

Add to `~/.zshrc`:

```zsh
eval "$(uniman completion zsh)"
```

