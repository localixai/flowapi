# The site

The landing page and the docs of flowapi, served at <https://flowapi.localix.ai>, as a folder of
the generator's repository. It was made from the
[llx-sitekit](https://github.com/rsaryev/llx-sitekit) template (its commit `d21ee3d`, which
`sitekit.synced` in `package.json` names) and keeps the template's layout, so
[AGENTS.md](AGENTS.md) here holds for every change in this folder.

```sh
cd site
npm ci
npm run dev          # http://localhost:5173
npm run check        # the layers, the types, the build, the docs' standard
npm run sync         # the template's changes since sitekit.synced, as a patch
```

## What is whose

| Path | Owner | |
| --- | --- | --- |
| `content/` | the site | `site.ts` (name, address, the install commands), `landing.ts` (the landing's words), `docs/` (the pages and `nav.ts`), `theme.css` |
| `public/` | the site | the icons, the link preview (`og.png`, drawn from `og-card.html`), the two pictures, `llms.txt`, `robots.txt` |
| `src/`, `scripts/`, the configs | the template | change one only where the site cannot say it in `content/` |

## What the site says is the repository's

No figure here is typed by hand: each is copied from a result the repository's checks or its
benchmark wrote. After a run that moves one, change it here in the same commit.

| Figure | Where it shows | Taken from |
| --- | --- | --- |
| Soundness, precision and groups on the JSON Schema Test Suite (779 of 779, 373 of 540, 0 of 409) | `landing.ts` (Conformance, Two targets), `docs/conformance.md` | `../conformance/results/suite.md` |
| The same for openapi-typescript and @hey-api/openapi-ts, with their versions | `landing.ts` (Conformance), `docs/conformance.md` | `../conformance/results/compare.md` |
| The Rust target's soundness and precision (787 of 787, 330 of 543) | `landing.ts` (Two targets), `docs/conformance.md` | `../conformance/results/suite-rust.md` |
| Random schemas (600 schemas, 4,319 of 4,319 valid values) | `landing.ts`, `docs/conformance.md` | `../conformance/results/props.md` |
| The OAI's examples (78 of 78) | `landing.ts`, `docs/conformance.md` | `../conformance/results/oai.md` |
| The SDK's style examples and cases (51, 234, none differ) | `landing.ts` (Two targets), `docs/sdk.md` | `../conformance/results/sdk.md` |
| Times, sizes, peak memory, which outputs type-check, the machine and versions | `landing.ts` (Speed), `docs/speed.md` | `../bench/results/speed.md` |

- **The pictures** `public/output-dark.png` and `public/compare-dark.png` are copies of
  `../assets/`, which `bench/` draws (`shots` and `charts`, at the end of `bench/run.sh`). Copy
  them again when they are drawn again. Dark only, as every demo of the lab.
- **Code** on a card or a page is what flowapi prints for it, or the repository's README's own
  example. The link preview's type is the command's output for the schema the card shows.
- **The link preview** `public/og.png` is drawn from `public/og-card.html` with the command in
  its comment (headless Chrome, at twice 1200×630).

## What differs from a site cloned from the template

- **It has no repository of its own.** `content/site.ts` says so with `sourceDir: 'site'`, which
  `sourceUrl` in `src/shared/config/index.ts` puts into a page's "Edit on GitHub", as the lab's
  other product sites do.
- **The link preview is drawn larger**, 2400×1260; `image.width` and `image.height` say so in the
  page's head (`src/app/head.ts`).
- **No release list and no recording.** The landing leaves out `releases` (the changelog is the
  repository's, `../CHANGELOG.md`) and the hero has no film.
- **Its workflow is the repository's**, `../.github/workflows/site.yml`: it checks the site on a
  change to it on `main` and publishes `dist/` to Cloudflare Pages, project `flowapi`.
