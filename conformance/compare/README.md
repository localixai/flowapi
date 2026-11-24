# O3 for the other generators

The suite oracle (`conformance/suite`, O3) asks the TypeScript compiler whether each case of the
JSON-Schema-Test-Suite fits the type flowapi generates. This folder asks the same of the other
generators pinned in `bench/competitors` (openapi-typescript and hey-api), so the three can be
compared on the same cases, under the same compiler and flags.

## Running it

    node conformance/compare/compare.mjs     # from the repository's root, a few seconds

It needs `conformance/ts` (`npm ci` there once, the compiler O3 uses) and `bench/competitors`
(`npm ci` there, the tools), and adds no dependency. flowapi's figures are read from
`conformance/results/suite.json`, so run the suite oracle first when flowapi changed. It writes:

- `results/compare.md`: per tool, the groups run, generation errors, invalid-TypeScript groups,
  soundness and precision; soundness again on the groups every tool compiled; soundness per
  keyword file; then each tool's unsound cases (the first 50), generation errors and
  invalid-TypeScript groups.
- `results/compare.json`: the same, with counts per keyword file and tool.
- `compare/out/<tool>/`: the documents given to the tool, its raw results and the checked `.ts`
  files, ignored by git. To see why a case failed, open its file.

Both results are sorted and hold no time and no path of the machine: the same tools give the
same bytes.

## How it works

1. `load.mjs` reads the suite as `conformance/suite/src/load.rs` does: the files sorted by name,
   a group run when its `compatibility` admits 2020-12 and it is not tagged `format-assertion`
   (the same algorithm, term by term). Each case's data is kept as the literal the Rust harness
   prints: one line, keys in written order, numbers as written.
2. `wrap.mjs` puts each group into an OpenAPI 3.1 document with
   `"jsonSchemaDialect": "https://json-schema.org/draft/2020-12/schema"`: the schema is
   `components.schemas.Root`, each of its `externalSchemas` is `components.schemas.Remote<i>`.
   Every `$ref` and `$dynamicRef` in a schema position is resolved against its base URI (the
   harness's URI for the group, or a remote's retrieval URI, changed by each `$id` on the way);
   when it lands on one of those documents, on a resource an `$id` embeds in them, or on an
   `$anchor` or `$dynamicAnchor`, it is rewritten to the local pointer of that schema
   (`#` becomes `#/components/schemas/Root`, `#/$defs/a` becomes
   `#/components/schemas/Root/$defs/a`, `http://localhost:1234/tree.json` becomes
   `#/components/schemas/Remote0`). A reference that lands on nothing known is left as written.
   Values of `enum`, `const`, `default` and unknown keywords are not schemas and are not touched.
3. `generate.mjs` runs one tool over every document in one Node process (one process per tool,
   one after the other), through its programmatic API, with its defaults:
   - openapi-typescript: `openapiTS(document)` then `astToString`, followed by
     `type Root = components["schemas"]["Root"];`.
   - hey-api: `createClient` with the `@hey-api/typescript` plugin alone into a scratch folder,
     logs silent and no crash-log file; its `types.gen.ts` exports `Root` itself.

   A tool that throws, writes no file, does not export `Root`, or takes over 30 s is a
   generation error for that group.
4. After the types come the harness's case lines, `const c<i>: Root = <data>;`. `tsc.mjs` runs
   TypeScript 7 once per tool over its files with the harness's flags (`--ignoreConfig --strict
   --exactOptionalPropertyTypes --noEmit --target es2022 --skipLibCheck`) and maps each
   diagnostic to its case by file and line: a diagnostic above the first case makes the group
   invalid TypeScript; a valid case with a diagnostic is unsound; an invalid case with one is
   rejected.

## Caveats

- **The wrapping is ours.** flowapi reads each group as a JSON Schema document; the others take
  OpenAPI, so the group is moved into `components.schemas` and its references rewritten. The
  rewrite follows `$id` and anchors statically, which is right for `$ref`; for `$dynamicRef` it
  is the static target, as a tool without dynamic scope would read it. `$id`, `$anchor` and
  `$defs` stay where the schema had them. A tool that would do better with `$defs` hoisted into
  `components.schemas` is measured on the schema as written: openapi-typescript prints a
  schema's `$defs` as a required property of its object type, which accounts for about a third
  of its unsound cases, and with a `$ref` beside `$defs` its output does not compile (TS2502).
- **Defaults only.** Each tool runs with its defaults, as a user who installs it gets it.
  openapi-typescript's defaults that move the figures: `additionalProperties: false` (an object
  schema without `additionalProperties` gets no index signature, so a literal with another
  property is rejected), `emptyObjectsUnknown: false` (`{"type": "object"}` alone is
  `Record<string, never>`), `defaultNonNullable: true` (a property with a `default` becomes
  required). hey-api has no option for the first two; its objects are closed the same way.
- **Different denominators.** A group a tool cannot generate, or whose types do not compile, is
  left out of that tool's counts, as in the harness. Soundness on the groups every tool compiled
  is printed beside it so the shares can be compared on the same cases; precision is not, as
  `suite.json` keeps flowapi's rejections per file, not per group.
- **flowapi is not run here.** Its row is `conformance/results/suite.json` as last written by the
  suite oracle, which wraps nothing.
- The compiler is TypeScript 7 (`conformance/ts`), not the TypeScript the tools themselves
  depend on; the generated files use nothing newer than what both accept.
