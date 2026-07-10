# Quality Policy

Use SOLID, SDP, DRY, DDD, KISS/FIST, and IDEALS. Keep modules structurally
sound, depend toward stable code, organize by domain language, and prefer
simple, fast, inexpensive, tiny solutions.

Ship only long-term, A++ solid work:

- no jank
- no cruft
- no accidental duplication
- no compatibility wrappers unless the user explicitly asks for staged migration
- no legacy leftovers
- no temporary bridges
- no god modules

Keep public and private boundaries crisp. Prefer small focused modules over
large catch-all modules. Update all callers to the clean API instead of
preserving old surfaces.

When reducing complexity, prefer deleting, merging, rehoming, or renaming stale
surfaces over adding another layer.
