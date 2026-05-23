# `uniman` — Universal Man Pages

`uniman` read man pages of any operating system on any unix machine, natively.

```bash
uniman list
uniman install linux-upstream
uniman linux execve
```

## Dependencies

`uniman` delegates rendering to your system's man page renderer. Make sure you have either `man-db` or `mandoc` on your system.

`git` is required for cloning backends. `curl` is needed for HTTP-backed backends.

`fzf` is required for the interactive search and listing menus.


## Installation

### From Source

```bash
git clone https://github.com/your-org/uniman.git
cd uniman
cargo install --path .
```

## Configuration

`uniman` stores its data in two locations:

| Path | Purpose |
|------|---------|
| `~/.config/uniman/config.json` | Backend registry and settings |
| `~/.uniman/` | Backend data and SQLite index |

The config file is created automatically on first run with default backends. You can edit it to add custom man page backends.

To print the config path:

```bash
uniman config
```

## Default Backend

`uniman` sets a default backend for each OS automatically on first run:

to list all backends and the default backend, run:

```bash
uniman list
```

to change the default backend, run:

```bash
uniman default <backend-name>
```

