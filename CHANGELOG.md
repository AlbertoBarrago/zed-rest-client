# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2026-07-21

### Added

- File-level variable declarations (`@name = value`) with substitution in request URLs, headers, and bodies.
- File-level variables override values with the same name from `.rest-client.env.json`.
- Nested variable resolution, including values captured from cached named responses.

## [0.1.1] - 2026-05-05

### Fixed

- Corrected the extension author email and aligned the extension and runner versions.

## [0.1.0] - 2026-05-04

### Added

- Syntax highlighting for `.rest` and `.http` files via Tree-sitter grammar (`rest-nvim/tree-sitter-http`).
- Gutter run buttons and runnable tasks bound to HTTP request lines.
- `rest-runner` CLI for executing requests: supports `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`.
- Request parsing: headers, JSON/form/raw body, multiple requests per file separated by `###`.
- Named requests (`### Name` or `# @name name`) for targeted execution.
- Variable substitution with `{{variableName}}` in URL, headers, and body.
- Built-in variables: `{{$guid}}`, `{{$timestamp}}`, `{{$randomInt}}`, `{{$processEnv VAR}}`.
- Environment file support (`.rest-client.env.json`) for switching between `local`, `staging`, `prod`.
- Response chaining: reference cached response values (`status`, `headers`, `body.$...`) in subsequent requests.
- Coloured terminal output with pretty-printed JSON and a summary bar.
- `--output` flag to save response body (or full response with `--output-headers`) to a file.
- Configurable per-request timeout via `--timeout`.
- Verbose mode (`--verbose`) to print request headers before sending.

[unreleased]: https://github.com/AlbertoBarrago/zed-rest-client/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/AlbertoBarrago/zed-rest-client/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/AlbertoBarrago/zed-rest-client/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/AlbertoBarrago/zed-rest-client/releases/tag/v0.1.0
