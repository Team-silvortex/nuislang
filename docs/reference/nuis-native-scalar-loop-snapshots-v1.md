# Native Scalar Loop Snapshot Proof

This reference describes the current `beta-0.15.*` return-loop snapshot proof,
including the 2026-10-03 post-loop extension and capture normalization. It supplements
[native value returns](nuis-native-scalar-value-returns-v1.md) and the
[session bridge](nuis-native-scalar-session-bridge-v1.md); it does not replace
the historical `beta-0.15.6` release checkpoint.

Each extension below is an independent bounded proof, not universal admission.
Later proofs supersede earlier rejection boundaries only for matching shapes;
checkpoint counts remain historical. In particular, the stored exit-signal route
does not turn computed predicates into replayable facts.

Within this bounded return rewrite, a loop no longer invalidates every field of
a written record at exit. Fields proved unchanged throughout the loop retain
their evaluated identities for later code, including subsequent nested return
loops. Fields that may change receive new identities, independently of
loop-entry identities.

## Shared Summary

The [bounded proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariant_parent_entries.rs)
checks exact nominal types and leaf widths before computing one monotone loop
summary. This includes zero trips, every intermediate publication, nested writes
and arbitrarily many later trips. Each known origin can only become unknown;
the existing 65536-work and 64-depth budgets still apply.

Parent re-entry and post-loop boundaries materialize this same immutable summary
separately. Known origins survive unchanged. Each unknown field receives a fresh
version per binding and boundary: two unknowns cannot establish equality, and a
post-loop value cannot accidentally equal an old evaluated copy merely because
the entry and exit used the same summary. Only entry-visible bindings survive;
child locals cannot escape. Failed materialization publishes neither an
environment nor a partially advanced version clock.

This is deliberately not a final-assignment-only proof. A temporarily changed
field remains unknown even if a suffix restores it: guarded break/continue can
expose that intermediate value. Delayed aliases and separately evaluated opaque
calls also cannot recover an old identity without proof.

The [rewrite](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariants.rs)
retains every original RHS's computed work, selected arithmetic failure and observed
exit. Source admission and whole-function revalidation remain mandatory. An
exhausted or rejected nested rewrite still tries the admitted inner-only plan
and otherwise retains the original body. No source/public/callback/FFI ABI,
native carry bound, return-signal ownership or resource/provider authority changes.

## Execution Evidence

The preceding-loop fixture changes a right-hand field before entering the
existing mixed/nested return workflow. Native/reference probes cover 9- and
64-word state, zero/first/later trips, and ordinary/break/continue exits. Subsequent
loops preserve inner N-4/outer N-1 carries, hence 60/63 at width 64.

Before capture normalization, the width-64 sorted private iteration arities were
7/12/62 for an ordinary pre-loop, 7/12/63 for break and 7/12/62 for continue.
They are now 5/6/12, 6/6/12 and 5/6/12 respectively: the preceding helper itself
shrinks from 62/63 arguments to 5/6 without widening any native bound. Baseline
and parent-entry return helpers use 5/11 and 6/12; the narrow fixture uses 9/11.
Changed-tag and joined-preheader helpers use 4/11. Checked child break/continue
fixtures use 4/6/11 and 2/6/11; opaque counterparts retain 6/6/11 and 4/6/11.
Fewer carried words, fewer private arguments and measured speed are distinct claims.

The differential test exercises 108 combinations of pre-loop and later-loop
limits, exit choices, temporary tag changes and selected division checks. It
compares the optimized form, independent return storage and a hand-written
arithmetic oracle, including old and newly evaluated copies.

The source-free full-width workflows require repeated events, cache hits,
pre-open tamper rejection and byte-identical LLVM/state after deleting source,
manifest and the original output, then restoring the standalone artifact.
Changed tag/count fields must still reject an over-limit native carry layout.

## Typed Literal Origins

The [literal-origin proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_invariant_literals.rs)
recognizes exact `i64`, `i32`, `bool`, finite `f32` and finite `f64` values.
Float identity uses the reference scalar parser's typed bit representation,
not numeric equality: signed zero and scalar kinds remain distinct. Invalid,
nonfinite and overflowed float text grants no literal origin. Text length is
charged before parsing under the same 65536-work budget. Canonical NIR integer
width casts preserve only a proven literal operand, using wrapping narrowing and
signed widening after exact operand-type/width checks. Callee spellings, opaque
calls, arithmetic and word codecs grant no constant authority.

This proves stored identity, not permission to erase a constructor or fold
arbitrary computation. Every original RHS still executes once at its original
position. A matching literal must agree with the actual preheader snapshot and
every intermediate write, on every branch and across the loop fixed point.
Different values, delayed aliases and a temporary change later restored remain
varying. Join, parent-entry and post-loop summaries reuse these typed origins;
literal parsing exhaustion publishes neither a snapshot nor an advanced clock.
Whole-function admission/revalidation and inner-only fallback remain unchanged.

The new mixed/nested fixture initializes one wide-state leaf to `99` on both
preheader arms, then repeats that literal while changing tag and count. At width
64 its inner/outer carries fit 60/64 words, rather than 61/65 when that leaf is
backed by separately evaluated opaque calls. The opaque 65-word version still
rejects before native emission. Public state and the source helper's flattened
ABI remain 64 words; the fixture retains a 57-word readonly private record. A
second native variant initializes the repeated `i32/bool/f32/f64` right-hand
leaves too: its outer carry falls from 64 to 60 words, while independent inner
return storage still needs 60. All 64 output words and zero aggregate
allocation/drop remain checked. These are different bounds, not permission to
widen any native limit or mechanically remove every constant from every layer.

Seven origin tests cover typed identity, wrapping/sign extension, signed zero,
float rounding, every intermediate write, joins, nested/zero-trip summaries,
delayed values, opaque work and budget/depth failure. A separate differential
test compares 72 combinations of loop limits, ordinary/break/continue exits,
temporary tag changes, early returns and selected division checks with independent
return storage and an arithmetic oracle, with reversed YIR storage.
Native probes compare every state word at widths 10/63/64, overlapping transport,
canaries and zero aggregate allocation/drop. The source-free workflow checks
two events, cache reuse, pre-open tamper rejection and repeated materialization
after deleting source, project metadata and the original output. A selected
checked-work fixture must publish only the already accepted open state on trap;
zero trips skip the check and host `SIGKILL` cannot satisfy the failure test.

## Capture Normalization

The [private copy proof](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_record_copies.rs)
replaces only unobserved, total copies from unwritten value inputs, using exact
nominal fields/types and conservative whole/subrecord demand. Each binding must
have one definition and cannot shadow a parameter. Calls, arithmetic, codecs,
local versions and resources never grant input readiness. Non-total fields remain
at their original positions, even when their result is unused; their evaluation
count, field order, guards and selected failures are unchanged.

Canonical source/wire layouts come from proven word-input metadata, not source
spellings. Call and aggregate-return operands retain their immutable provenance
chains: following only single-definition locals protects later enclosing
all-caller codec proofs, while mutable publications stop the alias walk. Whole
escapes and ambiguous writes remain conservative. The 65536-work/64-depth ceiling
also bounds scanning, demand, expansion, zero construction and provenance; failure
does not install a partial body. Every caller must still validate before the
candidate body and private signature are committed together.

Ten focused unit tests include 48 guarded-return combinations compared with an
independent arithmetic oracle. They retain unused checked calls before/after an
early return, reversed YIR storage, exact call/codec transport, nominal types,
shadowed/local inputs, all-caller rollback and bounded-work/depth rejection.
Native probes separately retain zero trips, break/continue, exact mixed bits,
shared entry/loop fuel and failure-atomic aliased/disjoint publication.

## Scalar Input Aliases

The [scalar alias proof](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_scalar_aliases.rs)
now exposes single-definition scalar copies before record-view and capture
normalization. An origin must be an exact supported value path from an unwritten
input. Aliases inherit that origin, not the identity of an arbitrary evaluated
local. Exact declared types, lexical statement order and independent branch/loop
environments remain mandatory. Globally repeated names, parameter shadows,
changed inputs, calls, arithmetic, codecs and resource/reference types cannot
authorize substitution. No branch or loop local escapes into a following scope.

Source scanning, scope/path copying, field lookup, rewriting and expanded result
validation share one 65536-work budget. Input and expanded expressions must both
remain below 64 depth. Failure leaves the entire original body intact; the
surrounding capture projection must still validate every caller before committing
the body, signature and call operands together. Scoped transport helpers are not
normalized by this pass, and registered control identities are explicitly
protected by the calling helper's registry metadata, never a name heuristic.

The 64-field fixture previously demanded every input leaf through local aliases.
Its tested private guarded branches now use 2/3 arguments (including the guard),
or 3/3 when an unused selected field still evaluates an opaque scalar call.
The public record remains 64 words. Ten unit tests include a two-storage-order
64-to-1 capture proof and 48 guarded-return cases checked against an independent
arithmetic oracle; selected division failures and early-return timing remain.
Native tests check all 64 state words, canaries and zero aggregate allocation/drop,
including positive/negative values and an unselected zero divisor. Standalone
artifact acceptance previously covered three variants, cache reuse, selected failure, tamper rejection
before open and identical LLVM/state after deleting source and the original build.
The unused-call fixture observes the result as an ordinary field read. Forwarding
that computed record's field into another call also protected the complete
transport chain at that checkpoint; the follow-up below now distinguishes scalar
observations from aggregate transport. A separate native/storage test keeps the complete record in a
single readonly `[64 x i64]` parameter and executes it without heap aggregation.
Together with its guard this is two typed arguments, not 65 scalar arguments;
record width, native argument count and carry width are different contracts.
This is bounded input-alias closure, not general local/call-result optimization or
a measured performance improvement.

## Scalar Call-Field Demand

The [record-copy proof](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_record_copies.rs)
now classifies an exact scalar field of a single-definition nominal constructor
as a leaf observation, not aggregate transport. Scalar calls may
consume this field directly or through immutable locals without protecting every
sibling input copy. Field lookup is charged to the existing 65536-work budget;
the 64-depth preflight, exact nominal paths and whole-body rollback remain.
Unknown paths, repeated writes, registered source/wire transports, whole records
and record-valued subpaths keep the conservative provenance chain. Fields inside
composite/encoded operands or aggregate returns retain complete source identities;
this pass does not reinterpret registered wire construction as scalar demand.

Only unobserved total copies of unwritten value inputs may become zeroes. Computed
sibling fields, codecs, checked arithmetic and calls stay at their original sites;
the pass neither inlines a call nor assumes its result is an input or constant.
Every caller still validates before a generated body/signature/operand change.
No public/source/callback/FFI ABI or native limit is widened.

The computed 64-field snapshot forwarding `saved.f0` to a scalar helper now
captures two i64 fields plus its guard, instead of a readonly 64-word record plus
the guard. Both original scalar calls remain in the emitted branch. A separate
whole-record consumer still uses the complete readonly `[64 x i64]` parameter;
argument count and transported word width must not be conflated. Native checks
retain every public state word, overlapping output canaries and zero aggregate
allocation/drop. This is not a measured performance claim.

Six unit tests cover direct/immutable-local field operands, nested mixed scalar
paths, aggregate/subrecord/composite/codec negatives, unknown paths and budget rollback. One
test exercises 48 independent arithmetic-oracle cases before/after normalization,
with reversed YIR storage and unused selected checks before/after early returns.
Source-free workflows distinguish scalar and whole-record variants, check cache
reuse and pre-open tamper rejection, and retain byte-identical LLVM/state after
deleting source, manifest and build directories. A selected unused constructor
check fails with only the accepted open state; the untaken branch succeeds, and
two same-path restorations must retain both outcomes without retry or fallback.
This is bounded scalar field-demand closure, not general local/call-result capture closure.

## Evaluated Scalar Record Views

The [record-view pass](../../tools/nuisc/src/lowering/buffer_loop_outline/capture_record_views.rs)
has a separate evaluated-scalar mode. A single-definition local scalar becomes
readable only after its RHS has executed, with exact kind inference from immutable
input paths, preceding scalar locals and the completed scalar-only helper catalog.
Nested nominal constructors can then reuse that local value. Field observations
and record aliases resolve to total reads; only unreferenced, proven total record
copies disappear. Every scalar definition remains at its original site, including
unused calls, checked arithmetic and codecs. No RHS is duplicated, moved, inlined
or treated as a constant, and input-alias authority is unchanged.

Lexical scopes do not export child locals. Repeated writes, unknown calls, wrong
kinds, opaque/call-returned aggregates, registered wire types and control names
do not grant a view. Scoped transport helpers remain excluded. Inference, field
lookup, copying and final expanded-tree validation retain bounded work/depth and
whole-body rollback; every caller must validate before capture projection commits.
Whole-record consumers retain their complete transport. Native 64-word limits,
independent return storage and public/source/callback/FFI ABI are unchanged.

The evaluated-local 64-field fixture keeps all three original scalar calls while
using two i64 captures plus its guard, with no full-record reconstruction in that
private branch. Eight unit tests cover exact mixed kinds and signed zero, lexical
scope, mutations, unknown calls, type/control/wire negatives, budget/depth rollback,
all-caller veto and 48 independent arithmetic-oracle cases with reversed YIR storage.
Native acceptance checks every public state word, overlap canaries and zero
aggregate allocation/drop. Source-free workflows exercise cache reuse, pre-open
tamper rejection and byte-identical restoration. An unused checked local must
still trap on the selected branch, publish only the accepted open state and remain
skipped on the untaken branch, including repeated same-path restoration.
This is local CPU/profile evidence, not a benchmark, fresh device/cross-platform
execution, general aggregate-call-result closure or new changing-backedge proof.

## Completed Aggregate Call Result Views

A separate record-view mode now proves immutable locals holding already-completed
pure aggregate call results. Its catalog validates complete nonrecursive callees,
exact nominal parameter/result kinds and supported owned-value layouts before
callers; it grants no provisional, asynchronous, effectful or new loop-body
authority. Generated signatures that capture projection may change are excluded
from that catalog. Original input-alias and evaluated-scalar modes stay separate.

Each call remains at its original site and returns its complete record into
independent storage, even when unused. Only later total field/subrecord reads,
constructors and aliases may reuse that result. A call binding is never erasable,
and no call is moved, duplicated, inlined or treated as a constant. Whole-record
consumers retain full transport. Inline aggregate caller arguments are not spilled
or hoisted: one unproven caller vetoes the entire body/signature rewrite.

Single-definition, lexical readiness and exact declared kinds remain mandatory.
Child roots cannot escape or become ready before definition; changing versions,
unknown results, registered wire types and scoped/control identities remain
conservative. The 65536-work/64-depth ceiling, final expansion validation and
transactional whole-body/every-caller rollback remain unchanged.

The 64-field fixture executes two distinct full `[64 x i64]` returns while the
private branch captures two i64 values plus its guard. It reads the first result
after a second, different result and needs no redundant full-record reconstruction.
The whole-record consumer keeps its readonly 64-word input. Nine unit tests cover
mixed kinds, unused calls, catalog/type/version/lexical negatives, depth, budgets
1 through 511, all-caller veto and 48 independent selected-failure oracle cases
with reversed YIR storage. Native checks retain all 64 public words, overlap
canaries, independent returns and zero owned aggregate allocation/drop.
The source-free checked variant completes its first result before an unobserved
field in the second call fails. It publishes only the accepted open state, including
after repeated same-path materialization; the untaken branch skips both calls.
These are bounded local CPU/profile claims, not general aggregate-result closure,
new changing-backedge authority, a benchmark or a native/public ABI expansion.

## Inline Record Call Arguments

The completed-result mode now types directly written nominal record arguments,
including nested records, exact mixed scalar kinds, literals and previously
validated scalar calls. Every field must be present exactly once, match its
declared kind and contain only independently typed operands. Unknown fields,
missing/duplicate fields, generic constructors, wrong nominal kinds, effects,
recursion, unknown calls and unavailable/changing locals grant no result view.

This is a type proof, not an erasable-constructor proof. The entire original
call binding, record argument, source field order and all checked/unused field
work remain in place. Only subsequent total reads/copies of the completed result
may be projected. The 65536-work/64-depth ceiling and whole-body rollback remain;
budget exhaustion never installs a partially inferred argument or view.
Original input-alias and scalar-only authority remain separate.
Direct constructor bindings skip the scalar/call-root inference and first use the
existing total-view proof. A non-total materialized constructor may then use the
separate kind-only proof below; a successful total-view proof is not repeated.

In particular, `helper(make(Input { ... }))` is not spilled, hoisted, duplicated
or decomposed to make a projected private signature fit. One such unproven caller
still vetoes the complete body/signature rewrite. Tests retain both function
storage orders, unchanged producer signatures and independently stored results.

The native 64-field fixture builds two inline two-field arguments, keeps both
full record returns and every original scalar argument call, while the private
branch uses two i64 captures plus a guard. Its whole-record consumer remains
full-width. The checked variant completes the first result before an unused
field in the later argument traps, before entering the second producer. The
untaken branch skips the entire argument evaluation. Source-free workflows
retain cache reuse, tamper rejection and accepted-open-only failure publication.
Eight additional unit tests cover nested mixed kinds, same-shaped but distinct
nominal types, malformed fields, effect/recursion/unknown-call and lexical/version
negatives, original scalar-only authority, budgets 1 through 1023, depth and
every-caller rollback. Another 48 independent selected-check oracle cases retain
reversed YIR storage and source return timing; focused reruns are not additional
distinct tests.

## Materialized Record Call Arguments

Completed-result mode now also types an immutable local record after its
constructor has executed, and propagates the exact nominal kind through total
local views and aliases. A later call can consume that stored local without
reconstructing its arguments or repeating any field computation. Constructor
fields still require the same exact, bounded kind proof used for inline record
arguments. The original input-only and evaluated-scalar modes are unchanged.

Read readiness and erasability remain separate. A non-total constructor, including
checked or otherwise computed fields, is never erasable, even if unused. A total
record view may become erasable, but any whole call operand or whole consumer
retains its binding through the final read scan. Calls keep their original
arguments and source positions, full returns remain independent, and a later
failure cannot erase the already completed first call. There is no spilling of
`helper(make(Input { ... }))`, new constant identity or changing-backedge proof.

Single definitions, lexical readiness and exact declared kinds remain mandatory;
child-only definitions do not escape, registered controls and wire types remain
excluded, and effectful or unknown callees grant no readiness. Exhaustion of the
65536-work/64-depth budget rolls back the entire candidate. Every caller must
still agree before a body/signature rewrite is installed.

Eight additional unit tests cover unchanged checked constructors/alias/call sites,
total view operands, malformed nominal/field/declared kinds, effects, duplicate
writes, lexical/control/transport/scalar-mode exclusions, unused constructors and
whole consumers, budgets 1 through 2047, depth, idempotence and every-caller veto
in both function storage orders. Forty-eight independent before/after oracle
cases retain selected failures and return timing with reversed YIR storage.
The 64-field fixture retains two nominal argument bindings in NIR, five scalar
calls and two independent full record returns, while removing only downstream
64-word reconstructions. The existing native ABI flattens the two argument fields;
native assertions check evaluation order and exact SSA operand identity instead
of requiring a redundant LLVM aggregate construction. The source-free workflow includes ordinary, selected
checked-field and whole-record consumer variants, cache reuse, tamper rejection
and repeated materialization with unchanged LLVM bytes.

## Stored Call Result Projections

A separate completed-result proof now types a stored initializer such as
`let part = produce(...).selected` or a deeper nominal field chain. It resolves
only a call-rooted chain through the unchanged pure-value catalog and exact
declared field kinds. After the original RHS completes, the resulting immutable
local can serve as a ready scalar or subrecord for subsequent total reads/copies.
The initializer itself is never erasable, even if unused.

This is not inline projection authority. The general inline expression typer is
unchanged: unbound `produce(...).selected` inside another constructor or projected
caller still grants no readiness. The full producer remains at its original site,
executes once and computes its complete result before selection, including all
unselected checked fields. Independent calls never share result identity or
storage, and scalar projections are not constants. Whole consumers, lexical and
single-definition constraints, registered controls/transport types, unchanged
catalog signatures and every-caller veto remain conservative.
Transport exclusion checks every nominal kind along the path, including the
call's complete result and intermediate records; selecting a scalar cannot strip
a protected envelope's identity.

The existing 65536-work/64-depth budget charges each nominal field lookup. Depth
or work exhaustion retains the entire original candidate. Ten additional unit
tests cover nested call-rooted chains, mixed scalar kinds, unchanged RHS/independent
bindings, malformed fields and declared kinds, effects/recursion/async/reference
results, lexical/version/control/transport exclusions, unchanged scalar-only and
inline modes, whole/unused uses, budgets 1 through 1023 and both function storage
orders, including protected parent kinds above scalar projections. Forty-eight
independent before/after oracle cases put a checked field in an unselected portion
of the producer's result and retain original return timing with reversed YIR
storage.

The native fixture returns a full 64-word envelope with two 32-word subrecords,
stores only the selected half, and still performs two full independent producer
calls. Later full-record reconstruction can disappear with two i64 captures plus
a guard; a whole-record consumer retains its readonly 64-word input. The checked
variant completes the first call, then fails inside an unselected field of the
second producer's return. Source-free workflows retain accepted-open-only failure
publication, no fallback, cache reuse, tamper rejection, unchanged LLVM bytes and
repeated same-path restoration. No new opcode, native limit or public ABI is added.

The first native fixture revealed a separate admission gap: completed nested
results inside a function with an entirely flat scalar/record interface did not
reach the guarded control outliner. Control roots now also admit these flat
interfaces through the completed pure-value catalog. Nested parameter/return
interfaces remain on the established local-selection route; effects, unresolved
dependencies and cycles remain rejected. A regression checks both declaration
orders and these boundaries rather than widening all nested control authority.

## Total Inline Caller Records

A separate caller-side proof now allows a non-scoped projected private capture to select
fields directly from a total inline nominal constructor, for example
`helper(State { a: input.b, b: input.a, unused: input.unused }, flag)`.
Only supported owned scalar/record parameters that are never written in the
caller supply readiness. Constructors and field paths must preserve exact
nominal names, field sets and kinds; nested constructors and subrecords retain
the same rules. All fields, including unused fields, must pass before any field
is removed or any signature is changed.

The proof reuses the existing total unwritten-input-copy rule. Literals, calls,
arithmetic, codecs, local values, reference parameters, rebound inputs and
unbound call-result projections grant no new readiness. No call is hoisted or spilled.
Kept scalar/predicate arguments remain unchanged, in their original evaluation
order. Selecting a constructor field does not clone the complete constructor for
each projected operand. Every caller must pass before the candidate body,
signature and call operands are installed; budget/depth failure keeps them all.
The separate 65536-work/64-depth preflight also bounds the caller walk and field
lookups. Registered indexed/codec word inputs still require their original
canonical transport proof, even when a constructor passes ordinary record typing.
Scoped loop targets retain their existing complete caller/seed-map proof; the new
fallback is never used to relax that authority. The broad regression found this
boundary and the fix keeps the original transactional rejection assertion.

Ten new tests cover nested/mixed kinds, exact operand identity, full unused-field
validation, nominal/field/version/reference negatives, both caller storage orders,
idempotence, budgets 1 through 1023 and depth rollback. A behavioral transport
test proves that ordinary totality cannot bypass registered codec authority.
Forty-eight independent before/after oracle cases preserve scalar checks, guarded
timing and reached failures with reversed YIR storage.

The native test lowers explicitly registered projected IR through YIR and LLVM,
links a host executable and runs it under a ten-second execution deadline. Its
helper has three scalar operands and prints `27`, while the surrounding scalar
calls remain. This is not automatic projection authority for source functions,
source-free acceptance of a new public feature, a speed measurement or device
evidence. Generated-capture eligibility, public ABI and native limits are unchanged.

## Binding Root Caller Materialization

A separate non-scoped caller fallback now handles computed record arguments at
the root of a `let` or `const` initializer, for example
`let result = helper(checked(x), produce(input, y), checked(z), flag)`.
It reuses the completed pure-value catalog and the existing exact operand-kind
proof. All arguments must match their original nominal/scalar kinds and depend
only on supported, unwritten owned input parameters; effects, recursion,
provisional signatures, references, changed versions and registered controls do
not grant operand authority. Generated signatures grant no catalog authority.

When materialization is needed, all original operands receive separate typed
bindings in source order immediately before the original binding-root call.
This includes kept scalar/predicate operands and unused aggregate arguments.
Calls, complete constructors, field order and checks in unselected fields remain
at that statement, evaluated once; only subsequent reads of these completed
values are projected. Constructor typing does not become constructor elision.
Branch-local bindings remain inside their original branch. No materialization is lifted from a condition,
nested expression or loop; this binding-root checkpoint did not grant return
authority. A separate return-root proof below adds that specific statement root.
Unbound call-rooted field projections
remain outside this operand-kind proof. Scoped targets and canonical word
transports retain their original separate caller contracts.

Preflight, original/output expression depth, new bindings and fresh-name searches
are charged against a 65536-work/64-depth budget before installation. Names are
reserved across parameters, bindings and variable reads. Prepared callers remain
copies until every caller agrees; a veto keeps the original callee, all callers
and every operand, with no leaked temporary bindings.

Eight new tests cover exact ordered operand identity, full unused checked work,
record and mixed scalar kinds, nominal/missing/duplicate/generic field failures,
branch locality, idempotence, both function storage orders, effects/cycles/reference
and written-input exclusions, registered controls, budgets 1 through 2047, depth,
name collisions and every-caller rollback. Forty-eight independent before/after
oracle cases retain guarded return timing and reached checks with reversed YIR
storage. The retained word-transport regression also rejects this fallback.

The native probe explicitly registers the private-capture IR, emits YIR/LLVM and
links three executables. The producer's NIR definition is unchanged, and LLVM
assertions count its invocation only inside the caller, not unrelated task
wrappers. The existing lowerer may use its established physical result strategy;
this proof does not require redundant aggregate construction or alter that ABI.
The projected helper has four i64 operands plus a predicate. Success prints `26`,
an unselected branch prints `0`, and a reached check in an unused producer field
triggers the existing controlled LLVM trap rather than an abnormal memory signal.
A ten-second native deadline kills and reaps overdue children. This is explicit
IR acceptance, not automatic source-function eligibility, a new public feature's
source-free acceptance, fresh device evidence or a measured speedup.

## Return Root Caller Materialization

The same bounded non-scoped preparation now also handles a complete call at the
root of `return`, for example
`return helper(checked(x), produce(input, y), checked(z), flag)`.
All original operands receive typed, complete bindings immediately before that
return statement, in their original source order. No result binding, tail-call
guarantee, call duplication or partial producer evaluation is introduced.
The original return remains a return, with its original declared result kind.

Branch-local return operands stay inside their original branch. Earlier exits
retain their ordering and skip every operand of an unselected return. Root
returns inside loops, including returns in nested loop branches, remain
conservative; that checkpoint did not materialize conditions or nested return
expressions. The separate if-condition-root proof below adds only direct predicate calls.
Input/version, exact nominal/scalar kinds, pure completed catalog, work/depth,
fresh-name, scoped and word-transport rules are unchanged. A single unproven
caller keeps every caller and the callee unchanged, even when other callers use
the newly admitted root-return form or the preceding binding-root form.

Three new unit tests cover both function/struct storage orders, producer calls
and checked constructors, both return branches, disjoint temporary names,
unchanged scalar/nominal subrecord return kinds, mixed binding/return callers, idempotence, transaction
vetoes, references/effects/written inputs, scalar-kind mismatches, exhausted
work budgets, registered controls and 48
independent before/after guarded-check cases with reversed YIR storage.
The former root-return rejection is replaced by positive root-return coverage;
nested returns and loop returns still have exact whole-module rejection checks.

Seventeen historical transaction fixtures used newly admissible pure computed
root returns as their rejection cause. Their negative cases now use unsupported
nested return expressions or effectful producers, retaining the original
rollback/body assertions rather than removing those checks. An ordinary integer
codec case additionally proves complete root-return storage, with its original
conversion expression unchanged; its nested counterpart still rejects.
This gives no constructor elision or registered canonical word-transport authority.

A separate native test runs the return-root form through the shared explicitly
registered IR harness. It preserves the producer's complete NIR definition,
checks one producer invocation inside the caller and LLVM ordering of the
before-check, producer, after-division and projected helper. Its three binaries
print `26`, print `0` after an earlier exit, or retain the existing reached LLVM
trap in an unused producer field. Execution retains the ten-second deadline,
child kill/reap and temporary-directory cleanup. This is not automatic
source-function eligibility, fresh GPU/Linux evidence or a benchmark.

## If Condition Root Caller Materialization

The non-scoped preparation now handles direct `if helper(...)` predicate calls
outside loops. Its private projection plan records whether the declared result
is exactly owned, non-optional, non-generic `bool`; a scalar/reference/optional
result cannot grant this new predicate-materialization authority. This is a
private proof fact, not a new serialized ABI or a new function eligibility rule.

All complete operands receive ordered typed bindings immediately before the
original `if` statement. The predicate call stays in the condition, once, and
both original branch bodies stay unchanged. Earlier exits skip every operand;
an enclosing unselected branch performs no new work. The completed pure-value
catalog, unwritten owned inputs, exact operand kinds, fresh names, work/depth
budgets, every-caller transaction and scoped/word exclusions remain unchanged.
Computed calls in comparisons, `&&`/`||`, other nested expressions, `while`
conditions or any loop body remain outside that preparation proof. The selected
short-circuit RHS follow-up below supplies a separate, narrower extension.

Three unit tests cover complete constructors/producers, exact operand identity
and order, unchanged branch bodies and early exits, mixed binding/return/condition
callers, declaration order, idempotence, nested/short-circuit/loop vetoes,
predicate/result/operand kinds, effects, references, changed inputs, controls,
budgets, depth and names. An independent 192-case before/after oracle reverses
YIR storage and varies operand failures, earlier exits and selected/unselected
branch arithmetic. Fallible direct branch returns retain their existing guarded
lowering restriction; the probes use the already supported guarded local-result
bindings, not relaxed speculation.

A shared explicit-IR native harness preserves the full producer definition,
one caller-local producer invocation and before-check/producer/after-division/
helper ordering. The predicate signature remains `i1` with four i64 parameters
and one predicate. Five executables cover selected results `11` and `19`, early
exit `0`, an unused producer field trap and a selected branch trap. Unselected
branch checks do not run. Existing deadlines, kill/reap and cleanup remain.
This is not automatic source-function projection, fresh GPU evidence or a benchmark.

The frontend also fixes an adjacent delimiter bug: while an outer condition or
match scrutinee disables ambiguous bare struct literals, call arguments have
their own delimiters and now admit nested/generic literals. The argument parser
restores the outer literal mode on success and error. Three parser regressions
cover if/while/match contexts, nested calls/conditions, the unchanged outer block
boundary, error restoration and retained expression-depth limits. This syntax
fix does not authorize loop operand materialization or wider private projection.

## Selected Short-Circuit RHS Materialization

The non-scoped caller proof now handles one logical edge in an `if` condition:
`gate && helper(...)` or `gate || helper(...)`. The gate must be an unwritten,
owned exact bool caller input or a bool literal; the predicate must declare an
owned exact bool result. Comparisons, computed gates, helper-on-the-left forms,
nested logical edges, locals/changing versions, loops and scoped/word transport
remain outside this new projection proof.

Every complete RHS operand is staged in the selected gate arm, in original order,
before its one predicate call. The skipped arm does no RHS work. A fresh bool
starts at false for AND or true for OR and receives the selected predicate result.
The original business branches and early exits remain unchanged and are not
duplicated. Input readiness, exact kinds, pure completed catalog, work/depth/name
budgets and every-caller atomic installation remain mandatory.

This exposed a shared lowering gap: guarded selections previously accepted one
value binding, not a complete multi-binding operand prefix. The shared value
contract now admits up to 32 let/const bindings, 4096 expression nodes and depth
below 64, checked before prefix kind inference/cloning. Intermediate locals must
be fresh, source-ordered and exactly typed; only the terminal result may rebind
an outer value. Forward reads, duplicate locals, outer writes, effects, references,
unsupported kinds and nested logical expressions remain excluded. Free-input
capture removes only proven prefix-local names, and complete unused work stays
inside the guarded helper. Private caller preparation uses the same prefix bound.

The independent source-condition route also fixes eager RHS checks for a bool
literal/current owned bool variable gate and a bounded pure exact-bool RHS with
calls or checked work, including projections/comparisons left by inlining. Both
local-selection and full-control routes use this normalization.
It uses the existing guarded value-helper contract, not capture eligibility,
speculative arithmetic or new YIR/ABI instructions. Three shared-value tests
cover ordered fresh prefixes, skipped and selected failures, original guard seeds,
effects/rebindings/kinds and bounds, plus eight source short-circuit cases.

Three caller tests cover literal/input gates, both operators, producer/constructor
operands, nesting in an enclosing branch, declaration order, mixed callers,
unchanged producers and business branches, idempotence, every-caller rollback,
gate/result kinds, mutations, loops, controls, work budgets and fresh names.
An independent 384-case before/after oracle reverses YIR storage while varying
gate/flag values, operand failures, early exits and branch failures.
Seven explicit-IR native binaries check skipped zero-divisor RHS work, selected
results `11`/`19`, early exit `0`, unused producer-field and selected branch traps.
The shared harness retains full producer NIR, exact i1 predicate signatures, one
ordered operand-producing guarded LLVM body, deadlines, kill/reap and cleanup.
LLVM order checks identify the original `20 / divisor` operand explicitly rather
than confusing it with preceding, total bool-word decoder divisions by two.
This is not general short-circuit capture closure, automatic source-function
projection eligibility, a new public ABI, fresh device evidence or a benchmark.

## Source Logical Value Roots

The independent source normalization now reuses the same guarded helper for one
`gate && rhs` or `gate || rhs` edge at a `let`, inferred `let`, `const`, or declared
owned-bool `return` root outside loops. The gate remains a bool literal or current
owned exact-bool variable, and the RHS must be a bounded pure exact-bool value
with calls or checked arithmetic. Complete constructors and unused-field checks
remain selected work; this is not wider private capture projection or an ABI
change. Computed gates, deeper logical trees, arbitrary nested expressions,
effectful RHS values and loop value roots still need separate proof. The original
nested return execution cases use the admitted pure control route. A separate
top-level effectful-parent return proof is described below; returns inside an
enclosing source branch still retain the guarded-lowering rejection.

The existing root statement, binding name, declared kind and return position
remain intact. Capture happens before destination rebinding, so same-name local
versions use the current input, not a stale parameter. Admitted local selection
arms also normalize their return bodies before extraction; otherwise a branch
binding could bypass the root fix. This temporary normalization restores the
parent's selection and return-kind flags and does not recursively authorize
new local selection shapes. Earlier exits and business branch timing stay intact.
Inferred bindings retain the guard's proven bool result kind even before the
generated helper enters the refreshed catalog. Following logical edges therefore
see the current local type rather than losing it during the rewrite.

Runtime coverage includes 144 source cases and 288 reference executions with
ordinary/reversed YIR storage, typed/inferred bindings, consts, return roots,
pure/effectful parents, both operators, skipped/reached zero divisors, positive
and negative predicates, same-name and changed local versions, both enclosing
arms and earlier exits. Chained inferred predicates also check the exact number
of selected helper invocations, not just final values. Sixteen structural
combinations check complete RHS
identity, guarded captures, binding kinds, unaffected functions and idempotence;
36 negative combinations retain whole-module equality for kinds, references,
loops, computed gates, nested logic, effects and work/depth limits. Ten native
binaries use the default source-to-AOT path, without explicit private projection
registration or a foreign runtime shim, and check output, skipped work, reached
LLVM traps, deadlines, kill/reap and scratch cleanup.

These tests exposed a separate reference-execution gap. Entry graph traversal
now observes each operation's registered `function_exit` hook just as nested
invocations do. An entry-local return publishes the actual value at its declared
result identity and stops only that entry's owned tail; global work continues.
The CPU provider also registers the existing `guard_print_return` composite as a
three-argument selected exit. Its lowering now advances the statement-effect
frontier, so later output cannot overtake the selected return. Registration errors
propagate without fallback or retry, and single-entry verification is unchanged. This uses provider contracts,
not a CPU instruction list embedded in the graph executor. A custom registered
provider tests early exit, global continuation, storage reversal, actual result
publication, error propagation and malformed multiple-entry rejection.

## Effectful-Parent Conditional Returns

A separate local-return outliner admits top-level `if` statements with a single
final pure value return in one or both arms of a non-control-root synchronous function.
The declared result must be exactly owned `bool` or `i64`; the condition must be a
bool literal or current owned exact-bool variable. Expression admission reuses
the bounded pure-value catalog and expression budget. One outer logical edge may
have an atomic bool gate and a bounded nonlogical RHS, using the existing source
short-circuit normalization. Complete constructors and unused checks remain work
inside the selected helper, not precomputed arguments at the parent call site.

The condition is bound once. Only current owned bool/i64 values are captured, and
the pure return computation uses the existing scalar-control helper contract.
An absent arm returns a harmless scalar seed from the helper; the parent's exit
then sees only a ready atom and remains conditional on the original branch.
Two terminal arms return the selected helper result directly. Parent prints,
earlier exits and the unchanged tail stay in the parent; no suffix or business
effects are copied into the helper. No new public ABI or YIR instruction is
introduced, and the original speculation barrier remains intact.

This does not recurse into enclosing source branches or loops and does not admit
unproven or effectful branch prefixes, branch-local effects, borrowed/resource/aggregate captures,
optional or mismatched kinds, async/generic parents, computed outer gates,
deeper logical expressions or unbounded work. Unsupported candidates remain
unchanged. Pure control roots retain their existing whole-function route.

The tests cover 210 runtime source cases and 420 reference executions under both
ordinary and reversed graph storage. They check then-only, else-only and dual
return arms, outer/inner lazy guards, reached/skipped zero divisors, earlier
exits, current typed/inferred bool and changed i64 versions, ordered prints,
actual entry results and exact selected consumer-helper invocation counts.
Structural checks retain complete return expressions, parent prefix/tail,
captured inputs and idempotence; negative checks retain whole-module equality.
Ten default-source native binaries check success, parent-tail suppression and
reached LLVM traps with deadlines and scratch cleanup. This is local CPU/source
correctness evidence, not broader private projection, device execution,
performance or formal-safety evidence.

## Pure Prefixes Before Conditional Returns

The local-return proof now separately admits fresh pure `let/const` prefixes
ending in the same exact bool/i64 return contract. Each complete arm is bounded
before recursive inference or cloning: at most 32 statements including the final return,
4096 shared expression nodes and depth below 64. Only the final return may use
the previously admitted single logical edge; prefix initializers remain
nonlogical bounded pure expressions. This leaves the old single-return admission
and existing local-value selection budget unchanged.

Sequential kind inference sees only parent inputs and earlier prefix locals.
Explicit declarations must exactly match their inferred owned kind; completed
local records are allowed by the same pure-value catalog. No outer-state rebinding,
capture shadowing, duplicate local names, forward references, branch-local effects,
nested control flow or resource creation is authorized. Locals are removed from
the captured input set, and outer captures still require current owned bool/i64
values. Local record computation is not resource/aggregate capture transport.

The complete prefix and final expression remain in their original order inside
the selected helper arm. Parent effects and exits stay in place, and only a
ready result reaches the parent return shortcut; its speculation barrier is
unchanged. In particular, prefix checks still run when the inner RHS is skipped:
`let ignored = 10 / divisor; return gate && consume(...)` must still trap on zero
after entering the outer branch, even when `gate` is false. Only a skipped outer
branch or earlier parent exit suppresses that prefix work. The helper predicate
name avoids both captured inputs and every arm-local binding, including reserved-
looking source names. The original return location, initializer/const kinds,
unused constructor fields and parent prefix/tail are preserved without copying
business effects or a continuation suffix.

Six new compiler tests cover 468 source cases and 936 reference executions with
ordinary/reversed graph storage: then-only, else-only and dual return arms;
typed/inferred locals, constants, local records, unused checks and name collisions;
outer/inner selected paths, earlier exits, exact consumer call counts, actual
entry results and ordered prints. Eighteen structural combinations check complete
arm identity, capture filtering, predicate hygiene, unaffected functions and
idempotence. Negative source cases distinguish frontend kind/scope rejection from
unchanged NIR admission vetoes. Statement/depth/work checks include 31/32 prefix
bindings, the exact whole-arm 4096-node boundary and multiple individually small
roots exceeding the shared budget. Twelve default-source native binaries reuse
the deadline, kill/reap, signal and scratch-cleanup probe, including selected
unused-prefix traps despite a skipped inner RHS. This is local CPU/source proof,
not device execution, wider private projection, new public ABI or formal safety.

## Pure Terminal Return Trees

A separate local-return fallback now admits pure terminal trees inside the
already selected outer arm. Fresh pure `let/const` prefixes may precede an atomic
owned-bool `if`; both arms can contain further such trees, but every leaf must return
the exact owned bool/i64 result. A body ends either in a value return or in a
complete two-arm terminal branch. A missing return, a fallthrough branch, an
intermediate branch followed by a suffix, an effect or a loop rejects the whole
candidate. The old single-return and straight-prefix routes keep their budgets
and admission behavior; this fallback requires at least one nested branch.

An iterative preflight bounds 32 statements across the entire tree of each outer
arm, 4096 shared expression nodes and statement/expression depth below 64 before recursive kind
inference or cloning. The statement budget includes all mutually exclusive arms,
not just the largest path, and the expression budget includes every prefix,
condition and return. Only a return root may use the previously supported single
logical edge; nested branch predicates must be literals or current owned-bool
bindings. Computed predicates and logical prefix initializers gain no new authority.

Every arm receives its own exact lexical type scope. It sees earlier locals but
never sibling locals; explicit kinds must agree, and outer-state rebinding,
shadowing and duplicate locals on one path are rejected. Sibling arms may use the
same fresh name with different types because those bindings never escape.
Completed pure local records are retained in full; only owned bool/i64 parent
inputs are captured. The existing predicate-name hygiene covers every nested
binding. Parent effects and the continuation suffix remain outside the helper:
every leaf returns without copying the parent suffix, and the selected complete
tree uses the existing scalar-control contract and speculation barrier.

Skipping a nested leaf suppresses that leaf's checked work. It does not suppress
an earlier local constructor: if a record producer is evaluated before the inner
branch, its unused field checks still run after the outer branch is entered.
Seven new compiler tests contain 507 runtime source cases and 1014 reference executions
under ordinary/reversed graph storage: 432 three-guard boolean cases, 72 typed/
inferred i64 cases and three deeper lexical-tree cases. Twelve structural cases
retain exact arms, parent prefix/tail, scalar captures, sibling/predicate hygiene,
unaffected functions and idempotence. Negative cases distinguish frontend scope
rejection from complete unchanged-NIR vetoes; budget cases check 32/33 statements,
the exact shared 4096-node boundary and excessive expression depth. Ten default-
source AOT binaries reuse the existing deadline, kill/reap, native-trap and scratch
cleanup harness. This is local CPU/source evidence, not general nested effectful return closure,
resource capture transport, wider public ABI, device execution or formal safety.

## Pure Fallthrough Return Trees

The empty-leaf baseline admits a bounded pure tree with both original return leaves
and empty fallthrough leaves. Fresh pure prefixes remain in their selected helper
arm, with the same exact bool/i64 result kinds, scalar parent captures, isolated
sibling scopes and shared whole-tree preflight. No parent suffix is copied into
the helper. Nonempty pure binding leaves require the independent extension below;
intermediate branches followed by a suffix, effects, rebinding and direct loops
still reject this fallback.

The return value and exit readiness are independent. Only empty helper leaves
receive an unused false/zero seed, making the helper a complete pure value
function. The parent first calls that helper behind the original outer guard,
then computes a total boolean exit predicate, then returns the value if ready.
A real false/zero return suppresses the parent suffix; a fallthrough seed never
does. Earlier parent exits and effects remain at their original sites.

Every tree predicate in this fallback must be an owned-bool literal or current
binding already available in the parent. Helper-local aliases or computed exit
predicates are excluded, even if they appear pure. Readiness combines only these
stable atoms with total boolean selection, never repeats calls or checked work,
and does not observe a helper's local values. Complete all-return trees retain
their older independent route, including its admitted local atom predicates.

The existing per-arm 32-statement, shared 4096-expression-node and below-64-depth
preflight runs before recursive inference, readiness construction or cloning.
Empty source leaves use no source-statement budget; synthesized seeds add at
most one return per empty leaf. Boolean readiness grows linearly with the
bounded branch tree, not by enumerating paths. Predicate and exit-binding names
remain hygienic across all nested source locals.

The regression suite covers 600 runtime source cases and 1200 reference executions
with ordinary/reversed storage: then-only, else-only and dual outer arms; opposite
nested exits; independent deeper predicates; mixed complete/partial arms; early
exits; exact consumer calls; typed/inferred i64 returns; literal and rebound
parent gates. Checked local records still execute before a skipped nested return,
including unused-field traps. Structural tests permit only empty-leaf seeding,
retain parent prefix/tail and unaffected functions, and check captures, hygiene
and idempotence. Vetoes retain computed/local predicates, effects, invalid kinds,
resources, rebinding and logical prefix initializers; budgets include exact shared
4096-node and 32/33-statement boundaries. A 31-level tree compiles and emits LLVM;
the 32-level tree remains unchanged when its 33 statements exceed the shared bound.
Fourteen default-source AOT variants
reuse the native deadline, kill/reap, trap and scratch-cleanup harness.
This is local CPU/source proof, not general nonterminal/effectful return closure,
new public ABI, resource capture transport, device execution or formal safety.

## Pure Work Before Continuation Leaves

A separate extension admits a whole fresh pure `let/const` prefix as a
continuation leaf. The leaf can sit inside a nested tree or be a direct outer arm
opposite a return. An all-fallthrough subtree can also sit opposite a returning
path. At least one original return leaf must exist across the two outer arms;
internal seeds do not grant exit authority to a computation-only candidate.

The shared grammar splits each body into fresh local bindings and an optional
final return/branch. Without such a final control statement, every original
statement must be a fresh binding. An intermediate branch followed by more work,
prints, loops, outer-state rebinding, duplicate locals, invalid declared kinds or
logical prefix initializers still reject the whole candidate. Type inference
validates each initializer through the unchanged pure-value catalog, sibling
scopes remain isolated, and only exact owned bool/i64 parent inputs are captured.
Branch predicates remain stable parent-owned bool atoms/literals, not helper
locals or computed conditions. Complete terminal trees retain their older mode.

Every original continuation binding stays intact and in order; the seed is
appended after all original leaf work. Constructors, unused fields and discarded
consumer results are still evaluated, including selected checks. The independent
parent readiness remains false on that path, so the untouched parent suffix runs
after the helper finishes. Real false/zero returns still suppress that suffix,
including checks in its fallible return expression. No suffix is copied into the
helper, and no new ABI, source resource transport or speculation exemption exists.

The original per-arm 32-statement, shared 4096-expression-node and below-64-depth
preflight counts all source leaf work before inference, cloning or seeding.
Synthesized returns are not source work. Exact 32-binding outer continuations and
whole nested 32-statement trees compile and emit LLVM; an extra original binding
rejects unchanged. Shared exact 4096-node and excessive-depth probes still apply.

Eight compiler tests cover 652 runtime source cases and 1304 reference executions
with ordinary/reversed storage: 480 boolean path/call/check cases, 120 typed/
inferred i64 cases, four fallible-parent-suffix cases and 48 all-fallthrough
subtree cases. Twenty-five structural combinations preserve every original
binding, parent prefix/tail, scalar captures, hygiene, unaffected functions and
idempotence. Source/direct-NIR vetoes retain effects, rebinding, scope/kind drift,
aggregate/borrowed captures and computation-only candidates. The former fresh
scalar continuation veto now independently compiles; logical initializers and
branch effects remain rejected. Sixteen default-source AOT variants reuse the
existing deadline, kill/reap, native-trap and scratch-cleanup harness.
This is local CPU/source proof, not effectful continuation closure, formal safety,
fresh device execution or expanded public/private resource authority.

## Local Atom Aliases For Exit Readiness

Fresh local boolean aliases may now select exits in a bounded partial-return
tree. Inferred `let`, declared `let` and `const` chains must resolve to a current
exact owned parent bool variable or a bool literal. Only the readiness expression
uses the flattened atom; every original binding, initializer, check and call
remains in its selected helper scope. Parent rebinding before the outer branch
is observed through the current variable, not an earlier value.

The existing shared preflight and pure-value/type/capture validation run first.
A separate readiness proof walks only the admitted lexical tree. Its alias map
starts with exact owned parent bool atoms, records only literals or already
proven aliases, and clones scope separately for siblings. A call, comparison,
field access or other computed initializer cannot become a readiness atom, even
through another alias. A rejected readiness proof leaves the candidate unchanged
before helper cloning or leaf seeding. Complete all-return trees keep their
older independent mode; this extension does not tighten or expand that mode.

Source statements still share the 32-statement budget, expressions the 4096-node
budget, and depth stays below 64. Exact 30-binding plus branch/return trees fit
32 statements; one extra alias rejects unchanged. Literal aliases do not erase
original branch work or manufacture source exits. The return value and readiness
stay independent, including false/zero results, and the parent suffix stays in
place rather than being copied or evaluated before the selected helper finishes.

Nine compiler tests cover 200 runtime source cases and 400 reference executions,
four structural combinations and twelve default-source AOT binary variants.
The alias regressions exercise ordinary and reversed YIR storage, short-circuit
checks, exact consumer invocation counts, selected unused prefix work, literal
and current-parent aliases, zero/false exits and fallible parent suffixes.
Structural checks retain original lexical bodies, isolated siblings, scalar
captures, hygiene and idempotence. Computed/call-derived conditions, alias
laundering, effects, rebinding, sibling leakage, intermediate branch suffixes and
oversized trees retain explicit rejection probes. Default-source AOT cases use
the existing deadline, kill/reap, native-trap and scratch-cleanup harness.
This is local CPU/source proof, not public/FFI ABI expansion, wider capture
authority, device execution, formal safety or effectful continuation closure.

## Total Comparisons For Exit Readiness

A separate extension admits a fresh local bool binding computed by one total
comparison. Bool operands permit `==` and `!=`; exact owned i64 operands permit
all six equality/ordering comparisons. Each operand must normalize to a current
parent scalar, literal or its scalar atom alias. The compared bool may itself be
forwarded through `let/const` aliases before the final atomic branch predicate.
Other inline computed branch conditions remain outside this proof; a separate
partial-tree extension below admits only the same bounded total comparisons.

The proof is isolated in the conditional-return predicate module and reuses the
common pure binary type contract. Only exact owned bool/i64 parent values seed
its typed facts. A normalized value is either one scalar atom or a comparison
of two atoms, at most three expression nodes. Comparison results cannot become
comparison operands, so repeated bool self-comparisons or alias DAGs cannot
inflate the readiness expression. Unfolded local arithmetic, checked division/remainder,
calls, field reads, conversions and logical expressions grant no operand fact.
Parent values already evaluated before the outer branch remain current atoms;
this does not authorize replaying their original initializers.

Whole-tree shared preflight, exact type/purity/capture validation and isolated
sibling scopes still precede readiness derivation, cloning and leaf seeding.
Readiness may repeat only the proven total comparison after complete helper
evaluation; no call, constructor, arithmetic or check is replayed. All original
bindings, unused checks and discarded calls stay in their selected source
scopes. Real false/zero returns suppress the unchanged parent suffix; a seeded
continuation still finishes its own work before reaching that suffix.

The 32-original-statement, 4096-source-expression-node and below-64-depth bounds
remain unchanged. A 30-binding comparison/alias chain plus branch/return fits
32 statements; one extra binding rejects unchanged. Complete all-return mode,
source-return eligibility, scalar capture authority, resource limits and public/
FFI/callback ABI remain unchanged. This is local CPU/source proof, not fresh
device execution, formal safety or general effectful continuation closure.

The linked constant-folding fix preserves the bool kind of literal i64
comparisons instead of emitting integer `1/0`. Typed aliases, record fields and
constant branch selection retain that kind and the selected effects. If folding
erases every return in a candidate, the local return outliner still grants no
exit authority; an existing guarded value-binding route may independently run
the remaining pure work. Current-parent constant cases have separate reference
and default-source AOT probes, rather than bypassing optimization.

Nine conditional-return tests cover 503 runtime source cases and 1006 ordinary/
reversed reference executions, four structural combinations, selected checks,
early exits, false/zero results and current parent versions. Fourteen default-source
AOT variants use the existing deadline, kill/reap, trap and scratch-cleanup harness.
Two further optimizer tests cover typed aliases/record fields and 36 direct-NIR
signed/extreme branch-selection cases without changing the selected effects.

## Inline Total Comparisons In Partial Return Trees

An internal `if` in a partial return tree may now directly use a total comparison
of exact owned bool/i64 atoms. Bool permits `==/!=`; i64 permits all six comparisons.
Each operand must be a current parent scalar, literal or fresh scalar atom alias.
For example, `if input > zero { return checked_value; }` no longer needs an
intermediate bool binding when its sibling continues. Original return eligibility
still requires at least one source return in the enclosing candidate.

The predicate-fact module is now shared by conditional-return validation and
partial-tree readiness. Shared whole-tree preflight runs before type inference,
fact derivation, cloning or seeding. In the restricted replay route, the
lexical fact proof admits only the bounded inline comparison and rejects call/field/
arithmetic-derived local operands and nested comparison results. Normalization
still produces at most three normalized expression nodes, not an expanded DAG.
No initializer, call, arithmetic or check is replayed. The complete selected
helper work runs before readiness; real false/zero returns remain independent
of continuation seeds and the unchanged parent suffix.

This partial-tree proof does not grant complete-tree computed-predicate authority;
the separate complete-tree extension below provides that proof. Outer entry
conditions retain their earlier atomic grammar in this proof; the separate
outer-entry extension below supersedes only its own admitted source shapes.
This extension does not widen logical conditions, casts/codecs, borrowing,
resource or aggregate capture transport, source/public/FFI/callback ABI or native
limits. Shared 32-statement, 4096-source-expression-node and below-64-depth bounds
remain intact; 30 scalar atom bindings plus branch/return fit, one extra rejects
unchanged. Sibling locals never leak and parent rebinding uses the current version.

Nine compiler tests cover 502 runtime source cases and 1004 reference executions,
four structural combinations, selected checks/calls, early exits, fallible parent
suffixes, signed extreme atoms, zero returns and current bool/i64 parent versions.
Fourteen default-source AOT variants reuse the deadline, kill/reap, reached-trap
and scratch-cleanup harness. Former inline bool/i64 comparison veto sources now
independently compile and emit LLVM. The separate stored-signal route below now
handles bounded pure computed partial predicates; outer-entry grammar vetoes
remain explicit. This is local CPU/source proof,
not general inline-predicate authority, fresh device execution or formal safety verification.

## Pure Computed Predicates In Complete Return Trees

A complete internal return tree may now use a bounded pure computed condition
with an exact owned bool result. It reuses the existing pure-value type contract,
including registered pure calls, typed local fields, arithmetic and nested
comparisons. For example, `if helper(produce(divisor)) { return yes; } else
{ return no; }` keeps the producer, condition call and all constructor checks at
their original selected evaluation point. A pure computation may fail a check:
pure is not total, and a reached failure must still happen before any return.

Whole-tree preflight proves every leaf returns and bounds all original condition,
prefix and return expressions before inference or cloning. All tree conditions
infer exact bool through the pure helper catalog and typed layouts. Partial
trees separately require restricted atom/comparison facts and lexical readiness
or the independent stored-signal proof below.
No exit-readiness expression or continuation seed is generated
for a complete arm. Its original condition is evaluated once, not copied into
the parent or replayed after the helper. Selected prefix work and real false/zero
returns still precede the unchanged parent suffix.

Outer entry predicates remain atomic within this earlier proof; see the separate
outer-entry extension below. The complete-tree proof grants no
partial-tree exit authority; the stored-signal design below provides that
independent once-only result/readiness proof. Condition logical edges remain
outside the bounded nonlogical expression grammar, even when both leaves return.
Branch-local effects, loops, rebinding, borrowed/resource/aggregate captures and
intermediate suffixes remain excluded. Source/public/FFI/callback ABI and native
limits do not change. The shared bounds remain 32 original statements,
4096 source-expression nodes and depth below 64. A 29-binding prefix plus a branch
and two returns fits 32 statements; one extra rejects unchanged. A separate
condition-plus-leaf probe admits exactly 4096 nodes, rejects 4097 without mutation,
and rejects a 64-deep condition before recursive inference.

Nine compiler tests cover 490 runtime source cases and 980 reference executions,
twelve structural combinations, exact original-body/capture/hygiene preservation,
idempotence, selected call counts, division/remainder and constructor checks,
early exits, sibling paths, current bool/i64 parent versions and zero returns.
Sixteen default-source AOT variants reuse the deadline, kill/reap, reached-trap
and scratch-cleanup harness. Former complete inline-comparison veto sources now
independently compile and emit LLVM. Former partial computed/call veto fixtures
now independently compile through the separate stored-signal route; effectful
producers, malformed nominal arguments, wrong kinds and outer-entry predicates
retain unchanged-rejection probes. This is local CPU/source proof,
not partial-tree computed-predicate authority, fresh device execution or formal safety verification.

## Stored Exit Signals For Partial Return Trees

A bounded pure partial return tree can now use exact owned bool conditions from
pure calls, computed local aliases, typed fields, checked arithmetic or nested
comparisons. The existing replay route still handles stable total facts without
extra transport. If that proof fails, a separate stored-signal route validates
the original whole tree through the same pure-value catalog and typed layouts.
Purity does not make a condition total or safe to replay.

The helper returns one private typed record with an `exited: bool` field and an
independent `value: bool/i64` field. Each original return first materializes its
complete expression at a fresh typed root, then returns a record with `exited`
true. Keeping logical returns at an expression root is essential: placing a
short-circuit expression directly inside a record field can eagerly lower its
RHS. The shared logical-root outliner guards that RHS before record construction.
Continuation leaves retain all original work before returning `exited` false
and an unused scalar seed. Empty outer arms are continuations, never real exits.

The parent stores the complete helper result exactly once, reads its value and
exit flag through total field projections, then either returns or resumes the
unchanged parent suffix. No condition, initializer, constructor, call or check is
replayed. False and zero results cannot stand in for an exit flag. Earlier parent
exits and effects remain in place. Mixed complete/replayable/computed arms wrap
original leaves rather than another route's synthetic seeds. Sibling locals stay
isolated, generated names avoid type/local collisions, and current parent scalar
versions are captured. Private layouts and pure catalogs are refreshed before
the shared logical/control lowering; public function results stay unchanged.

At least one original source return is still required. Original per-arm bounds
remain 32 statements, 4096 expression nodes and depth below 64, checked before
recursive inference or cloning. The new probes admit exactly 32 statements and
4096 nodes, reject one extra without mutation, and reject a 64-deep condition.
Only exact owned bool/i64 parent captures are admitted. Branch effects, loops,
local rebinding, intermediate branch suffixes and logical prefix/condition forms
remain outside this proof. No source/public/FFI/callback ABI, native word limit,
resource transport or ownership/GLM exemption changes.

Eight compiler tests cover 334 runtime source cases and 668 ordinary/reversed
reference executions, once-only call counts, selected constructor/division
checks, continuation work, real false/zero exits, fallible parent suffixes, mixed
arms, sibling identities, current parent versions, private snapshot structure,
hygiene, idempotence and unchanged vetoes. Twenty default-source AOT variants
reuse the existing deadline, kill/reap, reached-trap and scratch-cleanup harness.
Four variants explicitly select or skip a zero-divisor RHS behind both `&&` and
`||`, independently of the successfully computed tree predicate.
Earlier computed-predicate veto fixtures are retained as independent positive
source/LLVM checks rather than deleted. The retained logical-root regression also
executes eight call-alias source cases in both reference orders with exact call
counts, while its branch-effect fixture remains rejected. This is local CPU/source evidence,
not fresh Linux/GPU acceptance, a benchmark, formal safety proof or general
effectful/nonterminal return closure.

## Pure Computed Outer-Entry Predicates

The effectful-parent return route now admits a bounded pure nonlogical expression
with an exact owned bool result as its outer `if` condition. The shared iterative
preflight checks at most 4096 nodes and depth below 64 before recursive typing or
cloning. Exact types, layouts and the existing pure helper catalog then validate
calls, typed field reads, arithmetic and nested comparisons. This is a separate
original-site proof, not widened logical-return or internal readiness grammar.

The original expression is stored once in a fresh typed parent binding exactly
where the source `if` stood. Earlier parent exits skip it; earlier effects precede
it. Its complete checked operands finish before any arm or continuation work.
The generated helper receives only the saved bool, and replay-based readiness
also reads that bool rather than reconstructing the original computation.
Complete arms and partial replay/stored-signal arms keep their distinct proofs.
Pure does not mean total: reached entry checks fail, even if an arm would skip
all its own work. Skipped arm checks and skipped logical return RHS stay skipped.

Current canonical owned record operands can be used locally at this entry site;
they are not new helper captures. Arm captures remain exact owned bool/i64.
An entry call or division/remainder can itself supply the required computation
when the admitted returning arms contain only scalar constants. At least one
original return remains required. Logical entry expressions, effectful calls,
branch effects, resource/borrowed/optional/generic operands, loops, intermediate
suffixes and invalid capture kinds remain excluded from this earlier proof;
bounded pure suffixes have the independent proof below. Original per-arm bounds and
source/public/FFI/callback ABI, native limits and ownership/GLM rules are unchanged.

Eight compiler tests cover 201 runtime source cases and 402 ordinary/reversed
reference executions, exact once-only call counts, parent effects/early exits,
complete/empty/partial arms, real false/zero results, current record operands,
condition-only checked work, and fallible parent suffixes. Structural probes
check source-site placement, saved-bool-only helper entry, capture isolation,
name hygiene and idempotence. Exact 4096/4097-node and depth-63/64 probes check
admission and unchanged rejection before recursive typing. Eighteen default-source
AOT variants use the retained timeout, kill/reap, reached-trap and cleanup harness,
including both short-circuit operators and both partial-exit routes. Three former
outer-entry veto fixtures now independently execute in both reference orders,
rather than being removed. This is local CPU/source evidence, not fresh Linux/GPU
acceptance, a benchmark, formal verification or general effectful return closure.

## Bounded Pure Intermediate Suffixes

Pure source continuations after an internal `if` now have an independent bounded
normalization proof. A borrowed plan distributes the remaining suffix into only
continuing paths. An observed return ends its path without any inherited suffix;
the actual selected suffix runs once, not once for every statically expanded arm.
Parent effects, earlier exits and the parent's own continuation stay in place.

Original lexical scopes, exact declared/inferred types and reachability are
validated before moving any suffix. Branch-local values do not become inputs to
their former parent suffix. Only fresh pure bindings, exact bool conditions and
bool/i64 returns are admitted through the existing layouts and helper catalog.
At least one original return remains required. Original and expanded trees are
independently limited to 32 statements and 4096 expression nodes, with expression
depth below 64; the borrowed expanded plan is bounded before expression cloning.
Existing complete-arm, replay-readiness, stored-signal and capture proofs then
revalidate the normalized tree. A normalization-induced local name collision is
conservatively rejected rather than silently renamed.

Eight compiler tests cover 442 runtime source cases and 884 ordinary/reversed
reference executions. They check selected/skipped suffix traps, exact call counts,
complete/partial/computed/sequential/diamond paths, earlier parent exits, current
scalar versions, real false/zero exits, parent continuation work and fallible tails.
Structural probes check helper hygiene, untouched parent statements and idempotence.
Exact original/expanded statement, 4096/4097-node and depth-63/64 probes check
admission and unchanged rejection. Six former suffix veto fixtures are retained
as positive execution probes; the two discovered by the broad run now each cover
twelve additional source cases in both reference orders. Unrelated vetoes remain
intact. Eighteen default-source
AOT variants reuse the existing deadline, kill/reap, reached-trap and cleanup harness.

Effects, loops, rebinding, unreachable source suffixes, logical conditions/bindings,
resource/borrowed/aggregate parent captures and expansions outside those budgets
remain excluded. No public/FFI/callback ABI, native limit or ownership/GLM exemption
changes. This is local CPU/source evidence, not fresh Linux/GPU acceptance,
a benchmark, formal safety verification or general effectful return closure.

## Single-Edge Logical Outer Entries

An outer return entry now independently admits `gate && rhs` or `gate || rhs`,
where `gate` is an exact owned bool atom/literal and `rhs` is a bounded pure
nonlogical owned bool computation. The whole logical root shares the existing
4096-node and below-64-depth preflight before recursive typing or cloning; exact
types, layouts and the pure catalog validate both operands. Nested logical RHS,
computed/derived left gates and non-bool/borrowed/optional/generic gates are not
newly admitted. Internal arm condition and prefix grammars are unchanged
at this historical checkpoint; the independent arm-root proof below extends them.

The existing original-site return route saves the original expression in a typed
bool value root. Shared logical value-root lowering then guards its complete RHS
inside the selected helper arm before publishing that bool. Both returning work
and exit readiness consume only the saved result: no entry predicate or RHS call
is replayed. Earlier parent exits skip the entry; earlier effects precede it.
Skipped RHS constructors/division stay skipped, while selected checks still fail.
Original-return authority, parent continuation ordering and exact bool/i64 arm
capture limits remain unchanged. Complete, replay-readiness, stored-signal and
bounded pure intermediate-suffix routes keep their independent proofs.

Eight compiler tests cover 412 runtime source cases and 824 ordinary/reversed
reference executions: 144 complete-arm cases, 216 partial/suffix composition
cases and 52 real-zero/current-gate cases. They check once-only calls, parent
effects/exits, selected/skipped constructor and division checks, real zero exits,
fallible parent continuations, current local rebinding and independent snapshots.
Structural probes check original-site typed storage, saved-bool-only helper input,
hygiene, idempotence and shared guarded-value helper generation. Exact whole-root
4096/4097-node and depth-63/64 probes test admission and unchanged rejection.
The previous single-edge entry veto source is retained as a positive execution
probe. Invalid bool/record operands retain their front-end type diagnostic.
Eighteen default-source AOT variants reuse the existing deadline, kill/reap,
reached-trap and cleanup harness. This is local CPU/source evidence, not fresh
Linux/GPU acceptance, a benchmark, formal safety or general logical/effectful
return closure; no ABI, native limit, ownership/GLM or resource exemption changes.

## Guarded Logical Roots in Return Arms

Internal `if` conditions and fresh owned-bool `let/const` initializers now admit
one `gate && rhs` or `gate || rhs` edge under the shared guarded value-root route.
The left operand is an exact owned bool atom/literal in the current lexical scope;
the RHS is bounded, pure and nonlogical. Whole-tree preflight precedes recursive
typing/cloning: 32 original/expanded statements, 4096 nodes and depth below 64.
Original and expanded suffix roots share these limits. General selection-prefix
admission remains nonlogical; only the independently validated return-arm roots
are extended. Declared/inferred types, purity, original-return authority, fresh
names and exact bool/i64 parent captures retain their separate validation.

Each original logical condition or binding stays at its source position. Shared
value-root lowering guards its complete RHS, including unused constructor fields
and checked division, without eager evaluation or duplicate calls. Complete trees
need no exit-readiness replay. Non-total partial decisions use the existing private
typed `{exited, value}` signal instead of flattening or replaying a logical gate
or initializer. Continuing bindings run before continuation seeds; real false/zero
returns stop inherited suffixes and skip the original parent continuation.

Eight compiler tests cover 996 runtime source cases and 1992 ordinary/reversed
reference executions, plus eighteen default-source AOT variants. The matrices
combine complete/partial/suffix arms, let/const and continuation bindings, calls,
fields/division, earlier local checks, local computed gates, outer logical entries,
earlier exits and real-zero exits beside fallible parent tails. Structural probes
check hygiene, rooted shared guards, scalar captures, private signal layouts and
idempotence. Exact 4096/4097-node, depth-63/64 and 32/33-statement probes cover
original and expanded work. Ten earlier suites retain their newly admitted source
fixtures as positive executions, rather than deleting obsolete rejection evidence.
Nested logical roots, derived left gates, effects/rebinding, loops, name collisions
and borrowed/resource/aggregate parent capture transport remain excluded. No public
ABI, native limits or ownership/GLM exemptions change. This is selected local CPU
evidence, not fresh Linux/GPU acceptance, a benchmark or general return closure.

## Validation

Run sequentially from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- return_invariant --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --test native_application_bridge -- typed_sparse_literal_returns --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_literal_snapshots --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- private_record_copies --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --test native_application_bridge -- typed_sparse_scalar --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_scalar --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib --test native_application_bridge -- evaluated_record_views typed_sparse_evaluated_scalar --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_evaluated_scalar_records --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib --test native_application_bridge -- call_result_views typed_sparse_aggregate_result --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_aggregate_result_views --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib --test native_application_bridge -- inline_record_args typed_sparse_inline_record --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_inline_record_arguments --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib --test native_application_bridge -- materialized_record_args typed_sparse_materialized_record --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_materialized_record_arguments --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib --test native_application_bridge -- stored_projection typed_sparse_stored_projections flat_control_interfaces nested_value_admission --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_stored_projections --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- caller_record_constructors --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- caller_spills --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_return_terminal --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_return_fallthrough --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_return_continuations --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_return_aliases --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_return_comparisons --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_return_inline_comparisons --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_return_complete_predicates --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_return_stored_signals --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_return_entry_predicates --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_return_suffixes --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_return_logical_entries --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib --test native_application_bridge -- frontend lowering optimize:: typed_sparse_ typed_nested_preheader_snapshots typed_materialized_ gain_guarded_lowering gain_shared_keep_lowering --test-threads=1 --skip ::native:: --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- optimize:: --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_prefix_values --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_root_values --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_returns --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- conditional_return_prefixes --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p yir-exec -p yir-domain-cpu -p yir-runtime-host -- --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- condition_call_arguments expression_reentry --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib --test native_application_bridge -- private_scalar_aliases typed_sparse_scalar_aliases --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --test native_application_bridge -- typed_sparse_nested_ typed_sparse_post_loop typed_nested_preheader_snapshots --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_post_loop_snapshots native_parent_entries native_materialized_nested_returns native_full_width_nested_returns native_checked_child_snapshots --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_scalar_alias_copies --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked -j1 -p nuis -- dev-tensor --json
```

The earlier post-loop checkpoint passed 1778 compiler unit tests, 89 native bridge tests,
26 ordinary native tests, four source-free workflow tests (seven artifact
variants), five reference image sessions, 26 tensor tests and one host-path policy
test: 1929 distinct selected tests, excluding focused reruns. The rebuilt CLI
reports 1675 clean drift checks with clean coverage/hierarchy/lineage. Formatting,
2984 local documentation links, 4609 tracked UTF-8 files and all 22 changed/new
file limits pass. The capture follow-up has its own tensor evidence and rerun;
these earlier counts are historical, not that follow-up's acceptance. Neither is
a full-workspace test run.

The capture-normalization rerun on 2026-10-03 passed 1788 compiler units, 89 native
bridge tests, 26 ordinary native tests, five source-free workflows (eight artifact
variants), five reference image sessions, 26 tensor tests and one host-path policy
test: 1940 distinct selected tests, excluding focused reruns. The CLI reports
1680 clean drift checks and clean coverage/hierarchy/lineage. Formatting, 2985
local documentation links, 4611 tracked UTF-8 files and all 28 changed/new file
limits pass. These are local CPU/profile checks, not fresh Linux/GPU evidence.

The scalar-alias follow-up checked 1798 compiler units, 104 native bridge tests
across broad/focused runs, 26 ordinary native tests, three source-free workflows
(eight artifact variants), five reference image sessions, 26 tensor tests and one
host-path policy test: 1963 distinct selected tests, excluding focused reruns.
The six new alias guards bring the tensor to 1686 clean drift checks. Formatting,
2986 local links, 4614 tracked UTF-8 files and all 37 changed/new file limits pass.
These are selected local CPU checks, not a full-workspace or fresh device run.

The first combined scalar-alias artifact run encountered one host `SIGKILL`.
An explicit diagnostic rerun passed all three variants without adding a retry or
fallback to the test/launcher. Variant, output phase and input are now retained
in failure diagnostics. A minimal alternating-image cache test subsequently
reproduced a concrete `SIGKILL` class with a valid on-disk signature and an
overwritten file identity. Cache copies and binary materialization now publish
a fresh sibling file instead; see the [artifact publication proof](nuis-artifact-file-publication-v1.md).
The original phase/PID was not retained, so its exact attribution remains unknown.
No callback optimization or native profile change is used to hide this failure.

The typed-literal follow-up on 2026-10-03 passes 850 lowering units, 39 selected
native bridge tests, six cache tests, three source-free workflows (including the
earlier fresh-file publication regression), five reference image/window session
tests, 26 tensor tests and the host-path policy test. The CLI reports 1700 clean
drift guards and clean coverage/hierarchy/lineage. These are selected local tests,
not a full-workspace or fresh device/cross-platform run.

The direct scalar call-field follow-up after `beta-0.15.7` on 2026-10-03 passes
856 lowering units, 44 selected native bridge tests, 28 tensor-related tests,
two source-free workflows, five reference image/window session tests and the
host-path policy case. The workflows cover four scalar/whole-record variants and
a separate selected unused-check variant, including cache hits, pre-open tamper
rejection and repeated same-path restoration. Seven new drift guards bring the
live tensor to 1707 clean checks with clean coverage/hierarchy/lineage; the
coordinate remains `active/86`. UTF-8, changed-file caps, formatting, diff
whitespace and 3002 local documentation links pass.

An initial broader traversal changed existing nested-record transport layouts.
The admitted proof was therefore restricted to direct scalar operands, preserving
composite/codec and aggregate-return provenance. All original wide-layout
assertions pass unchanged; they were not weakened to accept the broader attempt.
This is selected local macOS aarch64 CPU/reference evidence, not fresh Linux/GPU
execution, a full-workspace run or a benchmark.

The evaluated-scalar follow-up passed 864 lowering unit tests, 46 distinct
selected native bridge regressions, three
source-free workflow tests (eight scalar/whole-record/check variants), five
reference image/window sessions, 28 tensor tests and the host-path policy test.
Focused reruns include the final depth-budget test and the complete evaluated
whole-record source-free variant; the fixture adds no unused-function warning.
Eight additional guards bring the tensor to 1715 clean checks with clean
coverage/hierarchy/lineage. Formatting, changed-file limits, UTF-8, diff whitespace
and 3004 local documentation links pass. These are selected local CPU checks,
not a full-workspace, fresh Linux/GPU, benchmark or formal-verification run.

The completed aggregate-call result follow-up passed 873 lowering units, 48
distinct selected native bridge tests, four source-free workflows (11 artifact
variants), five reference image/window sessions, 28 tensor tests and the host-path
policy test. Focused reruns include the strengthened later-unused-field failure:
the first full result completes, the second call fails, and source-free repeated
materialization still publishes only the accepted open state. The original
native layout assertions remain unchanged. Eight additional guards bring the
tensor to 1723 clean checks with clean coverage/hierarchy/lineage; the coordinate
remains `active/86`. The initial task-card prefix edit was corrected without
weakening its existing assertion, and all 28 tensor tests pass on rerun.
Formatting, diff whitespace, all 22 changed/new file limits, 4634 tracked UTF-8
files and 3005 local documentation links pass. These are selected local macOS
aarch64 CPU/reference checks, not a full-workspace, fresh Linux/GPU, benchmark
or formal-verification run.

The inline record call-argument follow-up passed 881 lowering units, 50 distinct
selected native bridge tests, two source-free workflows (six artifact variants),
five reference image/window sessions, 28 tensor tests and the host-path policy
test. The final rerun includes the same-shaped nominal-type negative and the
direct-constructor fast path; all original native layout assertions remain
unchanged. The checked argument variant completes its first full result before
the later argument fails, including cache reuse and repeated same-path source-free
restoration, without successful event publication or fallback. Eight new guards
bring the tensor to 1731 clean checks with clean coverage/hierarchy/lineage;
`active/86` remains unchanged. Formatting, diff whitespace, all 24 changed/new
file caps, 4636 tracked UTF-8 files and 3006 local documentation links pass.
This is selected local macOS aarch64 CPU/reference evidence, not a full-workspace
run, fresh Linux/GPU execution, a benchmark or formal safety verification.

Native test counts above now deduplicate the three `lowering` bridge regressions
already included in the broad capture filters. Their later rerun is not three
additional distinct tests; executed binaries and existing assertions are unchanged.

The materialized record call-argument follow-up passed 889 lowering units, 52
distinct selected native bridge tests, two source-free workflows (six artifact
variants), five reference image/window/registration sessions, 28 tensor tests and
the host-path policy test. The final native rerun checks flattened argument SSA
identity and source field evaluation order; it does not require a redundant LLVM
record construction. All previous transport/layout assertions pass unchanged.
The source-free selected-check variant preserves accepted-open-only publication,
cache reuse, tamper rejection and repeated same-path restoration. Eight new guards
bring the rebuilt CLI to 1739 clean drift checks with clean coverage, hierarchy and
lineage; `active/86` is unchanged. Formatting, diff whitespace, all 26 changed/new
file caps, 4638 tracked UTF-8 files and 3007 local documentation links pass.
These are selected local macOS aarch64 CPU/reference checks, not a full-workspace,
fresh Linux/GPU, benchmark or formal safety-verification run.

The stored-projection follow-up passed 900 lowering-related units, 58 distinct
selected native bridge regressions, two source-free workflows across six artifact
variants, five reference image/window/registration sessions, 28 tensor-related
tests and the host-path policy case. Focused reruns are not additional distinct
tests. The native tests keep both independent full 64-word returns, two scalar
captures plus a guard, and conservative whole-record consumers. The first native
failure exposed the flat-interface control admission gap described above; its
repair preserves the original nested-interface and transport assertions rather
than weakening them. Source-free checked failure retains only the accepted open
state, skips untaken work, rejects tampering and preserves LLVM bytes across
cache reuse and repeated same-path restoration.

Ten new drift guards bring the rebuilt CLI to 1749 clean checks, with clean
coverage, hierarchy and lineage; `active/86` is unchanged. Formatting, diff
whitespace, all 29 changed/new file caps, 4640 tracked UTF-8 files and 3008 local
documentation links pass. This is selected local macOS aarch64 CPU/reference
evidence, not a full-workspace run, fresh Linux/GPU execution, a benchmark or
formal safety verification.

The total inline caller-record checkpoint passes 910 lowering-related tests,
including the new explicit projected-IR native execution probe, plus 58 distinct
native bridge regressions. Two retained source-free workflows cover six artifact
variants, alongside five reference image/window/registration sessions, 28
tensor-related tests and the host-path policy case. Focused reruns and the native
probe already included in the unit suite are not additional distinct tests.
The first broad rerun found scoped caller leakage; the non-scoped fallback gate
fixes it while preserving the original carry-map transactional rejection test.
Canonical codec transport also has a behavioral regression, not only a source
pattern check.

Nine new drift guards bring the rebuilt CLI to 1758 clean checks, with clean
coverage, hierarchy and lineage; `active/86` remains unchanged. Formatting, diff
whitespace, all 34 changed/new file caps, 4644 tracked UTF-8 files and 3010 local
documentation links pass. This is selected local macOS aarch64 CPU/reference
acceptance, not a full-workspace run, fresh Linux/GPU execution, a benchmark,
formal safety verification or computed-inline-call spilling.

The binding-root materialization checkpoint passes 918 lowering-related tests,
including the explicit projected-IR probe with three native executables, plus
58 distinct native bridge regressions. A final focused rerun also checks two
identical producer expressions receiving distinct complete bindings and LLVM
ordering of the before-check, producer, after-division and helper invocation.
Focused reruns and native variants already inside that unit test do not add
distinct test counts. The existing return/condition/loop/scoped rejection and
every-caller rollback assertions remain unchanged.

Two retained source-free workflows cover six artifact variants, alongside five
reference image/window/registration sessions, 28 tensor-related tests and the
host-path policy case. Nine new guards bring the rebuilt CLI to 1767 clean drift
checks, with clean coverage, hierarchy and lineage; `active/86` is unchanged.
Formatting, diff whitespace, all 38 changed/new file caps, 4648 tracked UTF-8
files and 3012 local documentation links pass. This is selected local macOS
aarch64 CPU/reference acceptance, not a full-workspace run, fresh Linux/GPU
execution, a benchmark, formal safety verification or general expression spilling.

The return-root checkpoint passes 922 lowering-related tests and 58 distinct
native bridge regressions. A final 12-test focused rerun additionally verifies
nominal Pair returns, with independent before/after results of `13`, while the
scalar return-root native probe retains success `26`, early exit `0` and reached
unused-field traps. The two explicit-IR native tests each run three binaries;
those variants and focused reruns are already inside the unit acceptance and are
not additional distinct tests.

Two retained source-free workflows cover six artifact variants, with five
reference image/window/registration sessions, 28 tensor-related tests and the
host-path policy case passing. Five new guards bring the rebuilt CLI to 1772
clean drift checks with clean coverage, hierarchy and lineage; `active/86`
remains unchanged. Formatting, diff whitespace, all 48 changed/new file caps,
4649 tracked UTF-8 files and 3014 local documentation links pass. This is selected
local macOS aarch64 CPU/reference acceptance, not a full-workspace run, fresh
Linux/GPU execution, a benchmark, formal safety proof or general expression
spilling.

The if-condition-root checkpoint passes 1828 frontend/lowering unit tests and
58 distinct selected native bridge regressions. The final 21-test focused rerun
includes all 16 caller-materialization tests, the three new parser regressions
and two existing expression-depth cases; it is not additional distinct coverage.
Seven new test functions include the independent 192-case before/after oracle
and five new native executables. The three shared binding/return/condition native
tests cover 11 executable variants already counted inside the unit suite.

Two retained source-free workflows cover six artifact variants, alongside five
reference image/window/registration sessions, 28 tensor-related tests and the
host-path policy case. Nine new guards bring the rebuilt CLI to 1781 clean drift
checks, with clean coverage, hierarchy and lineage; `active/86` remains unchanged.
Formatting, diff whitespace, all 52 changed/new file caps, 4651 tracked UTF-8
files and 3016 local documentation links pass. Fallible direct branch returns
retain their original guarded-lowering restriction; the native probes use
supported local-result bindings. This is selected local macOS aarch64
CPU/reference and explicit private-IR acceptance, not automatic source-function
projection eligibility, a full-workspace run, fresh Linux/GPU execution, a
benchmark, formal safety verification or general expression spilling.

The selected short-circuit RHS checkpoint passes 1835 frontend/lowering unit
tests and 58 distinct selected native bridge regressions. Seven new test
functions include the independent 384-case before/after oracle, eight default
source-condition cases and seven new native binaries. The final broad run also
covers all 28 focused caller/prefix/parser/depth regressions; focused reruns and
the four native tests' 18 binary variants are not additional distinct tests.

Two retained source-free workflows cover six artifact variants, alongside five
reference image/window/registration sessions, 28 tensor-related tests and the
host-path policy case. Ten new guards bring the actual rebuilt CLI to 1791 clean
drift checks with clean coverage, hierarchy and lineage; `active/86` remains
unchanged. The old local-selection drift guard now separately requires its
full-control exclusion flag and branch, without removing the kind/capture checks.
Formatting, diff whitespace, all 57 changed/new file caps, 4654 tracked UTF-8
files and 3018 local documentation links pass. This is selected local macOS
aarch64 CPU/reference acceptance, including default-source laziness and explicit
private-IR projection, not a full-workspace run, fresh Linux/GPU execution,
a benchmark, formal safety proof or general nested/short-circuit capture closure.

The source logical value-root checkpoint passes 1842 frontend/lowering unit tests
and 58 selected native bridge regressions on the final inferred-kind refinement.
Seven new compiler test functions cover 144 runtime source cases, 288 reference
executions, complete root/arm identity, current versions, exact selected helper
counts, idempotence, negative admission and ten default-source native binaries.
Those variants and focused reruns are included in the broad test count, not
additional distinct tests. Two source-free workflows cover six artifact variants;
five compiled reference sessions and the host-path policy regression also pass.

The CPU and graph-executor suites pass 133 tests, including three new generic
registered-entry-exit regressions and expanded CPU composite-exit kind/arity
checks. Runtime-host unit coverage passes 81 of 84 cases; three temporary Unix
socket admission tests fail at socket binding under the filesystem/network
sandbox before executor assertions. The requested unsandboxed rerun was not
executed because the approval review service disconnected, so those cases remain
pending, not accepted. All eleven non-socket application-session integration
cases pass. This checkpoint does not claim a fully green runtime-host suite.

Twenty-eight tensor-related tests and the rebuilt CLI retain `active/86` with
1800/1800 passing drift guards and clean coverage, hierarchy and lineage. Nine
new guards cover source-root typing/version windows, selected execution counts,
default AOT probes, registered exits, composite statement-effect ordering and
documentation/tensor boundaries. Formatting, diff whitespace, all 64 changed/new
file caps, 4657 tracked UTF-8 files and 3020 local documentation links pass.
This remains local macOS aarch64 CPU/reference evidence, not full-workspace
acceptance, fresh Linux/GPU execution, a benchmark or a formal safety proof.

The effectful-parent conditional-return checkpoint passes 1848 frontend/lowering
unit tests and 58 selected native bridge regressions. Its six new compiler tests
include 210 source cases, 420 reference executions and ten default-source native
binary variants; those variants are included in the broad count, not additional
distinct tests. Two source-free workflows cover six artifact variants, and five
compiled reference application sessions pass. Twenty-eight tensor-related tests
also pass after retaining the prior callback, codec, capture and projection
boundary ledger and updating the registration guard for both outliner passes.

The actual CLI reports 1808/1808 passing drift checks, clean coverage, hierarchy
and lineage, and unchanged `active/86`. Eight new guards cover exact kinds, local
statement windows, complete guarded returns, pipeline registration, the intact
speculation barrier, execution/veto proofs, default AOT and documented scope.
Formatting, diff whitespace, all 69 changed/new file caps, 4661 tracked UTF-8
files and 3022 local documentation links pass. The prior three sandbox-blocked
Unix socket admission tests remain pending; they are not counted as accepted.
This is selected local macOS aarch64 CPU/reference acceptance, not full-workspace
or fresh Linux/GPU acceptance, a benchmark, formal safety proof, broader private
capture projection or arbitrary effectful control-flow closure.

The pure conditional-return prefix checkpoint passes 1854 frontend/lowering unit
tests and 58 selected native bridge regressions. Six new compiler tests contain
468 source cases, 936 ordinary/reversed reference executions, 18 structural
combinations, frontend and direct-NIR vetoes, shared work/depth/statement boundary
probes and twelve default-source native binary variants. Those variants and
focused reruns are already included in the broad count, not additional distinct
tests. The negative-source test now distinguishes frontend type/scope rejection
from lowering admission; direct-NIR mutations independently retain the lowering
kind, capture and whole-module-equality checks.

Two retained source-free workflows cover six artifact variants; five compiled
reference application sessions and the host-absolute-path policy regression
also pass. Twenty-eight tensor-related tests and the actual CLI retain
`active/86`, with 1814/1814 passing drift checks and clean coverage, hierarchy and
lineage. Six new guards cover complete fresh local admission, shared work/depth
limits, execution/hygiene/veto proof, default AOT, documentation and tensor scope.
All 72 current changed/new file caps, 4664 tracked UTF-8 files and 3024 local
documentation links pass, alongside formatting and diff whitespace checks.
The earlier three sandbox-blocked Unix socket admission tests remain pending;
this is selected local macOS aarch64 CPU/reference acceptance, not full-workspace
or fresh Linux/GPU acceptance, a benchmark, formal safety proof, resource/aggregate
capture transport or general effectful/nested return closure.

The pure terminal-return tree checkpoint passes 1861 frontend/lowering unit
tests and 58 selected native bridge regressions. Seven new compiler tests include
507 runtime source cases, 1014 ordinary/reversed reference executions, twelve
structural combinations, frontend/direct-NIR veto checks, exact shared statement/
expression/depth boundaries and ten default-source native binary variants.
These variants and focused reruns are already in the broad count, not additional
distinct tests. Two retained source-free workflows cover six artifact variants;
five compiled reference application sessions and the host-absolute-path policy
regression also pass.

Twenty-eight tensor-related tests and the actual CLI retain `active/86` with
1821/1821 passing drift checks and clean coverage, hierarchy and lineage. Seven
new guards cover fresh lexical admission, whole-tree preflight, independent
fallback registration, execution/identity/veto proof, default AOT, documentation
and tensor scope. Formatting, diff whitespace, all 75 current changed/new file
caps, 4667 tracked UTF-8 files and 3026 local documentation links pass. The earlier
three sandbox-blocked Unix socket admission tests remain pending and are not
counted as accepted. This is selected local macOS aarch64 CPU/reference acceptance,
not a full-workspace or fresh Linux/GPU run, a benchmark, formal safety proof,
resource capture transport or general nonterminal/effectful return closure.

The pure fallthrough-return tree checkpoint passes 1870 frontend/lowering units
and 58 selected native bridge tests. All nine new compiler tests and fourteen
new default-source native binary variants are included in those counts rather
than counted again. A wider run found one obsolete rejection assertion for the
newly admitted pure tree. The original fixture now independently compiles and
emits LLVM, while branch-local effects and helper-local exit predicates still
reject under the unchanged fallible-return speculation barrier. The final broad
rerun passes without weakening a type, capture, budget or speculation check.

Two retained source-free workflows cover six artifact variants; five compiled
reference application sessions, the host-absolute-path policy test and 28
tensor-related tests also pass. The actual CLI retains `active/86` with 1829/1829
passing drift checks and clean coverage, hierarchy and lineage. Eight new guards
cover stable-parent preflight, readiness/seeding, value-before-exit installation,
execution/identity/veto/budget proof, default AOT, documentation, original-root
admission/rejection boundaries and tensor scope. Formatting, diff whitespace,
all 78 current changed/new file caps, 4670 tracked UTF-8 files and 3028 local
documentation links pass. The earlier three sandbox-blocked Unix socket tests
remain pending and are not counted as accepted. This is selected local macOS
aarch64 CPU/reference acceptance, not a full-workspace or fresh Linux/GPU run,
a benchmark, formal safety proof, resource capture transport or general
effectful/nonterminal return closure.

The pure continuation-leaf work checkpoint passes 1878 frontend/lowering unit
tests and 58 selected native bridge regressions. All eight new compiler tests
and sixteen new default-source native binary variants are included in those
counts, not added again. The existing empty-leaf, complete terminal-tree and
speculation-veto regressions remain accepted without widening a capture or ABI.

Two retained source-free workflows cover six artifact variants; five compiled
reference application sessions, the host-absolute-path policy regression and
28 tensor-related tests also pass. The actual CLI retains `active/86` with
1836/1836 passing drift checks and clean coverage, hierarchy and lineage. Seven
new guards cover whole-leaf grammar/type/preflight, original-return eligibility,
seed-after-work/readiness separation, execution/identity/veto/budget proof,
default AOT, documentation and tensor scope. Formatting, diff whitespace,
all 80 current changed/new file caps, 4672 tracked UTF-8 files and 3030 local
documentation links pass. Earlier three sandbox-blocked Unix socket tests remain
pending and are not counted as accepted. This is selected local macOS aarch64
CPU/reference acceptance, not a full-workspace or fresh Linux/GPU run, a benchmark,
formal safety proof, resource transport or effectful continuation closure.

The local atom-alias readiness checkpoint passes 1887 frontend/lowering unit
tests and 58 selected native bridge regressions. All nine new compiler tests and
twelve new default-source binary variants are included in those counts, not
added again. Former atom-alias veto sources independently compile; computed and
call-derived predicates, alias laundering and sibling leakage retain rejection
evidence. Exact 30-binding plus branch/return 32-statement trees compile and emit
LLVM; one extra original alias rejects unchanged.

Two retained source-free workflows cover six artifact variants. Five compiled
reference application sessions, the host-absolute-path policy regression and
28 tensor-related tests also pass. The actual CLI retains `active/86` with
1843/1843 passing drift checks and clean coverage, hierarchy and lineage. Seven
new guards cover bounded atom proof, lexical scopes/no initializer replay,
reference order/exit proof, computed vetoes/shared budgets, default AOT,
documentation and tensor scope. Formatting, diff whitespace, all 82 current
changed/new file caps, 4674 tracked UTF-8 files and 3032 local documentation links
pass. Earlier three sandbox-blocked Unix socket tests remain pending and are not
counted as accepted. This is selected local macOS aarch64 CPU/reference evidence,
not a full-workspace or fresh Linux/GPU run, benchmark, formal safety proof,
resource transport or effectful continuation closure.

The total comparison-readiness checkpoint passes 1917 selected frontend/lowering/
optimizer unit tests and 58 selected native bridge regressions. Its nine new
conditional-return tests and two new optimizer tests are included in that count;
the fourteen default-source binary variants and focused reruns are not counted
again. Former atomic-comparison veto sources independently compile and emit LLVM,
while nested comparisons, alias DAGs, unfolded arithmetic, calls, field reads,
inline computed conditions and effectful work retain rejection proofs.

Two retained source-free workflows cover six artifact variants. Five compiled
reference application sessions, the host-absolute-path policy regression and
28 tensor-related tests also pass. The actual CLI retains `active/86` with
1852/1852 passing drift checks and clean coverage, hierarchy and lineage. Nine
new guards cover bounded typed comparison facts, shared preflight, reference
execution/order and non-total vetoes, default AOT, bool-kind-preserving constant
folding and its type/effect regressions, documentation and tensor scope.
Formatting, diff whitespace, all 87 current changed/new file caps, 4677 tracked
UTF-8 files and 3034 local documentation links pass. Earlier three sandbox-blocked
Unix socket tests remain pending and are not counted as accepted. This is selected
local macOS aarch64 CPU/reference evidence, not full-workspace or fresh Linux/GPU
acceptance, a benchmark, formal safety proof, resource transport or general
effectful continuation closure.

The inline partial-tree comparison checkpoint passes 1926 selected frontend/
lowering/optimizer unit tests and 58 selected native bridge regressions. Its nine
new compiler tests and fourteen default-source binary variants are included in
those counts, not counted again. Two retained source-free workflows cover six
artifact variants; five compiled reference application sessions, one host-path
policy regression and 28 tensor-related tests also pass: 2020 distinct selected
tests, excluding focused reruns.

The actual rebuilt CLI retains `active/86` with 1860/1860 passing drift checks and
clean coverage, hierarchy and lineage. Eight new guards cover shared registration,
partial-only typed grammar, independent lexical totality/readiness, reference
execution/order, structural identity and budget/veto proofs, default-source AOT,
documentation and tensor scope. Formatting, diff whitespace, all 90 changed/new
file caps, 4680 tracked UTF-8 files and 3036 local documentation links pass.
The earlier three sandbox-blocked Unix socket tests remain pending and are not
counted as accepted. This is selected local macOS aarch64 CPU/reference evidence,
not full-workspace or fresh Linux/GPU acceptance, a benchmark, formal safety proof,
resource transport or general inline/effectful continuation closure.

The complete-tree computed-predicate checkpoint passes 1935 selected frontend/
lowering/optimizer unit tests and 58 selected native bridge regressions. Its nine
new compiler tests and sixteen default-source binary variants are included in
those counts, not added again. Two retained source-free workflows cover six
artifact variants; five compiled reference application sessions, one host-path
policy regression and 28 tensor-related tests also pass: 2029 distinct selected
tests, excluding focused reruns. A first compile attempt caught a name-shadowing
error in the new fixture; the corrected final-code focused and broad runs pass.

The actual rebuilt CLI retains `active/86` with 1868/1868 passing drift checks and
clean coverage, hierarchy and lineage. Eight new guards cover exact bool inference
and mode separation, shared whole-tree budgets/scalar captures, once-only runtime
order and checks, original-body/budget/veto proofs, default AOT, the retained
former complete inline fixture, documentation and tensor scope. The older
complete-tree drift rule now requires the precise bool type contract rather than
the superseded atomic-predicate spelling. Formatting, diff whitespace, all 92
changed/new file caps, 4682 tracked UTF-8 files and 3038 local documentation links
pass. The earlier three sandbox-blocked Unix socket tests remain pending and are
not counted as accepted. This is selected local macOS aarch64 CPU/reference
evidence, not full-workspace or fresh Linux/GPU acceptance, a benchmark, formal
safety proof, resource transport or partial-tree computed/effectful exit closure.

The stored partial-tree exit-signal checkpoint covers 1943 selected compiler
units: 1942 passed in the broad frontend/lowering/optimizer run, and the retained
logical-root fixture passed its corrected focused rerun. That fixture now checks
pure predicate aliases positively and still rejects branch effects; the compiler
implementation was not loosened to satisfy it. All eight new compiler tests
passed again on final code, including twenty default-source binary variants.
The initial behavioral probe caught eager RHS lowering when a logical return was
placed in a record field; rooted value materialization fixes it, with both
operators independently checking skipped and reached zero-divisor RHS paths.

The 58 selected native bridge regressions, two retained source-free workflows
(six artifact variants), five compiled reference application sessions, one
host-path policy test and 28 tensor tests also pass. These are 2037 distinct
selected tests across the broad and corrected focused runs, not a second count
of repeated tests or binary variants. Tensor validation initially caught a
displaced historical evidence prefix; its original ordering and unchanged entry
assertion are preserved, with the new receipt beside its predecessor checkpoint.

The rebuilt CLI reports `active/87`, 1876/1876 passing drift checks and clean
coverage, hierarchy and lineage. Eight new guards cover bounded original-tree
admission, private independent exit/value transport and rooted logical returns,
original-leaf wrapping instead of replay seeds, shared layout/catalog refresh,
reference structure/budget checks, default-source binaries, documentation and
tensor scope. Formatting, diff whitespace, all 96 changed/new file caps, 4686
tracked UTF-8 files and 3040 local documentation links pass. The prior three
sandbox-blocked Unix socket tests remain pending and are not included. This is
selected local macOS aarch64 CPU/reference evidence, not full-workspace or fresh
Linux/GPU acceptance, a benchmark, formal safety, resource transport or general
effectful/nonterminal return closure.

The computed outer-entry checkpoint passes all 1951 selected compiler units in
one broad frontend/lowering/optimizer run, plus 58 native bridge regressions.
All eight new tests are included, with 201 runtime source cases, 402 reference
executions and eighteen default-source binary variants. The three retained
outer-entry veto fixtures now have independent positive execution probes, while
their unrelated effect/type/budget vetoes remain intact.

The 28 tensor tests, two source-free workflows (six artifact variants), five
compiled reference application sessions and one host-path policy test also pass.
These are 2045 distinct selected tests; repeated runs, source cases and binary
variants are not counted again as tests. The rebuilt CLI reports `active/88`,
1884/1884 passing drift checks and clean coverage, hierarchy and lineage, with
the original historical evidence prefix preserved. Eight new guards cover entry
preflight/type admission, original-site once-only storage and saved-bool transport,
reference ordering/partial exits, structure/budgets/vetoes, default-source binaries,
documentation, retained fixtures and tensor scope. Static checks cover 1882
definitions and 12735 patterns without missing evidence. Formatting, diff whitespace,
all 100 changed/new file caps, 4690 tracked UTF-8 files and 3042 local documentation
links pass. The prior three sandbox-blocked Unix socket tests remain pending and
are not counted. This is selected local macOS aarch64 CPU/reference evidence,
not full-workspace or fresh Linux/GPU acceptance, performance/formal-safety proof,
resource transport or general effectful/nonterminal return closure.

The bounded intermediate-suffix checkpoint passes 1959 distinct selected compiler
units: 1957 pass the broad frontend/lowering/optimizer run, and two obsolete
pure-suffix rejection fixtures pass their corrected focused rerun without any
production changes. Those original sources are retained as positive execution
probes, each expanded to twelve source cases in both reference orders. All eight
new compiler tests pass, including 442 runtime source cases, 884 reference executions
and eighteen default-source AOT variants. The retained fixtures are additional
source evidence, not additional distinct tests.

The 58 native bridge regressions, 28 tensor tests, two source-free workflows
(six artifact variants), five compiled reference application sessions and one
host-path policy test also pass: 2053 distinct selected tests in total, excluding
repeated runs and binary variants. This is combined broad/focused acceptance,
not an assertion that the initial broad run was entirely green.

The rebuilt CLI reports `active/89`, 1892/1892 passing drift checks and clean
coverage, hierarchy and lineage, preserving the original historical evidence
prefix. Eight new guards cover original preflight, lexical/type/reachability
validation, bounded borrowed expansion before cloning, independent arm/capture
revalidation, reference structure/budget/veto probes, default-source binaries and
documentation/tensor scope. Static checks pass 1890 definitions and 12793 patterns.
Formatting, diff whitespace, all 104 changed/new file caps, 4694 tracked UTF-8
files and 3044 local documentation links pass. The previous three sandbox-blocked
Unix socket tests remain pending and are not counted. No full-workspace, fresh
Linux/GPU, benchmark, formal safety, resource capture or general effectful-return
acceptance is claimed.

The single-edge logical-entry checkpoint passes 1938 compiler units in a broad
frontend/lowering/optimizer rerun with `--skip ::native::`. Both new default-source
AOT tests independently pass eighteen binary variants on the same final production
implementation. All six new non-native tests pass in the broad rerun, including
412 runtime source cases and 824 reference executions. An initial veto fixture
incorrectly expected lowering rejection for `bool && Packet`; it now explicitly
asserts the earlier front-end operand-type diagnostic. Production admission was
not loosened to bypass the failure. The former single-edge entry veto source is
retained as an additional positive execution probe.

The 58 native bridge regressions, 28 tensor tests, two source-free workflows
(six artifact variants), five compiled reference application sessions and one
host-path policy test also pass: 2034 distinct selected tests, counting the two
new AOT tests once, not their eighteen variants or focused reruns. The 27 other
`::native::` tests from earlier checkpoints were not rerun here; their historical
receipts are retained but are not claimed as this checkpoint's acceptance.

The rebuilt CLI reports `active/90`, 1900/1900 passing drift checks and clean
coverage, hierarchy and lineage, preserving the original historical evidence
prefix. Eight new guards cover single-edge whole-root admission, shared guarded
value-root consumption, reference ordering/partial composition, structure/budgets/
vetoes, default-source binaries, the retained fixture and documentation/tensor
scope. Static checks pass 1898 definitions and 12840 patterns. Formatting, diff
whitespace, all 107 changed/new file caps, 4697 tracked UTF-8 files and 3046 local
documentation links pass. The previous three sandbox-blocked Unix socket tests
remain pending and are not counted. This is selected local macOS aarch64 CPU/source
evidence, not full-workspace, fresh Linux/GPU, benchmark, formal safety, resource
capture or general logical/effectful-return acceptance.

The guarded logical-arm-root checkpoint passes 1944 compiler units in the broad
frontend/lowering/optimizer rerun with `--skip ::native::`, plus two independently
passing new default-source AOT tests on the same production implementation.
All eight new compiler tests pass across those runs: 996 runtime source cases,
1992 reference executions and eighteen binary variants. An initial prefix fixture
reused `ignored` across an earlier local and a branch; it was renamed
`prefix_checked`, not admitted by relaxing name-collision validation. Ten earlier
suites retain their newly admitted source fixtures as positive execution probes.

The 58 native bridge regressions, 28 tensor tests, two source-free workflows
(six artifact variants), five compiled reference application sessions and one
host-path policy test also pass: 2040 distinct selected tests, counting each new
AOT test once, not its variants or reruns. The 29 other `::native::` tests from
earlier checkpoints were not rerun here; their historical receipts are retained.
The rebuilt CLI reports `active/91`, 1908/1908 passing drift checks and clean
coverage, hierarchy and lineage, with the historical evidence prefix intact.
Eight new guards pass; static checks cover 1906 definitions and 12899 patterns.
Formatting, diff whitespace, 110 changed/new file caps, 4700 tracked UTF-8 files
and 3048 local documentation links pass. Three previously sandbox-blocked Unix
socket tests remain pending and uncounted. This is selected local CPU/source
evidence, not full-workspace, fresh Linux/GPU, benchmark, formal safety or general
logical/effectful-return acceptance.

## Remaining Boundaries

The persistent-session coordinate is now `active/94`. Empty fallthrough leaves
and fresh pure continuation bindings now have independent bounded proofs above.
Fresh helper-local atom aliases now have a separate bounded readiness proof.
Local total comparisons of stable bool/i64 atoms now have a separate proof.
Inline total comparisons inside partial return trees now share the bounded facts.
Complete all-return computed predicates now have their own once-only pure-value
proof; outer-entry predicates now have the independent original-site proof above.
Bounded pure computed/call-derived partial predicates now use the separate
stored-signal proof above. Bounded pure computed outer-entry gates now store one
owned bool at their original parent site, without replay or new helper captures.
Bounded pure intermediate branch suffixes now have the independent original-scope
and expanded-budget proof above, without moving work past an observed exit.
Single-edge logical outer entries now have their separate original-site value-root
proof, using exact owned bool atom/literal left gates and bounded nonlogical RHS.
Internal single-edge logical conditions and fresh bool binding roots now have the
independent shared-guard/stored-signal proof above, retaining earlier checks.
A separate [computed logical gate checkpoint](nuis-native-computed-logical-gates-v1.md)
extends source value roots and outer entries with whole-root preflight and
once-only pure left computation. Its separate computed return-arm follow-up
covers bounded conditions, fresh bindings and return roots without new captures.
A further [nested logical-tree proof](nuis-native-computed-logical-gates-v1.md#nested-logical-trees)
guards direct logical children at those roots under a shared 32-edge budget;
ordinary leaf embeddings and effectful prefixes remain excluded.
Logical outer gates and other inline conditions
outside the computed-gate, single-edge outer-entry/return-arm and earlier proofs,
suffixes outside the original/expanded budgets and
effectful-prefixed source returns inside partial trees, plus branch-local effects,
still require their
own guarded-computation and parent-ordering proof; the top-level scalar return,
terminal-tree, fallthrough and pure-continuation proofs do not authorize them. Further changing
backedges, changing local versions, opaque/effectful aggregate snapshots and computed
inline aggregate caller arguments outside binding/return/direct-if or selected
single-edge short-circuit RHS materialization need additional bounded proof,
not a wider native limit.
Completed pure call-result locals and stored call-rooted field projections
now have the separate proofs above; directly written and materialized immutable
record arguments also have exact
typing without erasing checked constructors or computed-call spilling at
projected helper call sites. Total inline constructors from unwritten input
paths now have a distinct caller-side proof, not general expression spilling;
computed binding/return/if-condition-root operands now have a separate complete,
ordered materialization proof outside loops, with a separate selected single-edge
short-circuit RHS proof, not general nested-condition or nested-expression authority;
this is not general aggregate-call-backed capture closure. The near-boundary
preceding-loop captures above are reduced, not general capture closure. Checked
constructors, opaque evaluation, observed exits and the reference image/window
regressions must remain intact.

This is local macOS aarch64 CPU evidence, not a fresh Linux/GPU run, a benchmark,
formal safety verification or full persistent-application closure.

Strict all-target Clippy for `nuisc`/`nuis` remains blocked by 11 existing compiler
lints, chiefly long lowering signatures plus simple style/borrow findings. The
observed LLVM lints were corrected without suppressions or a newer Rust
API requirement; their carry pair-shape and declared-result behavior remain
separate regression boundaries. Broad interface refactoring is not silently
included in the literal-origin proof or its runtime acceptance.
The LLVM library rerun passes 167 tests with one pre-existing ignored test,
including the new zero/odd/exact-64/over-limit carry pair-shape guard. Its
all-target strict Clippy check passes independently of the wider compiler gate.
