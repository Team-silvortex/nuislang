# Nuis Compiler Component Reproducibility

`nuis-compiler-component-reproducibility-v1` binds two clean productions of
the same Nuis-written stage1 candidate into one canonical aggregate. The v2
successor preserves those exact bytes while directly binding each root's
selected-representation sidecar. The machine-readable contracts are
[v1](nuis-compiler-component-reproducibility-v1.toml) and
[v2](nuis-compiler-component-reproducibility-v2.toml).

This protocol closes the first repeated-build evidence loop. It does not make
the current bounded token-summary transform a complete compiler stage, and it
never authorizes replacing stage0.

## Frontdoor

Run the two-build gate with:

```bash
nuis bootstrap-reproducibility path/to/project path/to/empty-output
```

The output path must be absent or a real empty directory. Symlinks, files, and
non-empty directories fail before compilation. The frontdoor creates exactly
`clean-build-0/` and `clean-build-1/`, runs the existing candidate production
chain in each, then writes the unchanged
`nuis.compiler-component-reproducibility.toml` and its
`nuis.compiler-component-reproducibility-v2.toml` successor beside them. Both
reports are built before either top-level report is persisted.

Both stage0 compilations use an explicit compile-cache bypass. Neither reads
nor writes the normal project compile cache, and the aggregate reader requires
each bound build manifest to retain `compile_cache_status = "bypass"`.
Dependency-resolution inputs may still come from their verified project and
Galaxy stores; v1 is a clean compiler-production proof, not an offline package
refetch protocol.

## Identity Layers

Each run retains its exact stage0 record, candidate record, production proof,
stage-transformation manifest, and thirteen-comparison differential report
identities. Candidate production v11 binds the transformation, semantic
differential, and producer-neutral handoff v2 selection, so the aggregate
covers them transitively. These exact audit
identities may differ because build manifests record their physical output
locations.

The aggregate separately requires these path-neutral values to be identical:

* stage0 `reproducible_build_sha256`
* candidate `reproducible_build_sha256`
* candidate compiler-image SHA-256
* native-output SHA-256
* `equivalent-awaiting-authorization` differential verdict

Every run must independently reach `13/13`. The aggregate verdict is
`reproducible-equivalent-awaiting-authorization`, with
`replacement_authorized = false`.

V2 first rebuilds v1 from both roots. It then reparses and independently
rebuilds each root's representation sidecar against the exact stage0 and
candidate component records. Each run binds its clean-root witness, production
proof, base `13/13` report, canonical sidecar byte length and SHA-256, internal
representation report SHA-256, and `2/2` verdict. The aggregate therefore
binds four equivalent selected representations.

The two sidecar hashes are not required to be equal. Each sidecar intentionally
binds root-specific audit record hashes even when path-neutral reproducible
identities and semantics agree. V2 proves individual binding and replay, not
false byte identity between different audit roots.

The aggregate stores no physical path or timestamp. Each clean root receives a
distinct SHA-256 witness derived by the local frontdoor. That witness prevents
two run slots from sharing one identity and records the procedural invocation;
it is deliberately labelled as having no independent attester trust.

## Failure Boundary

The artifact reader canonicalizes both supplied roots, rejects aliases, reads
their complete component payloads, checks the cache-bypass manifests, rebuilds
both differential reports, and then rebuilds the aggregate. Component,
adapter, transformation word or file, production proof, report, native binary,
witness, stable identity, sidecar byte/report hash, or either aggregate
mutation fails closed. Rejecting a v2 sidecar does not mutate v1.

The first real run exposed a process-global `?` expansion counter: a second
compile generated different NIR temporary names even though its source and YIR
were unchanged. The counter is now thread-local and reset at each project
lowering boundary, with a same-thread repeated-lowering regression test.

## External Attestation

The separate
[component attestation v1](nuis-compiler-component-attestation.md) frontdoors
can now sign this exact aggregate after rereading both roots and verify it with
a fresh challenge, strict Ed25519 signature, environment-scoped attester key,
and caller-owned trust-registry SHA-256 pin. The attestation remains a separate
artifact and preserves `replacement_authorized = false`.

Attestation v1 continues to consume the unchanged v1 file. The local v2
successor grants no trust or replacement authority and does not retroactively
change a generation-one signature. A future remote v2 attestation must use a
separate versioned protocol.

The downstream
[replacement authorization v1](nuis-compiler-component-replacement-authorization.md)
frontdoors now consume this unchanged aggregate and attestation through a
second pinned registry and distinct component-owner key. Authorization does
not mutate this report or retroactively grant authority to its attester.

## Honest Boundary

V1 proves two fresh compiler productions under one local frontdoor.
Reproducibility v2 closes the bounded local selected-representation binding
while keeping v1 signature-compatible. Handoff v2 now binds a byte-different
reversible NIR representation and its semantic
equivalence, while attestation v1 supplies the cryptographic external-witness
boundary. The repository now retains one separately operated Linux amd64
attester generation with two cache-bypassed `13/13` runs, an exact registry
pin, and a no-private-key verification regression. Cryptography still cannot
prove physical-machine independence. Canonical active-state v1 closes genesis
authorization consumption, and transition v2 signs exact stage0 restoration
plus candidate forward retention. Dispatch v1 now executes the exact current
image while retaining the forward candidate. Candidate preselection v1 now
owner-signs the exact delegated capability and provider dependency. Direct
capability v2 and candidate successor v1 now bind provider-free front-end
execution into generation three. Candidate-owned equivalent Nsld input is now
proven; general source coverage, candidate-owned native object bytes, remote v2
attestation, and final generation-three replacement remain open.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_reproducibility -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_reproducibility_v2 -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_attestation -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_replacement -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_remote_attestation_evidence -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuisc --lib repeated_same_thread_lowering_resets_try_expansion_names -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate two_uncached_clean_candidates_bind_one_reproducibility_aggregate -j 1 -- --test-threads=1
```
