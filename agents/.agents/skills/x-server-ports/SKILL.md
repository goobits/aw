---
name: x-server-ports
description: 'Use when the user invokes $x-server-ports or /x-server-ports, asks to fix dev server port drift, design fixed-port server startup, audit port/PID ownership, prevent auto-incremented ports, or make local server lifecycle management fail fast and predictable.'
---

# X Server Ports

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use this skill to design, audit, or implement fixed-port server lifecycle
behavior. It is based on `.llm/scratch/prompt-palette/server-ports.md`.

Read `.agents.local/project.md` and `.agents/policies/git.md` when present.
Follow local dev-server ownership rules exactly. In this repo, do not stop,
restart, or clean-restart shared dev services unless the user explicitly asks,
or you started that exact service in the current turn.

## Objective

Prevent port drift. A service that callers expect on one port should either run
on that port or fail with a clear conflict message. It should not silently move
to `PORT + 1`, random ports, or framework defaults.

## Fixed-Port Rules

- Define one explicit default port per service in the nearest config owner.
- Track self-owned server instances with a PID file or equivalent local state.
- If the expected port is free, start on the expected port.
- If the expected port is occupied by the tracked self-owned process, reuse it
  or prompt before restart when interaction is appropriate.
- If the expected port is occupied by an unknown process, fail fast with a clear
  diagnostic and next command.
- A `--restart` flag may stop only the tracked self-owned process, then wait for
  port release and retry the same port.
- Never kill unknown processes automatically.
- Never auto-increment ports unless the user explicitly asks for fallback-port
  behavior and all dependents can discover it.

## Audit Workflow

1. Identify the service, expected port, callers, and documented URLs.
2. Search for port definitions and fallback behavior:
   - `rg "<port>|PORT|listen|server|pid|restart" <service paths>`
3. Check whether startup behavior matches documented URLs and local policy.
4. Check for hidden drift:
   - framework auto-port fallback
   - random free-port helpers
   - stale PID files
   - docs pointing at a different port
   - dependent scripts hardcoding old ports
5. Propose or implement the smallest change that makes ownership explicit.

## Implementation Shape

Prefer the repo's existing server manager or dev-service helper. If adding new
logic, keep it narrow:

- `DEFAULT_PORT` or config-owned equivalent.
- PID/state file scoped to the service name.
- `isTrackedProcessRunning(pid)` check.
- `isPortFree(port)` or equivalent bind/probe.
- graceful shutdown handlers for self-owned servers.
- clear exit codes and messages for unknown conflicts.

## Verification

Run lightweight checks appropriate to the change:

- Unit tests for port/PID decision logic when present.
- Focused smoke command for the service when allowed by local policy.
- `rg` for stale ports or old URLs after changes.
- No broad build unless explicitly approved.

## Output

Style final output directly with the shared colorful vocabulary. The fenced
block is a structure template, not literal output.

```text
▌ Server Ports
~ path - fixed-port behavior, PID ownership, or docs updated.
! path - remaining port drift or unknown ownership risk.

▌ Verified
· command/result, or not run with reason.

▌ Remaining
· services, docs, or callers still needing port ownership decisions.
```
