import type { SiteConfig } from '@/shared/config';

// What the site knows about itself: the generator's name, where the site is published, the
// repository, and the two ways to run it (the npm package, or the Rust command built from a
// release tag: the crates are not on crates.io).
const GITHUB = 'https://github.com/localixai/flowapi';
const NPM = 'https://www.npmjs.com/package/@localix-ai/flowapi';
const URL = 'https://flowapi.localix.ai';
const DESCRIPTION =
  'Types from API descriptions that follow the specification: Swagger 2.0, OpenAPI 3.0 to 3.2 and JSON Schema in, TypeScript or Rust out.';

const site: SiteConfig = {
  name: 'flowapi',
  // The family's pattern, "<Product>: <its line>", short enough for a search result.
  title: 'flowapi: types from API descriptions',
  description: DESCRIPTION,
  url: URL,
  github: GITHUB,
  // The site is a folder of the generator's repository, not a repository of its own.
  sourceDir: 'site',
  install: [
    ['npm', 'npx @localix-ai/flowapi openapi.json -o types.ts'],
    ['Cargo', `cargo install --locked --git ${GITHUB} --tag v0.1.0 flowapi-cli`],
  ],
  image: {
    path: '/og.png',
    width: 2400,
    height: 1260,
    alt: 'flowapi, types from API descriptions: a Pet schema from an OpenAPI description and the TypeScript type flowapi prints for it',
  },
  bar: {
    links: [
      { label: 'Docs', href: '/docs' },
      { label: 'Conformance', href: '/docs/conformance' },
      { label: 'GitHub', href: GITHUB },
    ],
    action: { label: 'Install', href: '/#start' },
  },
  maker: { label: 'Localix Lab', href: 'https://localix.ai' },
  footer: {
    note: 'flowapi is MIT-licensed and open source.',
    links: [
      { label: 'Docs', href: '/docs' },
      { label: 'GitHub', href: GITHUB },
      { label: 'npm', href: NPM },
      { label: 'Changelog', href: `${GITHUB}/blob/main/CHANGELOG.md` },
    ],
  },
  // The landing's WebSite gets these: what it is, beyond a page about it.
  jsonLd: {
    mainEntity: {
      '@type': 'SoftwareSourceCode',
      '@id': `${URL}/#generator`,
      name: 'flowapi',
      description: DESCRIPTION,
      programmingLanguage: ['Rust', 'TypeScript'],
      codeRepository: GITHUB,
      license: 'https://opensource.org/licenses/MIT',
      url: `${URL}/`,
      image: `${URL}/og.png`,
      sameAs: [GITHUB, NPM],
      author: { '@type': 'Organization', name: 'Localix Lab', url: 'https://localix.ai' },
    },
    inLanguage: 'en',
  },
  // Usage statistics: Google Analytics 4, only after a visitor's yes (the lab's one property).
  analytics: {
    ga4: 'G-Q8WCW7ZLDD',
    notice: 'We count visits with Google Analytics, only with your yes, to see which pages help.',
    privacy: 'https://localix.ai/privacy',
  },
  // IndexNow: the lab's one key, on every site (lab-seo indexnow announces their pages).
  indexnow: '11254c49281269eef603b4537b75fb4c',
  llms: `> ${DESCRIPTION}`,
};

export default site;
