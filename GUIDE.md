# REST Client for Zed — Complete Guide

## Project structure

```
rest/
├── extension.toml          # Zed extension manifest
├── Cargo.toml              # Rust workspace
├── languages/
│   └── http/
│       ├── config.toml     # .rest / .http file associations
│       └── highlights.scm  # syntax highlighting
└── crates/
    └── rest-runner/        # CLI binary that executes requests
        ├── Cargo.toml
        └── src/
            ├── main.rs
            ├── parser.rs   # .rest file parser
            ├── executor.rs # HTTP client (reqwest)
            ├── env.rs      # environment variable loading
            └── output.rs   # colored terminal output
```

---

## 1. Prerequisites

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Follow the on-screen instructions (option 1 — default). When done, reopen your terminal or run:
```bash
source ~/.cargo/env
```
Verify the installation:
```bash
rustc --version   # e.g. rustc 1.78.0
cargo --version   # e.g. cargo 1.78.0
```

---

## 2. Build and install the CLI

The `rest-runner` CLI is the binary Zed will call to execute HTTP requests.

```bash
cd ~/Code/zed-plugin/rest
cargo install --path runner
```

This compiles the project and copies `rest-runner` to `~/.cargo/bin/`, which is already in your PATH.

Verify:
```bash
rest-runner --version
rest-runner --help
```

---

## 3. Load the extension in Zed (dev mode)

1. Open **Zed**
2. Go to **Extensions** (puzzle icon bottom-left, or `Cmd+Shift+X`)
3. Click **Install Dev Extension…**
4. Select the folder `~/Code/zed-plugin/rest`
5. Zed automatically downloads the TreeSitter grammar from GitHub and compiles the language support

From this point on, all `.rest` and `.http` files are recognized with syntax highlighting.

---

## 4. Add the "Run Request" task

Open (or create) `~/.config/zed/tasks.json` and add:

```json
[
  {
    "label": "REST: Run Request at Cursor",
    "command": "rest-runner",
    "args": ["$ZED_FILE", "--line", "$ZED_ROW"],
    "reveal": "always"
  }
]
```

### Optional keybinding

In `~/.config/zed/keymap.json` add:

```json
[
  {
    "context": "Editor",
    "bindings": {
      "ctrl-alt-r": "task::Spawn"
    }
  }
]
```

Or, to run the REST task directly without going through the menu:

```json
[
  {
    "context": "Editor && extension == rest || extension == http",
    "bindings": {
      "ctrl-alt-r": ["task::Spawn", { "task_name": "REST: Run Request at Cursor" }]
    }
  }
]
```

---

## 5. Test with a .rest file

Create a `test.rest` file in any folder and open it in Zed:

```http
### Get a user
GET https://jsonplaceholder.typicode.com/users/1

###

### Create a post
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "Hello",
  "body": "World",
  "userId": 1
}

###

### Test with variables
GET {{baseUrl}}/users/{{userId}}
Authorization: Bearer {{token}}
```

To run the request at the cursor position:
- Command palette (`Cmd+Shift+P`) → search `task: spawn` → select `REST: Run Request at Cursor`

The integrated terminal opens and shows:

```
HTTP/1.1 200 OK

content-type: application/json; charset=utf-8
...

{
  "id": 1,
  "name": "Leanne Graham",
  ...
}

────────────────────────────────────────────────────
200 OK  42ms  1234 bytes
```

---

## 6. Environment variables

Create a `.rest-client.env.json` file in the same folder as your `.rest` file:

```json
{
  "local": {
    "baseUrl": "http://localhost:3000",
    "userId": "1",
    "token": "my-local-token"
  },
  "prod": {
    "baseUrl": "https://api.example.com",
    "userId": "42",
    "token": "prod-secret-token"
  }
}
```

To use a specific environment, add the `--env` flag:

```json
"args": ["$ZED_FILE", "--line", "$ZED_ROW", "--env", "local"]
```

Or call it manually from the terminal:

```bash
rest-runner test.rest --line 15 --env prod
rest-runner test.rest --name "Create a post"
rest-runner test.rest --verbose   # also shows request headers
```

### Built-in variables

| Syntax | Result |
|---|---|
| `{{$guid}}` | Random UUID v4 |
| `{{$timestamp}}` | Unix timestamp in seconds |
| `{{$randomInt}}` | Random integer 0–999 |
| `{{$processEnv VAR}}` | Value of a system environment variable |

---

## 7. Release — publishing the extension

When the extension is stable and you want to make it available to all Zed users:

### 7.1 Prepare the repository

```bash
cd ~/Code/zed-plugin/rest
git init
git add .
git commit -m "feat: initial release"
```

Create a public repository on GitHub (e.g. `github.com/albz/zed-rest-client`) and push:

```bash
git remote add origin https://github.com/albz/zed-rest-client.git
git push -u origin main
```

Update `extension.toml` with your actual repository:

```toml
repository = "https://github.com/albz/zed-rest-client"
```

### 7.2 Fork the Zed registry

The Zed extension registry is a public GitHub repository:
```
https://github.com/zed-industries/extensions
```

Fork it, then clone your fork:

```bash
git clone https://github.com/YOUR_USERNAME/extensions
cd extensions
```

### 7.3 Add your extension

In the registry's `extensions.toml` file, add:

```toml
[rest-client]
submodule = "extensions/rest-client"
version = "0.1.0"
```

Add your repo as a submodule:

```bash
git submodule add https://github.com/albz/zed-rest-client extensions/rest-client
git add .
git commit -m "Add rest-client extension"
git push
```

### 7.4 Open a Pull Request

Go to your fork on GitHub and open a PR toward `zed-industries/extensions`. The Zed team typically reviews and approves PRs within a few days. Once merged, the extension appears in the Zed marketplace for all users.

---

## 8. Future updates

To update an already-published version:

1. Bump the version in `extension.toml` (e.g. `0.1.0` → `0.2.0`)
2. Commit and push your repository
3. Update the submodule in the registry fork and open a new PR

---

## Troubleshooting

| Problem | Solution |
|---|---|
| `rest-runner: command not found` | `~/.cargo/bin` is not in PATH — add `export PATH="$HOME/.cargo/bin:$PATH"` to your `.zshrc` |
| No syntax highlighting | Reload the dev extension: Extensions → right-click → Reload |
| TreeSitter grammar won't download | Check your connection. Zed downloads from GitHub on first load |
| SSL / certificate error | Try `rest-runner test.rest --verbose` in the terminal to see the full error |
| Request times out | Default is 30s. Pass `--timeout 120` or add it to your task args in `tasks.json` |
