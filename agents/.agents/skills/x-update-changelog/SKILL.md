---
name: x-update-changelog
description: 'Use when the user invokes $x-update-changelog or /x-update-changelog, asks to update CHANGELOG.md, sync the changelog with recent commits, refresh the [Unreleased] section, advance the changelog audit cutoff, or audit the changelog for duplicates and formatting drift.'
---

# X Update Changelog

Use the shared colorful output vocabulary in `.agents/souls.md` for user-facing reports when it improves scanning; keep any stricter skill-specific output contract below.

Use when the user asks to update `CHANGELOG.md`, refresh `[Unreleased]`, sync the changelog with recent commits, or advance the audit cutoff.

This is an editing skill. Do not commit unless the user explicitly asks, or another active skill delegates a verified scoped slice to `x-commit` (`.agents/skills/x-commit/SKILL.md`).

Read `.agents/policies/docs.md` and `.agents/policies/git.md` when present. This
skill owns changelog-specific editorial rules; repo-specific Git mechanics stay
in the Git policy.

## Workflow

1. Read `CHANGELOG.md` from the repo root.
2. Parse the audit cutoff comment near the top, which looks like `<!-- CHANGELOG audit cutoff: <date>. commit <sha> on <branch>. -->`. Capture `<sha>` as the cutoff commit.
3. Pin the audit head: run the repo-approved HEAD-inspection command once and capture it as `<head>`. Audit exactly the `<sha>..<head>` range so the range cannot drift mid-audit, and so the cutoff you stamp in step 10 matches the commits you actually classified. Re-confirm `<head>` immediately before step 10, because commits frequently land while you work; if HEAD moved, classify the new commits too before stamping.
4. Run the repo-approved log command to list every commit in the range. Use a
   stat/log variant when commit subjects are too terse to classify. Always scope
   by the `<sha>..<head>` SHA range, never by a date window. Do not use
   `--since`, `--until`, or `--after` to decide what changed: commit and author
   dates cluster, so date windows silently undercount. The SHA range from the
   recorded cutoff is the only authoritative source of "what changed."
5. Skip routine noise: submodule pointer bumps with no other change, guide media pointer bumps, lockfile-only commits, version bumps that ship no behavior, and merge commits.
6. Classify each remaining commit into Keep a Changelog categories using the **consumer-vs-internal filter** (see below). User-facing and developer-facing API changes that consumers will observe in their day-to-day use go under `Added`, `Changed`, `Fixed`, `Performance`, `Security`, or `Removed`. Everything else, including package extractions, code moves between packages, namespace cleanup, build/test infrastructure, and internal type tightening, goes under `Internal`.
7. Draft one user-meaningful bullet per logical change, not per commit. Consolidate related commits into a single entry when they describe one effort. Do not add one bullet per commit unless the user explicitly asks for that level of detail.
8. Dedupe against existing `[Unreleased]` bullets in the same category before inserting. If a new entry overlaps substantively with an existing bullet, merge clarifying detail into the existing bullet instead of adding a duplicate. Treat substring overlap of roughly 70 percent or higher as a duplicate signal; use judgment when wording differs but meaning matches.
9. Insert new entries in the correct category, preserving existing bullets and ordering.
10. Re-run the repo-approved HEAD-inspection command. If it differs from the `<head>` you pinned in step 3, classify and insert the newly landed commits before continuing. Then move the audit cutoff comment forward to that confirmed HEAD short hash and today's date. The stamped cutoff must equal the HEAD you audited through, so the next run starts exactly where this one stopped with no gap.

## Consumer-vs-internal filter

For every bullet, ask: **"Would a product user or API consumer observe this in their day-to-day use?"** If the answer is no, the bullet belongs in `Internal`, not in a user-facing section, regardless of how significant the work was.

**Goes to `Internal`:**

- Package extractions or renames without behavior change (e.g. `New package extracted from an existing module`)
- Code moves between packages with the same public behavior (e.g. `Text package internals moved further out of API globals`)
- Namespace, global, shim, or wrapper removal that was never in the documented public API (e.g. `Deprecated runtime/import wrappers removed`)
- Build, cold-start, CI, or test-runtime performance work (`Lazy license-billing services for faster cold starts`)
- Typecheck or package-boundary fixes that don't change observed behavior
- Test infrastructure, fixtures, smoke coverage, regression harnesses
- Internal type tightening, ESM migrations, source-folder reorganizations
- Repo tooling, dev gateway, dev shell ergonomics, agent skills
- New canonical packages whose existence consumers don't write code against

**Stays in user-facing sections:**

- Observable features in product UI or public package APIs
- Observable bug fixes consumers can reproduce against current behavior
- Real public API renames or removals that callers must update (mark with `⚠️`)
- Runtime performance wins users feel (text render hot paths, paint allocation)
- Real deprecations of documented surface (mark with `📛`)
- Consumer-visible perf wins (LCP, time-to-interactive, app startup)
- Visible service launches such as login, billing, public status, or other user-facing services

When a bullet describes a mix of internal moves and user-visible behavior, split it: write a tight user-facing bullet for what consumers see and put the move work under `Internal`. Do not duplicate the same item across both sections.

## Style rules

- **Front-loaded**: each bullet leads with the noun or subject, not a verb. Prefer `Vector-first PDF export pipeline with improved parity` over `Added a vector-first PDF export pipeline`. Reject draft bullets that open with `Added`, `Fixed`, `Made`, `Improved`, or similar verbs and rewrite to lead with the subject.
- **Concise**: one sentence per bullet. Strip implementation jargon that is not user- or developer-meaningful. Keep package names, public API names, and feature names; drop internal file paths.
- **Concrete, not vague**: never use `hardened`, `tightened`, `consolidated`, `cleaned up`, or `reorganized` as the load-bearing verb in a user-facing bullet. These describe effort, not observable behavior. Either name the specific behavior that improved (`Save cancellation now reports progress through postMessage retries` instead of `cancellation hardened`) or move the bullet to `Internal`. The Internal section is allowed to use effort verbs because nobody outside the repo reads it for behavior changes.
- **Split mega-bullets**: in user-facing sections, keep bullets under roughly 200 characters. When one bullet packs many independent items (e.g. several integrations or tool interactions), split it into one bullet per item. The `Internal` section may carry longer bullets since it is reference material.
- **Grouped**: keep bullets in their category. Never repeat the same item under multiple categories.
- **No commit-style prefixes**: do not start bullets with `feat:`, `fix:`, or other conventional-commit tags.
- **No em-dashes**: do not use the em dash character anywhere in CHANGELOG prose. Restructure with periods, commas, colons, or parentheses. Apply this to headers, intros, and bullets alike.

## Emojis

Two layers:

1. **Section headers** carry a **category emoji** that identifies the Keep a Changelog group.
2. **Each bullet** starts with a **topic emoji** drawn from a fixed vocabulary that identifies what that specific line is about. The section already tells the reader whether it's an add/fix/removal; the per-line emoji adds the _what_. Do not duplicate the section's category emoji on each bullet.

### Section headers

| Section     | Header               |
| ----------- | -------------------- |
| Added       | `### ✨ Added`       |
| Changed     | `### 🔧 Changed`     |
| Fixed       | `### 🐛 Fixed`       |
| Performance | `### ⚡ Performance` |
| Security    | `### 🔒 Security`    |
| Removed     | `### 🗑️ Removed`     |
| Internal    | `### 🏠 Internal`    |

### Bullet topic vocabulary

Reuse the target `CHANGELOG.md` topic vocabulary when it already has one. Match
the file's existing section headers, emoji topics, and grouping conventions
instead of imposing this fallback vocabulary.

If the changelog has no obvious topic vocabulary, pick exactly one generic topic
below. Prefer the more specific option when a line touches multiple topics; pick
the dominant subject.

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

### Stackable markers

After the topic emoji, two markers may stack, used sparingly:

- `⚠️` for a **breaking change** (API removal, schema change requiring migration, renamed public surface without alias). Example: `- 📦 ⚠️ Public X API removed; callers must migrate to Y`.
- `📛` for a **deprecation** scheduled for removal in a later version. Example: `- 📦 📛 X marked deprecated; will be removed in v9`.

### Idempotency

When a section header or bullet lacks its emoji, add it on this run. When an emoji is already present, leave it; do not reassign topic emojis on routine updates. Do not add decorative emojis outside this vocabulary.

## Highlights subsection

Each version block (including `[Unreleased]` while in flight) opens with a `### 🌟 Highlights` subsection of 5 to 8 bullets that capture the most user-impactful wins of the release. The highlights are a curated TLDR, not an exhaustive list. They sit above all other sections so a reader can grok the release in 30 seconds without scanning hundreds of bullets.

Each highlight is a single short sentence using the same topic-emoji prefix convention. Highlights duplicate content already present in the detailed sections; that duplication is intentional. Update highlights when major new work lands or when a previous highlight no longer represents the top of the release.

Skip the Highlights subsection only if the release has fewer than 10 user-facing bullets total.

## Structure within a section

Bullets within each section are grouped by topic emoji and listed in a fixed canonical order. The order moves from user-visible surface → services → operations → engineering plumbing, so a reader scanning the changelog sees the most product-meaningful changes first.

Canonical topic order:

1. 🪟 UI & Shell
2. 🧩 Components
3. 🎨 Rendering
4. 🖌️ Brush
5. 📐 Vector & Path
6. ✏️ Text
7. 🖼️ Image & Asset
8. 🗂️ File IO
9. 🎬 Media
10. 🎯 Tool
11. ⌨️ Input
12. 👆 Touch
13. 👤 Accounts & Auth
14. 🛒 Commerce
15. ☁️ Sync & Storage
16. 🌐 Network
17. 🚀 Deploy & Operations
18. 📦 Package Boundaries
19. 🧪 Tests & Smoke
20. 📚 Docs & Proposals

### Subsections

When a section has **25 or more bullets**, insert `#### <emoji> <Label>` subsection headers (one per topic that has at least one bullet in that section) using the canonical order. Subsections make long sections navigable.

When a section has **fewer than 25 bullets**, omit subsection headers and simply sort the bullets by canonical topic order. The per-bullet emoji at the start of each line still provides visual grouping.

### Bullet order within a topic

Preserve the existing order of bullets within a topic group. Append new entries to the end of their topic; do not re-sort or rewrite intra-topic order on routine updates. This keeps diffs small and respects the order in which work landed.

## Versioning structure

Keep `[Unreleased]` as the working area until release. Do not introduce `#### YYYY-MM-DD` subheaders inside category sections by default; the changelog is per-version, not per-day. If the user explicitly asks for a per-day buffer during a long release cycle, add dated subheaders only inside `[Unreleased]` and flatten them away at release time.

If `[Unreleased]` is growing past a comfortable scan length and a release is not imminent, suggest cutting an intermediate version (e.g. `v8.0.0-beta.N`) rather than reorganizing the format. Do not cut a version without explicit user approval.

### Archive convention

Old release notes do not belong in the main `CHANGELOG.md`. They live in **per-era** sibling files at the repo root, one per major-version era. The main file holds only `[Unreleased]` plus the most recent shipped major version, and ends with an Archives section listing every era file.

Naming: `CHANGELOG_v<N>.md` for a single major version (e.g. `CHANGELOG_v5.md`, `CHANGELOG_v2019.md`, `CHANGELOG_v2022.md`). Use `CHANGELOG_pre-v<N>.md` only for the untagged or pre-history span before the first version era.

Each era file follows this skeleton:

```markdown
# Changelog (<era label>)

<one-paragraph orientation: era name, date range, what shipped at a high level>

## Audit status

<authored | pending | partial>. Source of truth is the git history if pending.

## Highlights

<5 to 15 bullets in the standard topic-emoji format; consumer-filter applies>

## Tags

| Date       | Tag  |
| ---------- | ---- |
| YYYY-MM-DD | v... |
```

Era files use the same topic-emoji vocabulary, consumer filter, front-loaded prose, and no-em-dash rule as the main file. Highlights are curated, not exhaustive; the tag table at the bottom gives anyone the exact git landmarks to inspect via `git show <tag>`.

When auditing an era to populate Highlights, work mechanically and avoid commit-by-commit transcription:

1. Read the shortlog for `<prev-tag>..<this-tag>` to gauge volume and
   contributors.
2. Skim milestone subjects with the local log command.
3. Find substantial features by churn with the local log/stat command.
4. Sample boundary docs: read `README.md` or `RELEASE_NOTES.md` at the era's
   tags with the local show command.
5. Synthesize 5 to 15 user-meaningful Highlights bullets. Apply the consumer filter.

Main file points at each era:

```markdown
## Archives

- [CHANGELOG_v2022.md](./CHANGELOG_v2022.md)
- [CHANGELOG_v2021.md](./CHANGELOG_v2021.md)
- [CHANGELOG_v2020.md](./CHANGELOG_v2020.md)
- ...
- [CHANGELOG_pre-v5.md](./CHANGELOG_pre-v5.md) for pre-tagging history back to 2012.
```

Do not duplicate archived content in the main file. Move it.

## Report

After editing, summarize:

Apply the shared colorful output vocabulary directly:

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

Do not commit by default. If the user explicitly asks to commit the changelog update, hand the scoped slice (`CHANGELOG.md` only) to `x-commit` (`.agents/skills/x-commit/SKILL.md`). Do not bundle the changelog edit with unrelated work.
