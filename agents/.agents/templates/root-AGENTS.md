# AGENTS.md

Keep always-on project scope, safety, and coordination rules concise in this
file. Put project-specific details in `.agents.local/project.md` and reusable
details in the matching `.agents/policies/*.md` owner.

Load only the instruction needed for the current action:

- Load an activated skill's `SKILL.md` for its workflow.
- Read the relevant shared policy before code, test, Git, documentation, or
  frontend work that needs its detailed rules.
- Read only the relevant section of `.agents.local/project.md` when local
  commands, layout, ports, storage, integrations, or overrides matter.
- Do not preload `.agents/`, every skill or policy, or the entire project
  adapter.

Before writing, follow the project's documented coordination workflow. Before
Git, package-manager, build, install, deploy-source, server-lifecycle, or broad
test operations, read the corresponding local-project section first.
