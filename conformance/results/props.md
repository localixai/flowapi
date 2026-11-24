# O5: random schemas, flowschema against the TypeScript compiler

Written by `cd conformance/props && cargo run --release`; do not edit. 600 schemas from a fixed seed (0 left out: a keyword flowschema does not check), 12 values each.

| Valid values | Accepted | Invalid values | Rejected | Diagnostics outside the cases |
| ---: | ---: | ---: | ---: | ---: |
| 3968 | 3968 | 3232 | 2587 | 0 |

## Unsound

None.
