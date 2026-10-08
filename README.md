# nuislang

> An AOT-first heterogeneous systems language and toolchain built around
> `nuis -> NIR -> YIR -> registered Nustar backends -> Nsld -> runtime`.

Nuis treats CPU, shader, kernel, data, network, and C compatibility as
registered execution domains under shared function/node, clock, GLM, artifact,
and lifecycle contracts. LLVM and host operating systems are bootstrap
backends, not the language's semantic root.

The [software manufacturing architecture](docs/reference/nuis-software-manufacturing-architecture.md)
sets the wider direction: the OS hosts execution; Nuis owns production contracts.
Nuis-rc is the intended machine-local resource control plane, with decentralized
acquisition and separate discovery, identity, integrity and trust. This is an
architecture commitment, not a claim of an implemented CAS or resident collector.

## Current Line

The repository is on `beta-0.16.*`; the current committed minor baseline is
[`beta-0.16.0`](docs/versioning/nuis-beta-0.16.0-snapshot.md) (`dff1bdbc`, 2026-10-07).
Git history is authoritative; Cargo package versions are independent of the
project release. Use the [beta-0.16 checklist](docs/versioning/nuis-beta-0.16.0-release-checklist.md)
for current work. The previous minor baseline is `05951bef` (`beta-0.15.0`, 2026-09-24).
The [beta-0.15 snapshot](docs/versioning/nuis-beta-0.15.0-snapshot.md) records
that baseline, not a compatibility freeze or acceptance of later changes. The
[beta-0.14 snapshot](docs/versioning/nuis-beta-0.14.0-snapshot.md) and
[beta-0.12 snapshot](docs/versioning/nuis-beta-0.12.0-snapshot.md) remain
historical checkpoints, not current acceptance results. The
[beta-0.15.9 patch](docs/versioning/nuis-beta-0.15.9-patch.md) remains a historical
receipt, not the current version.

The mainline is **ns-nova application-led development**: grow one Nuis-owned
interactive image application, fix the foundation gaps it exposes, measure
performance, and progressively replace independently testable compiler modules
with Nuis implementations. Writing an engine in Nuis validates the language;
only real compiler responsibility transfer counts as compiler self-hosting.

Start with the [current mainline map](docs/current-mainline-map.md),
[development tensor](docs/reference/nuis-development-tensor.md), and
[application-led roadmap](docs/versioning/nuis-beta-0.11-application-led-mainline.md).
This is a formal staged self-hosting migration line, not a claim of a self-hosted
compiler, stable public API, or production-ready engine.

The earlier committed proofs include [bounded leading return-print prefixes](docs/reference/nuis-native-return-print-prefixes-v1.md):
save the entry once, keep guarded integer prints in the parent, then execute the
independently admitted pure return tail. The subsequent
[computed print-argument proof](docs/reference/nuis-native-computed-return-prints-v1.md)
keeps checked/call-backed argument work inside selected pure value helpers, with
the actual prints still in the parent. [Interleaved scalar aliases](docs/reference/nuis-native-return-print-aliases-v1.md)
now compose with those prints under original-scope and nonexpanding-copy proof.
The [staged initializer proof](docs/reference/nuis-native-staged-return-effects-v1.md)
adds selected checked/call-backed scalar bindings, evaluated once and reused by
prints and return tails, including checks in unused bindings.
The [logical initializer follow-up](docs/reference/nuis-native-logical-staged-initializers-v1.md)
adds direct nested short-circuit roots to these selected bindings, preserving
once-only snapshots, original scalar captures and shared prefix/tail budgets.
The subsequent [continuing effect-region proof](docs/reference/nuis-native-continuing-effect-regions-v1.md)
adds bounded internal `if` regions before/between those initializers and prints,
with once-only selected conditions, isolated child scopes and saved parent/child
paths. The [effect-region exit follow-up](docs/reference/nuis-native-effect-region-exits-v1.md)
adds bounded internal scalar returns, separate exit/value snapshots and continued
path guards that suppress later checks, prints and pure-tail work. The
[exit-only region proof](docs/reference/nuis-native-exit-only-regions-v1.md)
also admits internal source exits with empty or pure continuing tails, without
promoting continuation seeds into returns. The
[paired result-join proof](docs/reference/nuis-native-effect-result-joins-v1.md)
now carries fresh owned bool/i64 if-expression results out of fully continuing
selection arms without replaying work or exporting other child locals.
The [partial-result follow-up](docs/reference/nuis-native-partial-effect-result-joins-v1.md)
also admits internal source returns when each result arm still proves a final
paired value on its continuing paths, using the existing merged live masks.
The [one-sided result follow-up](docs/reference/nuis-native-one-sided-effect-result-joins-v1.md)
also admits a continuing result opposite a wholly exiting arm, preserving explicit
source returns and result presence. The
[join-capture optimization](docs/reference/nuis-native-effect-join-captures-v1.md)
removes unused single-value selectors without dropping condition work or live masks.
The [equal-atom follow-up](docs/reference/nuis-native-equal-effect-result-joins-v1.md)
also removes selection between identical validated atoms while preserving both
source arms' selected work and result presence.
The [typed effect-snapshot follow-up](docs/reference/nuis-native-typed-effect-snapshots-v1.md)
adds exact owned i32/f32/f64 data within selected regions, with typed inactive seeds
and bool-only path masks. The
[selected-region return handoff](docs/reference/nuis-native-typed-effect-returns-v1.md)
now also carries those exact kinds into enclosing returns without widening
ordinary tails or computed-print authority; native evidence uses ordinary AOT,
while the pure scalar session bridge still rejects effectful callbacks.
The separate [literal-print policy](docs/reference/nuis-native-literal-print-policy-v1.md)
now provides explicitly authorized constant i64 prints through registered native
callbacks, with checked effect order and unchanged default vetoes. The
[explicit build policy](docs/reference/nuis-native-literal-print-build-policy-v1.md)
now binds those grants and work limits through build, cache, source-free artifact
restoration and verified native launch; it is not automatic authority or a general
effectful application host.
The [guarded effectful scalar-call follow-up](docs/reference/nuis-native-effectful-scalar-selection-v1.md)
adds bounded two-sided call selections and one-sided existing-scalar rebinding,
with once-only predicates, retained current values, guarded argument evaluation
and explicit effect/trap order. Bounded nested existing-scalar updates now keep
descendant predicates and their arguments inside ancestor guards; unsupported
eager effectful arms reject. Bounded sequential leaves now keep private scalar
staging, current-version rebindings and unused effectful calls behind those guards,
returning only one existing scalar rather than exporting child-scope bindings.
One bounded ordered-region walk now subsumes the original prefix, single-child
suffix and two-child proofs. It admits adjacent selections, longer sibling
sequences and regions ending in a child, carrying each actual merged target into
later predicates, arguments and private staging. Child count is constrained by
shared work budgets, not finite shape combinations; single-target publication,
exact types, capture/depth limits and original effect authority remain intact.
Single-edge `&&`/`||` child predicates now keep exact owned-bool left gates,
including bounded nonlogical calls/comparisons and checked arguments, once-only
at their original sites. Complete RHS work stays behind ancestor and short-circuit
guards; left-side calls participate in effect discovery and dependency order.
Bounded direct logical trees now extend this proof with one guarded helper per
original edge and a shared 32-edge region budget, preserving once-only left work,
complete selected RHS subtrees and original expression/depth/capture limits.
Fresh owned-bool `let`/`const` logical initializers now reuse the same proof in
selected scalar regions and prefix/middle/suffix stages. Original declarations
stay private and after ancestor guards, with current versions and constant seals
carried across stages. Existing owned-bool `let` roots can now update private
stage bindings or the sole published bool target, with read-before-write
captures and constant seals. Single-statement logical child leaves and hidden
ordinary leaves remain separate; common guard/backend/effect authority is not widened.
Native fresh/cache/source-free runs check selection polarities, poisoned inactive
paths and selected-stage traps without exporting child-private bindings.
The [floating literal negation repair](docs/reference/nuis-native-float-literal-negation-v1.md)
preserves source negative zero and nested literal signs with native bit-pattern
checks. The [nonliteral sign-negation follow-up](docs/reference/nuis-native-float-sign-negation-v1.md)
also preserves variable/call zero signs and NaN payload bits using existing typed
word operations, with once-only evaluation and linked native evidence.
General multi-statement rebinding, resources and arbitrary effectful helper shapes remain
outside these bounded routes. The continuing/exiting/result proofs are now
in the beta-0.16.0 baseline; capture, equal-atom and typed-data proofs remain worktree
follow-ups after it, as do both floating-negation repairs and typed return handoff. None certifies complete
application closure.

```text
nuis source / nuis.toml
  -> nuis project frontdoor
  -> nuisc frontend and semantic checks
  -> NIR
  -> YIR + GLM / clock / domain verification
  -> registered Nustar lowering and backend artifacts
  -> Nsld link planning, NSB assembly, host-shell finalization
  -> lifecycle/runtime dispatch
  -> run-artifact / Nsdb execution evidence and replay
```

## Capability Snapshot

| Surface | Verified Scope | Important Boundary |
| --- | --- | --- |
| Small host CLI tools | Native builds exercise argv, piped stdin, stdout/stderr, file reads/writes, text statistics, reports and simple PGM transformations. | Examples have bounded inputs; this is not blanket compatibility or production certification. |
| Compiler | Parsing, types, generics, control flow, NIR/YIR verification, LLVM lowering and AOT emission have focused regressions, including bounded Buffer-writing loops. | Supported syntax does not imply every combination lowers; arbitrary `while` bodies and some aggregate early-return shapes remain incomplete. |
| Image application | Nuis generates RGBA8 data, inline WGSL runs RGB inversion on real Metal, and compiled host execution exports checked PPM frames. | The host embeds the YIR lifecycle runtime and uses registered providers; it is not fully native CPU execution or a self-contained Nsld image. |
| Persistent window | Registered Nuis open/event/close callbacks retain state and one provider connection across AppKit events. A separate registered CPU parent can consume one terminal outcome. | Explicit mode, bounded dispatch/replay, one-child parent profile; not an unlimited engine loop or general supervisor. |
| Cancellation | Independent tickets carry provider-worker drain into a typed, non-success result. AppKit and the headless build/run-artifact route share the classifier and launch policy; Metal regressions retain prior success evidence. | AppKit cancellation policy remains scripted; headless uses bounded scalar scripts. Linux hardware evidence, Windows transport, parent cancellation and resource reuse remain open. |
| Other backends | Checked-in routes include Linux CUDA/Vulkan and Apple Metal/CoreML provider work. | Evidence is backend- and workload-specific; reference results and hardware-free conformance do not certify physical execution. |
| Nsld | Deterministic plans/NSB assembly, first ARM64 Mach-O and x86_64 Linux ELF private-shell routes, loader admission, publication and final-output selection. | Broader architecture/provider parity, PE/COFF final execution and self-contained application packaging remain incomplete. |
| Self-hosting | Five bounded preparation gates, Nuis compiler-component proofs, differential/reproducibility evidence and explicit selection/rollback contracts. | `stage0-to-stage1-migration/active` is not completed compiler replacement; the bounded candidate-to-Nsld path stops before native object emission. |

The [observable CLI regression](tools/nuis/tests/std_filesystem_smoke.rs)
builds and runs actual host executables, rather than only checking source.
For example, [cli_wc_demo](examples/projects/tooling/cli_wc_demo) reads one
4096-byte chunk and uses ASCII word separators; it is not a complete `wc`.
Large-file streaming, broader text semantics, long-running concurrency and
cross-platform distribution need their own acceptance tests.

Tensor `stable/100` means stable for the recorded bounded milestone, not
language, stdlib, ABI, package, or whole-engine stability. Current task selection
comes from `nuis dev-tensor --json`, not a hardcoded progress total in this page.

Development-resource management follows the
[Nuis-rc heap/stack contract](docs/reference/nuis-rc-cache-lifecycle.md): default
shared heap packages, explicit project-owned stack packages, and root/lease-based
GC. The read-only policy core is tested; shared-store migration, compiler leases,
and the actual resident collector remain unimplemented. Existing caches are unchanged.

## Current Mainline

The goal is `standard-library/ns-nova/interactive-image-workflow`; the selected
prerequisite remains `standard-library/ns-nova/persistent-application-session`.
The [dependency plan](docs/reference/nuis-development-tensor-mainline.md)
keeps correctness interrupts, application progress and compiler migration distinct.

The [image showcase](examples/projects/domains/ns_nova_image_showcase) currently
has a bounded three-frame lifecycle and an
explicit stateful `--window-session window` route. Space toggles the GPU-processed
checkerboard. Tests check actual pixels, one worker, cache reuse, live/replay
identity, explicit close and failure-preserving parent handoff.

Cancellation does not collapse three different facts into one:

| Observation | Meaning |
| --- | --- |
| Cancellation admitted | No further host operations are admitted; an in-flight callback may still finish. |
| Host retirement acknowledged | The worker-owned module, registry, execution state and transport have left scope. |
| Provider/device resources retired | Requires provider-owned completion/drain evidence; it cannot be inferred from either observation above. |

`WindowSession::cancel` and its C ABI only forward the independent ticket.
It can outlive the window. Accepted cancellation never invokes implicit close
or manufactures a terminal outcome or parent delivery. Provider sessions depend
on the small, statically bound `ScopeAdmission` interface, not concrete window
or cancellation internals. The [cancellation contract](docs/reference/nuis-yir-application-cancellation-v1.md)
defines the ownership and failure boundaries.

The explicit scripted cancellation path now frees the window and polls its
independent ticket without close or parent delivery. Normal window quit still
uses explicit close/Finish. The [provider-session drain extension](docs/reference/nuis-yir-provider-session-drain-v1.md)
now reaches host-library cancellation through explicit `cancel_with_provider_drain`,
with a separate target/count-validated observation after registered worker close.
Damaged exchanges are not retried, and cancelled results never replace replay.
Adding `--drain-provider` to scripted cancellation now requires a declared bundle
capability and produces typed cancellation, never application success. The launcher
requires both the provider's Drained terminal and the child's non-success exit 130;
it rejects Finish under drain intent before provider publication. Default
host-only cancellation still reports EOF as incomplete provider execution.
The terminal contract and launch policy contain no OS/window/provider implementation;
The [headless scalar-script profile](docs/reference/nuis-yir-application-scalar-script-v1.md)
now uses those contracts without AppKit. Compiled M2 tests check exact image hashes,
normal Finish and typed drain with unchanged old evidence. Ordinary builds select
`--packaging-mode headless-aot-bundle`; run-artifact admits explicit scalar scripts
only after capability, source and executable identity checks. Explicit build
selections are cache-isolated. Headless builds consume an explicit verified-YIR
checkpoint without requesting CPU LLVM or inventing an empty `.ll` file. Source,
tokens, AST, NIR and YIR remain bound by the portable compiler-stage handoff.
Native/window builds still require LLVM. With `packaging_mode = "headless-aot-bundle"`
in `nuis.toml`, check/dump, benchmark/binding inspection and `nuis workflow --json`
select verified YIR and report LLVM as not requested. This is semantic-stage
evidence, not proof of native AOT readiness or successful provider execution.
Bounded Buffer-writing callback loops now reuse private YIR helpers and registered
CPU execution, with ordered reads/writes, checked native indices and checked scalar
integer division/remainder. Nested `if`/`else` bodies use guarded private functions:
conditions are read once and unselected arms perform no branch-local access or math.
Reachable synchronous, acyclic `i64`/`bool` source helpers now support fresh scalar
bindings, nested `if`/`else` and early returns as real ordered YIR calls. Guarded
function blocks defer branch-local computation; shared suffixes avoid duplicating
the rest of the function. PixelMagic uses guarded coordinate/color helpers before
its red/blue writes. The CLI
headless build/run-artifact test checks two real M2 Metal frames,
direct-session replay, exact output identity and failure admission; independent
native/reference tests compare every generated pixel. Multiple i64 accumulators now
cross Buffer-writing iterations through explicit LoopState fields, with strict
seed/result layouts, source-order rebinding and zero-trip preservation. PixelMagic
uses them for fill-and-statistics, with exact native/reference red-count and pixel-sum
checks and compatible red-count/boolean APIs. State updates also compose inside
nested `if`/`else` arms: untaken arms preserve their incoming values, conditions
are snapshotted once, and repeated updates retain source order. PixelMagic counts
red pixels inside the selected write branch. Ordinary LLVM multi-carry loops
retain aggregate return storage; the explicit native-session value profile below
has a separate allocation-free aggregate return path.
Nested bounded loops now compose through those same function contracts, including
branch-local child loops, reset/persistent counters and shared callback fuel.
PixelMagic uses real row/pixel loops with clamped partial rows; native/reference
tests check both pixels and carried statistics. Inner loops cannot mutate ancestor
headers. Guarded `continue` now skips the remaining iteration through those same
functions, provided its path explicitly performs the matching unit step. PixelMagic
uses it after red-pixel writes and statistics. Guarded `break` now signals the CPU
loop driver and LLVM to exit before stepping, preserving prefix writes and i64
state without empty remaining iterations. Native/reference and budgeted-session
regressions cover this subset. PixelMagic's `recolor_run` now exercises it in the
ordinary M2 headless build/run-artifact path: the first same-color run is marked
before GPU inversion, with exact stop state, write counts, frame bytes and replay.
Owned-Bytes cleanup returns now use the declared aggregate layout for LLVM packing,
including nested fields and two terminal branches. A pure Nuis regression compares
both return paths with reference execution and checks zero live Bytes after native
execution, also with reordered declarations. The default AOT image regression now
passes its LLVM checkpoint and real Metal export again without switching packaging
modes. Its compiled host still executes CPU callbacks as embedded YIR; this does not
establish native CPU callback dispatch or self-contained Nsld packaging.
An experimental [static scalar callback bridge](docs/reference/nuis-native-scalar-session-bridge-v1.md)
now invokes registered open/event/close helpers in a native executable, with nested
state, typed slot validation, declaration-order invariance and reference parity.
An explicit `yir-pack-aot --native-session ID` profile now routes these static
exports through the shared application-session host, preserving failure/cleanup
state and rejecting mixed YIR artifacts before open without interpreter fallback.
The explicit `native-session-aot-bundle:<id>` build profile now carries that
registration through LLVM checkpoints, cache identity, standalone materialization
and `nuis run-artifact --native-session <id>`. Real M2 regressions reject argument
and artifact drift before open and run again after deleting the original build
directory. The selected native profile now includes bounded, acyclic scalar helper
calls with exact parameter/result kinds and all-five-scalar native/reference parity.
Supported synchronous `@noinline` boundaries survive NIR-to-YIR lowering rather than
silently expanding. Counted i64 loops now compose with those helpers and ordered
add/multiply carries. Constant induction is proven at compile time; runtime i64
start/bound/step values are checked before loop entry for finite, non-wrapping
induction within 65536 iterations. Native/reference and real process-trap tests
cover this boundary. Scoped loop bodies now call admitted scalar helpers, either
discarding a scalar result or carrying one or multiple i64 returns into the next iteration.
All five scalar capture kinds retain exact typing; scoped edges share the same
acyclic call-graph limits. Multi-carry calls bind a checked flat return layout and
release the temporary aggregate before the next iteration. Registered callback
roots now retain their aggregate helpers even when main does not call them;
unsupported outer updates cannot silently degrade to a counter-only loop.
Scoped guarded break now commits validated carries, releases the returned aggregate
and exits before the induction step. Pure scalar source loops reuse the existing
normalizer with one private break bit; full induction preflight still applies.
Checked flat-i64 branch-helper returns now support multi-state guarded updates,
`break` and matching explicit-step `continue`, including nested-loop scope.
Ordinary and scoped edges share exact layout/kind checks and bounded call-graph
admission; nested temporary returns are unpacked and released before the next call.
Checked i64 division/remainder now reuse the existing native emitter with exact
operand kinds and zero/overflow traps before LLVM arithmetic. Guarded scalar helpers
preserve selected-path evaluation, including discarded results and unused arguments.
Fallible flat-i64 aggregate branches now share that guarded normalization, including
nested returns, existing record captures, shared suffixes and same-callee arguments.
Ordinary helper fields keep their declared names; scoped carry/control schemas stay
separate. Real allocation/drop probes cover selected and skipped paths. Counted
loops now compose inside these guarded helpers, including inline arms and shared
suffixes. An i64 induction update can precede ordered i64 cumulative updates via
the existing chained-loop contract. Selected loops keep finite/non-wrapping
induction preflight; unselected loops skip it, even without division. Cumulative
arithmetic wraps, and reference scalar arithmetic now agrees in debug builds too.
Conditional carry updates now reuse the existing conditional-chain contract:
each nonempty arm updates the same local i64; an omitted or empty arm keeps that
carry's own value, just like an explicit self-assignment. Leaf
comparisons read the stepped index or an earlier updated carry against an invariant
atom; reversed comparisons share normalization. Native preflight and cooperative
reference execution remain separate. Pure leaf comparisons now compose with
`&&`/`||`, including nesting and grouping, through the same YIR condition tree.
LLVM emits short-circuit blocks; every RHS retains exact-kind checking, including
unreached leaves. General verification and native admission have independent
condition-depth limits. Nested carry-update arms now normalize to one private
scoped iteration helper, preserving step-first decisions, ordered sibling reads
and each carry's own value on empty arms. Guarded bool helpers preserve short-circuit
`&&`/`||` inside the outlined iteration; no new CPU opcode or callback ABI is added.
Source guard depth and native call-graph/induction budgets remain independently
checked. Ordered multi-statement carry bodies now use the same helper: repeated
writes share one return slot, branch arms may update different carry sets, and
untaken arms retain incoming values. Later statements observe preceding writes;
short-circuit guards and source-order sibling restrictions remain enforced.
Iteration-local i64/bool temporaries now remain inside that helper rather than
becoming loop state. Typed and inferred bindings preserve initialized snapshots;
branch-local names never escape their arms, and mutable i64/bool locals may feed later
statements without adding driver return slots. Checked i64 `/` and `%` now execute
inside selected iterations, including local initializers, carry updates and lazy
predicates. Zero-trip/unselected paths do not evaluate them; reached zero divisors
and signed overflow trap even for unused results. Invariant induction preflight
still runs before any body work. Loop-local expressions can now call exact i64/bool
helpers from the completed pure, acyclic catalog. Logical arguments stay
lazy even inside an i64-returning call, and unused arguments retain reached-path
checks. Flat-i64 helper results now work as iteration-local snapshots: exact-layout
copies, field projections, aggregate arguments and branch captures use the shared
value-aware outliner without widening Buffer admission. Dependency-ordered validation
now admits bounded loop-bearing callees, including transitive wrappers. In the
native-session profile, each selected invocation preflights its own complete
induction after argument evaluation. Newly emitted callbacks also share one
stack-owned loop-work counter across synchronous helpers: each selected loop reserves
its full induction count before its body, with a default total of 1,048,576 per callback.
Early exits do not refund work; zero-trip and unselected paths do not consume it.
An independent stack counter now limits actual YIR function entries to 1,048,576,
including roots and outlined guards, closing loop-free call-tree expansion as well.
Conditional calls in the admitted scalar catalog retain guards rather than executing
both branches before a select. Exhaustion traps without publishing callback output;
cache and standalone restoration retain both policies. These are not elapsed-time,
memory or device-work bounds. Mixed/nested/resource aggregates remain separate.
Iteration-local bool rebinding now crosses nested, one-sided and mixed scalar
branch joins using canonical 0/1 private transport, without adding outer loop state
or changing the public ABI. Native execution and build/cache/standalone restoration
retain old snapshots, lazy checked calls and both independent execution budgets.
Iteration-local flat-i64 records can also be rebound through nested and mixed-value
joins. Private transport follows declared field order and reconstructs the exact
nominal record, without mutating old snapshots or widening Buffer/resource authority.
The aggregate-rebinding fixture retains build/cache/source-free restoration evidence.
Seeded mutable flat-i64 records now also survive loop backedges, alongside scalar
carries and other records. Zero trips retain their seeds; declared-order slots
reconstruct exact nominal values without overwriting pre-loop snapshots. The
aggregate-carries fixture passes build/cache/source-free restoration using the
existing loop contract and both budgets; Buffer and resource authority stay unchanged.
Seeded outer bool locals now use explicit canonical word conversion at iteration
entry, return and loop exit, including singleton and mixed-value carries. Zero-trip
seeds, pre-loop snapshots and lazy checked calls survive native execution and
build/cache/source-free restoration without changing the callback ABI.
Literal nested counted loops now reuse the same scoped helpers and value transport.
Each selected child invocation retains its own complete induction preflight and
shares both callback work counters. Native/reference checks cover rectangular and
triangular bounds, ascending/descending induction, local and persistent child indices,
bool/record snapshots, skipped paths and late failures; the literal-loops fixture
also crosses build/cache/source-free standalone restoration. Child declarations
remain local, and nested header/ancestor-induction mutation is rejected. Guarded
`break`/`continue` now also work in these leading-step pure-value bodies: each exit
belongs to its own loop, keeps earlier updates and skips only its remaining suffix.
Private recovery preserves the already-advanced index on break and the seed on zero
trips, reusing the existing scoped-break contract. Full preflight and reservations
still apply even to immediate exits; the value-loop-exits fixture passes build,
cache and source-free standalone restoration. Trailing-step pure-value loops now
use that same contract: effects observe the pre-step index, break suppresses the
step, and continue requires an identical explicit step immediately before it.
Leading/trailing child loops may mix without sharing exit scope. Pure-value and
Buffer rewriting are mutually exclusive for admitted value functions. The
trailing-value-loops fixture exercises build/cache/source-free restoration.
Counted value bodies now also return i64, bool, i32, f32, f64 or exact flat bool/i32/i64/f32/f64 records from
inside loops. The selected payload is evaluated before a private pending flag
unwinds child and parent loops; neither skipped suffixes nor trailing steps run.
The counted-returns fixture crosses the same source-free workflow. Ready-value
returns now avoid private branch/continuation helpers: integer/bool literals and
existing scalar/flat-record values need no speculative work. Conditions still run
once, and calls, projections and constructors stay guarded. The full previously
over-budget control-composition fixture now fits in 51 reachable native functions,
without raising the 64-function bound or changing either work-budget policy.
Single-use terminal computations also lose their forwarding helper but remain
inside the selected guard; multiple uses still share one body. Single-use
multi-statement suffixes also fold without dropping unused calls or changing their
order. Scope-aware private names now separate colliding locals while preserving
outer loop updates, fields and function symbols. The expanded composition includes
a same-name record/scalar prefix and retains the 51-function bound.
The selected [native value-return profile](docs/reference/nuis-native-scalar-value-returns-v1.md)
now returns admitted nested scalar records as LLVM slot values, with zero aggregate
heap allocations in the tested helper and callback paths. This does not change
the ordinary owned-aggregate ABI or promise faster wall time. Typed publication
retains raw scalar bits, independent snapshots, overlapping input/output buffers
and failure sentinels. Private boolean packing and field-selective capture transport
reduce generated helper arguments without widening public signatures or native limits.
Stable record aliases and straight-line rebound snapshots now retain lexical identity
and source evaluation order through projection; selected failures still trap.
The sparse, alias and snapshot workflows cover build/cache/tamper rejection and
source-free restoration. Same-name branch-local aliases now receive separate lexical
identities before field projection, while existing outer writes keep their identity.
Nested scopes and later same-name declarations stay separate; computed callers still
veto the candidate transaction. The scoped-alias workflow retains guarded native
results and source-free restoration. Straight-line branch-local record rebindings
now have scope-confined snapshot identities; old aliases and self-rebinding reads
retain their previous values without erasing constructor work. Cross-scope writes
in non-loop branches proven to return now receive separate snapshot versions too;
the suffix retains its preceding value. Field-only record copy families now admit
fallthrough and loop-written input projection by rebuilding private inputs while
keeping assignments, per-trip snapshots and backedges intact. Full-record escapes stay whole.
Single-definition loop aliases now expose fields when their pure-value origin is
an unwritten parameter. Generated scoped iterations now project invariant fields while
preserving induction/carry maps and nested break identities: the 64-field loop fixture
uses four arguments instead of 66. Scoped drivers retain complete flat-i64 initial state
independently of partial field arguments. Generated scoped helpers now project proven
partial and entirely unread record inputs, preserving full state and initializer work.
Generated non-scoped helpers now transport whole pure records through typed YIR parameters
and LLVM slot values when scalar capture compaction still exceeds 64 parameters.
The 64-field-plus-predicate case runs without widening source/public/FFI or callback ABI;
nominal layouts, scalar bits, independent snapshots and allocation-free publication stay checked.
Proven generated scoped flat-i64 records now use explicit per-trip field mappings,
preserving independent induction, carry and complete seed storage. Generated loop
branch helpers now share this private record transport while keeping unselected work guarded.
Generated scoped bool/i32/i64/f32/f64 record carries now use checked private word seeds and
typed reconstruction, including zero trips, snapshots and loop exits. The YIR loop
contract and source/public/FFI signatures stay unchanged. Signed i32 fields retain
sign extension, width-correct arithmetic and exact backedge decoding. F32/F64 fields use
bit-preserving word packing, not numeric casts. Nested pure-scalar carries now retain exact
field-segment seed/backedge maps and nominal reconstruction at every record level.
Same-named leaves remain distinct; complete word maps include nested all-i64 trees.
Sparse mixed/nested iteration inputs now retain complete initial storage while passing only
proven typed leaf words. A 64-leaf state uses six iteration arguments in the bounded probe.
Branch-local record snapshots and identity-preserving alias writes now retain explicit lexical/version proofs.
Materialized outer joins now retain real assignments while backward field demand projects only needed input words.
Changing nested-loop snapshots now retain zero-trip/header demand and nearest-loop break/continue targets through bounded fixed-point proof.
Nested early returns now preserve current mixed snapshots and failure-atomic publication in native probes.
Exact shared-budget success/failure and overlapping input/output checks retain real traps and canonical input validation.
Admitted returns can reuse an exact, initialized mutable record after a second source-order check.
Unobserved leading-loop exit indices no longer need a recovery carry; visible suffixes and generated helper outputs still do.
Proven return-only loop exits now share the compiler-owned pending bit with the canonical break slot; ordinary breaks stay independent.
Bounded per-write fixed points now prove invariant leaves across nested return loops; changing leaves share compact private typed records.
Every evaluated RHS remains once; only proven unused, total input-record copies may disappear. Whole-function revalidation can veto the rewrite.
Ordinary child break/continue paths now retain assignment own-read order, registered control
identities and checked compact-record seeds through final lowering, without relaxing source admission.
Native probes retain 64-word state with zero/one/two child trips and atomic selected-trap failure.
Ordinary child loops now share per-write invariant proof without sharing return-signal authority.
Observed child exit indices run at 9/63/64 words: the break fixture needs only two child
control words, and the continue fixture needs no child record backedge. Unused ready-input
reconstructions can disappear; this rule does not erase calls, checked fields, codecs or resources.
Partly observed total reconstructions now project exact typed fields before private capture planning.
Their 64-word break/continue workflows execute and restore without source; checked field uses,
observed indices and failure-atomic publication remain. Opaque snapshot calls in child breaks
and no-carry/single-carry continues retain evaluation through private read-only mixed-record inputs;
all callers must prove invariant roots. Checked constructors now fit through cached nominal
field ranges within the original analysis budget, without discarding field evaluation or traps.
Versioned preheader snapshots now prove equality across saved records, subrecords and fields,
with ordered-pair branch joins and per-field loop summaries across all trips and intermediate writes.
Parent entries and post-loop uses share one bounded summary, but freshen varying fields separately at each boundary.
Stable fields retain their evaluated origins; see the [loop snapshot proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md).
Unobserved total input-field copies can now disappear without erasing computed fields or transport provenance.
Bounded scalar input-alias chains also expose those copies: a 64-field guarded record
fixture uses 2/3 private arguments, or 3/3 when it retains an unused selected call.
That input-alias proof does not promote computed/local versions or codecs, and it
leaves scoped control identities and public inputs unchanged.
Exact scalar call-field demand now distinguishes a field observation from whole-record
transport. The computed 64-field fixture forwards a scalar with two `i64` captures
plus its guard, while its calls still execute and a real whole-record consumer
retains the readonly 64-word input. Public state and native bounds are unchanged.
Already-evaluated immutable scalar locals now have a separate record-view proof:
private record copies may disappear, but their original calls, codecs and checked
work remain in place. This does not grant constant or loop-backedge authority.
Completed pure aggregate calls now have a separate bounded result-view proof:
two independent 64-word returns stay fully evaluated and separately stored, while
later total copies no longer force a full private capture. The selected branch
uses two `i64` captures plus its guard; whole-record consumers remain conservative.
Inline computed calls at projected helper sites, opaque/effectful results and changing versions are
not granted this authority, and no public ABI or native limit is widened.
Completed calls can now retain directly written, nested record arguments under
exact nominal/field-kind proof. Every original argument field and selected check
still executes once; only later copies of the completed result can disappear.
This does not spill or project an inline computed argument at a private helper
call site, and one unproven caller still vetoes the complete rewrite.
Materialized immutable record arguments now carry the same exact kind proof
through local aliases into later calls. Checked constructors remain at their
original bindings, and whole call operands retain their storage; only later
total result copies can disappear. Scope, version and transport exclusions remain.
Stored call-result field initializers now have a separate bounded proof:
`let part = produce(...).field` keeps the entire producer and all unselected
result work, then permits later total copies of the stored part to disappear.
Unbound inline projections remain conservative; this stored-result proof does not
authorize caller spilling.
An additional [caller-side total record proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#total-inline-caller-records)
selects fields from inline constructors built only from unwritten input paths.
Every field and caller must pass; computed fields and registered codec boundaries
remain conservative. Scoped targets keep their original caller proof, and this
does not expand source-function projection eligibility.
Computed record operands now have a separate [binding-root materialization proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#binding-root-caller-materialization):
`let/const` root calls can preserve every complete operand in ordered typed bindings
before private projection. Checks in unused fields still run. A separate
[return-root proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#return-root-caller-materialization)
now preserves complete operands immediately before `return helper(...)`, without
moving work across branches or early exits. A separate
[if-condition-root proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#if-condition-root-caller-materialization)
preserves complete operands before direct `if helper(...)` calls with an exact
owned `bool` result. The separate [selected short-circuit RHS proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#selected-short-circuit-rhs-materialization)
keeps complete operands inside the selected arm of `gate && helper(...)` or
`gate || helper(...)`, with a stable owned bool input/literal gate. Shared guarded
value lowering now supports bounded fresh pure prefixes and prevents eager RHS
checks in these source conditions without requiring private projection.
The shared [source logical value-root route](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#source-logical-value-roots)
also guards supported `let/const` initializers and owned-bool `return` roots outside
loops, including admitted selected arms and current local versions. Reference
entry exits now use the registered return contract instead of executing the tail.
The separate [effectful-parent conditional-return proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#effectful-parent-conditional-returns)
guards top-level pure `bool`/`i64` return computations while retaining parent
effects, current scalar versions and exits. It does not relax the speculation
barrier or authorize branch-local effects, enclosing branches or loop returns.
A separate [pure return-prefix proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#pure-prefixes-before-conditional-returns)
keeps bounded fresh `let/const` calculations and local records inside the selected
return arm, including unused checks. Parent effects and scalar capture rules stay
unchanged; prefix work is not skipped merely because a later RHS short-circuits.
A separate [pure terminal-tree proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#pure-terminal-return-trees)
admits bounded complete nested return branches under the original outer guard.
Every leaf returns, sibling locals stay isolated, and parent effects or a
continuation suffix are never copied into the pure helper.
A separate [pure fallthrough-tree proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#pure-fallthrough-return-trees)
admits empty continuation leaves beside pure return leaves. Stable parent bool
inputs determine whether to exit after the selected helper runs; a real `false/0`
return is not confused with the helper's unused seed. Computed exit predicates
and effectful continuation bodies remain outside that proof.
A separate [pure continuation-work proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#pure-work-before-continuation-leaves)
also keeps fresh `let/const` work in continuation leaves and direct outer arms.
Unused checks and calls remain selected; an internal seed follows that work
without becoming a source exit. At least one original return is still required.
A separate [local atom-alias proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#local-atom-aliases-for-exit-readiness)
lets fresh boolean alias chains select a partial return. Only aliases of current
parent bool atoms or literals are flattened for readiness; original local work
stays in place. Call-derived/computed predicates and branch effects remain excluded.
A separate [total comparison-readiness proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#total-comparisons-for-exit-readiness)
admits local bool equality/inequality and signed i64 comparisons of stable atoms
or their aliases. Normalized conditions stay at most three nodes; calls, fields,
arithmetic-derived operands and nested comparisons remain excluded.
A separate [inline partial-tree comparison proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#inline-total-comparisons-in-partial-return-trees)
allows those total comparisons directly in internal `if` conditions beside
continuation leaves, reusing the shared typed facts and unchanged budgets.
This does not authorize call-derived partial-tree predicates or replay any original work.
A separate [complete-tree computed-predicate proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#pure-computed-predicates-in-complete-return-trees)
allows exact owned bool conditions from bounded pure calls, field reads and
checked calculations when every internal leaf returns. The original condition
runs once inside selected work, with no reconstructed exit flag.
A separate [stored partial-tree exit signal](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#stored-exit-signals-for-partial-return-trees)
now admits bounded pure computed conditions beside continuation leaves. A private
typed result stores exit readiness and the actual bool/i64 value together;
the parent reads that completed snapshot without replaying a condition or call.
Original logical returns are materialized before record construction so their
short-circuit RHS remains guarded.
A separate [computed outer-entry proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#pure-computed-outer-entry-predicates)
now evaluates bounded pure bool calls, field reads and arithmetic/comparisons
once at the original parent statement site. Helpers and exit readiness read only
the saved bool; earlier exits, parent effects and checked operands stay ordered.
This does not widen logical entry conditions, branch effects or resource captures.
A separate [bounded intermediate-suffix proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#bounded-pure-intermediate-suffixes)
now carries pure branch tails into continuing paths, never past an observed return.
Original lexical scopes and expanded statement/expression budgets are checked
before cloning; branch effects and resource captures remain excluded.
A separate [single-edge logical entry proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#single-edge-logical-outer-entries)
now admits an owned bool atom/literal on the left of `&&` or `||` with a bounded
pure nonlogical RHS. Shared value-root lowering guards the complete RHS before
publishing the saved entry bool; later bounded extensions are listed below.
A separate [logical return-arm root proof](docs/reference/nuis-native-scalar-loop-snapshots-v1.md#guarded-logical-roots-in-return-arms)
adds the same single edge at internal `if` conditions and fresh bool `let/const`
roots, including partial returns and bounded pure suffixes. Shared guards retain
selected checks; non-total exit decisions use stored signals, never predicate replay.
A separate [computed logical gate proof](docs/reference/nuis-native-computed-logical-gates-v1.md)
adds bounded pure calls, field checks and comparisons on the left of source bool
value roots and outer return entries. The left runs once before the guarded RHS;
The [computed return-arm extension](docs/reference/nuis-native-computed-logical-gates-v1.md#computed-return-arm-gates)
adds this gate to bounded return-tree conditions, fresh bool bindings and return
roots, retaining shared original/expanded budgets and scalar-only parent captures.
Non-total partial exits use stored signals; local record checks stay selected.
A [nested logical-tree extension](docs/reference/nuis-native-computed-logical-gates-v1.md#nested-logical-trees)
guards direct `&&`/`||` children at these roots with shared work/depth/32-edge
budgets and linear helper growth, retaining original scope, exits and selected checks.
Other logical leaf embeddings, loops and scoped/word transports retain their
original projection rules. Delimited call arguments now admit
struct literals inside conditions without consuming the enclosing statement block.
Cache and restored host binaries now use [fresh-file publication](docs/reference/nuis-artifact-file-publication-v1.md).
A minimal cache regression reproduced host `SIGKILL` after in-place replacement despite
a valid on-disk signature; the fix changes file publication, not LLVM or callback admission.
The full-width preceding-loop helper shrinks from 62/63 private arguments to 5/6; this is not a benchmark. The return fixture
uses outer N-1 and inner N-4 carries: 60/63 at width 64. A changed outer tag now runs with
60/64 carries, including an admitted preheader `if`; in that baseline, changing both tag and count still requires 65 outer slots and rejects.
Typed literal origins now prove repeated integer/bool/finite-float fields across joined
preheaders and every intermediate write. A separate literal-seeded wide variant fits
60/64 carries despite changed tag/count; repeated mixed constants reduce its outer
carry to 60, while opaque calls still reject 65. All 64 output words, original RHS
work and source-free restoration remain checked; no native bound is widened.
Branch joins retain equality proved on every arm, not equality between arms, old versions or separate calls.
Native bounds, proof budgets, return signals, computed fields, observed exits and resource boundaries are unchanged.
Carried-word reductions and private argument counts are separate; this is not a measured speedup.
Recursive shapes remain bounded. Earlier shared-codec round trips
also passed with host clang at `-O0` and `-O2`. Neither this work nor
the repository cleanup raises the persistent-session coordinate above its bounded `active/86`.
Deep call/group nesting now reports a bounded parser diagnostic;
other frontend recursion and native call-depth limits remain separate boundaries.
Unrestricted aggregate calls, resources and provider effects remain outside this
profile; the default image host is unchanged.
The separate Buffer profile still excludes step-before-break and unstepped
`continue`; arbitrary carry types and fully native CPU callbacks remain separate work.
Portable protocol tests do not certify Linux GPU execution or Windows transport.
Resource-capability state, recovery, richer image bindings,
long-duration measurements and native CPU frame dispatch remain separate work.

ns-nova's intended role is a real-time world engine combining rendering,
control, ML and audio workflows, not just a widget kit. That
[capability horizon](docs/versioning/nuis-beta-0.11-application-led-mainline.md#engine-capability-horizon)
is not an integrated ML/audio engine today. PixelMagic, WitSage and registered
Nustars keep their own semantics; shared YIR/GLM/time contracts do not require
engine-specific compiler branches or a finite list of backend combinations.

## Quick Start

Run from the repository root with the Rust toolchain, the host LLVM/linker
requirements for the selected profile, and any provider-specific prerequisites.
Start with a host CLI route; GPU/window examples require their registered backend.

```bash
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
cargo run -p nuis -- dev-tensor
cargo run -p nuis -- workflow examples/projects/tooling/filesystem_io_report_demo
cargo run -p nuis -- check examples/projects/tooling/filesystem_io_report_demo
cargo run -p nuis -- build examples/projects/tooling/filesystem_io_report_demo target/nuis-readme/cli-report
cargo run -p nuis -- run-artifact target/nuis-readme/cli-report
```

For a compiled Metal image application on a supported Apple Silicon host, follow
the [image showcase instructions](examples/projects/domains/ns_nova_image_showcase/README.md)
for `--export-frame`, `--window-session` and optional
`--window-parent-session`. The launcher still supplies registered provider
sessions and artifact sidecars; copying only the executable is not this route.

Use `--json` on supported inspection/workflow surfaces for structured evidence.
Do not treat inspection as execution: `run-artifact --json` does not run the app.

### Native Artifact Closure

The checked-in linker pressure route is:

```bash
cargo run -p nuis -- build examples/projects/tooling/native_artifact_closure_demo target/nuis-readme/native-artifact
cargo run -p nuis -- artifact-doctor target/nuis-readme/native-artifact
cargo run -p nsld -- drive target/nuis-readme/native-artifact/nuis.build.manifest.toml
cargo run -p nsld -- drive target/nuis-readme/native-artifact/nuis.build.manifest.toml --apply --until-clean --json
cargo run -p nuis -- run-artifact target/nuis-readme/native-artifact
```

`nsld drive` is non-mutating without `--apply`. Applying writes whitelisted
next artifacts and reports blocked boundaries instead of bypassing them.
Compatibility output remains the default; explicit private-image selection
requires independently validated admission and selection evidence.
Read the [native workflow](docs/reference/nuis-native-artifact-workflow.md),
[Nsld frontdoor](docs/reference/nsld-linker-frontdoor.md), and
[binary assembly gap map](docs/reference/nsld-binary-assembly-gap-map.md).

## Repository Map

| Path | Responsibility |
| --- | --- |
| [`tools/`](tools) | Project/compiler/linker/debugger/bundler frontdoors and host/YIR tools |
| [`crates/`](crates) | Reusable compiler, semantic, artifact, runtime and domain capabilities |
| [`nustar-packages/`](nustar-packages) | Static manifests, backend registrations, ABI targets and packaged assets |
| [`stdlib/`](stdlib) | Nuis sources for core, std, PixelMagic, WitSage and ns-nova |
| [`examples/`](examples) | Runnable projects, source probes, verifier cases and explicit legacy material |
| [`docs/reference/`](docs/reference) | Current implementation contracts and boundaries |
| [`docs/versioning/`](docs/versioning) | Minor-line history and long-range policy |
| [`subprojects/`](subprojects) | Separately owned Vulpoya and Yalivia shells |
| [`scripts/`](scripts) | Maintenance and developer-machine helpers |

See the [repository layout](docs/repo-layout.md) and
[documentation index](docs/README.md) before adding another top-level entry.

## Toolchain Boundaries

```text
nuis             workflow and project frontdoor
nuisc            compiler core and AOT artifact producer
nsld             deterministic linking and binary assembly
nuis-runtime     lifecycle loader and execution context
yir-runtime-host embedded-YIR application/window host boundary
nuis-host-runner host compatibility launcher
nsdb             YIR debugging and replay
nsbdr            OS bundle/distribution adapter over final Nsld outputs
```

Frontdoors should be thin adapters over reusable capabilities. The compiler knows
Nustar registration contract shapes and asks registered packages for domain
behavior; the CFFI package owns the generated GNU resolver and symbol-version registry.
The C world is an explicit compatibility domain, not a privileged linker model.
See the [CFFI contract](docs/reference/cffi-von-neumann-domain-contract.md),
[pointer safety boundary](docs/reference/ffi-pointer-safety-boundary.md), and
[toolchain capability boundary](docs/reference/toolchain-galaxy-core-boundary.md).

## Libraries And Examples

```text
core -> std -> pixelmagic
core -> std -> witsage
core -> std -> ns-nova
```

`core` is the semantic base; `std` owns practical systems contracts. PixelMagic
owns image algorithms, WitSage owns classical ML, and ns-nova composes application
policy without absorbing either package. The [stdlib index](stdlib/README.md)
and [examples router](examples/README.md) distinguish runnable projects from
recipes, metadata proofs and legacy probes. A file's existence or a backend
declaration alone is not execution evidence.

## Development

The [CI cold-start contract](docs/reference/nuis-ci-cold-start.md) separates
source-only checks, locked dependency fetch, focused portability validation and
the workspace build. A clean checkout must not need existing Cargo caches or
generated `target/` files to pass its source guards.

The current Ubuntu [Build workflow](.github/workflows/build.yml) runs those
checks, including one exact host-path portability test; it does not run the full
compiler semantic, native-session, self-hosting candidate or physical GPU suites.
A green Build is not whole-toolchain or cross-platform certification.

Prefer focused checks rather than rebuilding the workspace on every edit:

```bash
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test std_filesystem_smoke std_tooling_observable_cli_smoke_checks_reports_and_stdin -j 1 -- --exact --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis dev_tensor -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit -j 1 -- --test-threads=1
cargo fmt --all -- --check
git diff --check
```

The [beta-0.15 validation checklist](docs/versioning/nuis-beta-0.15.0-release-checklist.md)
separates existing CI coverage from focused native, migration and provider checks.
The [beta-0.12 checklist](docs/versioning/nuis-beta-0.12.0-release-checklist.md)
remains historical evidence, not a current acceptance result.
For disk cleanup, inspect `scripts/disk-clean-safe.sh` output before choosing
`--apply`. Add `--build-binaries` to retire hashed test/build executables while
keeping root CLI binaries and libraries; `--workspace` explicitly drops the
whole root build tree. Cleanup is workspace-only and refuses tracked or symlinked
paths. See the [cleanup policy](docs/repo-cleanup-candidates.md); do not remove
source, live regressions or unrelated project data. Generated example bundles
are rebuilt from source, not shipped in the checkout.

Use repository-relative paths and registered target/provider identities, not
personal directories, private host addresses or a fixed macOS version. Prefer
remote Linux infrastructure for Docker and CUDA-heavy tests. Rust/Nuis source
defaults to 800 lines, tests to 1000, Markdown to 2000; see the
[file-line policy](docs/repo-file-line-policy.md).

## Long-Range Direction

Staged migration began at `beta-0.10.*`; ns-nova now supplies application pressure
for foundation hardening, measured optimization and module-by-module ownership
transfer. The five closed preparation gates do not replace the Rust compiler.
The bounded Nuis candidate, trust/rollback chain and remaining native-object
boundary are documented in [self-hosting readiness](docs/reference/nuis-self-hosting-readiness.md)
and [candidate-to-Nsld materialization](docs/reference/nuis-compiler-candidate-nsld-materialization.md).

The approximate stage2-equivalent target remains `gamma-0.5.*` through
`gamma-0.10.*`, not a deadline or a claim of full engine maturity. Wider
Vulpoya/Yalivia collaboration, runtime/framework maturity and a self-owned
heterogeneous OS/hardware stack are longer-range work. Read the
[OS roadmap](docs/versioning/nuis-long-range-heterogeneous-os-roadmap.md),
[GLM positioning](docs/glm-spec/glm-heterogeneous-flow-graph-positioning.md) and
[Vulpoya/YIR review boundary](docs/glm-spec/vulpoya-yir-secondary-review-positioning.md).
Hardware extensibility and scheduling performance are design goals, not measured
advantages over other compiler infrastructures.
