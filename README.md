# Untitled 1

# `uman` — Universal Man Page Reader

## Overview

`uman` is a universal CLI tool for reading man pages across multiple operating systems.

It unifies access to:

- Linux man pages
- macOS man pages
- BSD man pages
- Other man page sources

without requiring a VM, container, or remote system.

Example:

```bash
uman linux-upstream 2 execve
```

This lets a macOS user read Linux syscall documentation locally.

---

# Goals

- Unified interface for multiple man page sources
- Offline support
- Easy backend installation
- Easy backend updates
- Cross-platform compatibility
- Zero custom rendering logic

---

# Basic Usage

```bash
uman <backend> <section> <topic>
```

Example:

```bash
uman linux-upstream 2 execve
```

Meaning:

- `backend` → documentation source (Linux, BSD, macOS, etc.)
- `section` → man section number
- `topic` → page name

---

# Backend Management

Install a backend:

```bash
uman install linux-upstream
```

List installed backends:

```bash
uman backend list
```

Remove a backend:

```bash
uman remove freebsd
```

---

# Updating

Update all backends:

```bash
uman update
```

Update a single backend:

```bash
uman update linux-upstream
```

---

# Planned Backends

- linux-upstream
- macos

---

# Search

```bash
uman search execve
(returns like linux-upstream execve \n freebsd execve etc. 
```

---

# Backend Format (the configuration per backend so we know where to pull the data, how to pull the data (via git or https or http) and what the format is for that man page

```json
{
  "name": "linux-upstream",
  "source": "https://github.com/mkerrisk/man-pages",
  "format": "roff"
  "fetching": "git"
}
```

---

# IMPLEMENTATION DETAILS

## Language Choice

- Use **Rust**

---

## Dependencies (do not worry about them being present or not for now, take them as present)
 - a man page renderer (man-db or mandoc)
 - git (for fetching option)

## Rust Stack

- CLI: `clap`
- Async: `tokio` + `reqwest`
- Serialization: `serde`
- Index/search: `sqlite`
---

## Rendering Strategy (CRITICAL)

`uman` does NOT render man pages but makes sure there is a renderer installed on the system and adapts it as its renderer.

All rendering is delegated to system tooling:

- `man-db` (Linux)
- `mandoc` (BSD/macOS)

---

## Execution Model

```bash
MANPATH=~/.uman/backends/linux-upstream uman 2 execve
```

or:

```bash
uman 2 execve
```

---

## Architecture

- `uman` → CLI + backend manager + resolver
- ` man-db / mandoc` → rendering engine
- backends → local man page datasets

---

## Storage Layout

```text
~/.config/uman/
├backends/
|── cache/
└── index/
```

---

## Backend Installation

Backends are installed via git or archives (configured by the json url):

```bash
uman install linux-upstream
```

Internally:

```bash
git clone https://github.com/mkerrisk/man-pages ~/.uman/backends/linux-upstream
```

---

## Search System

- full-text search
- per-backend indexing

---

## IMPLEMENTATION DETAIL (IMPORTANT ADDITION)

### Local Cache + SQLite Index Layer

`uman` maintains a **two-layer backend system**:

#### 1. Raw Source Layer (Source of Truth)

- downloaded man page files stored per backend (actual man pages to render)
- used for re-indexing

#### 2. SQLite Index Layer (Query Engine)

- parsed man pages stored in SQLite
- enriched with useful metadata such as backend, format, last-updated etc.
- used for fast search and lookup
- supports  FTS (full-text search)
- not used for the rendering, just the stdout info

#### Behavior:

- backends are downloaded once and cached permanently
- all man pages are parsed into SQLite for fast access
- updates re-fetch only changed backends
- re-indexing can be full or incremental
- content hashing is used to avoid unnecessary reprocessing (sha256)

#### Example flow:

```text
download backend → store raw files → parse → insert into SQLite → query via CLI
```

#### Update flow:

```text
update backend → re-download changes → re-parse backend → reinsert into SQLite
```

---

# Design Goals

- fast
- portable
- offline-first
- minimal dependencies
- extensible backend system
- reuse existing Unix tooling
- no custom man-page parsing
- conventional commands

