# Current Mainline Map

This is the short reading map for current implementation, not a history catalog.
When documents disagree, inspect the code/tests and current tensor evidence
before changing a capability claim.

## Fast Reading Order

The current `beta-0.16.*` priority is the ns-nova application-led dependency
chain agreed in [beta 0.11](versioning/nuis-beta-0.11-application-led-mainline.md).
The current committed baseline is [`beta-0.16.0`](versioning/nuis-beta-0.16.0-snapshot.md)
(`dff1bdbc`, 2026-10-07), with its [current checklist](versioning/nuis-beta-0.16.0-release-checklist.md).
The [beta-0.15.9 patch](versioning/nuis-beta-0.15.9-patch.md) remains historical;
bounded computed return predicates, nested logical trees and the following
parent-effect proofs retain their original receipt scope.
[Leading return-print prefixes](reference/nuis-native-return-print-prefixes-v1.md)
retain parent-only effects, a once-only entry and
an independently admitted pure return tail, not general effectful helpers.
The subsequent [computed print-argument proof](reference/nuis-native-computed-return-prints-v1.md)
guards pure checked/call-backed argument work before each parent print, without
granting new return-tail eligibility or logical-leaf/capture authority.
The [interleaved scalar-alias follow-up](reference/nuis-native-return-print-aliases-v1.md)
adds fresh total atom copies/literals around prints, preserving original lexical
validation and charging removed copies against the original/expanded budgets.
The [staged initializer follow-up](reference/nuis-native-staged-return-effects-v1.md)
preserves ordered selected bool/i64 computations as once-only ready snapshots,
including unused checked work. Only its separately proven initializer work may
support a pure return tail; actual source exits and ordinary-root limits remain.
The [logical initializer follow-up](reference/nuis-native-logical-staged-initializers-v1.md)
extends only direct logical initializer roots, reusing the existing short-circuit
pass while charging prefix expressions and expanded tails together. Ordinary
print/call leaves and original scalar capture authority are not widened.
The subsequent [continuing effect-region proof](reference/nuis-native-continuing-effect-regions-v1.md)
was developed after that historical patch and is included in beta-0.16.0.
Bounded internal `if` regions retain
once-only selected conditions, saved parent/child paths and isolated child scopes;
the [effect-region exit follow-up](reference/nuis-native-effect-region-exits-v1.md)
adds internal owned bool/i64 returns and masks later prefix/tail work without
replaying conditions. The [exit-only region proof](reference/nuis-native-exit-only-regions-v1.md)
also admits empty or pure continuing tails after validated internal source exits;
the [paired result-join proof](reference/nuis-native-effect-result-joins-v1.md)
adds fresh owned bool/i64 if-expression results from fully continuing selection
arms, retaining all other child-scope isolation. The
[partial-result follow-up](reference/nuis-native-partial-effect-result-joins-v1.md)
permits internal source returns when each result arm still reaches its paired
value on continuing paths. The
[one-sided result follow-up](reference/nuis-native-one-sided-effect-result-joins-v1.md)
adds one continuing result opposite a wholly exiting arm, with explicit source
returns distinct from initializer tails. The
[join-capture optimization](reference/nuis-native-effect-join-captures-v1.md)
removes only unused single-value selection inputs, retaining source work and live masks.
The [equal-atom proof](reference/nuis-native-equal-effect-result-joins-v1.md)
also removes redundant result selection only after both source arms validate
identical ready atoms, retaining selected work and merged live authority.
The [typed effect-snapshot follow-up](reference/nuis-native-typed-effect-snapshots-v1.md)
adds exact owned i32/f32/f64 data inside selected regions with kind-specific seeds;
path masks and enclosing return authority remain separate.
The [floating literal negation repair](reference/nuis-native-float-literal-negation-v1.md)
adds source signed-zero and native bit-pattern evidence without widening that
profile. The [nonliteral sign-negation follow-up](reference/nuis-native-float-sign-negation-v1.md)
preserves zero signs and NaN payload bits with once-only evaluation and existing
typed word operations. The
[typed selected-return handoff](reference/nuis-native-typed-effect-returns-v1.md)
adds exact i32/f32/f64 enclosing returns only for admitted selected regions.
Ordinary AOT checks actual returned words; the pure native session bridge still
rejects effectful callbacks. The separate
[literal-print policy](reference/nuis-native-literal-print-policy-v1.md) checks explicit
constant i64 effects through registered native callbacks. The
[explicit build policy](reference/nuis-native-literal-print-build-policy-v1.md)
binds grants and limits through build/cache/launch identity without implicit
authority. The [guarded scalar-call repair](reference/nuis-native-effectful-scalar-selection-v1.md)
now preserves source effect/trap order for bounded two-sided selections and
one-sided existing-scalar rebinding, keeping inactive arguments behind guards and
retaining the current value when an update is skipped. Bounded nested updates
keep descendant conditions inside ancestor guards. Bounded sequential scalar
leaves now preserve private staging and repeated binding versions. A bounded
prefix before a final child selection now forwards new versions and retains
completed prefix updates on child skips. A separately proved bounded suffix now
consumes the child's selected or retained value and private prefix staging in
source order. Two sibling child selections now have a separate bounded proof,
carrying each merged target and private staging into later predicates and calls.
Adjacent children, longer sibling sequences, broader typed selected-return CLI
proof and generalized effect transport remain
separate. General rebinding, resource work and child-scope
exports remain open.
The [beta-0.15 snapshot](versioning/nuis-beta-0.15.0-snapshot.md) records
the baseline `05951bef` (`beta-0.15.0`, 2026-09-24); Git remains authoritative.

1. [Repository overview](../README.md)
2. [Mainline selection and acceptance coordinates](reference/nuis-development-tensor-mainline.md)
3. [Machine-readable application boundary](reference/nuis-ns-nova-application-lifecycle-v1.toml)
4. [Native scalar session bridge](reference/nuis-native-scalar-session-bridge-v1.md)
5. [Stateful window contract](reference/nuis-yir-window-session-v3.md)
6. [Cancellation and host retirement](reference/nuis-yir-application-cancellation-v1.md)
7. [Runnable image application](../examples/projects/domains/ns_nova_image_showcase/README.md)
8. [Focused validation checklist](versioning/nuis-beta-0.15.0-release-checklist.md)

Use the [versioning index](versioning/README.md) for the historical
[beta-0.14 snapshot](versioning/nuis-beta-0.14.0-snapshot.md),
[beta-0.12 snapshot](versioning/nuis-beta-0.12.0-snapshot.md) and
[checklist](versioning/nuis-beta-0.12.0-release-checklist.md), earlier
[beta-0.10 migration entry](versioning/nuis-beta-0.10.0-self-hosting-entry.md),
[beta-0.6 foundation](versioning/nuis-beta-0.6.0-mainline-entry.md) and older
alpha/pre-alpha anchors. Those checkpoints must not replace current behavior.

## Start Here

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis -- dev-tensor --json
```

The current goal is `standard-library/ns-nova/interactive-image-workflow`.
Its selected prerequisite is
`standard-library/ns-nova/persistent-application-session`.
The committed `beta-0.15.8` checkpoint retained `active/94`; the leading print-prefix
worktree checkpoint retained `active/95`; the computed print-argument follow-up
retained `active/96`; the interleaved atom-alias follow-up retained `active/97`;
the staged initializer follow-up retained `active/98`; the logical initializer
follow-up retained `active/99`, included in `beta-0.15.9`. The continuing-region
and effect-region exit follow-ups keep `active/99`, with separate validation evidence. This remains
an application prerequisite, not a completed native application.
Selection follows the [declared dependency plan](reference/nuis-development-tensor.mainline.toml),
not the globally lowest percentage. Correctness regressions may interrupt it.
The `05951bef` checkpoint and `beta-0.15.7` retain `active/86`. Current work adds
exact sparse mixed/nested bool/i32/i64/f32/f64 field-path maps and typed read-only scoped inputs.
The next task is broader internal
control/effect regions, preserving observed exits, ready snapshots and bounded fallback.
The separate backlog of further changing backedges and wide private captures
retains its own proof gates and does not replace this selected frontier.
Branch-local record/subrecord aliases and identity-preserving inherited writes now retain
lexical/version proofs. Materialized outer joins retain assignments under backward field-demand proof;
changing nested-loop snapshots now use bounded fixed-point demand with lexical break/continue targets.
Nested early-return probes now cover 9/30/31/57/58/59/60/61/62/63/64-word states by reusing an exact initialized mutable record.
Original source admission remains mandatory; a second validation can veto reuse and retain independent storage.
Bounded continuation analysis removes only unobserved leading-index recovery; suffixes, generated outputs and parent backedges retain it.
Return-only exits share the private pending/break signal under bounded, compiler-minted provenance; ordinary breaks stay independent.
Bounded fixed points join zero trips and every child write, keeping proven invariant leaves outside nested backedges and grouping changing leaves into private records.
Versioned preheader record/subrecord/field snapshots now prove cross-binding equality, with fresh identities for separately evaluated opaque results, ordered-pair branch joins and per-field loop fixed points. Parent entries and post-loop snapshots share one bounded summary of zero trips and every intermediate write on all trips: stable leaves retain origins, varying leaves get independent identities per binding and boundary, and child locals cannot escape. Boundary materialization publishes neither a partial environment nor a partially advanced clock on failure. A join preserves a relation only if it holds on every arm; swapped arms/old versions/separate calls remain distinct. The original 65536-work/64-depth budget and inner-only/original-body fallback remain; see the [loop snapshot proof](reference/nuis-native-scalar-loop-snapshots-v1.md).
The fixture carries outer N-1 and inner N-4 words (60/63 at width 64); a changed outer tag executes with 60/64, including an admitted preheader `if`. An opaque count snapshot on one arm still rejects 65 outer slots, as does changing both tag and count in that baseline. The immutable-parameter parent-entry fixture uses 6/12 private arguments, versus 5/11 for the baseline; fewer carried words do not establish a speedup. Existing scalar rebinding outside loops is not newly admitted.
Typed literal origins now preserve exact integer/bool/finite-float storage identities across joined preheaders and every intermediate write, including bounded integer casts and signed-zero/kind distinctions. One repeated literal leaf lets a separate changed-tag/count fixture fit inner/outer 60/64; mixed constant leaves further reduce the outer carry to 60, while independent child return storage remains 60. Native probes retain all 64 outputs, overlap/canaries and zero aggregate allocation/drop; source-free workflows retain repeated events, cache reuse, tamper rejection and selected checks. Calls, arithmetic, codecs and nonfinite text grant no constant authority, and no native limit or public ABI is widened.
Preceding ordinary/break/continue loops retain the subsequent full-width 60/63 layout. Safe unobserved input-copy pruning reduces their private iteration arities to 5/6/12, 6/6/12 and 5/6/12; the preceding helper shrinks from 62/63 arguments to 5/6. Calls, checked work, codecs and local versions stay evaluated. Registered source/wire layouts and immutable call/aggregate-return provenance chains remain intact; no source spelling grants transport authority. This is not a measured speedup.
Single-definition scalar aliases of unwritten inputs now expose total copies through exact lexical origins. The 64-field guarded fixture uses 2/3 private arguments, or 3/3 with an unused selected call retained; public inputs stay at 64 words. Computed/local versions and codecs do not grant alias readiness. Registered controls and scoped transport helpers are excluded; expansion and rollback retain the 65536-work/64-depth ceiling and every-caller validation.
Cache restoration and host-binary materialization now use [fresh-file publication](reference/nuis-artifact-file-publication-v1.md), rather than overwriting a previously executed inode. A minimal alternating-image cache test reproduced `SIGKILL` despite a valid on-disk signature; publication retains byte identity and existing admission without launch retries or callback-profile changes.
Exact scalar call-field observations no longer protect every sibling copy of a single-definition constructor. The computed 64-field scalar-forwarding fixture uses two i64 captures plus a guard with both calls retained, while a whole-record consumer keeps its readonly 64-word transport. Unknown paths, registered transports and computed siblings remain conservative; the [loop snapshot contract](reference/nuis-native-scalar-loop-snapshots-v1.md) keeps this bounded field-demand proof separate from general call-result capture closure.

A separate evaluated-scalar record view now reuses immutable local scalar values after their RHS has executed. Exact constructors and record aliases can lose their redundant copies while every scalar definition, call and selected check remains at its original site. Original input-alias authority is unchanged; aggregate call results, changing versions, registered transports and scoped control identities remain outside this proof. See [evaluated scalar views](reference/nuis-native-scalar-loop-snapshots-v1.md#evaluated-scalar-record-views).
A separate completed aggregate-call result mode now admits immutable locals only through a fully validated pure-value catalog. Generated signatures that may be projected grant no catalog authority. Both independent 64-word results and their original calls remain intact; only subsequent total reads/copies are projected, reducing the private branch to two i64 captures plus a guard. Whole-record consumers keep full transport. Inline computed arguments at projected helper sites, effects, recursion, changed versions and scoped/wire identities remain conservative; see [completed call-result views](reference/nuis-native-scalar-loop-snapshots-v1.md#completed-aggregate-call-result-views).
Completed calls now also accept directly written nested record arguments with exact nominal/field-kind validation inside the original bounded proof. Argument constructors, field order and every checked/unused scalar call remain intact; only subsequent result copies can disappear. This is argument typing, not constructor elision or computed-call spilling at a projected helper call site. Every-caller veto, unchanged return storage and native/public bounds remain mandatory; see [inline record arguments](reference/nuis-native-scalar-loop-snapshots-v1.md#inline-record-call-arguments).
Already materialized immutable record constructors and total local views now retain exact kinds through aliases into unchanged later call operands. Checked constructors are never erasable, whole uses preserve their local bindings, and branch-local/changing/control/transport identities grant no new authority. See [materialized record arguments](reference/nuis-native-scalar-loop-snapshots-v1.md#materialized-record-call-arguments); this is not computed-call spilling or a public ABI change.
Stored call-rooted projections now gain exact scalar/subrecord readiness only after their original initializer completes. The full producer, unselected result checks and independently stored results stay intact; inline expression typing and computed caller spilling are unchanged. See [stored projections](reference/nuis-native-scalar-loop-snapshots-v1.md#stored-call-result-projections).
Total inline caller constructors now have a separate non-scoped proof using exact unwritten input paths, with complete unused-field validation and every-caller rollback. It selects fields without repeating the constructor, leaves kept scalar operands in place and retains registered word-codec authority. Scoped targets keep their original complete caller/seed-map proof. This totality proof grants no computed-field, local-version or call-spilling authority; native evidence is explicitly registered projected IR, not expanded source-function eligibility. See [caller-side records](reference/nuis-native-scalar-loop-snapshots-v1.md#total-inline-caller-records).
Computed record operands at non-scoped `let/const` root calls now have a separate complete, ordered materialization proof through the unchanged pure-value catalog. Every original operand, including kept scalar checks and unused aggregate fields, receives a typed binding before projection; branch-local work stays in its branch and every caller must agree. A separate [return-root proof](reference/nuis-native-scalar-loop-snapshots-v1.md#return-root-caller-materialization) adds complete ordered bindings immediately before root-call returns outside loops, preserving early exits. The [if-condition-root proof](reference/nuis-native-scalar-loop-snapshots-v1.md#if-condition-root-caller-materialization) adds direct predicate calls outside loops only with exact owned bool results; the enclosing branches and early exits stay unchanged. A separate [selected short-circuit RHS proof](reference/nuis-native-scalar-loop-snapshots-v1.md#selected-short-circuit-rhs-materialization) handles one `gate && helper(...)` or `gate || helper(...)` edge outside loops with a stable owned bool input/literal gate. Complete operands stay inside its selected arm and only a fresh predicate result is joined. Shared guarded value lowering accepts bounded fresh pure prefixes, and the source condition path independently guards pure bool RHS calls instead of eagerly evaluating their fallible arguments. The [source logical value-root route](reference/nuis-native-scalar-loop-snapshots-v1.md#source-logical-value-roots) extends this shared guard to let/const initializers and declared owned-bool returns outside loops, including admitted selected arms and current local versions. Registered entry exits now preserve reference return timing and actual result publication while allowing unrelated global work to continue; single-entry verification is unchanged. The separate [effectful-parent conditional-return proof](reference/nuis-native-scalar-loop-snapshots-v1.md#effectful-parent-conditional-returns) guards top-level single pure bool/i64 return computations with current scalar captures while retaining parent effects, exits and the existing speculation barrier. A separate [pure return-prefix proof](reference/nuis-native-scalar-loop-snapshots-v1.md#pure-prefixes-before-conditional-returns) retains bounded fresh let/const values and local records inside their selected return arm, including unused checks when a later logical RHS is skipped; current scalar captures, parent effects/exits and whole-arm work limits remain intact. A separate [pure terminal-tree proof](reference/nuis-native-scalar-loop-snapshots-v1.md#pure-terminal-return-trees) keeps bounded complete nested return branches behind their original outer guard; every leaf returns, sibling scopes stay isolated and earlier constructor checks remain selected work. Parent effects and the continuation suffix are not copied. A separate [pure fallthrough-tree proof](reference/nuis-native-scalar-loop-snapshots-v1.md#pure-fallthrough-return-trees) admits empty continuation leaves beside pure returns, with independent stable-parent exit readiness evaluated after guarded helper work. Real false/zero returns and unused helper seeds cannot be confused; parent suffixes remain untouched. A separate [pure continuation-work proof](reference/nuis-native-scalar-loop-snapshots-v1.md#pure-work-before-continuation-leaves) retains complete fresh let/const work in continuing leaves, direct outer arms and all-fallthrough subtrees opposite an original return. Seeds follow every selected call/check and cannot invent a source exit; computation-only candidates remain outside this return route. A separate [local atom-alias readiness proof](reference/nuis-native-scalar-loop-snapshots-v1.md#local-atom-aliases-for-exit-readiness) flattens fresh boolean aliases of current parent atoms/literals without replaying initializers or moving original local work. A separate [total comparison-readiness proof](reference/nuis-native-scalar-loop-snapshots-v1.md#total-comparisons-for-exit-readiness) admits local bool equality/inequality and signed i64 comparisons of stable scalar atoms/aliases, keeping normalized facts at most three nodes without replaying original calls/checks. A separate [inline partial-tree comparison proof](reference/nuis-native-scalar-loop-snapshots-v1.md#inline-total-comparisons-in-partial-return-trees) allows those same total comparisons directly in internal if conditions beside continuation leaves, sharing exact typed facts and preflight without replaying initializers or selected work. A separate [complete-tree computed-predicate proof](reference/nuis-native-scalar-loop-snapshots-v1.md#pure-computed-predicates-in-complete-return-trees) admits exact bool conditions from bounded pure calls, field reads, checked arithmetic and nested comparisons only when every internal leaf returns. Original conditions and checks stay once-only selected work, with no parent readiness replay or extra exit flag. A separate [stored partial-tree exit-signal proof](reference/nuis-native-scalar-loop-snapshots-v1.md#stored-exit-signals-for-partial-return-trees) now keeps computed pure bool conditions inside their original partial tree and returns one private typed exit/value snapshot. Parent field reads never replay a predicate or call; original logical returns remain rooted and guarded before record construction, continuation seeds cannot invent source exits, and mixed arms wrap original leaves. Shared exact types, budgets and scalar captures remain unchanged. A separate [computed outer-entry proof](reference/nuis-native-scalar-loop-snapshots-v1.md#pure-computed-outer-entry-predicates) now evaluates bounded pure calls, typed fields and arithmetic/comparisons once at the original parent statement site. Helpers and exit readiness consume only the saved owned bool; earlier effects/exits, original checks and both partial-exit routes stay ordered. Current record operands remain parent-local rather than becoming new helper captures. A separate [bounded intermediate-suffix proof](reference/nuis-native-scalar-loop-snapshots-v1.md#bounded-pure-intermediate-suffixes) now carries pure source tails into continuing paths only, stopping at observed returns. Original scopes/types/reachability and both original/expanded budgets are validated before cloning, then existing arm and capture proofs revalidate the normalized tree. Parent effects/exits/continuations remain in place. A separate [single-edge logical entry proof](reference/nuis-native-scalar-loop-snapshots-v1.md#single-edge-logical-outer-entries) admits exact owned bool atom/literal left gates with bounded pure nonlogical RHS. The original typed bool value root uses existing guarded-value lowering so only the selected complete RHS runs; return helpers/readiness consume the saved bool without replay. Whole-root bounds and internal-arm/capture contracts remain unchanged. A separate [logical return-arm root proof](reference/nuis-native-scalar-loop-snapshots-v1.md#guarded-logical-roots-in-return-arms) now admits single-edge internal if conditions and fresh owned-bool let/const roots, including complete/partial trees and bounded suffix/continuation bindings. Shared guards retain complete RHS checks at the original site; non-total exit decisions use private stored signals without replaying gates or initializers. Exact lexical bool gates, shared original/expanded work budgets, source-return authority and scalar-only parent transport remain intact. A separate [computed logical gate proof](reference/nuis-native-computed-logical-gates-v1.md) now admits bounded pure nonlogical owned-bool computations on the left of source value roots and outer return entries, under shared whole-root preflight before inference/cloning. The original left runs once as the first helper argument; the complete selected RHS stays guarded and outer return readiness consumes only the completed original-site bool. Parent records remain parent-local. A separate [computed return-arm proof](reference/nuis-native-computed-logical-gates-v1.md#computed-return-arm-gates) now covers bounded internal conditions, fresh bool bindings, direct return values and pure continuation/suffix bindings, under shared original/expanded budgets and exact fresh lexical scopes. Shared guards retain once-only computed left work and complete selected RHS; non-total exits use stored signals without replay, local record checks stay selected and scalar-only parent capture authority is unchanged. The bounded [nested logical-tree follow-up](reference/nuis-native-computed-logical-gates-v1.md#nested-logical-trees) now guards direct logical children at established source/entry/return roots with original exact typing, shared 32-edge/work/depth budgets and linear helper growth. Complete RHS work remains selected, total outer RHS cannot expose nested LHS checks, partial exits keep stored signals and source loops retain their independent atom-only gate. Logical gates hidden inside ordinary leaves remain unproven. Other inline conditions outside these distinct proofs, over-budget or name-colliding suffixes, effectful/rebinding prefixes, loops and resource captures remain unproven. Neither route grants wider private function eligibility. Delimited call arguments now permit struct literals in conditions, with the enclosing literal mode restored on success and failure. Other nested/short-circuit forms, loops, changing/local versions and scoped/word authority remain conservative. Native probes preserve the complete producer definition and check success, skipped work, selected branches and reached LLVM traps; this is explicit IR acceptance, not new source-function eligibility. See [binding-root materialization](reference/nuis-native-scalar-loop-snapshots-v1.md#binding-root-caller-materialization).
Ordinary child-exit rewrites now retain assignment own-read order, registered break identities and checked compact-record seed types through native lowering.
Break/continue probes run 9/63/64-word states across zero/one/two child trips, now including observed child indices at full width.
Per-write invariants also cover ordinary child loops, independently of return-signal sharing. Dead total input-record reconstructions no longer force full captures; calls, checked fields, codecs and resources stay intact.
The observed break fixture retains two child control words; continue needs no child record backedge. Partly observed total reconstructions now project exact typed fields, including local copies and subrecords, under bounded immutable-input proof.
Their 64-word break/continue workflows retain checked field uses and observed exits, with 4/2 child arguments and source-free restoration. Opaque snapshot calls use private read-only records in multi-carry breaks and no-carry/single-carry continues without discarding evaluation. Every caller must prove an unwritten source root; dynamic loop-state mappings remain i64 and record carry mappings require a multi-carry seed map. Checked constructors fit by reusing immutable nominal layouts and field ranges inside the same proof budget, with fresh per-write origins. The current preheader-snapshot proof retains native field-order traps and source-free 4/6/11 break and 2/6/11 continue checked workflows. Private entry arities are distinct from carry widths; this proves execution, not a measured speedup. Changing outer record backedges and opaque carry mutation remain bounded.
Failed or exhausted nested proof still retains an admitted inner-only plan or the original body, without relaxing source admission.
Scoped capture planning uses complete record seed or read-only input proofs instead of inapplicable ordinary-call boolean packing; read-only transport grants no seed/elision authority.
Exact shared budgets, selected traps and atomic failure publication remain required without widening the native profile.
Generated scoped flat-i64 inputs now use explicit per-trip record maps after every
caller agrees on the complete seed range. A 64-field record plus induction and a
bounded break loop execute natively. Generated loop branch helpers now retain whole
records with guarded call/constructor work instead of exceeding the 64-argument bound.
Generated flat bool/i32/i64/f32/f64 carries now retain typed initial state, per-trip decoding
and immutable snapshots. F32/F64 use exact bit packing; nested pure-scalar carries now use
complete field-segment maps and nominal reconstruction at every depth. Same-named leaves
cannot alias, and nested all-i64 carries require the same complete proof. Sparse typed inputs
now preserve full seed storage and checked initializer work, with exact per-trip slot maps.
All callers must prove canonical codecs before unused input reconstructions can disappear.
Over-limit private return/control carry expansion and resource inputs remain separate;
public signatures, source loop-shape admission and native bounds are unchanged.
Generated non-scoped capture plans now use typed YIR record parameters when scalar
compaction still exceeds 64 parameters. The incompressible 64-field-plus-predicate
case now executes with bounded value transport; public/source/FFI and callback ABI
remain unchanged. Registered parameter contracts check nominal layout and exact
runtime values; the shared codec handles native arguments and returns. Version
documentation and repository cleanup do not change capability scores.

Current session evidence:

- Registered Nuis open/event/close helpers retain scalar aggregate state and
  one provider connection across independent host events.
- Compiled AppKit/Metal regressions check input, actual image pixels, replay,
  worker/cache reuse, explicit cleanup and failure-preserving exit.
- An explicitly selected CPU parent opens before its child and receives one
  owned terminal outcome; its successful cleanup cannot certify child success.
- Pump, window and C ABI expose one independent cancellation ticket.
  It survives window destruction without implicit close, a fabricated terminal
  outcome or parent delivery. The packaged host now consumes it through explicit
  `--window-cancel-after-events`, with real Metal and compiled replay regressions.
- Cancellation admission, host-scope retirement and provider/device resource
  retirement are distinct. Explicit registered-worker drain now has provider-owned
  evidence; an opt-in host-library cancellation ticket separately observes it.

The [provider-session drain boundary](reference/nuis-yir-provider-session-drain-v1.md)
now distinguishes explicit worker-scope retirement from successful completion,
without publishing replacement replay. Pump/window/C ABI drain admission is now
connected through the generic provider-scope boundary, with first-fault and damaged
exchange guards. The packaged script now accepts opt-in `--drain-provider` behind
an exact bundle capability; the shared policy requires a typed Drained terminal
and cancellation exit, rejecting Finish before publication. It never publishes
cancelled work as successful launch/trace evidence. The terminal contract and
launch policy have no OS/window/backend dependency. A separate
[headless scalar-script profile](reference/nuis-yir-application-scalar-script-v1.md)
now packages the same pump without AppKit, with compiled protocol and real M2
image/drain evidence. Ordinary `nuis build/run-artifact` now select
`headless-aot-bundle`, verify exact capabilities, scalar signatures and image
identity before preparing providers, and reuse the existing typed launch policy.
Explicit host selections are cache-isolated. Headless builds now consume a
verified-YIR checkpoint without CPU LLVM codegen. The complete source-to-YIR
handoff travels with the artifact; no empty LLVM file substitutes for a stage.
Manifest-selected headless check/dump, benchmark/binding inspection and workflow JSON
now use the same checkpoint, with `llvm_emit=not_requested` and no invented LLVM
byte count. Default native diagnostics remain in place. Bounded Buffer-writing
callback loops now outline to private helpers under the existing scoped-call YIR
contract. CPU-owned cooperative execution invokes the helpers rather than logging
calls; expression-level memory effects preserve source order and native indices
are checked. PixelMagic now uses these loops instead of recursive pixel filling.
The actual CLI build/run-artifact regression passes two M2 Metal frames and exact
direct-session replay, rejects executable/YIR drift and invalid callback arguments,
and retains prior evidence on replay exhaustion. Independent native/reference
tests compare every generated pixel; invalid scalar division/remainder no longer
panics in constant evaluation or reaches undefined native arithmetic. Conditional
pixel writes now use nested guarded helpers and one-time condition snapshots;
untaken invalid reads/writes or arithmetic do not execute. Scalar-only branch
traps cannot be discarded as dead bindings. Source-level scalar helper composition
now admits synchronous, acyclic `i64`/`bool` callees, with transitive
body checks and real ordered calls under shared fuel. Buffer-read arguments remain
inside the selected branch. PixelMagic's two-level coordinate/color helpers exercise
this surface, with imported private helpers retained in their owner's scope rather
than exposed as public functions or resolved against another module's same-named
implementation. Nested scalar-helper `if`/`else` and early returns now use typed
guarded function blocks with shared fallthrough continuations, rather than
speculating branch-local arguments or callee bodies. Regressions cover scope
isolation, both return types, traps, source order and shared callback fuel.
Multiple source-ordered i64 accumulators now cross Buffer-writing iterations through a
shared scoped-call contract and ordinary LoopState field projections. Updates keep
source order; zero-trip loops preserve all seeds and only fully validated helper
returns update the private state. PixelMagic exposes fill-and-statistics with exact
native/reference pixel, red-count and pixel-sum tests. Ordinary LLVM aggregate returns
still allocate and release storage; the explicit native-session value profile below
is separate. Neither route establishes performance parity.
The carried generator also passes ordinary M2 build/run-artifact with two exact
Metal frames, direct-session byte parity and unchanged failure admission.
Branch-local and repeated scalar updates now compose with those writes. Guarded
arms preserve incoming seeds when unselected; nested arms and subsequent branches
see source-ordered updates. PixelMagic counts red pixels within its red write arm.
Nested bounded Buffer-writing loops now compose through the same private functions,
with independent counters, scalar carries, guarded child bounds and shared fuel.
PixelMagic uses row/pixel loops, not a fixed two-dimensional runtime operation.
NIR branch joins invalidate stale literals and preserve live outgoing assignments.
Guarded `continue` now preserves prefix writes/carries and skips the whole suffix,
including nested loop bounds, when the source path explicitly performs the matching
unit step. Each loop owns its control scope; no runtime-specific opcode is added.
PixelMagic uses this path after finishing each red pixel.
Guarded `break` now exits before stepping through a shared flat-i64 control contract
consumed by the CPU registered driver and LLVM. Native/reference and session tests
preserve prefix state, reject invalid control values and stop a trillion-bound loop
within 1000 shared fuel. PixelMagic `recolor_run` now passes ordinary M2 headless
build/run-artifact: four same-color pixels are marked before real Metal inversion,
the fifth helper invocation stops at index 4, and exact pixels, writes, identity
admission and replay remain verified. Owned-Bytes cleanup returns now propagate
declared aggregate layouts through LLVM, restoring the default-AOT image checkpoint.
Nested aggregate cleanup helpers have native/reference parity, declaration-order
invariance and zero live native Bytes after execution. The executor asks each
registered module for function exits instead of dispatching on CPU operation names.
A [static scalar callback bridge](reference/nuis-native-scalar-session-bridge-v1.md)
now executes registered open/event/close helpers natively, with nested typed slots,
reference parity, in-place state and rejection before callback entry. This selected
profile now admits bounded acyclic scalar helper calls and bounded i64 loops with
constant or runtime-checked induction, but excludes resources and providers. Explicit packer
selection now uses the shared application-session host with static exports, full
YIR identity admission, last-valid-state preservation and one cleanup attempt.
Production-host tests reject stale objects and native failures without interpreter
fallback. Explicit `native-session-aot-bundle:<id>` selection now passes ordinary
`nuis build` and `run-artifact --native-session`, with real LLVM checkpoint
regeneration, cache registration isolation and standalone materialization after
removing the original output. Required documentation/package metadata is restored
with bound inputs rather than read from the old host paths. Nested helper calls
now retain all five scalar kinds, reject recursive/effectful/type-drifted closures,
and survive cache reuse and independent artifact restoration. Counted i64 loops
now compose with those helpers, source-ordered add/multiply carries and zero-trip
seeds. Constant induction is proven before emission; runtime i64 start, limit
and step receive an in-place preflight before loop entry. Both require finite,
non-wrapping induction within 65536 iterations per loop. The CPU registered driver now returns real
plain-chain state under shared reference fuel instead of logging a unit value;
shared loop input dependencies no longer duplicate effect edges. Real native
executions and the build/run-artifact restoration regression cover this subset.
A 1680-case native guard probe matches checked-step simulation across six
comparisons and add/sub, including signed extremes, zero trips and skipped guards.
Rejected inputs enter no loop iterations; six unmodified native trap executions
confirm process failure rather than a catchable callback/fuel result. Scoped
scalar loop-body calls now reuse the existing YIR action and LLVM emitter, with
discarded scalar results or one carried i64 return. Scoped targets join the same
bounded acyclic closure; captures require exact scalar kinds and caller-local
values. Six native/reference runs cover the scoped Nuis fixture and declaration
reordering. Two additional executables observe 36 callback cases and 84 actual
helper invocations, including all five scalar kinds, NaN/signed-zero bits,
pre-step carry order and zero-trip preservation. Three process-trap cases prove
invalid induction never enters the helper. The build/run-artifact restoration
regression now uses a multi-carry scoped-loop fixture. Flat multi-i64 scoped
returns now use the shared layout/seed contract and the same bounded helper
closure. Six more native/reference runs cover the multi-carry session fixture.
Six native executables observe 144 callback cases and 288 helper invocations
with 2/3/7 carries, reversed argument order, sequential updates, zero/one trips,
exact scalar captures and balanced real aggregate allocation/drop. Three more
process-trap runs reject invalid induction before the first helper call.
Aggregate helper reachability now includes registered, exported and noinline
roots, not only main; counter-only fallback rejects discarded outer updates.
Scoped guarded break now reuses the shared return/control contract: validate the
zero seed and binary returned control, release the aggregate, commit all carries
and exit before stepping. Pure scalar source loops reuse private normalization;
break-only loops need one control bit rather than an aggregate branch result.
The full bounded induction preflight still applies even for an immediate break.
Six native binaries cover 180 callback cases and balanced aggregate release;
eleven real traps cover seed/return/induction rejection. The frontdoor restoration
suite retains the multi-carry fixture and adds the guarded-break fixture.
Checked flat-i64 branch-helper returns now reuse ordinary aggregate calls with
exact return-layout, scalar-parameter and bounded-closure validation. Multi-state
guarded updates, break and explicit-step continue now execute natively, including
nested scope and both induction directions. Six nested-call binaries cover 144
callback cases and 576 inner helper calls with balanced real aggregate release;
selected-path tests skip an unreachable excessive-loop preflight but trap when it
is reached. The frontdoor adds the multi-state branch fixture through cache
isolation and standalone restoration. Checked i64 division/remainder now have
exact-kind native admission and reuse existing zero/overflow guards. Eight native
binaries compare 1840 callbacks against reference execution and a wide-integer
oracle; 20 dynamic and four literal invalid cases trap without returned state.
Four unselected invalid literal cases return safely. Frontend purity no longer
licenses arithmetic speculation: acyclic i64/bool helpers reuse guarded outlining.
Flat-i64 aggregate values now use the same outliner with neutral guards, existing
record captures and shared suffixes. Fourteen more native binaries compare 3220
callbacks across early/two-arm/nested returns, unused results/arguments and direct
registered-state branches; 32 dynamic and four literal invalid cases trap, while
four unselected literal cases return safely. Real allocation/drop probes stay
balanced after every successful callback and before the observed arithmetic leaf.
Ordinary fields retain declared names; scoped carry/control schemas remain strict.
Thirty-two source guards have linear helper growth. Single-induction counted
loops now compose in flat-value helper branches, including inline arms, prefix
work and shared suffixes. An iterative call-graph flag retains the loop boundary
even without checked arithmetic. Independent checked-step/reference probes cover
2090 accepted or skipped callbacks and 26 real traps, including zero iterations
before rejected preflight and the exact 65536-trip bound. Guarded helpers now also
admit ordered linear i64 carry updates through shared chained-loop preparation.
Another 2849 accepted/skipped callbacks cover all carry slots, wrapping arithmetic,
1/3/7-carry widths and selected-path evaluation; 27 real traps retain preflight
before any update and reached arithmetic failure. Extreme seeds exposed and fixed
Rust-debug-dependent scalar integer overflow in the reference CPU. No backend loop
opcode or Buffer admission was added. Full if/else carry updates now reuse the
existing conditional-chain opcode and LLVM emitter, with strict leaf-condition,
exact-i64 and metadata checks before emission. The reference CPU now executes the
same scalar conditional chains cooperatively rather than returning trace-only Unit.
Another 1357 accepted/skipped callbacks and 27 real traps check selected updates,
source order, all six comparisons, reversed operands, wrapping and preflight.
Omitted and explicit empty carry arms now share `keep` preparation without a new
opcode or ABI. Thirty-nine native binaries add 2502 accepted/skipped callbacks
and nine real traps, including own-value retention, earlier updated carry reads,
all six comparisons, variable widths, dynamic stride and the exact trip bound.
Six more typed lifecycle runs and reference-fuel failure preserve accepted state
and cleanup. Both-empty arms, invalid seeds and effectful/fallible updates still
reject. Pure comparison leaves now compose through existing `and/or` condition
trees and short-circuit LLVM blocks. Thirty-one native binaries add 1216
accepted/skipped callbacks and nine real traps; volatile comparison counters
agree with independent short-circuit evaluation, not just final state values.
Nested RHS kind drift rejects even on zero trips. A shared parser depth guard
also protects the general verifier, which runs before native admission.
Nested carry-update arms now use scoped iteration helpers with guarded branch
calls. The driver retains finite, non-wrapping induction preflight; decisions see
the source's stepped index and earlier updated carries. Three distinct leaf
updates and implicit keep no longer require collapse to two values. Compound
conditions inside these helpers use neutral-false guards and preserve actual
short-circuit comparison counts. Multi-statement bodies now support repeated
writes, asymmetric branch write sets, and ordered prefix/branch/suffix updates
through the same iteration helper. Each seeded carry has one return slot regardless
of its write count. Branch-local validation does not borrow writes from a sibling
arm; after the join, an untaken arm preserves the binding's own input value.
Iteration-local i64/bool temporaries now use lexical initialization scopes and
stay inside the iteration helper. Inferred bool expressions use the same lazy
predicate lowering as annotated ones; stored values survive later mutations.
Branch-local names do not escape even when both arms declare the same spelling.
Mutable i64 temporaries can cross inner branch joins, but only pre-existing carries
return to the outer driver. Checked i64 division/remainder now use that same scoped
route for direct carry updates, local initializers and short-circuit conditions,
without requiring a temporary declaration. Unselected and zero-trip paths skip
arithmetic; reached zero/signed-overflow errors trap even for unused values.
Independent result/iteration oracles retain induction preflight and successful-path
allocation/drop checks. Scalar helper calls now use the completed pure,
acyclic closure with exact i64/bool arguments and results. Logical call arguments
stay lazy, even in i64 initializers; native probes match actual argument values,
call order and iteration positions. Flat-i64 helper results now compose with scoped
locals, exact-layout copies, projections, aggregate arguments and guarded captures.
Shared effect outlining distinguishes pure values from Buffer access explicitly;
Buffer admission is unchanged. Dependency-ordered validation now admits bounded
loop-bearing helpers and their wrappers inside iterations without a fixed number of
catalog passes. In the native-session profile, every selected invocation checks its
own complete induction after argument evaluation; invalid or cyclic dependencies
never release callers. Newly emitted native callbacks also reserve complete induction
counts against one callback-stack counter forwarded through synchronous helpers.
The default is 1,048,576 per callback; early exits do not refund reservations,
zero-trip/skipped paths cost nothing, and exhaustion traps before the rejected body
or callback output. A second independent stack counter now limits actual YIR function
entries to 1,048,576, including roots, scoped helpers and neutral-return guards.
Conditional calls in the admitted scalar catalog stay behind guards even without
loops or checked arithmetic. Loop-free binary call-tree expansion now has real
native rejection evidence. Both policies survive cache reuse and standalone
restoration; neither promises wall-time, peak-memory or device-work bounds.
Iteration-local bool rebinding now preserves lexical initialization, old snapshots,
one-sided/nested joins and lazy calls. Private branch returns encode bools as 0/1
in existing i64 slots and restore their source types, without changing outer loop
state, the public ABI or Buffer admission. Iteration-local flat-i64 record rebinding
now uses declared-order private word transport and reconstructs exact nominal types;
old snapshots, nested/mixed joins, checked failures and shared budgets are retained.
Outer flat-i64 mutable records now cross loop backedges through the same private
word transport, preserving zero-trip seeds, nominal layouts, old snapshots and
source-ordered updates. Record slots are independent of helper capture order, including
single-field records and multiple records mixed with scalar carries. Outer bool
locals now retain typed seeds and snapshots through explicit canonical word
conversion at the private iteration boundary, including singleton bool carries.
The bool-carries fixture crosses build/cache/source-free standalone restoration.
Literal nested counted bodies now compose through the same scoped helpers, with
independent selected-invocation preflight and shared loop/entry budgets. Rectangular
and triangular loops retain typed bool/record carries and lexical child scope;
late failures and skipped invalid children have native probes. The literal-loops
fixture adds build/cache/source-free standalone restoration. Leading-step pure-value
loops now also admit guarded `break`/`continue` with loop-local control and ordered
prefix effects. Private carry recovery preserves the already-advanced induction
without a new opcode or callback ABI. Full preflight and work reservations are not
shortened by early exits; actual helper entries remain independently charged.
The value-loop-exits fixture crosses the same source-free frontdoor. Trailing-step
pure-value loops now retain pre-step effects, break indices and matching explicit-step
continue through the same contract, including mixed leading/trailing child loops.
Admitted value functions bypass Buffer rewriting, avoiding a double-normalization
crash. Existing ordinary-entry counted/flow fast paths stay intact. The
trailing-value-loops fixture follows the same source-free workflow. Counted-loop
returns now transport i64, bool, i32, f32, f64 and exact flat bool/i32/i64/f32/f64 records through seeded private
values and loop-local break, propagating child returns out of the whole function.
Return expressions run at their source position before the pending flag is set;
skipped suffixes do not run, entered loops retain full reservations and failed
callbacks do not publish partial output. The counted-returns fixture passes the
same source-free workflow. Ready-value, terminal and statement-suffix elision restore
the full control-composition fixture that exceeded the unchanged 64-function bound:
it emits 51 reachable native functions, down from 54, and retains native/reference parity.
Only integer/bool literals and already-initialized scalar/flat-record values bypass
private helpers. Conditions still execute once; calls, field projections and record
construction remain guarded. A terminal expression with one generated use no longer
needs a forwarding function; multiple uses still share their suffix instead of copying it.
Actual helper entries decrease, but loop reservations and both budget policies stay
unchanged. Multi-statement suffixes with a single generated use also fold, retaining
unused calls and shared inner suffixes. Scope-aware private naming now separates
colliding locals without renaming function symbols, fields or outer loop bindings;
bool/record backedges retain their original binding identities. The full composition
now includes a same-name record/scalar prefix within the same 51-function bound.
The selected [native value-return profile](reference/nuis-native-scalar-value-returns-v1.md)
now removes aggregate heap allocation from admitted nested scalar helper and callback
returns. It preserves exact layouts, raw scalar bits, independent snapshots, overlap
and failure publication. Ordinary owned/resource aggregates retain their separate ABI.
Generated capture transport packs boolean leaves and projects only demanded fields,
without changing user signatures or the independent native argument bound. A sparse
64-i64 state needs four private selection arguments; stable aliases and straight-line
rebound snapshots use two/three in their fixtures while public inputs remain unchanged.
Forward substitution materializes old snapshots before later writes; record versioning
retains initializer work and selected arithmetic failure. The sparse, alias and snapshot
frontdoor cases retain cache reuse, tamper rejection and source-free restoration.
Same-name branch-local aliases now acquire independent identities on the candidate
copy before projection; writes to an existing outer/parameter binding retain its
identity. Nested and suffix declarations remain isolated, and unsafe callers veto
the whole candidate. Scoped-alias native and frontdoor regressions retain exact state,
selected traps, cache identity and source-free restoration. Straight-line branch-local
record rebindings now use separate snapshot versions when all writes remain in one
non-loop scope with an exact type. Nested reads inherit the version at entry;
siblings and suffixes do not inherit branch-local versions. Returning non-loop branches
now receive separate versions for writes to ancestor records. A separate copy-family
proof now projects field-only fallthrough and loop-written inputs through nominal
reconstruction without changing assignments, per-trip aliases or backedges. Whole
escapes and unknown/mismatched types remain conservative. Single-definition
loop aliases now project field demand from unwritten pure-value parameter origins;
generated scoped iteration targets now project invariant fields and retain their
independent argument maps. The 64-field source loop uses four rather than 66 iteration
arguments; nested break identities retain their existing mapping.
Explicit flat-i64 field arguments can now omit unread slots while independent seeds preserve complete initial state.
Generated scoped helpers now project partially read and entirely unread flat-i64 record inputs after
separating parameter snapshots from updates and proving every scoped caller's exact
seed/reconstruction map. Source regressions retain 3/7/64 state slots with two/three
iteration arguments. Empty demand requires every scoped caller to agree on the nominal type,
width and result-slot range, with compiler-private proof passed to lowering only
after a successful rewrite. Full uses, unresolved joins and nested writes
remain conservative; sparse state storage is not implemented by this projection. These are bounded CPU
correctness/transport results, not native resource/provider dispatch or measured runtime speedups.
The parser also rejects excessive call/group expression re-entry before stack
exhaustion, independently of other frontend recursion, NIR expression-depth and
native call-graph admission.
The division, aggregate-division, aggregate-counted, aggregate-carried,
aggregate-conditional, aggregate-one-sided, aggregate-compound, aggregate-nested,
aggregate-sequences, aggregate-temporaries, aggregate-checked, aggregate-calls and
aggregate-local-values, aggregate-loop-calls, bool-rebinding, aggregate-rebinding,
aggregate-carries, bool-carries, literal-loops, value-loop-exits, trailing-value-loops,
counted-returns and control-composition
fixtures compose with typed lifecycle and multi-state break/continue. The frontdoor
suite also covers typed value returns, capture words, sparse fields, aliases and
snapshots, alongside loop-work/helper-entry exhaustion and lifecycle reset, cache isolation, tamper rejection and
standalone restoration. Use the execution logs for the tested revision/platform,
not the test count alone, as acceptance evidence.
The passing default image host still executes embedded YIR.
This does not certify arbitrary loop bodies, richer carry payloads or fully native CPU callbacks.
Linux hardware and Windows transport
are not certified by the portable tests.
Cross-session resource reuse remains a separate, unproven boundary.
The default host-only path acknowledges its own scope, while the live provider
still treats EOF without Finish as incomplete execution. Ordinary quit still uses
close/Finish; parent cancellation remains unsupported. Do not import provider
registries, transports or concrete cancellation state into the window adapter.

This does not close resource-capability state, peer recovery, general multi-child
routing, richer image bindings, sustained performance, native CPU frame dispatch
or a self-contained Nsld application image. The broad engine coordinate is not
an engine-completion percentage.

## Current Truth By Layer

| Area | Current Reference |
| --- | --- |
| Development model | [Tensor protocol](reference/nuis-development-tensor.md), [mainline selection](reference/nuis-development-tensor-mainline.md) |
| Production architecture | [Software manufacturing direction](reference/nuis-software-manufacturing-architecture.md), [RC policy and current limits](reference/nuis-rc-cache-lifecycle.md) |
| Frontend/lowering | [Control flow](reference/control-flow-lowering-contract.md), [generic diagnostics](reference/generic-diagnostic-ownership-contract.md), [source encoding](reference/source-text-encoding-contract.md) |
| Memory/task safety | [NIR memory](reference/nir-memory-model.md), [task/GLM](reference/cpu-task-glm-contract.md), [thread/lock boundary](reference/cpu-thread-lock-boundary.md), [FFI pointers](reference/ffi-pointer-safety-boundary.md) |
| Runtime sessions | [Application lifecycle](reference/nuis-yir-application-session-v1.md), [window](reference/nuis-yir-window-session-v3.md), [terminal outcomes](reference/nuis-yir-application-outcome-v1.md), [parent pump](reference/nuis-yir-application-outcome-pump-v1.md), [cancellation](reference/nuis-yir-application-cancellation-v1.md) |
| Nustars | [Capability ownership](reference/nustar-capability-split-boundary.md), [multi-backend artifacts](reference/nustar-multi-backend-artifact-contract.md), [CFFI domain](reference/cffi-von-neumann-domain-contract.md), [provider IPC](reference/nuis-yir-provider-runtime-ipc-v4.md) |
| Linking/launch | [Native workflow](reference/nuis-native-artifact-workflow.md), [NSB protocol](reference/nuis-binary-format-protocol.md), [Nsld](reference/nsld-linker-frontdoor.md), [assembly gaps](reference/nsld-binary-assembly-gap-map.md) |
| Debugging/distribution | [Nsdb](reference/nsdb-yir-debugger-frontdoor.md), [Nsbdr](reference/nsbdr-bundler-frontdoor.md), [reusable capability boundary](reference/toolchain-galaxy-core-boundary.md) |
| Official Galaxies | [Std](../stdlib/std/README.md), [PixelMagic](../stdlib/pixelmagic/README.md), [WitSage](../stdlib/witsage/README.md), [ns-nova](../stdlib/ns-nova/README.md) |
| Compiler migration | [Readiness](reference/nuis-self-hosting-readiness.md), [data model](reference/nuis-compiler-data-model.md), [stage handoff](reference/nuis-compiler-stage-handoff.md), [candidate production](reference/nuis-compiler-candidate-production.md), [candidate-to-Nsld boundary](reference/nuis-compiler-candidate-nsld-materialization.md) |

The five bounded migration-preparation gates remain closed; actual compiler
responsibility transfer is separate. Use the readiness/component references for
canonical payloads, trust pins, differential/reproducibility and rollback
commands rather than reproducing their entire history in this router.

## Fast Example Routes

| Task | Start With | Evidence Boundary |
| --- | --- | --- |
| Basic host CLI | [filesystem_io_report_demo](../examples/projects/tooling/filesystem_io_report_demo), [stdin_runtime_demo](../examples/projects/tooling/stdin_runtime_demo), [cli_report_file_demo](../examples/projects/tooling/cli_report_file_demo) | Native executables with observable I/O and output-file checks. |
| Text statistics | [cli_wc_demo](../examples/projects/tooling/cli_wc_demo) | One 4096-byte read and ASCII separators, not complete streaming `wc`. |
| Simple file/image transform | [cli_pgm_invert_demo](../examples/projects/tooling/cli_pgm_invert_demo) | Checked small PGM input/output, not a general image-codec library. |
| Current application goal | [ns_nova_image_showcase](../examples/projects/domains/ns_nova_image_showcase) | Real Metal processing, compiled export and explicit stateful AppKit window; embedded YIR host runtime remains. |
| Small render/uniform route | [ns_nova_showcase](../examples/projects/domains/ns_nova_showcase) | Bounded frame loop, registered render resources and live/replay evidence. |
| Linker closure | [native_artifact_closure_demo](../examples/projects/tooling/native_artifact_closure_demo) | Inspect the selected finalizer and admission evidence; a planned image is not automatically runnable. |
| Compiler component | [bootstrap_structural_projection_candidate](../examples/projects/tooling/bootstrap_structural_projection_candidate) | Bounded candidate proof, not general self-compilation or replacement authority. |
| Linux device work | [CUDA bring-up](reference/linux-cuda-provider-bringup.md), [domain examples](../examples/projects/domains/README.md) | Run provider-specific gates on suitable hardware; absent hardware is unverified, not fallback success. |

The [observable CLI regression](../tools/nuis/tests/std_filesystem_smoke.rs)
builds and runs seven project routes, including PixelMagic/WitSage reports.
Those reports alone are not device-execution evidence. The separate Metal tests
check actual dispatched image data.

The packaged AppKit application now exposes standalone scripted cancellation
through `--window-cancel-after-events`. This does not add a Nuis intrinsic,
CFFI signature grant, parent cancellation route or device-retirement authority.

## Deep Routers

- [Documentation index](README.md) and [reference index](reference/README.md)
- [Project examples](../examples/projects/README.md), [source examples](../examples/ns/README.md), [freshness audit](examples-freshness-audit.md)
- [Std tooling](../stdlib/std/tooling/README.md), [filesystem](../stdlib/std/filesystem/README.md), [network](../stdlib/std/network/README.md)
- [Repository layout](repo-layout.md) and [file-line policy](repo-file-line-policy.md)
- [Application-led design](versioning/nuis-beta-0.11-application-led-mainline.md) and [long-range OS roadmap](versioning/nuis-long-range-heterogeneous-os-roadmap.md)

Rendering, control, ML and audio are an engine horizon, not four completed
workflows. Shared YIR/GLM/time contracts preserve independent Galaxy/Nustar
ownership. Hardware extensibility and scheduling efficiency require separate
measurements; a smoke test is not an advantage over MLIR or a mature language.

## Cleanup Rule

Keep this map short. Current behavior belongs in focused reference documents;
minor checkpoints belong in versioning, and examples state their own input,
backend and execution limits. Keep paths repository-relative. Update tensor
evidence and documentation drift guards after each accepted change, without
raising capability scores for documentation work.
