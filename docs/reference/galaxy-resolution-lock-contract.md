# Galaxy Resolution Lock Contract

## Status

The compiler-side protocol is implemented as
`nuis-galaxy-resolution-lock-v1`.

Every project build writes `nuis.project.galaxy.lock` into its output metadata.
The build manifest records both that path and the snapshot's SHA-256 resolution
digest. Build-manifest verification reads the lock, verifies its canonical
payload hash, and rejects a digest mismatch.

This is the compiler-owned resolution snapshot. The root
`nuis.galaxy.lock` package-manager file still uses the older direct-bundle
format and has not yet migrated to this protocol.

## Bound State

The canonical payload binds:

* every direct and transitive Galaxy name, requested version, and package ID
* sorted `requested_by` and `depends_on` edges
* the library import policy and auto-injection decision
* auto-injection blockers
* `module.toml` byte length and SHA-256
* every declared source module's logical path, byte length, and SHA-256
* every library module's logical path, byte length, SHA-256, and selection mode
* aggregate dependency, source, library, and selected-library counts

Library selection is one of:

* `auto-injected`
* `explicit`
* `hidden`

The selected state is derived from the loaded project module origins, not from
a second package-manager interpretation.

## Portability

The snapshot contains only canonical forward-slash relative module paths.
Absolute paths, parent traversal, backslashes, empty path components, and the
record separator are rejected before rendering. Physical workspace paths are
used only while reading bytes for hashing and never enter the lock text.

The text contract requires UTF-8-compatible Rust strings, LF line endings, and
a final newline. Dependencies and set-like fields are sorted before rendering.
Rendering the same loaded project twice therefore produces identical bytes.

Content identity is captured from the same read that parses each Galaxy
manifest. Source and library identities are captured during resolution, and a
library file must still match that byte count and SHA-256 before it enters the
project AST. The lock writer renders these frozen identities and does not
reread mutable workspace files after compilation.

## Integrity

The header declares:

```toml
lock_schema = "nuis-galaxy-resolution-lock-v1"
digest_contract = "sha256"
resolution_sha256 = "sha256:<hex>"
payload_begin = true
```

`resolution_sha256` covers every byte after the canonical payload boundary.
The verifier checks schema, digest contract, line endings, digest shape, payload
hash, and required counts. Project-aware verification additionally renders the
current closure and requires byte-for-byte equality.

The build manifest carries the same digest independently. Artifact verification
therefore rejects both payload mutation and a lock/manifest digest mismatch.

## Current Boundary

The generated lock proves what a completed build consumed. It does not yet
prevent a later build from accepting a changed workspace and generating a new
snapshot, because the root package-manager lock is still the older bundle-only
format.

The next protocol tranche must migrate `nuis galaxy lock-deps`,
`verify-lock`, `sync-deps`, `nuis build`, and `project-status` to one canonical
root lock. Bundle provenance must use package/content identity rather than a
machine-specific absolute path. Remote solving, registry discovery, and a
content-addressed package cache remain later work.
