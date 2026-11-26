// Running one command for the judge: stdout and stderr together in the order they came, a time
// limit that ends the whole process group, and the exit code. Never throws: a command that cannot
// start is reported like one that failed.

import { spawn } from 'node:child_process'

/** How much output is kept per command; the report shows only its first lines. */
const KEEP_BYTES = 256 * 1024

/**
 * Runs `file` with `args` in `cwd`; with `shell`, `file` is a whole command line for the system
 * shell and `args` is empty. Resolves to `{ code, signal, timedOut, output }`.
 */
export function run(
  file,
  args,
  { cwd, env = process.env, timeoutMs = 600_000, shell = false } = {},
) {
  return new Promise((resolve) => {
    const chunks = []
    let kept = 0
    let timedOut = false
    const keep = (chunk) => {
      if (kept >= KEEP_BYTES) return
      chunks.push(chunk.subarray(0, KEEP_BYTES - kept))
      kept += chunk.length
    }
    let child
    try {
      child = spawn(file, args, {
        cwd,
        env,
        shell,
        stdio: ['ignore', 'pipe', 'pipe'],
        detached: process.platform !== 'win32', // its own group, so a time limit ends its children too
        windowsHide: true,
      })
    } catch (error) {
      resolve({ code: null, signal: null, timedOut, output: `could not start: ${error.message}\n` })
      return
    }
    const timer = setTimeout(() => {
      timedOut = true
      stop(child)
    }, timeoutMs)
    child.stdout.on('data', keep)
    child.stderr.on('data', keep)
    let startError = null
    child.on('error', (error) => {
      startError = error
    })
    child.on('close', (code, signal) => {
      clearTimeout(timer)
      let output = Buffer.concat(chunks).toString('utf8')
      if (startError) output += `could not start: ${startError.message}\n`
      resolve({ code: startError ? null : code, signal, timedOut, output })
    })
  })
}

function stop(child) {
  try {
    if (process.platform === 'win32') child.kill()
    else process.kill(-child.pid, 'SIGKILL')
  } catch {
    // already gone
  }
}

/** The first `lines` lines of `text`, with a note of how many were left out. */
export function head(text, lines = 40) {
  const all = text.replace(/\s+$/, '').split('\n')
  if (all.length <= lines) return all.join('\n')
  return `${all.slice(0, lines).join('\n')}\n… ${all.length - lines} more lines`
}

/** Quotes one argument for the system shell. */
export function shellQuote(value) {
  if (process.platform === 'win32') return `"${value.replace(/"/g, '""')}"`
  return `'${value.replace(/'/g, `'\\''`)}'`
}
