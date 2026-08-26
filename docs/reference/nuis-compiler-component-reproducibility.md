# Nuis Compiler Component Reproducibility

`nuis-compiler-component-reproducibility-v1` binds two clean productions of
the same Nuis-written stage1 candidate into one canonical aggregate. Its
machine-readable contract is
[nuis-compiler-component-reproducibility-v1.toml](nuis-compiler-component-reproducibility-v1.toml).

This protocol closes the first repeated-build evidence loop. It does not make
the current identity projection relay a complete compiler stage, and it never
authorizes replacing stage0.

## Frontdoor

Run the two-build gate with:

```bash
nuis bootstrap-reproducibility path/to/project path/to/empty-output
```

The output path must be absent or a real empty directory. Symlinks, files, and
non-empty directories fail before compilation. The frontdoor creates exactly
`clean-build-0/` and `clean-build-1/`, runs the existing candidate production
chain in each, then writes
`nuis.compiler-component-reproducibility.toml` beside them.

Both stage0 compilations use an explicit compile-cache bypass. Neither reads
nor writes the normal project compile cache, and the aggregate reader requires
each bound build manifest to retain `compile_cache_status = "bypass"`.
Dependency-resolution inputs may still come from their verified project and
Galaxy stores; v1 is a clean compiler-production proof, not an offline package
refetch protocol.

## Identity Layers

Each run retains its exact stage0 record, candidate record, production proof,
and thirteen-comparison differential report identities. These exact audit
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

The aggregate stores no physical path or timestamp. Each clean root receives a
distinct SHA-256 witness derived by the local frontdoor. That witness prevents
two run slots from sharing one identity and records the procedural invocation;
it is deliberately labelled as having no independent attester trust.

## Failure Boundary

The artifact reader canonicalizes both supplied roots, rejects aliases, reads
their complete component payloads, checks the cache-bypass manifests, rebuilds
both differential reports, and then rebuilds the aggregate. Component,
adapter, production proof, report, native binary, witness, stable identity, or
aggregate mutation fails closed.

The first real run exposed a process-global `?` expansion counter: a second
compile generated different NIR temporary names even though its source and YIR
were unchanged. The counter is now thread-local and reset at each project
lowering boundary, with a same-thread repeated-lowering regression test.

## Honest Boundary

V1 proves two fresh compiler productions under one local frontdoor. It does
not prove independent-machine diversity, trusted remote attestation, or a
non-identity Nuis transformation. Those remain separate upgrades, followed by
an independently versioned reversible replacement protocol.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_reproducibility -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuisc --lib repeated_same_thread_lowering_resets_try_expansion_names -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate two_uncached_clean_candidates_bind_one_reproducibility_aggregate -j 1 -- --test-threads=1
```
