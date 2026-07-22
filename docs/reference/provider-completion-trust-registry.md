# Provider Completion Trust Registry

The provider-completion signature verifier consumes an external static trust
registry. The registry is deliberately not embedded in an NSB artifact: an
artifact cannot declare the key that makes its own producer trustworthy.

Set `NUIS_PROVIDER_COMPLETION_TRUST_REGISTRY` to the registry path for both
Nsdb and Nuis verification. `NUIS_PROVIDER_COMPLETION_TRUSTED_PUBLIC_KEYS`
remains a compatibility fallback only when no registry path is configured.

```toml
protocol = "nuis-provider-completion-trust-registry-v1"
generation = 2

[[keys]]
key_id = "ed25519:sha256:<sha256-of-32-public-key-bytes>"
public_key_hex = "<64-lowercase-or-uppercase-hex-characters>"
status = "revoked"

[[keys]]
key_id = "ed25519:sha256:<new-key-id>"
public_key_hex = "<new-public-key-hex>"
status = "active"
```

`generation` must be nonzero and should increase for every rotation or
revocation. Key IDs must be unique and must match the SHA-256 digest of the
decoded Ed25519 public key. Every key status must be either `active` or
`revoked`.

The whole registry fails closed when its protocol, generation, key encoding,
key ID, uniqueness, or status is invalid. A configured registry path also
disables the inline compatibility fallback, so a missing or malformed external
registry cannot silently weaken verification.

Verification statuses distinguish:

* `signature-verified`
* `signature-key-revoked`
* `signature-key-untrusted`
* `signature-trust-registry-invalid`
* malformed, missing, unsupported, or mismatched signatures

Nsdb and Nuis implement registry parsing independently. Nsld only transports
the resulting signature identity and status as neutral final-output metadata.

## Generation anchor

Rollback protection persists a separate
`nuis-provider-completion-trust-anchor-v1` file. By default it is stored beside
the registry with `.anchor` appended to the registry path. Set
`NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR` to place it in a separately protected
location.

The anchor records the registry protocol, highest accepted generation, and the
SHA-256 hash of the complete registry file. Verification rejects a lower
generation as `signature-trust-registry-rollback` and rejects different content
at the same generation as `signature-trust-registry-fork`.

Anchor creation and upgrades use a synchronized temporary file, file sync,
atomic rename, directory sync, and a cross-process create-new lock. Nsdb and
Nuis implement the anchor reader/writer independently and share only the file
protocol. The lock uses
`nuis-provider-completion-trust-anchor-lock-v1` and records its owner PID,
creation timestamp, and owner token. Locks older than 30 seconds are recovered;
the token prevents a recovered lock's former owner from deleting its successor.
Fresh malformed locks fail closed; malformed locks older than the lease are
recovered using their filesystem modification timestamp.

Initialization also persists
`nuis-provider-completion-trust-anchor-marker-v1`. It defaults to
`<anchor>.initialized`; set `NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR_MARKER` to
place it in a separately protected location. A marker with a missing anchor is
treated as anchor deletion and fails closed. Existing anchor-v1 deployments
without a marker migrate on their next successful verification, which also
recovers the safe crash window between initial anchor and marker writes. The
marker binds the anchor protocol, registry protocol, initial generation, and
initial registry hash.

`NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR_BACKEND` selects the storage adapter.
The defaults are:

* `file-v1` (default): uses `<registry path>.anchor` and `<anchor>.initialized`
  with optional overrides.
* `protected-file-v1`: requires explicit
  `NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR`,
  `NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR_MARKER`,
  `NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR_ROOT`, and
  `NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR_MARKER_ROOT`. The two roots must be
  different absolute Unix directories with mode `0700`; each configured file
  must be a direct child of its declared root. Anchor and marker writes use
  owner-only `0600` files, protected temporary files use create-new semantics,
  and lock files are also `0600`. Existing symlinks, non-regular files,
  group/other-accessible roots or files, relative paths, and root/path mismatch
  fail closed. This filesystem adapter is unavailable on non-Unix hosts until
  an equivalent ACL verifier exists.

Unknown adapters fail closed as `signature-trust-anchor-invalid`; this contract
leaves room for future Keychain, TPM, or protected-directory adapters without
coupling the signature verifier to one operating system. Protected deployment
should use host-level access control to protect registry, anchor, and marker with
separate directories, and avoid sharing symlinked locations. Deleting both a
registry-adjacent `file-v1` anchor and marker cannot reset an active
`protected-file-v1` deployment because neither independent verifier consults
those ordinary paths. Deleting both protected files with authority over both
roots can still reset this filesystem-only adapter; preventing privileged root
compromise requires a later Keychain, TPM, or OS service backend.
