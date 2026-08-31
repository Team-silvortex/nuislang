# Nuis Compiler Candidate Direct Compile Capability

`nuis-compiler-candidate-compile-capability-v2` is the first execution proof
where the production-bound stage1 candidate completes a compiler stage without
invoking the stage0 compiler provider.

The scope is deliberately precise: this is direct ownership of the canonical
five-stage front-end handoff and its normalized result. It is not yet fresh
project parsing, native object emission, final linking, or self-hosting.

## Frontdoor

```text
nuis bootstrap-candidate-direct-compile \
  <candidate-build-root> \
  <front-end-result-output> \
  <capability-output>
```

The candidate root must come from `nuis bootstrap-candidate-build`. Both
outputs are create-new files. Runtime paths never enter either canonical
artifact.

## Direct Ownership

The command deep-verifies candidate production v11, its exact adapter, the
candidate handoff, all five `source -> tokens -> AST -> NIR -> YIR` payloads,
and both AST/NIR transformation checkpoints. It then executes a private reread
copy of the adapter with exactly those five payload arguments.

The child receives closed stdin and a cleared process environment. It gets no
provider path, project path, build directory, shell command, or native-output
request. The three-argument delegated `bootstrap-build` route therefore cannot
be selected by this invocation.

The Nuis candidate emits the existing 53-line scalar protocol as a first-class
`nuis.compiler-candidate-front-end-result`. `nuis-artifact` parses it in exact
order and independently rebuilds the expected folds, token pagination, bundle,
and AST/NIR checkpoint lanes from production evidence. A raw output hash alone
is not sufficient.

## Capability Boundary

The path-free v2 capability binds:

* the stage1 candidate record, reproducible identity, producer, and image;
* candidate production v11 and its exact adapter bytes;
* the candidate handoff bundle and aggregate identity of all five inputs;
* the canonical front-end result protocol, bytes, hash, and bundle fold;
* exit `0`, empty stderr, cleared environment, and absent runtime provider;
* explicit false native materialization, replacement, and selection authority.

```toml
provider_dependency_required = false
direct_stage1_compile = true
native_materialization = false
replacement_authorized = false
selection_authorized = false
```

## Honest Next Step

Capability v1 remains valid evidence for the delegated full rebuild, and
preselection v1 remains the signed generation-three admission over that v1
dependency. V2 does not rewrite either artifact. Candidate successor v1 now
deep-verifies both branches and signs this direct proof under the continuing
component-owner key while keeping native materialization and final selection
false.

The next technical boundary is fresh source-to-front-end ownership, then
candidate-owned NIR/YIR-to-native materialization. Until those close, the
stage0 lineage remains provenance for the seed handoff even though it is not a
runtime provider for this direct execution.

The machine-readable contract is
[nuis-compiler-candidate-direct-compile-capability-v2.toml](nuis-compiler-candidate-direct-compile-capability-v2.toml).
