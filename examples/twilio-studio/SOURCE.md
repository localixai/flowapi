# Twilio Studio v2: where the example comes from

Not generated: the one file of this folder beside `flowapi-sdk.yml` and `openapi.yaml` that
flowapi does not write.

- **The description**, `openapi.yaml`: Twilio's own description of Studio v2 (`twilio/twilio-oai`,
  `spec/json/twilio_studio_v2.json`, version 1.55.0), as APIs-guru's openapi-directory keeps it,
  converted to YAML: `APIs/twilio.com/twilio_studio_v2/1.55.0/openapi.yaml` at commit
  `f04b8d0bcd39c52e1cf3ad7a5fe744709832ae49`, the commit `bench/corpus/corpus.tsv` pins for
  APIs-guru. sha256 `a07670d586313484ee0524ed14b45a1dd1c9b54bef53e6951e68d2f651b6e805`,
  48,061 bytes, kept as it is.
- **Its licence**: Apache License 2.0, as the description's `info.license` says; the text is
  `LICENSE` in this folder (the SDK is licensed the same way, `package.license: Apache-2.0`).
  APIs-guru's collection itself is CC0 1.0.
- **The config**, `flowapi-sdk.yml`: `flowapi config draft openapi.yaml`, then refined by hand;
  its first comment says what changed and why. `flowapi config check` finds no issue.
- **Everything else** is written by `flowapi generate flowapi-sdk.yml --out .` and listed in
  `.flowapi/lock.json`; `cargo test -p flowapi-cli --test examples` holds it to what `generate`
  writes, byte for byte.
