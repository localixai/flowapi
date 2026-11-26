---
"@localix-ai/flowapi": minor
---

`flowapi generate <flowapi-sdk.yml> --out <dir> [--check]` runs through the package as the Rust
command runs it: the same files byte for byte, the same `.flowapi/lock.json`, the same messages
and exit codes, by the same rules (all or nothing; a generated file edited by hand is a conflict;
`src/lib/` and `examples/` kept; an unchanged file no longer generated deleted, a changed one kept
with a warning; a path out of `--out` or through a symbolic link refused; `--check` writes
nothing). The rules run in `flowapi-sdk.wasm`; the package reads the disk and makes the changes.
`loadSdk()` gives the two halves without a disk: `repoFiles` (the repository and its lock) and
`repoPlan` (what writing it changes, given what is on disk).
