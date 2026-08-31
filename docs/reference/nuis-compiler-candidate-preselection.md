# Nuis Compiler Candidate Preselection

`nuis-compiler-candidate-preselection-v1` is the signed boundary between a
measured candidate compile capability and any future generation-three
selection. It lets the generation-two component owner acknowledge one exact
candidate, production proof, compile capability, and stage0 provider
dependency without rewriting generation two.

This is intentionally a preselection rather than a transition. It grants
neither replacement authority nor final selection authority, and it records
that direct stage1 compilation is not yet present.

## Frontdoor

```text
nuis bootstrap-preselect-candidate \
  <aggregate> <attestation> <attester-registry> <attester-registry-sha256> \
  <attestation-challenge-sha256> <authorization> \
  <authorizer-registry> <authorizer-registry-sha256> \
  <authorization-challenge-sha256> <active-state> <transition> \
  <transition-challenge-sha256> <candidate-build-root> \
  <candidate-compile-capability> <preselection-challenge-sha256> \
  <authorizer-id> <environment-id> <preselection-id> <output>
```

The command uses the existing component-owner signing key environment. The
key must resolve to the same owner identity, environment, and Ed25519 public
key already signed into generation two. The output is create-new and is
reread before success.

## Deep Verification

The frontdoor does not trust the capability receipt in isolation. It first
replays the complete attestation, replacement authorization, active-state,
generation-two transition, pinned registry, challenge, lineage, and signature
checks. It then deeply reads the candidate build root, including stage0,
candidate execution, handoff payloads, production v11, and the exact
production-bound adapter.

The production and capability sources must parse and rerender byte-for-byte.
Their identities must agree with the exact stage0 and candidate component
records selected by generation two. The capability result must retain the
stage0 reproducible identity, and its provider image must be the stage0
compiler image.

## Signed Identity

The path-free record signs:

* the complete predecessor transition source and proof;
* stage0 provider record, reproducible build, and compiler image identities;
* candidate record, build, producer, and compiler image identities;
* complete production-v11 and capability-v1 source and proof identities;
* compiled-artifact semantic and candidate-driven result identities;
* the explicit provider dependency, fresh challenge, and owner key identity.

Runtime paths are used only to locate inputs. No project, candidate-root,
registry, capability, transition, or output path enters canonical fields.

## Authority Boundary

Every valid record contains:

```toml
provider_dependency_required = true
direct_stage1_compile = false
replacement_authorized = false
selection_authorized = false
preselection_authorized = true
```

`preselection_authorized` means only that the component owner admits this
exact evidence into generation-three review. A future transition still has to
consume a stronger direct stage1 compile proof before it can change the
selected compiler generation.

## Honest Next Step

The weakest remaining bootstrap boundary is now execution rather than trust
binding: the production-bound candidate still exact-execs a verified stage0
provider. The next versioned capability must prove one canonical compile whose
front-end stages are owned directly by stage1, while preserving capability v1,
this preselection, and generation two as immutable predecessors.

The machine-readable contract is
[nuis-compiler-candidate-preselection-v1.toml](nuis-compiler-candidate-preselection-v1.toml).
