# Nuis Compiler Component Dispatch

`nuis-compiler-component-dispatch-v1` is the first runtime stage-driver
evidence emitted after a signed generation-two component transition. Its
machine-readable contract is
[nuis-compiler-component-dispatch-v1.toml](nuis-compiler-component-dispatch-v1.toml).

The protocol closes one narrow boundary: a signed `current` reproducible build
identity can now be resolved to exact compiler-image bytes and executed without
placing a physical path, timestamp, or registration ordinal in the receipt.
The signed stage1 candidate remains registered as `forward` but is not
executed by this rollback dispatch.

## Resolution

The resolver consumes the fully reverified generation-two transition and an
unordered inventory containing exactly two entries:

* the stage0 component-build record plus its compiler image
* the stage1-candidate component-build record plus its compiler image

Each build record recomputes its dependency closure, reproducible identity,
and exact record identity. Each supplied image must match the byte length and
SHA-256 carried by its record. The records must agree on component, bootstrap
subset, domain, unit, semantic handoff, dependency closure, and native output.

Selection is by signed component role and reproducible build identity, never
by inventory order or input filename. The candidate compiler-image identity is
also checked against the transition. Missing, duplicate, additional, wrong-
role, wrong-component, or hash-drifting entries fail closed.

## Execution

The frontdoor is:

```bash
nuis bootstrap-dispatch-component \
  <aggregate> <attestation> <attester-registry> \
  <attester-registry-sha256> <attestation-challenge-sha256> \
  <authorization> <authorizer-registry> <authorizer-registry-sha256> \
  <authorization-challenge-sha256> <active-state> <transition> \
  <transition-challenge-sha256> <current-component> <current-image> \
  <forward-component> <forward-image> <output>
```

Before image resolution, the command repeats the attester-registry pin,
component-owner registry pin, both challenges, authorization, active state,
generation-two signature, and predecessor lineage checks.

After resolution, the driver writes the verified current bytes into a private
create-new staging slot, rereads those bytes, marks the slot executable, and
runs the fixed `help` frontdoor request with closed stdin. The temporary image
is removed through scope cleanup. This prevents a mutable input pathname from
being swapped between verification and process launch.

The receipt is also create-new. It binds transition proof, selected and forward
build records, both compiler-image identities, the fixed request contract,
exit status, stdout/stderr lengths and hashes, verdict, and its own canonical
identity. It contains no physical path, staging name, timestamp, or captured
output bytes.

## Honest Boundary

This proves real execution of the generation-two `current` compiler frontdoor
while retaining the signed `forward` candidate. It does not yet route a source
project through that selected image, advance to generation three, execute the
forward candidate, rotate the owner key, or claim full self-hosting.

The repository integration extends the existing two-clean-build path through
attestation, independent authorization, active state, signed rollback, real
Mach-O stage0 execution, canonical receipt verification, and staging cleanup.
Artifact tests additionally reverse inventory order and reject image,
registration, or receipt drift.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_dispatch -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis parses_bootstrap_component_transition_commands -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate two_uncached_clean_candidates_bind_one_reproducibility_aggregate -j 1 -- --test-threads=1
```
