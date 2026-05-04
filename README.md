# REST Client for Zed

A HTTP client extension for [Zed](https://zed.dev) that lets you write, run, and inspect HTTP requests directly inside the editor — no browser, no external app.

Inspired by the [VS Code REST Client](https://github.com/Huachao/vscode-restclient) extension.

---

## Features

- Syntax highlighting for `.rest` and `.http` files
- Multiple requests per file, separated by `###`
- Named requests (`### My Request` or `# @name myRequest`)
- Request headers and body (JSON, form, raw)
- Variable substitution with `{{variableName}}`
- Environment files for switching between `local`, `staging`, `prod`
- Built-in variables: `{{$guid}}`, `{{$timestamp}}`, `{{$randomInt}}`, `{{$processEnv VAR}}`
- **Response chaining** — use values from a previous response in the next request
- **Save to file** — write the response body (or full response) to a file with `--output`
- **Configurable timeout** — set per-request timeout with `--timeout`
- Colored, pretty-printed output in the integrated terminal

---

## Installation

### 1. Install the Zed extension

Open Zed → **Extensions** (`Cmd+Shift+X`) → search for **REST Client** → **Install**.

### 2. Install the CLI runner

The extension provides syntax highlighting out of the box. To **execute** requests you also need the `rest-runner` CLI:

```bash
cd rest
cargo install --path runner
```

> Don't have Rust? Install it first: https://rustup.rs

### 3. Register the run task

Add the following to `~/.config/zed/tasks.json` (create the file if it doesn't exist):

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

### 4. (Optional) Add a keyboard shortcut

In `~/.config/zed/keymap.json`:

```json
[
  {
    "context": "Editor",
    "bindings": {
      "ctrl-alt-r": ["task::Spawn", { "task_name": "REST: Run Request at Cursor" }]
    }
  }
]
```

---

## Usage

### Writing requests

Create a file with a `.rest` or `.http` extension and write your requests in plain HTTP format:

```http
GET https://api.example.com/users
```

Separate multiple requests with `###`:

```http
GET https://api.example.com/users/1

###

POST https://api.example.com/users
Content-Type: application/json

{
  "name": "Jane",
  "email": "jane@example.com"
}
```

### Running a request

Place your cursor anywhere inside the request you want to run, then:

- **Command palette** (`Cmd+Shift+P`) → `task: spawn` → `REST: Run Request at Cursor`
- Or use your keyboard shortcut if you configured one

The integrated terminal opens and shows the full response:

```
HTTP/1.1 200 OK

content-type: application/json; charset=utf-8
x-ratelimit-limit: 100

{
  "id": 1,
  "name": "Jane",
  "email": "jane@example.com"
}

────────────────────────────────────────────────────
200 OK  84ms  312 bytes
```

---

## Request format

### Method and URL

```http
GET     https://api.example.com/resource
POST    https://api.example.com/resource
PUT     https://api.example.com/resource/1
PATCH   https://api.example.com/resource/1
DELETE  https://api.example.com/resource/1
HEAD    https://api.example.com/resource
```

### Headers

Add headers on the lines immediately after the request line:

```http
POST https://api.example.com/login
Content-Type: application/json
Accept: application/json
Authorization: Bearer my-token
```

### Body

Leave one blank line after the headers, then write the body:

```http
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "Jane",
  "role": "admin"
}
```

For `application/x-www-form-urlencoded`:

```http
POST https://api.example.com/auth
Content-Type: application/x-www-form-urlencoded

grant_type=password&username=jane&password=secret
```

### Request names

Name a request using the separator comment or a `# @name` annotation:

```http
### Create user
POST https://api.example.com/users
Content-Type: application/json

{ "name": "Jane" }

###

# @name deleteUser
DELETE https://api.example.com/users/1
```

Run a named request from the terminal:

```bash
rest-runner api.rest --name "Create user"
rest-runner api.rest --name deleteUser
```

### Comments

```http
# This is a comment
// This is also a comment

GET https://api.example.com/users
```

---

## Variables

### User-defined variables

Use `{{variableName}}` anywhere in the URL, headers, or body:

```http
GET {{baseUrl}}/users/{{userId}}
Authorization: Bearer {{token}}
```

### Built-in variables

| Variable | Description |
|---|---|
| `{{$guid}}` | A random UUID v4 |
| `{{$timestamp}}` | Current Unix timestamp (seconds) |
| `{{$randomInt}}` | A random integer between 0 and 999 |
| `{{$processEnv VAR}}` | Value of the `VAR` environment variable |

### Response chaining

After running a **named** request, its response is cached locally. Subsequent requests can read from that cache using:

| Syntax | Returns |
|---|---|
| `{{requestName.response.status}}` | HTTP status code (e.g. `201`) |
| `{{requestName.response.headers.content-type}}` | Value of a response header |
| `{{requestName.response.body.$.id}}` | A field from the JSON body |
| `{{requestName.response.body.$.items[0].name}}` | Array element field |

**Example — create a user, then immediately delete it:**

```http
### Create user
POST https://api.example.com/users
Content-Type: application/json

{ "name": "Jane" }

###

### Delete the user just created
DELETE https://api.example.com/users/{{Create user.response.body.$.id}}
```

> The cache is stored in `~/.cache/rest-runner/` and persists between runs, so you can also reference responses across different terminal sessions.

---

### Environment files

Create a `.rest-client.env.json` file in the same directory as your `.rest` file:

```json
{
  "local": {
    "baseUrl": "http://localhost:3000",
    "token": "dev-token-123",
    "userId": "1"
  },
  "staging": {
    "baseUrl": "https://staging.api.example.com",
    "token": "staging-token-456",
    "userId": "42"
  },
  "prod": {
    "baseUrl": "https://api.example.com",
    "token": "prod-token-789",
    "userId": "42"
  }
}
```

By default, the `local` environment is used. To switch environment, edit your task args:

```json
"args": ["$ZED_FILE", "--line", "$ZED_ROW", "--env", "staging"]
```

Or call the CLI directly:

```bash
rest-runner api.rest --line 12 --env prod
```

---

## CLI reference

```
rest-runner <FILE> [OPTIONS]

Arguments:
  <FILE>  Path to the .rest or .http file

Options:
  -l, --line <LINE>          Line number of the request to run (1-based)
  -n, --name <NAME>          Name of the request to run
  -e, --env <ENV>            Environment name (from .rest-client.env.json)
      --env-file <ENV_FILE>  Path to a custom env file
  -t, --timeout <SECS>       Request timeout in seconds [default: 30]
  -o, --output <FILE>        Save the response body to a file
      --output-headers       Include status line and headers in the output file
  -v, --verbose              Print request headers before sending
  -h, --help                 Print help
  -V, --version              Print version
```

---

## Example file

```http
# .rest-client.env.json should be in the same folder:
# {
#   "local": { "baseUrl": "https://jsonplaceholder.typicode.com" }
# }

### Get all posts
GET {{baseUrl}}/posts

###

### Get single post
GET {{baseUrl}}/posts/1
Accept: application/json

###

### Create a post
POST {{baseUrl}}/posts
Content-Type: application/json

{
  "title": "Hello from Zed",
  "body": "REST Client extension",
  "userId": 1
}

###

### Update a post
PUT {{baseUrl}}/posts/1
Content-Type: application/json

{
  "id": 1,
  "title": "Updated title",
  "body": "Updated body",
  "userId": 1
}

###

### Delete a post
DELETE {{baseUrl}}/posts/1
```

---

## Troubleshooting

**`rest-runner: command not found`**
`~/.cargo/bin` is not in your PATH. Add this to your `~/.zshrc` or `~/.bashrc`:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

**No syntax highlighting**
Reload the extension: Zed → Extensions → right-click REST Client → Reload.

**Request times out**
The default timeout is 30 seconds. Pass `--timeout 120` for slower endpoints, or add it to your task args in `tasks.json`.

**Response body is not pretty-printed**
The server must return `Content-Type: application/json`. If it doesn't, the raw body is shown as-is.

---

## License

MIT
