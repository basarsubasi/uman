# `uniman` — Universal Man Pages

Read man pages of any operating system on any unix machine, natively.

https://github.com/user-attachments/assets/f292953a-0d72-4a23-b896-9bb3f6da32a8

## Dependencies


`git` is required for cloning backends. `curl` is needed for HTTP-backed backends.

`fzf` is required for the interactive search and listing menus. (optional)

`man-db` or `mandoc` is required for the rendering of the man pages (optional)

## Installation

### From crates.io

```bash
cargo install uniman
```

### From Source

```bash
git clone https://github.com/basarsubasi/uniman.git
cd uniman
cargo build --release 
```
(move the binary from target/release/uniman to somewhere in path)

## Configuration

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

