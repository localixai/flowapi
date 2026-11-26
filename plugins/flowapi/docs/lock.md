# `.flowapi/lock.json`

The state of a generated SDK repository, written by `flowapi generate` alone
(`crates/flowapi-sdk/src/repo/lock.rs` in this repository). The plugin reads it in two places: the guard
(`lib/guard.mjs`), to know which files are generated, and the judge (`lib/judge.mjs`), for the
flowapi version that regenerates the repository. Nothing in the plugin writes it.

This page is the shape the plugin relies on. flowapi's `generate` defines the file; when the two
disagree, flowapi is right and this page and the plugin are changed.

## Where it is

At `<repo>/.flowapi/lock.json`, next to `<repo>/flowapi-sdk.yml`. The repository's root is
the nearest directory above a file that holds `.flowapi/lock.json`; every path in the lock is
relative to it.

## Fields

```json
{
  "version": 1,
  "flowapi": "0.3.0",
  "spec_sha256": "3b1f…",
  "config_sha256": "9a0c…",
  "configured_endpoints": 42,
  "files": {
    "package.json": "5e88…",
    "src/client.ts": "c1d2…",
    "src/resources/projects.ts": "77ab…",
    "examples/basic.ts": "0f3e…"
  },
  "names": {
    "GET /v1/projects/{project_id}/members": "projects.members.list"
  }
}
```

| Field | Type | What it is |
| --- | --- | --- |
| `version` | integer | The lock's format version: `1` |
| `flowapi` | string | The exact flowapi version that wrote the repository; the judge regenerates with it |
| `spec_sha256` | string | SHA-256 (hex) of the OpenAPI description's bytes as read |
| `config_sha256` | string | SHA-256 (hex) of `flowapi-sdk.yml`'s bytes |
| `configured_endpoints` | integer | The number of operations the config maps to methods |
| `files` | object | Every file `generate` wrote: repo-relative path → SHA-256 (hex) of its bytes |
| `names` | object | Names pinned when two would collide: the operation (`"METHOD /path"` or `operationId`) → the name it keeps |

Paths in `files` use `/` on every platform, never start with `/` or `./`, and never contain `..`.
`flowapi-sdk.yml` and the lock itself are not in `files`.

## What the guard does with it

On Claude Code's Edit, Write, MultiEdit and NotebookEdit, and on each path of an Agent Plugins core
client's `file.modify` operations (an edit, a write, a delete, and both ends of a move):

- A target named `.flowapi/lock.json` is refused, wherever it is.
- A target under a directory that holds the lock, whose repo-relative path is a key of `files` and
  is not under `src/lib/` or `examples/` (where the person's own code lives), is refused with the hint to
  change `flowapi-sdk.yml` and generate again.
- Everything else goes through. A lock that cannot be read or is not JSON, or whose `files` is not
  an object, refuses nothing: a broken lock must not block the person's work.

On Windows and macOS the path is compared without case, as their file systems do. The guard reads
the paths the tool call names; a write through a shell command or a symbolic link is not seen.
`generate` covers those: a file whose bytes no longer match its hash in `files` is not overwritten
but reported as a conflict.
