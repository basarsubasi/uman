# `uman` — Universal Man Page Reader

`uman` lets you read man pages from any operating system on any machine, without VMs, containers, or remote access.

```bash
uman install linux-upstream
uman linux execve
```

Reading Linux man pages on macOS, BSD pages on Linux — locally, offline, instantly.

## Installation

### From source

Requirements:

- [Rust](https://rustup.rs/) (1.70+)
- `git` (for git-backed backends)
- `man` (man-db) or `mandoc` (for rendering)

```bash
git clone https://github.com/your-org/uman.git
cd uman
cargo install --path .
```

### Dependencies

`uman` delegates rendering to your system's man page renderer. Make sure one of these is installed:

| Tool | Platform | Install |
|------|----------|---------|
| `man-db` | Linux | `apt install man-db` / `pacman -S man-db` |
| `mandoc` | macOS, BSD | pre-installed / `brew install mandoc` |

`git` is required for cloning backends. `curl` is needed for HTTP-backed backends.

## Configuration

`uman` stores its data in two locations:

| Path | Purpose |
|------|---------|
| `~/.config/uman/config.json` | Backend registry and settings |
| `~/.uman/` | Backend data and SQLite index |

The config file is created automatically on first run with default backends. You can edit it to add custom backends:

```json
{
  "backends": {
    "linux-upstream": {
      "name": "linux-upstream",
      "source": "https://github.com/mkerrisk/man-pages",
      "format": "roff",
      "fetching": "git",
      "aliases": ["linux"]
    },
    "freebsd": {
      "name": "freebsd",
      "source": "https://gitlab.freebsd.org/freebsd/doc-manual.git",
      "format": "roff",
      "fetching": "git",
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
uman backend default              # show current default
uman backend default linux        # set by alias
uman backend default linux-upstream  # set by name
```

### Storage layout

```
~/.config/uman/
  config.json

~/.uman/
  backends/
    linux-upstream/    # raw git clone
    freebsd/
  index/
    uman.db            # SQLite db
```

## Usage

### Reading man pages

```bash
uman <backend> [<section>] <topic>     # explicit backend
uman <topic>                            # default backend
uman <section> <topic>                  # default backend with section
```

```bash
uman linux-upstream 2 execve           # full form
uman linux execve                      # alias, section auto-resolved
uman execve                            # default backend, section auto-resolved
uman 2 execve                          # default backend, explicit section
```

When section is omitted, `uman` resolves it automatically by looking up the lowest section number in the index.

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
uman backend list
```

Output:

```
NAME                 DEFAULT    STATUS     FORMAT SOURCE
linux-upstream       *          installed  roff   https://github.com/mkerrisk/man-pages
freebsd                         available  roff   https://gitlab.freebsd.org/freebsd/doc-manual.git
```

### Removing backends

```bash
uman remove <backend>
```

```bash
uman remove linux-upstream
```

Removes the backend data and its index entries. If the removed backend was the default, a warning is printed.

### Updating backends

```bash
uman update            # update all installed backends
uman update linux      # update a single backend (alias works too)
```

For git backends, this runs `git pull` and re-indexes changed pages. For curl backends, it re-downloads the archive.

### Setting the default backend

```bash
uman backend default              # show current default
uman backend default linux        # set by alias
uman backend default linux-upstream  # set by name
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

## Architecture

`uman` does not render man pages itself. It manages local copies of man page collections (backends) and delegates all rendering to your system's `man` or `mandoc` binary, setting `MANPATH` to the appropriate backend directory. Output goes directly to the terminal through the renderer's built-in pager.

Each backend is stored as a raw git clone. On read, `uman` resolves the backend directory and invokes the renderer with `MANPATH=<backend_dir>:` (trailing colon preserves system man paths for cross-references).

A SQLite index with FTS5 is maintained for fast search. Indexing happens automatically after install and update. Content hashing (SHA-256) drives incremental re-indexing — only changed pages are updated. The NAME section of each roff file is parsed to extract descriptions for keyword search.

```
install backend → git clone → index into SQLite
read page       → locate dir → exec man with MANPATH → pager
update backend  → git pull   → re-index changes
search          → query SQLite FTS5
```

## Command reference

| Command | Description |
|---------|-------------|
| `uman <backend> [<section>] <topic>` | Read a man page (backend name or alias) |
| `uman <topic>` | Read using default backend |
| `uman <section> <topic>` | Read with section using default backend |
| `uman install <backend>` | Install a backend |
| `uman remove <backend>` | Remove a backend |
| `uman update [<backend>]` | Update one or all backends |
| `uman search [-k] <topic>` | Search for man pages |
| `uman backend list` | List configured backends |
| `uman backend default` | Show default backend |
| `uman backend default <name>` | Set default backend |