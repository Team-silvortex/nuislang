# Nuis Compiler Candidate Successor

`nuis-compiler-candidate-successor-v1` is the signed generation-three
strengthening record that joins the immutable delegated preselection chain to
the direct stage1 front-end capability.

It does not rewrite generation two, capability v1, or preselection v1. It lets
the continuing component owner sign one exact capability-v2 result into the
same generation-three review while keeping native materialization and final
selection closed.

## Frontdoor

```text
nuis bootstrap-sign-candidate-successor \
  <aggregate> <attestation> <attester-registry> \
  <attester-registry-sha256> <attestation-challenge-sha256> \
  <authorization> <authorizer-registry> <authorizer-registry-sha256> \
  <authorization-challenge-sha256> <active-state> <transition> \
  <transition-challenge-sha256> <candidate-build-root> \
  <candidate-compile-capability-v1> <preselection> \
  <preselection-challenge-sha256> <direct-compile-capability-v2> \
  <front-end-result> <successor-challenge-sha256> \
  <authorizer-id> <environment-id> <successor-id> <output>
```

The command uses the existing component-owner signing-key environment. The
key, identity, and environment must exactly match the owner already signed
into generation two and preselection v1. The output is create-new, canonical,
path-free, and reread before success.

## Verification Chain

The signer does not trust either capability receipt in isolation. It first
replays attestation, replacement authorization, active state, generation-two
transition, both pinned registries, and every challenge and signature. It then
deep-verifies candidate production v11, delegated capability v1, and the exact
preselection source.

The direct side independently verifies the stage1 candidate record, production
proof, adapter bytes, five handoff payloads, AST/NIR transformations, canonical
53-line result, and capability-v2 proof. The direct result must be exactly the
same bytes named by capability v2, and the candidate and production identities
must match the preselection lineage.

## Signed Identity

The successor signs:

* the complete preselection-v1 source, proof, and identifier;
* candidate record, reproducible build, producer, and image identities;
* production-v11 protocol and proof;
* the complete direct-capability-v2 source, proof, driver, provider, and input identity;
* the canonical front-end result protocol, bytes, SHA-256, and bundle fold;
* the continuing component-owner identity, public key, and fresh challenge.

The relation is `same-generation-capability-strengthening-v1`: target
generation remains `3`. This is not a component transition and does not mutate
the selected compiler.

## Authority Boundary

Every valid record contains:

```toml
provider_dependency_required = false
direct_stage1_compile = true
fresh_source_compile = false
native_materialization = false
replacement_authorized = false
selection_authorized = false
preselection_authorized = true
successor_authorized = true
```

`successor_authorized` means only that the component owner admits the stronger
direct front-end evidence into generation-three review. It cannot be used as a
native image, a replacement authorization, or a final selector.

## Downstream Boundary

The successor remains immutable and therefore still carries
`fresh_source_compile = false`. The separate downstream
`nuis-compiler-candidate-fresh-source-capability-v1` now binds this successor's
canonical source and proof identity to one candidate-owned 56-byte source-to-
token/AST/NIR/YIR execution without a preexisting stage0 handoff. It inherits
no signing, replacement, or selection authority from this record.

The next weakest boundary is candidate-owned native object materialization for
that frozen source snapshot. See
[Nuis Compiler Candidate Fresh-Source Capability](nuis-compiler-candidate-fresh-source-capability.md).

The machine-readable contract is
[nuis-compiler-candidate-successor-v1.toml](nuis-compiler-candidate-successor-v1.toml).
