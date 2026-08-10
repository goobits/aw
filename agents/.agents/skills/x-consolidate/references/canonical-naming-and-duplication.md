# Canonical Naming And Semantic Duplication

Load this reference when naming, module ownership, semantic duplication,
concept migration, or cross-surface vocabulary is material to the task.

## Canonical Concept Rule

- Give one abstraction one canonical owner and one canonical vocabulary.
- Use the same term for the same concept across code, schemas, errors, logs,
  tests, and docs.
- Translate a canonical term only at an external protocol boundary. Keep that
  translation explicit and local to the boundary.
- Finish renames repository-wide. Remove retired synonyms, aliases, wrappers,
  tests, and docs unless an approved staged migration still requires them.

## Naming Grammar

- Name operations as `verb + object + optional qualifier`. Render the grammar
  idiomatically per language: `resolveSymbol` in TypeScript and
  `resolve_symbol` in Rust or Python.
- Prefix state and capability predicates with `is`, `has`, `can`, or
  `supports`. Reserve `should` for an actual policy decision, not stored state.
- Use language-native constructors and factories: `new` or `from_*` in Rust;
  class construction, `createThing`, or `fromThing` in TypeScript. Reserve
  `on*` for event handlers.
- Keep stable role suffixes consistent: `Request`, `Plan`, `Receipt`,
  `Resolver`, `Adapter`, and `Backend`. Reuse an established domain suffix
  instead of inventing a near-synonym.
- Use fixed directional pairs: `source`/`destination`, `before`/`after`,
  `incoming`/`outgoing`, and `readSet`/`writeSet`, adapted only for language
  casing.
- Give parallel operations the same semantic stem across languages and layers.
- Prefer complete, searchable domain terms over abbreviations.

## Module Ownership

- Give each module one responsibility and a name that predicts its contents.
- Split independent responsibilities at stable domain boundaries; do not split
  solely because a file is long.
- Avoid bucket owners such as `utils`, `helpers`, `common`, and `manager`.
  Name the actual domain operation or role instead.
- Name public and private files according to local policy without hiding a
  broad responsibility behind a technically valid filename.

## Semantic Duplication Check

Before adding or preserving a type, function, module, schema, test owner,
fixture, or helper, compare it with existing candidates by:

- data shape
- callers and consumers
- responsibility
- role suffix
- operation family
- observable behavior

Treat matching semantics as a duplication signal even when spellings differ.
Identify the canonical owner before adding another surface. Similar data alone
does not require consolidation when lifecycle, security, performance, or domain
ownership differs materially; name that distinction explicitly.

## Completion Check

- Search exact names, stems, aliases, schema fields, error codes, log fields,
  tests, and docs before declaring a vocabulary migration complete.
- Reuse an existing vocabulary or architecture check when one protects a
  credible regression. Do not add static-copy tests merely to police wording.
- Report the canonical owner, canonical term, retired terms, boundary
  translations, and any intentionally distinct owner that remains.
