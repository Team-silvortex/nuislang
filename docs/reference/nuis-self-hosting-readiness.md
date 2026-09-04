# Nuis Self-Hosting Readiness

This reference defines the executable control boundary for the formal
`stage0 -> stage1` compiler migration, active from `beta-0.10.0`. The checked-in
machine-readable v2 source is
[nuis-self-hosting-readiness.toml](nuis-self-hosting-readiness.toml).
Completed stage2-equivalent compiler ownership remains a later
`gamma-0.5.*` through `gamma-0.10.*` closure window.

## Roadmap Is Not Readiness

`developer-system/dev-tensor/self-hosting-phase-roadmap` is `stable/100`
because the schedule is agreed and versioned. It does not mean a Nuis-written
compiler is self-hosted or ready to replace stage0.

The readiness manifest therefore owns five separate bootstrap-critical
coordinates with their own evidence, status, progress, blocker, next action,
validation command, and expected artifact. These coordinates begin at honest
nonterminal scores and can only close independently.

## Active Migration Semantics

Readiness v2 separates phase activation from final readiness. The current
manifest reports `stage0-to-stage1-migration/active` because Git has entered
`beta-0.10.*`. All `5/5` preparation gates are now closed and the frontdoor
reports `ready = true`, so candidate-owned vertical slices may proceed as the
mainline. Readiness still does not authorize stage0 removal, component
replacement, or final selection.

## Frontdoor

Use the repository-relative default manifest:

```bash
cargo run -q -p nuis -- bootstrap-status
cargo run -q -p nuis -- bootstrap-status --json
```

The installed public form is `nuis bootstrap-status --json`.

An explicit manifest can be inspected without changing repository state:

```bash
cargo run -q -p nuis -- bootstrap-status --json path/to/readiness.toml
```

The command validates the exact protocol, required gate set, coordinate
identity, unique IDs, field completeness, status vocabulary, progress range,
and declared gate count. A structurally valid but unfinished manifest exits
successfully with `ready=false`; invalid protocol state fails closed.

`ready=true` is emitted only when all five gates are exactly `stable/100`.
The command selects the weakest incomplete gate by status rank, progress, then
coordinate, matching the recursive development tensor rather than relying on
the roadmap score or registration order.

## Required Gates

### `bootstrap-language-subset`

Coordinate: `language-core/nuisc/bootstrap-language-subset`.

Freeze the syntax, type, effect, control-flow, generic, pointer, FFI, and
library surface permitted in compiler-authoring sources. Both accepted and
rejected fixtures are required so stage1 cannot acquire accidental dependency
on an unstable language feature. V8 is now executable through
`nuisc bootstrap-check`; see
[Nuis Bootstrap Language Subset](nuis-bootstrap-language-subset.md).
This gate is `stable/100`; its twenty-one exact scalar exports include generic
AST/NIR page continuation and complete token-page hashing without admitting a
new language capability. Widening it requires a new protocol version.

### `compiler-data-model`

Coordinate: `standard-library/std/compiler-data-model`.

Provide the minimum owned text, vector, map, arena, source-span, diagnostic,
and path contracts needed by a real compiler component. Data model v11 is now
`stable/100`: `StdLanguageCore` owns the foundational representation,
`StdCompilerData` owns materialized token records and bounded bytes,
`StdCompilerPayload` owns the typed logical-page view,
`StdCompilerPayloadRegistry` owns kind/schema registration and the shared
aggregate arena,
`StdCompilerTokens` owns the standalone token DFA, and
`StdCompilerTokenEmit` reconstructs canonical `nuis-token-stream-v1` bytes
while materializing a complete bounded page into the owned store. The frozen
subset accepts these modules, and
`bootstrap_compiler_data_model_demo` crosses bootstrap-check, NIR/YIR/LLVM,
native build, and deterministic exit `130` without FFI or host collections. It
materializes four records, emits 59 canonical bytes, then materializes the
real `use cpu StdLanguageCore;` token prefix and emits its exact 91-byte
canonical page with pinned hash `1277127995`. It decodes those bytes into a
fresh store and re-emits the identical length and hash. A two-vector, seven-byte-word
packing keeps the 128-byte output buffer near the old compile-time baseline.
The same native program builds an eight-entry map across two vector pages,
updates a key without reordering it, pins ordered identity `415394959`, fills
all sixteen entries, and rejects both overflow and malformed column shapes.
It also stores two four-slot compiler objects at stable indices, reads their
fields through checked projections, and pins ordered arena identity
`1064756829`; capacity, kind, index, slot, and malformed-shape failures remain
inside ordinary Nuis execution.
V7 layers `CompilerTextArena` over that unchanged envelope. The native program
stores `nuislang` and U+03BB as ten canonical UTF-8 payload bytes, rebuilds
owned texts at stable indices, and pins typed identity `1643761726`. Wrong
kind, invalid index, malformed UTF-8, forged hash, object exhaustion, and
payload exhaustion fail closed without changing the pre-failure identity.
V8 adds `CompilerPagedTextArena` beside that frozen type. The native program
stores `nuislang`, U+03BB, and `nuislang` as 18 bytes across two logical pages,
projects the third record across the boundary, pins page identities
`712007164` and `132664649` plus complete identity `322532187`, and preserves
the deterministic exit `130` without widening the twenty-one-export ceiling.
V9 then registers kind-one canonical text and kind-two fixed twelve-byte source
spans. One 20-byte aggregate arena projects both owned values across the page
boundary and pins registry identity `1630830726`, source-span identity
`1383365918`, page identities `934788601` and `1229397900`, envelope identity
`1109161393`, and complete identity `1274791798`. Duplicate registration,
wrong kind, absent index, and malformed fixed length fail with exact codes;
failed storage leaves identity unchanged and the native exit remains `130`.
V10 adds kind-three `CompilerChunkedPayload` without changing those identities.
The 24-byte `nuis-compiler-payload-v1` value is copied and projected through
eight fixed Nuis chunks, spans three aggregate pages, and pins typed identity
`94500080`, extended registry identity `1593840720`, envelope identity
`1520342505`, page identities `934788601`, `1001962162`, and `1407376619`, plus
complete identity `551151124`. Forged identity and full-capacity failures leave
the input arena unchanged; native exit and the twenty-one-export ceiling remain
unchanged.
V11 then sends that complete three-object, 44-byte arena and its registry
through `compiler_aggregate_arena_forward` and a second checked Nuis helper
before projection. Registry identity `1593840720`, every stable index and page,
and complete identity `551151124` remain unchanged. Supplying the valid v9
registry fails with code `3` and leaves the original arena unchanged under its
correct registry. This adds no import, type, FFI dependency, pointer identity,
or scalar export.
See [Nuis Compiler Data Model](nuis-compiler-data-model.md).

This bounded gate is `stable/100`. Each token materialization window remains
bounded to four token records, 64 payload bytes, and 128 output bytes, but
production now covers the complete token stream with contiguous 128-byte pages
whose boundaries may cross records. Nuis and the artifact layer independently
recompute every page hash and chain link while preserving the canonical legacy
page identity. `StdCompilerProjection` also owns and resumes the first two AST
and NIR structural pages through opaque cursors. Vectors and maps remain
`i64`-specific and bounded to sixteen entries. One text remains limited to
sixteen bytes, the v11 aggregate payload is bounded to 128 bytes, and registered
typed codecs now cover text, source spans, and canonical chunked bytes. Generic nested-page
specialization still lacks defining-module provenance in the general case, and
arbitrary aggregate loop-carried state still requires general backedge
lowering. These are growth limits rather than blockers for the bounded
self-hosting preparation gate.

### `stage-neutral-ir-boundary`

Coordinate: `language-core/nuisc/stage-neutral-ir-boundary`.

Freeze producer-neutral source, token, AST, NIR, and YIR handoff records. The
serialized identity must not depend on Rust layout so the existing stage0 and
a future Nuis stage1 producer can be compared against the same contract.

This gate is now `stable/100` for its bounded three-page migration slice. Normal AOT builds emit the ordered five-stage
`nuis-compiler-stage-handoff-v1` SHA-256 chain, hash its source/token/manifest
artifacts in the build manifest, and preserve bundle identity across cache
hits. The shared `nuis-compiler-structural-projection-v1` codec independently
parses and canonically re-renders AST/NIR hierarchy and module identity without
source reconstruction or producer-private layout. Explicit YIR crosses parse,
verify, and canonical re-render. `StdCompilerProjection` now supplies a typed
streaming consumer whose pure Nuis candidate crosses bootstrap-check, native
AOT execution, malformed-sequence rejection, and tamper-checked execution
proof. The exact scalar producer ABI consumes every serialized stage byte,
emits a Nuis-owned bundle fold, and drives `StdCompilerTokens` across the exact
token header, seven record kinds, payload shape, and LF boundaries. Candidate
production v11 now binds its record count, decoded semantic fold, every raw
token page and page-chain identity, plus the preserved four-record canonical
prefix identity `164749511446`, to an independent artifact-layer result. It
also binds three
completed AST records, unfinished-line continuation, state hash `1349056749`,
and identity `174028320749`, all recomputed independently by the artifact
layer. The same scanner binds four completed NIR records, its 25-byte
continuation body, state hash `1026894471`, and identity `132469386887` to a
second independent result. Nuis then serializes opaque cursors and resumes the
AST and NIR streams into independently replayed second pages. Adapter
protocol v9 now emits both complete AST and NIR cursor pairs as two ordered
22-word non-identity checkpoints. `nuis-compiler-stage-transformation-v3`
binds each checkpoint to canonical ULEB128 structural records without appending one complete source
blob. Its decoder reconstructs indentation and LF boundaries, reparses every
ordinal, depth, kind, and body, and independently replays every word and source
byte before production can attest it. The semantic differential passes `2/2`
for the byte-different representations. `nuis-compiler-stage-handoff-v2` now
selects every registered transform in order and binds its canonical source
record, derived payload, checkpoint, recovery hash, and semantic verdict
without stage-specific selection logic or replacement authority.

The production-bound adapter now has a disjoint
`structural-pagination-v1` mode whose canonical 62-line result carries three
AST and three NIR pages with every page identity, cursor identity, and opaque
cursor lane. `nuis-compiler-candidate-structural-pagination-v1` binds that
result to production-v11, the complete payload hashes, and the exact adapter;
the artifact layer independently replays all six pages and proves predecessor
pages one and two unchanged. Two cache-bypassed roots preserve identical raw
results and projection semantics while retaining their distinct root-bound
component and production lineage. Complete-stream pagination remains a later
versioned slice. See
[Nuis Compiler Stage Handoff](nuis-compiler-stage-handoff.md).

### `stage0-stage1-driver`

Coordinate: `compiler-toolchain/bootstrap/stage0-stage1-driver`.

This gate is now `stable/100` for its bounded canonical migration slice. `nuis bootstrap-build` is a dedicated project-only
driver over the frozen bootstrap gate and normal AOT pipeline. It consumes the
five-stage handoff and emits `nuis-compiler-component-build-v1`, binding the
exact stage0 compiler image, native output, build outputs, project/Galaxy/
Nustar dependency closure, a cache-stable reproducible identity, and an exact
audit identity. It also emits `nuis-compiler-diagnostic-report-v1`, bound to
the exact component record and producer. See
[Nuis Compiler Component Build](nuis-compiler-component-build.md).

`nuis bootstrap-candidate-probe` additionally executes the first pure Nuis
candidate with empty argv, closed stdin, required exit `0`, and empty output,
then binds its image and result to
`nuis-compiler-candidate-execution-v1`. See
[Nuis Compiler Candidate Execution](nuis-compiler-candidate-execution.md).

The probe authority remains explicitly execution-only. The separate
`nuis bootstrap-candidate-build` frontdoor feeds all five payloads through the
candidate's exact scalar exports, independently verifies the folds, token
decode summary, canonical token page, AST/NIR continuation identities, and
both checkpoint sets, materializes lossless byte-different AST- and NIR-derived
payloads, emits their semantic differential and producer-neutral handoff v2,
binds both through `nuis-compiler-candidate-production-v11`, and then runs the
differential gate.
See [Nuis Compiler Candidate Production](nuis-compiler-candidate-production.md).
The current producer is the bounded
`nuis-stage1-compact-structured-nir-producer-v10` leaf; it preserves canonical
handoff bytes, emits two reversible compact-record binaries plus `2/2`
semantic comparison, and grants no replacement authority.

`nuis bootstrap-reproducibility` now runs that complete production chain in
two initially empty roots with compile-cache read/write bypass. Its path-free
`nuis-compiler-component-reproducibility-v1` aggregate binds distinct local
witnesses, both exact report lineages, and stable component/native identities.
The local witness intentionally carries no independent attester authority.
`nuis-compiler-component-attestation-v1` now adds a separate Ed25519 claim over
the exact aggregate, both v11 production proofs, fresh verifier challenge, and
registered environment identity. Verification requires a caller-owned exact
trust-registry hash pin and never grants replacement authority. The checked-in
[Linux amd64 generation-one evidence](../evidence/compiler-attestation/linux-amd64-cleanroom/generation-1/nuis.compiler-component-remote-evidence.toml)
now records two cache-bypassed clean builds from a separately operated server,
the exact signed aggregate, and registry pin `90b8f7f4c9d336c72caa7dc4dc9a91c41ec263a7bfffa282ee8211088b164f01`.
Its private key remained on the attester; repository tests verify the real
claim and reject a wrong challenge or registry pin.

`nuis-compiler-component-replacement-authorization-v1` now adds a deliberately
separate component-owner authority. Its two frontdoors re-verify that false-
authority attestation, require another caller-pinned component-scoped registry
and fresh challenge, reject reuse of the attester identity or public key, and
sign one generation-one stage0-to-candidate transition with stage0 retained as
the rollback target. The authorization is written without replacement and
does not itself switch the active compiler. See
[Nuis Compiler Component Replacement Authorization](nuis-compiler-component-replacement-authorization.md).

`nuis bootstrap-activate-component` now consumes that permission through
`nuis-compiler-component-active-state-v1`. It repeats both pinned trust checks,
binds the immutable authorization source, authorization proof, and attestation
proof, and creates one canonical state without overwrite. The same
provider-neutral selector resolves `active` to the stage1 candidate build and
`rollback` to the exact stage0 build.

`nuis bootstrap-rollback-component` now emits the signed
`nuis-compiler-component-transition-v2` successor. It binds the generation-one
authorization proof and active-state identity, requires the same component-
owner role and key, restores stage0 as `current`, and retains the candidate as
`forward`. The private-key-free verification frontdoor repeats the full
predecessor chain.

`nuis bootstrap-dispatch-component` now replays that complete trust chain,
resolves an unordered exact-two component/image inventory by the signed
reproducible identities, and verifies both image byte hashes. It executes only
the restored stage0 image from a private create-new staging slot, rereads the
staged bytes before launch, removes the slot afterward, and emits canonical
`nuis-compiler-component-dispatch-v1` evidence. The receipt retains the exact
candidate as `forward` and carries no physical path or timestamp. The current
request remains deliberately limited to the `help` frontdoor. Its separately
versioned `nuis bootstrap-dispatch-compile` companion derives a canonical
rebuild request from the exact selected stage0 record, runs real
`bootstrap-build`, rereads the complete result, and requires equal dependency,
handoff, native, reproducible, and decoded artifact-semantic identities. Both
raw path-bearing artifact hashes remain auditable while operational paths stay
out of the receipt. See [Nuis Compiler Component Dispatch](nuis-compiler-component-dispatch.md)
and [Nuis Compiler Component Compile Dispatch](nuis-compiler-component-compile-dispatch.md).

`nuis bootstrap-candidate-compile-capability` now proves the next narrower
boundary without rewriting that transition. It reuses the exact adapter bound
by candidate production v11, folds the command and all runtime path bytes
through Nuis exports, and invokes only a byte-verified stage0 provider through
separate `execl` arguments. The rebuilt component must satisfy the same
component and decoded artifact-semantic predicate as compile dispatch v1.
Missing providers, adapter or provider drift, and absent Nuis admission fail
before a receipt is written. The receipt explicitly grants neither replacement
nor selection authority. See
[Nuis Compiler Candidate Compile Capability](nuis-compiler-candidate-compile-capability.md).

`nuis bootstrap-candidate-direct-compile` now proves the first non-delegating
stage1 execution slice. It deep-verifies the production-v11 candidate and its
five-stage handoff, launches a private adapter copy with closed stdin, a cleared
environment, and exactly five payload arguments, then parses the exact 53-line
front-end result. Capability v2 independently reconstructs its folds, token
pagination, bundle, and AST/NIR checkpoints from production evidence. It binds
`provider_dependency_required = false` and `direct_stage1_compile = true`, while
keeping native materialization, replacement, and selection false. See
[Nuis Compiler Candidate Direct Compile Capability](nuis-compiler-candidate-direct-compile-capability.md).

`nuis bootstrap-sign-candidate-successor` now joins that direct proof to the
immutable preselection lineage. It replays the complete generation-two trust
chain, delegated capability v1, production v11, direct capability v2, and the
canonical front-end result, then signs one path-free
`nuis-compiler-candidate-successor-v1` under the continuing component-owner key.
The relation strengthens generation three without changing the selected
compiler. Provider dependency is false and direct stage1 compilation is true;
fresh-source compilation, native materialization, replacement, and final
selection remain false. See
[Nuis Compiler Candidate Successor](nuis-compiler-candidate-successor.md).

`nuis bootstrap-candidate-fresh-source` now crosses the next ownership boundary
without mutating that signed successor. It stages the exact production-v11
adapter, clears the environment, closes stdin, and drives one canonical
56-byte Nuis snapshot through five candidate-owned source, token, AST, NIR, and
YIR identities. The independent artifact implementation reproduces all counts,
identities, and the bundle fold before create-new evidence is persisted. No
stage0 handoff or runtime provider is loaded, while native materialization,
replacement, and selection remain false. See
[Nuis Compiler Candidate Fresh-Source Capability](nuis-compiler-candidate-fresh-source-capability.md).

The same frontdoor then invokes `nsld-input-v1` on the same verified adapter.
The Nuis candidate uses reserved subset-v8 ordinals rather than adding an
export, and emits fourteen semantic values binding source identity
`12832741133`, YIR identity `9279238763`, `Main.main`, return value `7`, time
ordinal zero, no dependencies/relocations/GLM resources, and materialization
fold `1403051547`. The artifact layer independently rebuilds the exact
`nuis-compiler-candidate-nsld-input-v1`; capability v1 binds it to the complete
fresh-source and signed-successor lineage. `nsld candidate-input` consumes the
target-neutral record and stops at registered object-writer selection. It does
not claim native object bytes or grant replacement/selection authority. See
[Nuis Compiler Candidate to Nsld Materialization](nuis-compiler-candidate-nsld-materialization.md).

The developer path now indexes lowering helper lanes once, verifies lowering
and GLM graphs through dense integer node IDs, uses hash-backed lowering
node/resource and edge membership indexes, and compiles SHA-256 proof hashing
with targeted optimization. Bootstrap subset admission also feeds one normal
AOT invocation instead of compiling the project once for admission and again
for the artifact. The handoff verifier now compares disk-verified payloads with
the already rendered pipeline text instead of reparsing AST/NIR or rerendering
producer-private AST/NIR/YIR structures. Controlled compiler-data samples
reduced a cold cache miss from 268.59 seconds to 19.94 seconds. The established
cache-hit baseline is 1.57 seconds and the latest sample is 1.62 seconds.
Successive graph-index samples reduced retired instructions from 175.44 billion
through 166.73 billion to 159.15 billion and peak footprint from 266.7 MB through
259.6 MB to 244.1 MB. The measured cold runs used no swap; AST, NIR, YIR, LLVM IR,
and shim outputs remained byte-identical and native exit remained `130`.
Recursive async helpers now enter their precompiled direct-call path before
unsupported inline recursion is rejected. This is implementation evidence only:
it does not freeze the aggregate ABI or relax semantic, canonical, hash,
disk-read, or YIR verification. The separate v11 nested-arena proof is what
closes the bounded data-model gate at `stable/100`.

### `differential-reproducibility-gate`

Coordinate: `developer-system/bootstrap/differential-reproducibility-gate`.

This gate is now `stable/100`. `nuis bootstrap-diff` consumes verified stage0 and
explicit `stage1-candidate` records plus their handoffs, payloads, normalized
diagnostics, dependency closures, and native outputs. Its fixed thirteen-check
report emits `blocked-drift` or `equivalent-awaiting-authorization`; both keep
`replacement_authorized = false`. Valid drift is retained as an audit report
and returns a failing command status. See
[Nuis Compiler Component Differential Gate](nuis-compiler-component-differential.md).

The checked-in Nuis token materializer now enters this path as a real
`stage1-candidate` leaf and reaches repository-native `13/13` equivalence. The
path frontdoor verifies its execution and production proofs, including exact
adapter bytes, all stage folds, independently reproduced token summary,
canonical token identity, complete token pagination, AST/NIR continuation
identities, and the exact independently replayed transformation, semantic, and
handoff v2 selection manifests before writing the report. Two local clean,
cache-bypassed runs now retain stable reproducible identities and 13/13
verdicts; production proof identity transitively binds the transformation in
both runs, including compact-record metadata and semantic recovery, and root or
aggregate tampering fails closed. A separately versioned signed attestation
binds both runs to a challenge and a pinned environment-scoped key; the real
generation-one Linux amd64 claim is checked in and verified without its private
key. Claim, signature, registry, challenge, or lineage tampering fails closed.
The thirteen canonical comparisons remain byte-stable for generation-one
verification. A second canonical sidecar now walks every registered v2
selection without stage-specific branches and binds the actual selected bytes,
recovered payload, transform, checkpoint, base comparison, and handoff proof.
The real AST and NIR paths therefore reach `2/2` selected-representation
equivalence despite non-identical bytes while retaining false replacement
authority. Reproducibility v2 preserves the exact canonical v1 predecessor and
its signed aggregate, independently rebuilds that predecessor from both clean
roots, and directly binds each root's canonical sidecar bytes, internal report
hash, base report, production proof, and clean-root witness. The two sidecar
hashes intentionally may differ because each binds a distinct root audit; the
closure criterion is per-root replay plus `4/4` semantic equivalence. Sidecar
tampering fails closed without invalidating immutable generation-one v1
evidence. Replacement authorization v1 now feeds canonical active-state v1,
transition v2 signs generation-two stage0 restoration while retaining the
candidate as the forward target, dispatch v1 executes that exact current image,
and compile dispatch v1 rebuilds one canonical project without rewriting a
predecessor. Candidate compile capability v1 closes the production-bound
delegating driver route while retaining the verified stage0 provider as an
explicit dependency. Candidate preselection v1 signs that exact capability,
production proof, provider dependency, and immutable generation-two transition
under the continuing component-owner key. Direct capability v2 closes front-end
execution without that runtime provider, and candidate successor v1 now signs
the exact proof and result into generation three without mutating any
predecessor. The bounded fresh-source capability now gives the candidate
ownership of one canonical source-to-YIR identity path. The bounded
materialization capability now extends that ownership to one equivalent Nsld
input, but not to general parsing or native object bytes. Remote mirrors of the
direct-successor, fresh-source, and materialization evidence remain open.
Cryptography does not prove physical-machine
independence; that remains an operational fact.

## Migration Rule

The preparation order is:

```text
freeze subset and compiler data contracts
  -> freeze stage-neutral handoffs
  -> build one component through stage0
  -> compare stage0 and candidate stage1 outputs
  -> sign candidate capability into generation-three preselection
  -> prove direct stage1-owned compilation
  -> bind the direct proof into a signed generation-three successor
  -> prove candidate-owned fresh-source front-end compilation
  -> prove one candidate-owned equivalent Nsld materialization input
  -> emit and independently verify candidate-owned native object bytes
  -> permit one reversible component replacement
```

The first stage1 component should be deliberately small and leaf-like. It must
exercise source parsing or diagnostic production while avoiding ownership of
the whole compiler driver. Replacement remains reversible until repeated
stage1 builds and differential reports are deterministic.

## Honest Boundary

This protocol is migration infrastructure. It does not claim that Nuis is
self-hosted, that the bootstrap subset is stable public language surface, or
that entering `beta-0.10.*` grants immediate compiler replacement. It keeps the
open prerequisites visible while bounded ownership transfers proceed under an
immutable stage0 rollback chain.
