# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**fmq** (frontmatter query) - A Rust CLI that helps AI agents programmatically manage frontmatter properties of markdown files.

### Design

1. Stream input file to extract the frontmatter section
2. Delegate manipulation to `yq` or equivalent library with familiar syntax
3. Output query results

The implementation should be simple Rust focused on frontmatter parsing; complex querying is delegated to `yq`-style tooling.

## Development Commands

```bash
# Build
cargo build

# Run tests
cargo test

# Run a single test
cargo test <test_name>

# Run with release optimizations
cargo build --release

# Format code
cargo fmt

# Lint
cargo clippy
```

## Task Tracking

Use `bd` (Beads) for issue tracking instead of external tools. Issues live in `.beads/` directory.
