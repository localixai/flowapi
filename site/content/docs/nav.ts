import type { DocsConfig } from '@/shared/config';

// The docs' table of contents. Every markdown file under content/docs is a page listed here, in
// a group, or is named under `unlisted` with why: a file in neither fails the build.
const docs: DocsConfig = {
  index: {
    title: 'flowapi docs',
    description:
      'How to make TypeScript or Rust types from an OpenAPI or JSON Schema description with flowapi, how they are checked against the specification, and how fast it is.',
  },
  groups: [
    {
      title: 'Start',
      blurb: 'From the install to a file of types, and the request and response functions.',
      pages: ['getting-started', 'sdk'],
    },
    {
      title: 'Proof',
      blurb: 'How the types are held to the specification, and what the benchmark measured.',
      pages: ['conformance', 'speed'],
    },
  ],
  unlisted: {},
};

export default docs;
