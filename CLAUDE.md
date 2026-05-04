# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

A Zed editor extension that adds `.rest` / `.http` file support. It has two independent parts:

- **The Zed extension** (root) — static, no WASM. Registers the `HTTP` language via `extension.toml`, points to the `rest-nvim/tree-sitter-http` grammar, and provides `languages/http/` config and highlights.
- **`runner/`** — a standalone Rust binary (`rest-runner`) that Zed calls via a task to execute requests and print responses to the integrated terminal.

The two parts are intentionally decoupled: the extension only handles syntax; the binary handles execution. There is no WASM extension code.

## CLI commands

All Cargo commands run from `runner/`:

```bash
# Build
cargo build

# Build + install to ~/.cargo/bin (required for the Zed task to work)
cargo install --path .

# Run directly (useful for manual testing)
cargo run -- ../../test.rest --method GET --url https://example.com
cargo run -- ../../test.rest --name "Get a single user" --verbose
cargo run -- ../../test.rest --env prod --timeout 60

# Check without building
cargo check
cargo clippy
```

## Architecture of `runner/`

Request execution flows through these modules in order:

```
main.rs  →  parser.rs  →  executor.rs  →  output.rs
               ↓                              ↑
            env.rs (vars)              cache.rs (response chaining)
            jsonpath.rs (chaining resolution)
```

**`parser.rs`** — Splits the file on `###` boundaries into sections, then parses each section into a `Request` struct. Zed runnable queries pass the captured method and URL to the runner, which selects the matching parsed request. Variable substitution (`{{expr}}`) happens after selection; it calls into `cache` and `jsonpath` to resolve chained expressions.

**`executor.rs`** — Builds a `reqwest::blocking` request from the parsed `Request` and fires it. Auto-sets `Content-Type: application/json` if the body starts with `{` and no content-type header is present.

**`cache.rs`** — Persists named-request responses to `~/.cache/rest-runner/<name>.json`. This is what enables response chaining across separate CLI invocations.

**`jsonpath.rs`** — Minimal dot-notation resolver (`$.field`, `$.a.b`, `$.arr[0].field`). Not a full JSONPath implementation — only what's needed for chaining.

**`output.rs`** — Prints the coloured request header (`▶ METHOD url`) before execution and the response (status, headers, pretty-printed body, summary bar) after. Also writes to file when `--output` is passed.

## Key design decisions

- `grammars/` is gitignored — Zed generates it on first load by cloning the tree-sitter grammar repo. Never commit it.
- `runner/Cargo.lock` is committed (binary crate convention).
- The Zed task is bound to the `rest-request` runnable tag and passes `$ZED_CUSTOM_METHOD` / `$ZED_CUSTOM_URL`; it does not depend on cursor row detection.

## Zed extension loading

To reload the extension after changes to `languages/` or `extension.toml`:
- Zed → Extensions → right-click REST Client → Reload

After changes to `runner/` source:
```bash
cargo install --path runner   # run from repo root
```
No Zed reload needed — the task spawns a fresh process each time.
