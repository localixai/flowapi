// The tools the judge runs, at the versions it pins, and the one way to replace them in tests.

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

/** The plugin's folder. */
export const ROOT = fileURLToPath(new URL('..', import.meta.url))

/**
 * The flowapi the plugin runs, from `lib/flowapi-version`, the one place it is written: `bin/flowapi`
 * reads it too. The judge regenerates with the version a repository's lock names, else this one.
 */
export const FLOWAPI_VERSION = readFileSync(new URL('flowapi-version', import.meta.url), 'utf8').trim()
export const PUBLINT = 'publint@0.3.25'
export const ATTW = '@arethetypeswrong/cli@0.18.5'

/**
 * The `npx` the judge calls. `FLOWAPI_PLUGIN_NPX` names another program with the same arguments:
 * the tests put a stand-in there, so no step reaches the network.
 */
export function npx(env = process.env) {
  return env.FLOWAPI_PLUGIN_NPX || 'npx'
}

/**
 * The generate command when none is given: `{out}` is the folder, `{flowapi}` the version.
 * `FLOWAPI_PLUGIN_FLOWAPI` replaces flowapi here as in `bin/flowapi`, so step 0 regenerates with the
 * flowapi the agent used.
 */
export function defaultGenerate(env = process.env) {
  const flowapi = env.FLOWAPI_PLUGIN_FLOWAPI || `${npx(env)} -y @localix-ai/flowapi@{flowapi}`
  return `${flowapi} generate flowapi-sdk.yml --out {out}`
}
