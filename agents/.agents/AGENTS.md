# Shared Agent System Instructions

These instructions apply when changing the reusable `.agents` system itself.
Projects that consume this system own their always-on startup contract in their
root `AGENTS.md`.

## Loading Model

- Keep the root `AGENTS.md` small and project-owned.
- Load only the skill, policy, reference, or local-project section needed for
  the current action. Never require agents to preload this directory.
- Load a skill body when its workflow is explicitly invoked or clearly matches
  the task. Skill frontmatter and harness discovery own the skill inventory.
- Read `.agents.local/project.md` only when local commands, layout, ports,
  storage, coordination, or overrides matter.
- Read the relevant file under `policies/` when a detailed reusable standard
  matters; do not copy the policy into startup instructions or skills.

## Canonical Owners

- Root `AGENTS.md`: always-on project scope, safety, and routing.
- `.agents.local/project.md`: project-specific commands, paths, infrastructure,
  integration owners, and policy overrides.
- `policies/*.md`: reusable standards grouped by responsibility.
- `skills/*/SKILL.md`: one invoked workflow per skill.
- `skills/x-consolidate/references/canonical-naming-and-duplication.md`:
  canonical vocabulary, semantic duplication, and module ownership.
- `souls.md`: optional communication style and output vocabulary.
- `README.md` and `templates/`: installation guidance for people setting up the
  system in another project.

One concept has one canonical owner. Link to that owner instead of restating
its detailed commands, examples, vocabulary, or output contract.

## Portability

- Keep reusable policies and skills free of project-specific commands, package
  names, ports, paths, attribution, and infrastructure facts.
- Put those facts in `.agents.local/project.md`. A deliberately integration-
  specific workflow, such as the Shelly queue mode in `x-commit`, may name its
  integration explicitly.
- When one skill delegates to another, link the direct `SKILL.md` path.
- Keep skill frontmatter names and descriptions accurate. Do not maintain a
  second prose catalog of installed skills.
- Keep modules and documents focused. Do not add generic buckets or parallel
  owners for an existing policy, workflow, or concept.

## Verification

- Search for the retired wording, copied command, or stale path after a change.
- Check affected links and skill frontmatter directly.
- Use the lightest relevant project check; instruction-only edits do not need a
  build or a new test unless an existing validator covers the changed contract.
