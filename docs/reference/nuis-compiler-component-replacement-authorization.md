# Nuis Compiler Component Replacement Authorization

`nuis-compiler-component-replacement-authorization-v1` is the first explicit
permission boundary between reproducible stage1 evidence and an eventual
component switch. It does not alter the attestation protocol: every attestation
still has `replacement_authorized = false`.

The machine contract is
[replacement authorization v1](nuis-compiler-component-replacement-authorization-v1.toml).

## Separate Authority

Replacement permission uses a second registry protocol and trust scope:

* attesters prove that two clean builds and their candidate lineage were seen;
* component owners may authorize one exact transition;
* the two registries have independent caller-owned SHA-256 pins;
* the authorizer identity and Ed25519 public key must both differ from the
  attester identity and key;
* an authorizer registry entry is scoped to one exact component ID.

This means adding an attester to the reproducibility registry cannot grant
replacement authority, even if that attester produces a valid signature.

## Genesis Transition

Version 1 signs only a generation-one `activate-candidate` transition. It binds
the canonical reproducibility aggregate, canonical attestation, their complete
hash lineage, and these state identities:

```text
from     = stage0 reproducible build
to       = candidate reproducible build
rollback = stage0 reproducible build
```

`from` and `to` must differ. The rollback target must exactly equal the prior
stage0 build. The record also binds a fresh authorization challenge, component
and authorization IDs, authorizer role, candidate compiler image, and native
output. It is created with `create_new` and cannot silently overwrite an older
authorization.

The authorization record says permission exists; it does not itself apply the
transition. The separate
[active-component state v1](nuis-compiler-component-active-state-v1.toml)
consumes a verified record without changing these v1 bytes.

## Signing Frontdoor

The component owner provisions a canonical authorizer registry and both
registry pins through trusted channels, then runs:

```bash
NUIS_COMPILER_REPLACEMENT_SIGNING_KEY_HEX=<private-key> \
  nuis bootstrap-authorize-component-replacement \
  <aggregate> <attestation> \
  <attester-registry> <attester-registry-sha256> \
  <attestation-challenge-sha256> \
  <authorizer-registry> <authorizer-registry-sha256> \
  <authorization-challenge-sha256> \
  <authorizer-id> <environment-id> <authorization-id> <output>
```

Before signing, the frontdoor verifies the original attestation under its own
pin and challenge. It then builds the authorization, resolves the signing key
through the separate component-scoped registry, self-verifies the complete
record, and finally writes it without replacement. The private key is accepted
only through the dedicated environment variable.

## Verification Frontdoor

Verification requires both trust domains again:

```bash
nuis bootstrap-verify-component-replacement \
  <aggregate> <attestation> \
  <attester-registry> <attester-registry-sha256> \
  <attestation-challenge-sha256> <authorization> \
  <authorizer-registry> <authorizer-registry-sha256> \
  <authorization-challenge-sha256>
```

Wrong challenges, changed pins, revoked keys, component drift, lineage drift,
same-role keys, transition drift, proof changes, and signature changes fail
closed.

## Active-State Consumer

After verification, the same pinned inputs can derive one canonical state:

```bash
nuis bootstrap-activate-component \
  <aggregate> <attestation> \
  <attester-registry> <attester-registry-sha256> \
  <attestation-challenge-sha256> <authorization> \
  <authorizer-registry> <authorizer-registry-sha256> \
  <authorization-challenge-sha256> <active-state-output>
```

The frontdoor repeats both trust-domain checks before creating a new file. Its
provider-neutral selector resolves `active` to the stage1 candidate build and
`rollback` to the exact stage0 build retained by the authorization. State,
authorization, and attestation files are never overwritten. Re-deriving the
same authorization yields the same state identity rather than a second logical
transition.

## Honest Boundary

The repository regression combines the checked-in Linux amd64 generation-one
attestation with a temporary local component-owner key. This proves protocol
composition and role separation, not an independently operated release-owner
ceremony.

The repository now proves a canonical active-component state consumer with an
exact stage0 rollback selection. A signed generation-two rollback or forward
transition, runtime execution through the selected compiler image, threshold
authorization, and checked-in operational authorizer evidence remain open.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_replacement -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_active_state -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis parses_bootstrap_component_replacement_commands -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_remote_attestation_evidence -j 1 -- --test-threads=1
```
