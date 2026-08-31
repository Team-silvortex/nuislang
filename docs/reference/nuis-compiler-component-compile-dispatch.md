# Nuis Compiler Component Compile Dispatch

`nuis-compiler-component-compile-dispatch-v1` is the first protocol that sends
a real project-form bootstrap request through compiler-image bytes selected by
a signed component transition. Its machine-readable contract is
[nuis-compiler-component-compile-dispatch-v1.toml](nuis-compiler-component-compile-dispatch-v1.toml).

It is a versioned companion to
[dispatch v1](nuis-compiler-component-dispatch.md). Dispatch v1 remains the
small fixed-`help` process probe; compile dispatch v1 adds a real
`bootstrap-build` without changing the older receipt.

## Canonical Request

The exact stage0 component-build record selected as generation-two `current`
is also the request template. That record already canonically binds the frozen
bootstrap subset, component coordinate, compiler image, dependency closure,
stage handoff, native output, and cache-stable reproducible identity.

The project input and absent output directory are operational carriers. They
are deliberately not request identities and never enter the receipt. A source,
manifest, Galaxy lock, Nustar registration, or standard-library change instead
changes the result dependency closure and causes request verification to fail.

## Frontdoor

```bash
nuis bootstrap-dispatch-compile \
  <aggregate> <attestation> <attester-registry> \
  <attester-registry-sha256> <attestation-challenge-sha256> \
  <authorization> <authorizer-registry> <authorizer-registry-sha256> \
  <authorization-challenge-sha256> <active-state> <transition> \
  <transition-challenge-sha256> <current-component> <current-image> \
  <forward-component> <forward-image> <project> <fresh-build-output> <output>
```

The command first replays the complete attestation, authorization, active-state,
and generation-two transition lineage. It fully reads both component records
and their bound disk payloads, byte-verifies both images, and resolves the
unordered inventory by signed identity.

Only the selected current bytes are copied into a private create-new staging
slot. The slot is reread, made executable, invoked as `bootstrap-build` with
closed stdin, and removed by scope cleanup. The fresh result record and every
record-bound payload are then read back from disk before receipt construction.

## Artifact Identity

The compiled artifact embeds its build manifest. That manifest intentionally
contains operational output paths, a generation time, and cache status, so two
honest builds may have different raw container lengths and SHA-256 values.
Compile dispatch does not ignore this difference: both raw identities remain
in the receipt as audit evidence.

For semantic comparison, both containers are decoded and hashed under
`nuis-compiled-artifact-semantic-identity-v1`. The hash includes packaging and
target ABI metadata, the executable envelope, lifecycle contract, native blob,
and ordered host objects. It excludes only the embedded manifest source and its
byte count. Dependency closure, stage handoff, native output, and reproducible
build identity must also match independently.

## Receipt

The create-new canonical receipt binds:

* the generation-two transition proof
* the exact request/current record and compiler image
* the retained forward record and compiler image
* request and result dependency, handoff, native, and reproducible identities
* both raw compiled-artifact identities and their shared semantic identity
* process exit status and stdout/stderr lengths and hashes
* the `current-compiled-forward-retained` verdict and receipt identity

It carries no project path, output path, staging name, timestamp, captured
output bytes, or mutable registration ordinal.

## Honest Boundary

The repository integration now proves that the generation-two current stage0
image can rebuild the canonical structural-projection component through the
real Nuis frontdoor while retaining the candidate as forward. Cache/path noise
changes the raw container but not its semantic identity.

The forward image is still a specialized Nuis structural producer, not a full
`bootstrap-build` frontdoor. This protocol therefore does not authorize a
generation-three switch, claim the candidate can compile projects, or claim
self-hosting. That capability is the next stage-driver gap.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_dispatch -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis parses_bootstrap_component_transition_commands -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate two_uncached_clean_candidates_bind_one_reproducibility_aggregate -j 1 -- --test-threads=1
```
