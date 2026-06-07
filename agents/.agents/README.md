# Shared Agent System

This directory is designed to be copied, vendored, or submoduled into multiple
projects as a portable agent workflow system.

## Install Shape

Each project should have:

```text
AGENTS.md                 # tiny loader committed by the project
.agents/                  # shared reusable agent system
.agents.local/project.md  # project-specific adapter committed by the project
```

`AGENTS.md` should point agents at `.agents/AGENTS.md`, then at
`.agents.local/project.md`.

## What Is Shared

- `.agents/AGENTS.md`: portable safety, style, skill, and policy map.
- `.agents/skills/`: reusable workflows.
- `.agents/policies/`: reusable default policies.
- `.agents/templates/`: starter files for a new project.
- `.agents/souls.md`: expanded communication/style preferences.

## What Is Local

Project-specific commands, layout, ports, package names, repo quirks, and
overrides belong in:

```text
.agents.local/project.md
```

Do not put local project facts into shared skills unless the workflow itself is
not portable.

## New Project Setup

1. Copy or mount `.agents/` into the project.
2. Copy `.agents/templates/root-AGENTS.md` to `AGENTS.md`.
3. Copy `.agents/templates/project.md` to `.agents.local/project.md`.
4. Fill every relevant local command, path, and override.
5. Keep future reusable workflow changes in `.agents/`; keep project facts in
   `.agents.local/`.
