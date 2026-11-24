# Where this suite comes from

Copied unchanged from `github.com/json-schema-org/JSON-Schema-Test-Suite`, at commit
`e2a68a15cd490750fd6b95e604e4054d7d978d1f` (2026-11-16), MIT (`LICENSE` beside this file).
`tests/` holds the top-level files of the suite's `validation/tests/`, one per keyword. Not taken
for now: `validation/tests/optional/`, `validation/tests/proposals/` and `annotations/`.

The suite is unified across dialects: each test case says in `compatibility` which drafts it
applies to (the grammar is in the suite's `validation/README.md`), and the oracle keeps those that
admit 2020-12. There is no `remotes/` directory at this commit: the schemas a case references are
in the case itself, under `externalSchemas`, keyed by their retrieval URIs.

Nothing here is ours: change nothing in place, so an update stays a plain copy.

To update: clone the repository, read what changed since the commit above, copy the top-level
`validation/tests/*.json` and `LICENSE` over these, put the new commit here, and run the oracle
(`conformance/README.md`).

    git clone --depth 1 https://github.com/json-schema-org/JSON-Schema-Test-Suite
    git -C JSON-Schema-Test-Suite log -1 --format='%H %cs'
