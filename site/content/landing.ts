import type { Block, Card, LandingContent, Line, Media } from '@/shared/config';

// The landing's words. The design is the template's; this file is the site's. Every figure is
// the latest run's, in the repository's conformance/results and bench/results (README.md here
// says which file holds which): change one there first, then here. The two pictures are copies
// of the repository's assets/, which bench/ draws.
const DOCS = '/docs';
const GITHUB = 'https://github.com/localixai/flowapi';
const NPM = 'https://www.npmjs.com/package/@localix-ai/flowapi';

const terminal = (blocks: Block[], label: string): Media => ({ kind: 'terminal', blocks, label });
const ui = (...lines: Line[]): Block => ({ ui: lines });
// A line with nothing on it: a space, since a row with no text has no height.
const GAP: Line = [['', ' ']];

const CARDS: Card[] = [
  {
    tag: 'Output',
    title: 'Types you would have written',
    text: 'One type for each component schema and each operation, and `Paths` for the whole API. A schema’s description sits above its member, and the keywords TypeScript cannot state (`pattern`, `minimum`) are named in a comment over the type.',
    links: [{ href: `${DOCS}/getting-started`, label: 'Getting started' }],
    media: {
      kind: 'image',
      src: '/output-dark.png',
      width: 1906,
      height: 1282,
      alt: 'An OpenAPI description beside the TypeScript flowapi prints for it: Pet with its described id, the kind as a union of literals, tags as a non-empty tuple, owner as a string or null, x- extensions as a template literal index signature; GetPet’s path parameter and 200 response; and Paths',
    },
  },
  {
    tag: 'Conformance',
    title: 'Every value the schema accepts, the type accepts',
    text: 'Each schema of the official JSON Schema Test Suite becomes a type, and each test value is handed to the TypeScript compiler as a literal of it. All 779 valid values type-check. 373 of the 540 invalid ones are refused.',
    links: [{ href: `${DOCS}/conformance`, label: 'How it is checked' }],
    media: terminal(
      [
        {
          grid: [
            [['dim', 'JSON Schema Test Suite'], ['dim', 'valid in'], ['dim', 'invalid out'], ['dim', 'broken']],
            [['hi', 'flowapi 0.1.0'], ['hi', '100.0%'], ['hi', '69.0%'], ['hi', '0']],
            ['openapi-typescript 7.13', '90.6%', '51.4%', '56'],
            ['@hey-api/openapi-ts 0.99', '94.6%', '42.2%', '7'],
          ],
          align: 'lrrr',
        },
        ui(
          GAP,
          [['dim', 'Each schema as an OpenAPI 3.1 component, each tool with its defaults.']],
          [['dim', 'Broken: groups that failed to generate or printed invalid TypeScript.']],
        ),
      ],
      'On the JSON Schema Test Suite, flowapi accepts 100.0% of valid values and rejects 69.0% of invalid ones, with no broken group; openapi-typescript 90.6% and 51.4% with 56 broken groups; hey-api 94.6% and 42.2% with 7',
    ),
  },
  {
    tag: 'Speed',
    title: 'Cloudflare’s 30 MB description in under half a second',
    text: 'Each tool ran as a fresh process on real descriptions, five rounds, the median kept. On Cloudflare’s, flowapi peaks at 140 MB of memory where the other two take 937 MB and 1027 MB, and its output type-checks for all six descriptions.',
    links: [{ href: `${DOCS}/speed`, label: 'The benchmark' }],
    media: terminal(
      [
        {
          grid: [
            [['dim', 'description'], ['dim', 'flowapi'], ['dim', 'openapi-typescript'], ['dim', 'hey-api']],
            ['Slack, 999 KB', ['hi', '14 ms'], '892 ms', '992 ms'],
            ['Stripe, 8.3 MB', ['hi', '114 ms'], '2.44 s', '10.57 s'],
            ['GitHub, 14.3 MB', ['hi', '185 ms'], '3.73 s', '4.49 s'],
            ['Cloudflare, 29.8 MB', ['hi', '435 ms'], '7.96 s', '12.15 s'],
          ],
          align: 'lrrr',
        },
        ui(GAP, [['dim', 'The median of five rounds on an Intel N150.']]),
      ],
      'Time to generate: Slack 14 milliseconds against 892 and 992; Stripe 114 milliseconds against 2.44 and 10.57 seconds; GitHub 185 milliseconds against 3.73 and 4.49 seconds; Cloudflare 435 milliseconds against 7.96 and 12.15 seconds',
    ),
  },
  {
    tag: 'Beside the others',
    title: 'The whole comparison in one picture',
    text: 'Correct types, broken output, time and memory, each panel on its own scale, from the same runs as the two cards before. The tools compared and their versions are pinned in the repository, so anyone can run it again.',
    links: [{ href: `${GITHUB}/tree/main/bench`, label: 'The benchmarks on GitHub' }],
    media: {
      kind: 'image',
      src: '/compare-dark.png',
      width: 1760,
      height: 1290,
      alt: 'flowapi beside openapi-typescript and hey-api: valid values accepted on the official JSON Schema Test Suite (100% against 90.6% and 94.6%), invalid values rejected (69.0% against 51.4% and 42.2%), broken output (0 against 56 and 7 groups), and the time to generate Slack’s, Stripe’s, GitHub’s and Cloudflare’s descriptions, peak memory on Cloudflare’s and how many outputs type-check',
    },
  },
  {
    tag: 'SDK',
    title: 'Requests written, nothing sent',
    text: '`--sdk` adds two functions per operation: one writes the request by its `style` and `explode`, one reads the answer by status and `Content-Type`. Any HTTP client sends it, or none in a test. The output imports nothing.',
    links: [{ href: `${DOCS}/sdk`, label: 'The SDK' }],
    media: terminal(
      [
        ui(
          [['dim', 'getPetRequest({ path: { id: 7 }, query: { tags: ["a b", "c"] } })']],
          [['hi', '{ method: "GET", path: "/pets/7",']],
          [['hi', '  query: "tags=a%20b&tags=c", headers: {} }']],
          GAP,
          [['dim', 'getPetResponse(res.status, headers, await res.text())']],
          [['hi', '{ response: "200", status: 200,']],
          [['hi', '  mediaType: "application/json", body: Pet }']],
        ),
      ],
      'getPetRequest writes a GET to /pets/7 with the query tags=a%20b&tags=c; getPetResponse reads a 200 JSON answer into a Pet',
    ),
  },
  {
    tag: 'Two targets',
    title: 'TypeScript or Rust, from Rust or npm',
    text: 'The Rust target prints serde types and an SDK of its own, sound on the same suite. The npm package is the generator compiled to WebAssembly, with no runtime dependency, and prints the Rust command’s output byte for byte.',
    links: [
      { href: `${DOCS}/getting-started`, label: 'Getting started' },
      { href: NPM, label: 'On npm' },
    ],
    media: terminal(
      [
        {
          grid: [
            [['dim', 'oracle'], ['dim', 'result']],
            ['suite, TypeScript: valid in', ['hi', '779 of 779']],
            ['suite, Rust (serde): valid in', ['hi', '787 of 787']],
            ['random schemas: valid in', ['hi', '4,319 of 4,319']],
            ['OAI examples: type-check', ['hi', '78 of 78']],
            ['SDK style examples: differ', ['hi', '0 of 51']],
          ],
          align: 'lr',
        },
      ],
      'The oracles: 779 of 779 valid suite values for TypeScript, 787 of 787 for Rust, 4,319 of 4,319 valid values of random schemas, 78 of 78 OAI example descriptions type-check, and none of the 51 style examples differ',
    ),
  },
];

const landing: LandingContent = {
  hero: {
    title: { chip: 'Types', rest: 'that follow the specification' },
    lede: 'Swagger 2.0, OpenAPI 3.0 to 3.2 and JSON Schema in, TypeScript or Rust out. Every value a schema accepts, its type accepts, checked on the official JSON Schema Test Suite.',
    under: 'A Rust generator, also on npm · MIT · no runtime dependency · [Get started](/docs/getting-started)',
  },
  loop: {
    title: 'What flowapi gives you',
    aside: 'Each figure from its own runs',
    cards: CARDS,
  },
  docs: { title: 'Documentation' },
  closing: {
    title: 'Generate your types',
    aside: 'Free · MIT license',
    steps: [
      { title: 'Run it', text: '`npx @localix-ai/flowapi openapi.json -o types.ts`, JSON or YAML. The Rust command is the same: `cargo install --locked --git https://github.com/localixai/flowapi --tag v0.2.0 flowapi-cli`.' },
      { title: 'Import the types', text: 'Each component schema is a type of its own name, each operation one with its parameters and responses, and `Paths` maps every path to them.' },
      { title: 'Add the SDK if you want it', text: '`--sdk` prints the request and response functions after the types. `flowapi generate flowapi-sdk.yml --out sdk` writes a whole client package: resources, retries, errors, pagination, tests. `--target rust` prints serde types instead.' },
    ],
    links: [
      { label: 'Getting started', href: `${DOCS}/getting-started` },
      { label: 'Source on GitHub', href: GITHUB },
    ],
  },
};

export default landing;
