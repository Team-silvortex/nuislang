# Galaxy Resolution Lock Contract

## Status

The compiler and Galaxy package front door share
`nuis-galaxy-resolution-lock-v1` as their only project resolution-lock
protocol.

`nuis galaxy lock-deps` writes the canonical root `nuis.galaxy.lock`.
`verify-lock`, `sync-deps`, `project-status`, and project builds all compare
that file against the same loaded dependency closure. The old direct-bundle
lock, absolute bundle paths, and FNV-only project dependency authority have
been removed. Local `pack`, `publish-local`, and `install-local` remain a
separate package-cache surface and do not define compiler resolution.

Every project build writes `nuis.project.galaxy.lock` into its output metadata.
The build manifest records both that path and the snapshot's SHA-256 resolution
digest. Build-manifest verification reads the lock, verifies its canonical
payload hash, and rejects a digest mismatch.

When a committed root lock exists, build admission verifies it before cache
restore, lowering, linking, or output creation. The generated build lock is
byte-identical to that verified canonical resolution.

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

Library selection is `auto-injected`, `explicit`, or `hidden`. Selection comes
from loaded project module origins, not from a second package-manager
interpretation.

## Portability

The snapshot contains canonical forward-slash relative module paths only.
Absolute paths, parent traversal, backslashes, empty path components, and the
record separator are rejected before rendering. Physical workspace paths are
used only while reading bytes and never enter lock text.

The text contract requires UTF-8-compatible Rust strings, LF line endings, and
a final newline. Dependencies and set-like fields are sorted. Rendering the
same loaded project twice therefore produces identical bytes.

Content identity is captured from the same read that parses each Galaxy
manifest. Source and library identities are captured during resolution, and a
library file must still match that byte count and SHA-256 before entering the
project AST. Lock rendering uses these frozen identities rather than rereading
mutable workspace files after compilation.

## Integrity

The header declares:

```toml
lock_schema = "nuis-galaxy-resolution-lock-v1"
digest_contract = "sha256"
resolution_sha256 = "sha256:<hex>"
payload_begin = true
```

`resolution_sha256` covers every byte after the canonical payload boundary.
The verifier checks schema, digest contract, line endings, digest shape,
payload hash, and required counts. Project-aware verification also renders the
current closure and requires byte-for-byte equality.

The build manifest carries the same digest independently. Artifact verification
therefore rejects payload mutation and a lock/manifest digest mismatch.

## Materialization

`nuis galaxy sync-deps` first performs project-aware lock verification. It then
re-reads every manifest, source module, and library module through its frozen
byte-count and SHA-256 identity, writes a staged tree under
`.nuis/deps/galaxy`, copies the canonical root lock into that tree, and only
then replaces the previous materialization. A verification or staging failure
leaves the prior tree intact. Files absent from the verified resolution are
removed on successful replacement.

Materialized packages use `<name>/<version>/module.toml` plus canonical
relative source and library paths. No absolute source or bundle path is stored
in the lock or materialized metadata.

## Current Boundary

An existing root lock is fail-closed: package, edge, policy, library-selection,
manifest, source, or library-content drift rejects verification, sync, and
build admission. `project-status` reports the same resolution digest and
direct/transitive package identities.

The early-beta development workflow still permits a missing root lock so old
workspace examples can be checked and built before they are individually
locked. Release admission does not yet expose a dedicated required-lock mode.
The resolver also still reads the workspace closure and compares it with the
lock; it does not yet consume a content-addressed synchronized package cache as
its primary source. Required-lock release policy, cache-owned resolution,
remote solving, and registry discovery remain later work.
