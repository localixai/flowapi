# Where these files come from

Copied unchanged from `github.com/OAI/OpenAPI-Specification`, Apache-2.0 (`LICENSE` beside this
file): the test descriptions of the official schemas, which the OAI keeps beside each version.

| Here | Branch, path | Commit |
| --- | --- | --- |
| `v3.2/pass`, `v3.2/fail` | `v3.2-dev`, `tests/schema/pass`, `fail` | `6d86f6560681d0c461cd06b3faa80814aa120fda` (2026-11-12) |
| `v3.1/pass`, `v3.1/fail` | `v3.1-dev`, `tests/schema/pass`, `fail` | `642ee90575f60233302ad3e82f9de83e02996532` (2026-11-12) |
| `v3.0/pass` | `main`, `_archive_/schemas/v3.0/pass` | `4f86429f134f7e48b6351563549b4502fc69de52` (2026-10-18) |

`conformance/oai/run.mjs` holds flowapi to them (oracle O2): every `pass` description generates
with no error and its TypeScript type-checks. A `fail` description is invalid against the OAI
schema, often in ways no type depends on (a missing `info`), so its outcome is recorded, not judged.

To update: fetch the branches, read what changed, copy the folders over these, put the commits here.
