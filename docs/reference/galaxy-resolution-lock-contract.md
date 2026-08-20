# Galaxy Resolution Lock Contract

## Status

The compiler and Galaxy package front door share
`nuis-galaxy-resolution-lock-v1` as their only project resolution-lock
protocol.

`nuis galaxy lock-deps` writes the canonical root `nuis.galaxy.lock`.
`verify-lock`, `sync-deps`, `project-status`, and project builds all compare
that file against the same loaded dependency closure. Locked compilation then
resolves from the synchronized content-addressed cache rather than rereading
the workspace provider. The old direct-bundle
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

Project `release-check` is stricter than the ordinary early-beta development
path: it requires both the committed root lock and its synchronized cache
before compilation or output creation. Single-file release checks are not
project package-resolution operations and do not require a project lock.

## Resolution Provider

`nuis-galaxy-resolution-provider-v1` is the static request/result boundary in
front of lock rendering. The currently registered provider kinds are:

* `workspace-layout`
* `locked-resolution-cache`
* `offline-layout`

The request binds the provider ID and kind plus sorted exact package
requirements. The result repeats that request identity and records selected
name, version, package ID, canonical relative path, direct/transitive status,
and requester set. `request_sha256` authenticates the normalized request;
`selection_sha256` also binds selected manifests, source/library identities,
and dependency facts. Physical provider roots are deliberately excluded, so
equivalent mirrors remain portable.

Selection is exact in this version. A transitive index edge may pin its own
exact version; an unpinned edge is accepted only when the provider exposes one
candidate for that name. Missing, duplicate, conflicting, ambiguous, ranged,
malformed, unregistered, traversal, and symlink-escape inputs fail closed.

The offline front door is:

```bash
cargo run -p nuis -- galaxy resolve-deps <project-dir|nuis.toml> \
  --provider-root <offline-layout> \
  [--provider-id <id>] [--provider-kind <kind>]
```

It routes through the same generic project loader, writes the canonical root
lock, and materializes the same SHA-256-addressed compile cache. It does not add
registry-specific resolution branches to `nuisc`.

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
byte-count and SHA-256 identity and writes a staged
`nuis-galaxy-resolution-cache-v1` provider under:

```text
.nuis/deps/galaxy/sha256/<resolution-digest>/
```

The addressed root contains a minimal resolver `index.toml`, every verified
package, `nuis.galaxy.cache.toml`, and a byte-identical canonical lock copy.
Only after the complete tree exists does sync replace the previous Galaxy
cache base. A verification or staging failure leaves the prior tree intact.
Files absent from the verified resolution are removed on successful
replacement.

Materialized packages use `<name>/<version>/module.toml` plus canonical
relative source and library paths. No absolute source or bundle path is stored
in the lock or materialized metadata.

`load_project_for_compile` selects the addressed provider whenever a root lock
exists. Resolution rereads package identities from that cache and then renders
the complete closure again; the resulting bytes must equal the committed lock.
Every resolved package root, manifest, source, and library path is canonicalized
and must remain beneath the addressed cache root, so index traversal and
symlink escape fail even when outside bytes happen to match the lock.
Missing caches, cache-lock drift, manifest/source/library drift, and index
changes therefore fail before project AST admission. Workspace-backed
`load_project` remains available to lock, sync, status, and development tools
that need to inspect or refresh the source provider.

## Current Boundary

An existing root lock is fail-closed: package, edge, policy, library-selection,
manifest, source, cache-index, or library-content drift rejects verification,
sync, and build admission. `project-status` reports the same resolution digest
and direct/transitive package identities. Project release admission requires
the lock and addressed cache; it does not silently create or refresh either.

The early-beta development workflow still permits a missing root lock so old
workspace examples can be checked and built before they are individually
locked. This fallback is never used when a root lock exists. Remote candidate
discovery, semantic-version range solving, registry trust metadata, transport, and cache
garbage collection remain later work. Those providers must produce the same
canonical lock and addressed cache rather than becoming new compiler-side
resolution authorities.
