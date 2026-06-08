---
name: x-version-source
description: 'Use when the user invokes $x-version-source or /x-version-source, asks to find hardcoded version references, source versions from package manifests, remove stale README version claims, align displayed versions with package.json/pyproject/Cargo.toml, or audit version drift.'
---

# X Version Source

Use this skill to remove version drift by making maintained manifests the source
of truth. It is based on `.llm/scratch/prompt-palette/version.md`.

Read `.agents.local/project.md`, `.agents/policies/docs.md`, and nearby package
docs when present. Follow local package-manager and Git ownership rules.

## Objective

Version values should come from the owning manifest or runtime metadata, not
from duplicated constants, docs, tests, or generated output.

## Sources Of Truth

Prefer the nearest authoritative manifest for the package or service:

- JavaScript/TypeScript: `package.json`.
- Python: `pyproject.toml`, then `setup.py` only when it is the owner.
- Rust: `Cargo.toml`.
- Apps/services: their package manifest or explicit app metadata file.
- Generated docs or artifacts: generator input, not generated output.

README files should usually avoid repeating current version numbers. Keep a
README version only when it documents a compatibility floor, migration boundary,
browser/runtime baseline, protocol version, or known-version quirk.

## Workflow

1. Identify the scope: package, app, server, repo root, docs, or release tool.
2. Search for version claims:
   - `rg -n "version|v[0-9]+\\.|[0-9]+\\.[0-9]+\\.[0-9]+" <scope>`
3. Classify each hit:
   - current package/app version
   - compatibility minimum or maximum
   - protocol/schema/API version
   - dependency version
   - docs example
   - generated or vendored output
4. For duplicated current-version values, read from the source manifest instead
   of copying the string.
5. For docs, remove stale current-version claims or rewrite them as source-of-
   truth pointers.
6. For compatibility or protocol versions, keep them explicit and label why they
   are intentionally fixed.

## Rules

- Do not edit generated, minified, vendored, or lockfile content unless the
  owning generator/package-manager command is in scope.
- Do not change package versions unless the user explicitly asks for a version
  bump or dependency update.
- Do not replace meaningful compatibility baselines with package versions.
- Do not add fallback compatibility wrappers; update callers to the clean source
  of truth.
- When dependency upgrades are the real task, use a dedicated package-update
  workflow instead of this skill.

## Verification

Run lightweight checks appropriate to the change:

- `rg` for removed hardcoded version strings and stale docs claims.
- Focused tests for version display/export helpers when present.
- Type/lint checks for touched package code when practical.
- No package install, lockfile update, or broad build unless explicitly
  approved by local policy.

## Output

```text
▌ Version Source
~ path - version now reads from source of truth.
- path - removed stale duplicated version claim.
✓ path - explicit compatibility/protocol version intentionally preserved.

▌ Verified
· command/result, or not run with reason.

▌ Remaining
· generated outputs, compatibility baselines, or owner decisions not changed.
```
