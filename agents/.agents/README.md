# Shared Agent System

This directory is designed to be copied, vendored, or submoduled into multiple
projects as a portable, progressively loaded agent workflow system.

## Install Shape

Each project should have:

```text
AGENTS.md                 # small project-owned startup contract
.agents/                  # shared skills, policies, references, and templates
.agents.local/project.md  # project-specific adapter
```

The root `AGENTS.md` contains only always-on project scope, safety, and routing.
It tells agents when to read one relevant policy or local-project section; it
must not require eager reading of `.agents/AGENTS.md`, the whole `.agents/`
tree, or the whole project adapter.

Agent harnesses discover skills from `skills/*/SKILL.md` frontmatter and load a
complete skill body only when its workflow is activated. Do not copy the skill
inventory into `AGENTS.md` or another maintained list.

## Canonical Owners

- Root `AGENTS.md`: always-on project instructions and lazy routing.
- `.agents/AGENTS.md`: instructions for maintaining the shared system itself.
- `.agents/skills/`: reusable invoked workflows.
- `.agents/policies/`: reusable standards.
- `.agents/souls.md`: optional communication and output style.
- `.agents/templates/`: starter files for a new project.
- `.agents.local/project.md`: local commands, layout, ports, storage,
  coordination, integrations, and overrides.

Keep detailed rules in one owner and link to them. Project facts do not belong
in shared policies or generic skills unless the workflow explicitly targets a
named integration.

## New Project Setup

1. Copy or mount `.agents/` into the project.
2. Copy `.agents/templates/root-AGENTS.md` to `AGENTS.md`.
3. Copy `.agents/templates/project.md` to `.agents.local/project.md`.
4. Add the project's always-on scope and safety rules to root `AGENTS.md`.
5. Fill relevant local commands, paths, infrastructure, and overrides in the
   project adapter.
6. Confirm startup does not eagerly load the shared guide, every policy, every
   skill body, or the entire project adapter.
7. Keep reusable workflow changes in `.agents/`; keep project facts in
   `.agents.local/`.
