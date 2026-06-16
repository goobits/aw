---
name: x-update-changelog
description: 'Use when the user invokes $x-update-changelog or /x-update-changelog, asks to update CHANGELOG.md, sync the changelog with recent commits, refresh the [Unreleased] section, advance the changelog audit cutoff, or audit the changelog for duplicates and formatting drift.'
---

# X Update Changelog

Use `.agents/souls.md` vocabulary when it improves scanning; keep stricter local output rules.

Use when the user asks to update `CHANGELOG.md`, refresh `[Unreleased]`, sync
with recent commits, or advance the audit cutoff.

This is an editing skill. Do not commit unless the user asks, or another active
skill delegates a verified scoped slice to `x-commit`
(`.agents/skills/x-commit/SKILL.md`).

Read `.agents/policies/docs.md` and `.agents/policies/git.md` when present.
This skill owns changelog editorial rules; repo Git mechanics stay in policy.

## Workflow

1. Read root `CHANGELOG.md`.
2. Parse the cutoff comment near the top:
   `<!-- CHANGELOG audit cutoff: <date>. commit <sha> on <branch>. -->`.
   Capture `<sha>`.
3. Pin `<head>` with the repo-approved HEAD-inspection command. Audit exactly
   `<sha>..<head>`. Re-confirm HEAD before stamping; if it moved, classify the
   new commits too.
4. List every commit in `<sha>..<head>` with the repo-approved log command. Use
   stat/log detail when subjects are terse. Never use date windows such as
   `--since`, `--until`, or `--after`; the SHA range is authoritative.
5. Skip routine noise: submodule pointer bumps with no other change, guide media
   pointer bumps, lockfile-only commits, version bumps without behavior, and
   merge commits.
6. Classify remaining commits with the consumer-vs-internal filter. Observable
   user/API changes go under Keep a Changelog user-facing categories; internal
   package moves, tooling, tests, and type cleanup go under `Internal`.
7. Draft one user-meaningful bullet per logical change, not per commit. Merge
   related commits unless the user asks for commit-level detail.
8. Dedupe against existing `[Unreleased]` bullets in the same category. Merge
   overlapping entries; treat roughly 70 percent substring overlap as a signal,
   then use judgment.
9. Insert entries in the correct category, preserving existing bullets and order.
10. Move the cutoff comment to the confirmed HEAD short hash and today's date.
    The stamped cutoff must equal the HEAD audited through.

## Consumer Filter

Ask: **"Would a product user or API consumer observe this in day-to-day use?"**
If no, put it in `Internal` regardless of significance.

`Internal`: package extractions/renames without behavior change; code moves with
the same public behavior; undocumented namespace/global/shim/wrapper removal;
build, cold-start, CI, test-runtime, typecheck, package-boundary, test
infrastructure, fixtures, smoke/regression harnesses, ESM migrations,
source-folder moves, repo tooling, dev gateway/shell ergonomics, agent skills,
and canonical packages consumers do not code against.

User-facing sections: observable product UI or public API features; reproducible
bug fixes; public API renames/removals callers must update (`⚠️`); runtime
performance wins users feel; documented deprecations (`📛`); LCP,
time-to-interactive, or app-startup wins; visible login, billing, public status,
or other user-facing service launches.

For mixed work, split it: user-facing behavior in the relevant category,
move/plumbing work in `Internal`. Do not duplicate.

## Style Rules

- **Front-loaded**: lead with the noun/subject, not verbs like `Added`,
  `Fixed`, `Made`, or `Improved`.
- **Concise**: one sentence per bullet. Keep package names, public APIs, and
  feature names; drop internal paths and irrelevant jargon.
- **Concrete**: avoid `hardened`, `tightened`, `consolidated`, `cleaned up`, or
  `reorganized` as load-bearing verbs in user-facing bullets. Name the
  observable behavior or move the line to `Internal`.
- **Split mega-bullets**: keep user-facing bullets under roughly 200 characters.
  Split independent items; `Internal` may be longer reference material.
- **Grouped**: keep bullets in their category and never duplicate an item across
  categories.
- **No commit prefixes**: no `feat:`, `fix:`, or conventional-commit tags.
- **No em-dashes** in CHANGELOG prose. Use periods, commas, colons, or
  parentheses.

## Emojis

Section headers use category emojis. Bullets start with one topic emoji from the
fixed vocabulary. The section says add/fix/removal; the bullet emoji says
_what_. Do not duplicate section emojis on bullets.

Section headers: `### ✨ Added`, `### 🔧 Changed`, `### 🐛 Fixed`,
`### ⚡ Performance`, `### 🔒 Security`, `### 🗑️ Removed`,
`### 🏠 Internal`.

Reuse the target `CHANGELOG.md` topic vocabulary when it has one. Otherwise use
exactly one generic topic emoji, preferring the dominant specific topic:

| Emoji | Topic                                             |
| ----- | ------------------------------------------------- |
| 🪟    | UI, screens, layout, navigation, dialogs          |
| 🧩    | Components, design system, shared primitives      |
| 🎨    | Visual output, rendering, media, presentation     |
| 🗂️    | File IO, import/export, data formats, clipboard   |
| ⌨️    | Input, keyboard, shortcuts, gestures              |
| 👤    | Users, auth, billing, sessions, permissions       |
| 🛒    | Commerce, checkout, products, orders              |
| ☁️    | Sync, storage, filesystem, backup                 |
| 🌐    | Network, CDN, DNS, domains, gateways              |
| 🚀    | Deploy, operations, servers, releases             |
| 🧪    | Tests, smoke, coverage, regression, fixtures      |
| 📦    | Packages, API surface, exports, runtime contracts |
| 📚    | Docs, proposals, runbooks, changelog              |

Stackable markers after the topic emoji:

- `⚠️` for breaking changes requiring migration.
- `📛` for deprecations scheduled for later removal.

If a section header or bullet lacks its emoji, add it. If one is already
present, leave it. Do not add decorative emojis outside this vocabulary.

## Highlights

Each version block, including `[Unreleased]`, opens with `### 🌟 Highlights`: 5
to 8 curated bullets for the most user-impactful wins. Each highlight is one
short sentence using the topic-emoji convention. Highlights intentionally
duplicate detailed-section content. Skip Highlights only when the release has
fewer than 10 user-facing bullets total.

## Section Structure

Group bullets by this topic order: 🪟 UI & Shell, 🧩 Components,
🎨 Rendering, 🖌️ Brush, 📐 Vector & Path, ✏️ Text, 🖼️ Image & Asset,
🗂️ File IO, 🎬 Media, 🎯 Tool, ⌨️ Input, 👆 Touch, 👤 Accounts & Auth,
🛒 Commerce, ☁️ Sync & Storage, 🌐 Network, 🚀 Deploy & Operations,
📦 Package Boundaries, 🧪 Tests & Smoke, 📚 Docs & Proposals.

When a section has **25 or more bullets**, insert `#### <emoji> <Label>`
subsections for each topic present, in canonical order. With fewer than 25
bullets, omit subsections and sort by canonical topic order.

Within a topic, preserve existing order and append new entries. Do not re-sort
or rewrite intra-topic order on routine updates.

## Versioning And Archives

Keep `[Unreleased]` as the working area until release. Do not add
`#### YYYY-MM-DD` subheaders by default; the changelog is per-version, not
per-day. If explicitly requested, use dated subheaders only inside
`[Unreleased]` and flatten them at release.

If `[Unreleased]` is too long and release is not imminent, suggest an
intermediate version such as `v8.0.0-beta.N`; do not cut one without approval.

Old release notes live in per-era sibling files at the repo root. The main file
holds only `[Unreleased]` plus the most recent shipped major version, then an
Archives section listing every era file.

Names: `CHANGELOG_v<N>.md` for a major-version era, and `CHANGELOG_pre-v<N>.md`
only for untagged history before the first version era.

Era files use the same topic emojis, consumer filter, front-loaded prose, and
no-em-dash rule. They follow this shape:

```markdown
# Changelog (<era label>)

<one-paragraph orientation>

## Audit status

<authored | pending | partial>. Source of truth is git history if pending.

## Highlights

<5 to 15 bullets in standard topic-emoji format; consumer filter applies>

## Tags

| Date       | Tag  |
| ---------- | ---- |
| YYYY-MM-DD | v... |
```

When auditing an era, avoid commit-by-commit transcription: read the shortlog,
skim milestone subjects, find substantial features by churn, sample boundary
docs at tags, then synthesize 5 to 15 user-meaningful Highlights bullets.

The main file's `## Archives` section links each era file, newest to oldest,
including the pre-history file when one exists. Do not duplicate archived
content in the main file. Move it.

## Report

Use this output shape:

```text
▌ Changelog Update

✓ Audited
  <count> commits since cutoff <old-sha>

✓ Added
  bullets added per category, with counts

✓ Merged / Deduped
  bullets merged into existing entries

✓ Cutoff
  <old-sha> -> <new-sha>

◇ Skipped
  commits intentionally skipped as noise
```

If the commit range is empty, say that directly and do not edit the file.

## Commit Mode

Do not commit by default. If the user asks to commit, hand `CHANGELOG.md` only
to `x-commit` (`.agents/skills/x-commit/SKILL.md`). Do not bundle unrelated
work.
