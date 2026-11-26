#!/usr/bin/env node
// PreToolUse hook: the guard (lib/guard.mjs decides), for two clients. With no argument it is
// Claude Code's hook (hooks/hooks.json, matched to Edit, Write, MultiEdit and NotebookEdit): a
// denial is `hookSpecificOutput` with `permissionDecision: "deny"`. With the argument `core` it is
// the Agent Plugins core hook (hooks.json, every tool call, core 1.1.0 §7.3.6): it reads the
// `file.modify` tool profile and a denial is `{ "decision": "deny", "reason" }`. In both, silence
// (no output) is no opinion: it never answers `allow`, so the client's own permissions still apply.
//
// It runs before every edit, so it stays cheap: it reads the event and at most one lock per path,
// one file read per directory level, and starts no process. Any error lets the edit through with
// nothing printed; every answer is on stdout with exit code 0.

import { decide, decideCore } from '../lib/guard.mjs'

const core = process.argv[2] === 'core'
const chunks = []
for await (const chunk of process.stdin) chunks.push(chunk)
let decision = null
try {
  const event = JSON.parse(Buffer.concat(chunks).toString('utf8'))
  decision = core ? decideCore(event) : decide(event)
} catch {
  decision = null // fail open, and print nothing: the event may hold the person's file contents
}
if (decision) {
  const answer = core
    ? { decision: 'deny', reason: decision.reason }
    : {
        hookSpecificOutput: {
          hookEventName: 'PreToolUse',
          permissionDecision: 'deny',
          permissionDecisionReason: decision.reason,
        },
      }
  process.stdout.write(`${JSON.stringify(answer)}\n`)
}
