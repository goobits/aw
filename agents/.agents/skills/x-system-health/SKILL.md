---
name: x-system-health
description: 'Use when the user invokes $x-system-health or /x-system-health, asks for CPU/memory/load/swap usage, asks what is eating resources, asks whether system usage is bad, or asks to identify/kill stale Playwright/Xvfb/browser helper processes.'
---

# X System Health

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use this skill for local resource audits and safe process cleanup.

There are two modes:

- Report mode: inspect system health and recommend next steps only.
- Cleanup mode: kill or clean up processes only when the user explicitly asks.
  Never infer cleanup approval from a health question.

## Health Snapshot

Start with a concise snapshot:

```bash
printf 'SYSTEM\n'
uptime
nproc
free -h
printf '\nCPU_TOP\n'
ps -eo pid,ppid,pgid,pcpu,pmem,rss,etime,comm,args --sort=-pcpu | head -11
printf '\nMEM_TOP\n'
ps -eo pid,ppid,pgid,pcpu,pmem,rss,etime,comm,args --sort=-rss | head -11
```

Report in tables:

- System capacity: CPU cores, load average, RAM total/used/free/available, swap total/used/free.
- Worst CPU offenders: PID, process group, CPU, memory, RSS, age, process name.
- Worst memory offenders: PID, process group, memory, RSS, CPU, age, process name.

Convert RSS from KiB to MiB/GiB in the response. Keep command text out of the final answer unless the user asks for it.

## Interpretation

Call out:

- CPU pressure when 1-minute load approaches or exceeds core count, or one process is pegged above 100%.
- Memory pressure when available RAM is low or swap is actively used.
- Swap usage as a warning sign, not an immediate emergency, when available RAM is still healthy.
- The single biggest actionable offender before listing secondary noise.

When the user says Codex, Claude, Zellij, or another class is okay, exclude those from cleanup recommendations unless they are clearly orphaned and the user explicitly asks.

## Playwright/Xvfb Audit

For browser/test leftovers:

```bash
ps -eo pid,ppid,pgid,lstart,etime,pcpu,pmem,rss,stat,command \
	| rg 'playwright test|xvfb-run|Xvfb|playwright-mcp|@playwright/mcp|chrome-linux/chrome|firefox|webkit' \
	| rg -v rg
```

Classify:

- Active test groups: `playwright test`, `xvfb-run`, `Xvfb`, and browser children sharing a process group.
- Browser helpers: `playwright-mcp`, `@playwright/mcp`, and their browser children.
- Orphaned browser children: browser processes whose parent/test runner has exited, often with PPID `1`.

Default action:

- Do not kill anything unless the user asks to clean up or kill processes.
- Leave fresh jobs alone unless the user says to kill them.
- Treat helpers older than roughly 30 minutes as old when the user asks to kill old helpers.
- If ownership is unclear, report the process group and ask or stop; do not kill
  shared agent, shell, editor, queue, or database processes on suspicion.

## Killing Processes

Prefer terminating whole process groups so child browser/Xvfb processes exit together:

```bash
kill -TERM -- -<PGID>
sleep 2
```

Verify afterward with the Playwright/Xvfb audit command. Use `kill -KILL -- -<PGID>` only if `TERM` fails and the user asked for cleanup. Never kill unrelated Codex/Claude/Zellij process groups unless explicitly requested.

## Response Shape

Style healthy capacity, warnings, cleanup candidates, unsafe cleanup, and
report-only next steps with shared colors when useful.

For a normal report:

1. System capacity table.
2. Worst CPU table.
3. Worst memory table.
4. Browser/test audit table when matching processes exist.
5. `Suggested next steps` section in bullet form, marked as report-only unless
   cleanup was requested.

Format `Suggested next steps` like a small proposal:

- **Biggest issue**: name the one process group or resource condition most worth
  attention.
- **Safe now**: actions the agent can do without process cleanup, such as waiting
  for a fresh test to finish, rerunning the audit later, or checking a specific
  owner/tab.
- **Cleanup option**: exact process groups that are reasonable to kill only if
  the user explicitly asks for cleanup; include why they are safe candidates
  such as old helper, orphaned browser, or stale test.
- **Leave alone**: active, fresh, user-owned, shared, or unclear processes that
  should not be killed from report mode.
- **Longer-term**: optional prevention ideas such as stopping old dev servers,
  reducing parallel browser runs, adding timeouts, or restarting memory-heavy
  dev servers.

Use concise bullets. Do not present cleanup bullets as already approved in
report mode.

For cleanup:

1. List process groups killed.
2. List process groups intentionally left alone and why.
3. Confirm no matching stale processes remain, or name what remains.
4. End with `Suggested next steps` bullets:
    - `Done`: what was cleaned up.
    - `Still running`: active groups intentionally left alone.
    - `Watch`: any remaining CPU, RAM, or swap concern.
