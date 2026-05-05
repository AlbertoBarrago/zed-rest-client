# Contributing to REST Client for Zed

Thank you for considering a contribution! This project is split into two independent parts — please read the section that applies to your change.

## Project structure

```
.
├── extension.toml          # Zed extension manifest
├── languages/http/         # Syntax highlighting, runnables, tasks
├── runner/                 # Rust CLI that executes HTTP requests
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── parser.rs
│       ├── executor.rs
│       ├── output.rs
│       ├── env.rs
│       ├── cache.rs
│       └── jsonpath.rs
├── README.md
└── CONTRIBUTING.md         # this file
```

## Which part are you changing?

### 1. The Zed extension (syntax, tasks, highlights)

Files: `extension.toml`, `languages/**/*`

- Install the extension as a **dev extension** in Zed:
  1. Open Zed → Extensions (`Cmd+Shift+X`)
  2. Click **Install Dev Extension…**
  3. Select this repository root
- After editing `languages/http/*.scm` or `extension.toml`, reload the dev extension:
  - Extensions panel → right-click **REST Client** → **Reload**
- There is no manual build step; Zed compiles the Tree-sitter grammar automatically on load.

### 2. The `rest-runner` CLI

Files: `runner/**/*`

- Rust toolchain (via rustup) is required.
- All Cargo commands run from the `runner/` directory:

```bash
cd runner

# Check / lint
cargo check
cargo clippy

# Build
cargo build --release

# Install locally (required for Zed to pick up changes)
cargo install --path .

# Manual test
cargo run -- ../test.rest --method GET --url https://httpbin.org/get
```

## Submitting changes

1. **Open an issue first** for bug reports or feature proposals so we can discuss the approach.
2. **Fork the repository** and create a feature branch.
3. **Keep commits atomic** — one logical change per commit.
4. **Update documentation** if your change affects usage (`README.md` or inline `--help` text).
5. **Ensure the CLI builds cleanly:**
   ```bash
   cd runner
   cargo check
   cargo clippy
   ```
6. **Open a Pull Request** with a clear description of the problem and the solution.

## Code style

- Rust: follow `cargo fmt` and keep `cargo clippy` warnings-free.
- Tree-sitter queries: keep captures consistent with existing `highlights.scm` and `runnables.scm`.

## Reporting bugs

Please include:
- Zed version (`zed: copy system specs to clipboard`)
- `rest-runner --version`
- A minimal `.rest` file that reproduces the issue
- The exact command or UI action you used (gutter button, task override, CLI)
- Expected vs actual output

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
