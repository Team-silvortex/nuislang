# Native Scalar Session Bridge V1

Status: experimental static CPU bridge with explicit build, artifact and launch selection.
This is not the default application host, a native window renderer, or a
self-contained Nsld image. The ordinary compiled image host still runs embedded
YIR callbacks.

## Selection

`yir_lower_llvm::native_session::emit_registered(module, id)` consumes an existing
YIR application-session registration and its shared `ApplicationSessionSignature`.
It emits a complete LLVM unit containing the registered open/event/close helpers,
their admitted helper closure and one static export per role. No source function names or application policies
are hardcoded. Export symbols encode the registration ID as UTF-8 bytes in hex.
The contract identifier is `nuis-native-scalar-session-bridge-v1`.

The first admitted profile is deliberately small:

- CPU helpers containing scalar parameters/constants, selected arithmetic and
  comparisons, struct construction/fields, selection and aggregate returns.
- Nested nonempty scalar state made of `bool`, `i32`, `i64`, `f32` and `f64`.
- Acyclic helper calls with scalar parameters and one declared scalar return,
  including nested calls, zero-argument helpers and guarded scalar returns.
- Checked mixed/nested scalar aggregate helper returns through ordinary calls.
  Scoped calls retain flat-i64 carries, guarded branches and explicit-step continue.
- Checked i64 division/remainder, with exact operands and reached-path failure.
- Counted i64 loops with constant or runtime-checked induction inputs and ordered
  scalar add/multiply carries, with at most 64 carries and 65536 iterations per loop.
- Scoped loop-body calls with a discarded scalar result, one i64 carry, or a
  checked flat aggregate of at most 64 i64 carries returned before the induction step.
- At most 64 input slots, 64 state slots and 4096 body nodes per function.
- At most 64 reachable functions (including registered roots), 16384 total body
  nodes and 32 functions on any root-to-leaf call path.
- No recursion, unbounded loops, resource/provider effects, resource-bearing state
  or global init.

Each callback must have one canonical owned-aggregate return whose nominal type,
field order and scalar kinds agree with the event/close flattened state signature.
Parameter nodes must match their declared indices, types and function lanes.
Only explicit typed calls may cross a function boundary. Direct value dependencies
and incoming edges from other functions are rejected, not silently initialized or
discarded. Each reachable helper is checked, even if its result is unused or a guard
would bypass the call. Unselected helpers and unrelated main are not emitted or replayed.
The call graph is checked iteratively, with shared callees counted once and the
longest path bounded; it does not recursively traverse an untrusted host stack.

This bridge uses strict native value materialization: every reached value-producing
node must acquire an LLVM value, and the function must emit a terminal return.
It does not decide completeness by scanning diagnostics or LLVM comments.
Helper arguments and scalar returns must have the exact declared materialized kind,
including guarded returns; the generic backend's bool-to-i64 compatibility is not
admitted here. Guards require bool/i64 conditions; scalar helper guards cannot carry
an aggregate layout. General LLVM inspection keeps its existing partial-lowering behavior.
Supported synchronous `@noinline` functions now retain their NIR-to-YIR call boundary.
Aggregate-helper reachability includes registered host entries and explicit
export/noinline roots, not only main. This is a common lowering rule, not a
hardcoded application-session name. Unsupported outer-state updates reject
instead of degrading to a counter-only loop after scoped lowering declines.
Likewise, a single-carry scoped call cannot absorb another outer-state update
as a temporary prefix; multi-carry updates need the explicit supported shape.
This is not a guarantee that downstream LLVM optimization preserves machine calls.

## Checked Arithmetic

The profile admits existing `cpu.div` and `cpu.rem` only for two exact i64 values.
It reuses the ordinary LLVM scalar emitter: reached operations check zero divisors
and `i64::MIN` with `-1` before `sdiv` or `srem`. Both invalid cases trap for both
operators. Valid signed results truncate toward zero and remainder follows the
dividend. Implicit bool/i32/float conversion and typed i32/f32/f64 division are
not admitted by this profile. No new opcode, callback ABI or arithmetic runtime
is introduced.

Purity is not permission to speculate checked arithmetic. Shared source lowering
propagates division/remainder through a cached, iterative call graph, independent
of declaration order. Eligible acyclic i64/bool scalar helpers reuse the existing
guarded branch/continuation outliner. Prefix work keeps source order; an unselected
branch skips its arithmetic, even when its result is discarded or passed to a
callee that ignores the argument. Work before the guard still executes.

Acyclic helpers with i64/bool and flat-i64 record values now share this outliner.
It supports early/two-arm/nested returns, aggregate locals and captures, field
projections, same-callee arguments and shared fallthrough suffixes. Only existing
values cross a generated branch boundary; captures are flattened through the
existing scalar-parameter contract. An unselected private helper returns a typed
zero record before user arithmetic or callees execute. The final select operates
on already guarded values, never on deferred branch expressions.

The source value catalog is independent of the narrower Buffer-loop catalog and
the backend's slot/call-graph bounds. It rejects effects, recursive dependencies,
general rebinding and unsupported payloads. A shared suffix is outlined once
rather than copied into every branch. Counted loops now compose inside these
helpers. The compact metadata profile uses one induction update followed by ordered
i64 carry updates, with invariant
i64 literal/variable limit and stride. Every updated name must be a distinct,
already initialized local `let` of exact i64 type. The existing counted/chained-loop
parsers and native preflight remain authoritative; this source shape check does
not prove termination or duplicate backend slot/opcode admission.
Each carry uses the existing linear add/multiply preparation, reading the stepped
index or earlier updated carries. Full `if/else` may update the same carry once in
each arm, including explicit `let carry: i64 = carry;` to keep its previous value.
An omitted `else`, empty `else`, or empty `then` uses the same shared `keep`
contract for that carry. It never synthesizes a seed or copies a sibling's value.
Both-empty arms reject; value-producing conditional expressions still require
both values. This does not relax local initialization or mutable exact-i64 checks.
Leaf comparisons use an already stepped index or earlier updated carry against an
invariant i64 literal/variable. Reversed comparisons share the ordinary loop
preparation; mutable predicate RHS values are not captured as stale seeds.
These leaves may form nested `&&`/`||` predicates. Source grouping and precedence
are retained in the existing `LoopCondExpr` tree; no new condition IR or opcode
is introduced. LLVM branches around the right operand and merges the boolean
result. Calls, fallible expressions and mutable RHS values are still excluded
from every leaf, including a right operand that could be skipped at runtime.
Header/stride mutation, forward sibling reads,
duplicate updates, body calls and fallible update expressions are rejected.
Fallible seed expressions outside the loop retain ordinary source-order checks.
Constants, parameter rebinding and arbitrary loop bodies are not admitted by this
compact profile. Sequences, calls, projections and checked expressions use the scoped
normalization described below instead of widening metadata operands. The narrower Buffer catalog
is unchanged.

### Nested Carry-Update Arms

Nested `if`/`else`, including `else if` and empty arms, may update the same local i64
carry in source order along each selected path. Different leaves may compute three or more
distinct i64 results; an empty arm retains that carry's incoming value.
The basic nested profile compares the stepped index or an earlier updated carry
with an invariant i64 atom; local and checked conditions use the scoped expression
rules below. Every leaf is checked, including unreachable leaves, and forward
sibling reads, effects, unsupported calls and header mutation remain rejected. Ordered
multi-statement updates are described below. The source shape scan allows
at most 32 nested guards; native closure and slot limits remain independent.

The normalizer outlines one private iteration function through the existing
scoped-call and flat-i64 return contracts. It advances a private index copy before
the branch tree, then returns the carries. The outer driver advances its own index
exactly once, preserving the source's step-first semantics and the original finite,
non-wrapping induction preflight. Multi-carry projection retains source order and
the YIR ownership contract. Native flat helper transport is described below;
ordinary LLVM retains aggregate allocation/drop. No new loop opcode is added.

The effect outliner snapshots branch decisions and calls guarded helpers; a skipped
branch returns before its own decisions or updates execute. Boolean `&&`/`||` inside
these new helpers use guarded scalar calls, not eager boolean arithmetic. Both use
the existing neutral-false guard contract (`a || b = !(!a && !b)`), without duplicating
arm bodies or relaxing native guard admission. One helper per logical edge keeps
this part of normalization linear. Native flat neutral returns now use values rather
than heap owners. Ordinary LLVM retains its aggregate allocation/drop path.

### Ordered Multi-Statement Carry Bodies

The scoped iteration route also admits sequential writes to pre-existing mutable
i64 locals, multiple statements in either arm, and different write sets in the
then/else arms. Each unique carry is captured and returned once in first lexical
write order; repeated assignments do not add return slots or driver steps.
Branch decisions are snapshotted before their arms execute, so mutations cannot
retroactively change which arm was selected. Prefixes and suffixes run in order.

```text
let index: i64 = initial;
let total: i64 = seed;
let checksum: i64 = 0;
while index < limit {
  let index: i64 = index + stride;
  let total: i64 = total + index;
  if total < pivot {
    let total: i64 = total * 2;
    let checksum: i64 = checksum + total;
  } else {
    let total: i64 = total - 1;
  }
  let total: i64 = total + checksum;
}
```

The no-forward-sibling-read policy is unchanged: an update reads its own current
value, invariant atoms, or carries made available by preceding statements.
Conditions still compare the stepped index or a preceding carry with an invariant
i64 atom. Each arm is checked from the same incoming availability, independently
of the other arm. At the join, both write sets become available because all names
were seeded before the loop and every untaken write retains its own input.
All leaves, including unreachable ones, retain type, ownership and effect checks.
Iteration-local scalar bindings follow the scoped profile below. Constants/parameters
as update targets, unsupported body calls, general inner loops and a second
induction write remain rejected.
This is a bounded multi-statement carry profile, not arbitrary imperative code.

[Sequence execution regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_sequences.rs)
compare registered reference and native CPU execution with an independent wrapping
oracle and actual predicate/iteration/allocation counters. The
[sequence lifecycle fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_sequences_loops.ns)
also crosses the ordinary build, cache, tamper rejection and standalone-restoration
path. These test definitions are not Windows, macOS or GPU execution evidence.


### Iteration-Local Scalar Temporaries

A fresh `let` inside an admitted counted loop may hold i64 or bool, with an exact
annotation or inferred type. Initializers read already-visible values and execute
at their source position on each selected iteration. The compiler retains a value
binding, not an expression substitution: later changes to an initializer's inputs
do not retroactively change the temporary. Nonfallible i64 add/subtract/multiply
use the existing wrapping arithmetic contract.

```text
let index: i64 = 0;
let total: i64 = 0;
while index < limit {
  let index: i64 = index + 1;
  let delta = index * 2;
  let selected = delta < 5 || index == 4;
  let delta: i64 = delta + 100;
  if selected {
    let local = delta - 100;
    let total: i64 = total + local;
  } else {
    let local = 1;
    let total: i64 = total + local;
  }
}
```

The example leaves total at 15 when limit is 4. `selected` observes `delta` before
its later update. Annotation-free bool declarations receive the same guarded
short-circuit normalization as explicit bool declarations. Logical `&&`/`||`
compose at logical edges; equality of bool atoms is supported, but a compound
logical expression hidden beneath bool equality is not admitted by this profile.

Fresh bindings are initialized before entering their lexical scope. A declaration
in one arm cannot initialize another arm or the continuation; even two arms using
the same spelling do not export that name. A variable initialized before a branch
may be updated there when it is an iteration-local i64, bool or exact flat-i64 record.
Rebinding never recomputes an earlier snapshot. New const declarations are not part
of this slice; outer flat-i64 and bool carries follow the separate seeded rules
below. Parameters/constants never gain write authority.

Only pre-existing mutable carries are captured and returned to the outer driver;
iteration locals are never seeded from a preceding iteration. An iteration with
local work but no carried output discards its scalar helper result. Inner branch
helpers may return updated i64/bool/flat-record temporaries to their enclosing iteration without
turning them into driver state. Protected induction, bound and stride still cannot
be changed by the body. The existing no-forward-sibling-read rule also applies to
initializer reads; creating a temporary cannot hide an unavailable carry read.

Both arms and every logical operand are type/availability checked even on zero
trips or unreachable paths. The local expression walk is bounded at depth 64,
independently of the existing 32-guard, native closure, slot and induction limits.
Source admission is not a guarantee of native admission: a 32-level decision tree
can exceed the native call-depth bound after expansion into scoped helpers and
callback frames. The two limits are checked independently; this integration keeps
the explicit rejection rather than increasing the backend budget.
Provider/resource effects and general inner loops remain outside this local-expression
profile. Checked arithmetic and scalar helper calls follow the scoped rules below.

[Temporary execution regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_temporaries.rs)
reuse the independent sequence oracle and observe actual comparison/iteration and
allocation/drop counts. The [temporary lifecycle fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_temporaries_loops.ns)
also crosses ordinary build, registration-specific cache reuse, tamper rejection
and standalone restoration. Test definitions alone do not certify a platform;
Linux results must not be relabeled as Windows, macOS or GPU validation.

### Iteration-Local Bool Rebinding

Typed and inferred bool locals can be reassigned directly or through one-sided,
two-sided and nested branches. A branch snapshots its condition before any writes;
changing that binding in the selected arm cannot activate the other arm. Copies
retain their initialized values. Branch-local bindings do not escape even when both
arms use the same spelling, and every iteration initializes its own locals anew.
Outer bool carries and flat-i64 records use the separate seeded carry rules below.
Parameter/constant mutation remains closed.

The shared outliner retains exact source types. Only private branch return slots
encode bool values as canonical i64 0/1 words through existing YIR conversion
instructions; projected results are restored to bool before the next statement.
Single-value and mixed i64/bool joins use the existing scalar/flat-i64 return
contracts, not a new mixed aggregate ABI. Guard fallback seeds may only encode an
already-available bool parameter, not arbitrary expressions or calls. Buffer's
effect profile and the callback's public four-argument ABI are unchanged.

Logical rebinding RHSs remain lazy, including checked helper arguments. Reached
arithmetic still executes when its result is overwritten or unused. Whole-loop
preflight runs before body work; branch/iteration helpers consume the same shared
entry budget and never reset or refund either execution counter. These conversions
are not a new source-level permission for arbitrary cast expressions in the catalog.

[Native execution regressions](../../tools/nuisc/tests/native_application_bridge/bool_rebinding.rs)
compare independent states and actual checked-call/iteration traces after reversing
YIR declaration order, including successful-path allocation/drop balance, both loop
directions, 1/3/7 carries, skipped invalid arithmetic and selected unused-result traps.
Forged cast types/dependencies/arity reject. The
[budget regression](../../tools/nuisc/tests/native_application_bridge/helper_entries.rs)
checks an exact 16-entry/8-trip composition and independent exhaustion without
callback output. [Default native-entry tests](../../tools/nuisc/tests/control_flow_syntax_native/bool_rebinding.rs)
retain the same value and reached-failure behavior, without importing session budgets.
The [bool-rebinding lifecycle fixture](../../tools/nuisc/tests/native_application_bridge/bool_rebinding_loops.ns)
crosses build/run-artifact, registration-specific cache reuse, tamper rejection and
standalone restoration after removing both the original source and build directory.

### Checked Iteration Expressions

Exact-i64 `/` and `%` may appear in local initializers, direct carry updates,
nested branch bodies and lazy boolean conditions. Shared non-speculation analysis
selects the scoped iteration route even without a fresh local declaration; a
fallible expression is never captured as an eagerly evaluated loop-metadata operand.
The existing guarded branch and boolean helpers keep evaluation at its source
position. Stored results remain snapshots across later writes.

Zero iterations, untaken arms and short-circuited RHSs perform no arithmetic.
Reached zero divisors and `i64::MIN` with `-1` trap for both operators, including
when the result is unused or the iteration has no carried output. A failure on a
later iteration is not hoisted to an earlier one. Header bounds and strides remain
invariant atoms: whole-bound induction preflight rejects invalid strides, overflow
and excessive trips before executing any body expression. Scope, forward-sibling,
exact-type, effect and depth admission still checks unreachable paths.

[Checked-iteration regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_checked.rs)
compare native and registered reference execution with an independent i128 oracle
across branch positions, both directions, signed extremes and 1/3/7 carry widths.
Volatile probes observe actual iterations and traps; allocation/drop balance is
checked on successful paths, not promised after process termination. Reference
event failure retains accepted state and permits close, but native process traps
are not catchable session errors. Ordinary native-entry regressions also cover a
discarded checked expression without carries.
The [checked-iteration lifecycle fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_checked_loops.ns)
crosses build/cache reuse, tamper rejection and standalone artifact restoration.

### Iteration-Local Helper Calls

Local initializers, carry updates and conditions may call helpers with exact i64/bool
or flat-i64 value arguments and results. The loop-enabled source catalog validates
dependencies before callers and consults only completed, admitted callees, never
provisional signatures. Bounded loop-bearing helpers and their transitive wrappers
are eligible; transitive effects, recursion, cycles, nested/resource aggregates,
references, async functions and unresolved generic
signatures cannot gain admission through a hidden call. Declaration order does not
change closure discovery. All arguments retain scope, availability, type, arity and
the NIR expression-depth checks, including on unreachable paths.

Calls select scoped iteration lowering even without a fresh binding or checked
arithmetic in the caller. Recursive expression normalization preserves logical edges
inside arguments, including arguments to i64-returning calls. Inferred bool results
are retained as snapshots; comparisons may consume exact bool call results. An
unselected call does not evaluate its arguments, but a selected call must evaluate
even an argument its callee ignores. No new opcode, callback ABI or runtime dispatcher
is introduced. Bounds/strides remain invariant atoms, and native closure/depth limits
still apply after private helpers are generated.

[Call execution regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_calls.rs)
reuse the independent checked-arithmetic oracle with an ordered operand/iteration
trace. A test-only probe observes real native callee entries, while existing probes
retain whole-bound preflight, state results, successful-path allocation/drop and
process traps. The [call lifecycle fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_calls_loops.ns)
also uses bool arguments with skipped invalid helper calls and crosses ordinary
build/cache/standalone restoration. These probes are correctness, not performance evidence.

The source parser separately permits at most 32 simultaneously active expression
parse entries, counting the outer expression. Deep call arguments and parenthesized
groups now return a diagnostic before exhausting its precedence-parser stack.
Sibling expressions reuse this budget, including after errors. This is distinct from
the scoped NIR expression limit of 64 and the native function-call depth limit of 32;
it is not a claim that all recursive frontend structures have been made stack-safe.

### Iteration-Local Flat Values

Nonempty, nongeneric records containing only exact i64 fields may be constructed,
returned by helpers, copied to fresh local bindings, passed as arguments and projected
inside the iteration. Type identity is nominal, not just a matching field count;
initializers require each declared field exactly once. Field operands keep source
evaluation order even when their order differs from the declaration. Field accesses
and aggregate constructors retain the scoped expression-depth/availability checks.

Aggregate values are immutable snapshots, not extra loop carries. Later scalar
mutations do not change their contents, and branch-local names never escape. Guarded
branches and short-circuit predicates capture already-available aggregates; calls and
field expressions on skipped paths do not run. Reached unused results and ignored
aggregate arguments still evaluate checked arithmetic. Successful execution must
release aggregate allocations; process traps do not promise recoverable cleanup.

The shared effect validator/outliner now takes an explicit Buffer or pure-value type
mode. Capture discovery is shared with ordinary value control flow, while Buffer's
scalar-only helper catalog and effect authority remain unchanged. Local discovery
does not itself authorize outer aggregate writes; the carry rules below do.
Resource or mixed/nested records remain outside this profile. Literal counted
nesting follows the scoped rules below. Native layout/slot and closure limits remain independent of
source layout discovery, including source-only tests with 65-field records.

[Flat-local native regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_local_values.rs)
check independent states, real operand/iteration traces, source-order field evaluation,
snapshots across branch captures, whole-bound preflight and allocation/drop balance.
The [flat-local lifecycle fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_local_values_loops.ns)
also crosses formal build/run-artifact, cache reuse and standalone restoration.

### Iteration-Local Flat Rebinding

Typed and inferred flat-i64 local bindings can now be rebound from literals, existing
snapshots and admitted helper results, including self-references on the RHS. The old
binding is read before replacement. One-sided, two-sided and nested branches carry
only names initialized before that branch; matching arm-local names never escape.
Exact nominal types must agree even when two records have identical field layouts.

The private transport plan expands each record in declared field order into existing
i64 return slots. Bool companions keep canonical 0/1 encoding. Each branch call is
evaluated once; its projections rebuild fresh records with the original nominal type,
not an in-place update to shared storage. Even a one-field record uses this plan.
Source literal fields still evaluate in source order before packing. A leading guard
may project an already-captured flat parameter, but cannot evaluate arbitrary calls,
nested/resource fields or arithmetic as a pass-through default.

Outer aggregate carries follow the separate seeded rules below. Parameter/constant
mutation and resource or mixed/nested payloads remain separate. Literal counted
nesting follows the scoped rules below. Buffer retains its scalar-only
write authority. The public callback ABI, native flat-i64 slot limits, complete
induction preflight and both shared execution budgets are unchanged. Reached results
execute even when overwritten; successful paths release all temporary aggregates.
Process traps still do not promise recoverable cleanup or bounded peak memory.

[Rebinding execution tests](../../tools/nuisc/tests/native_application_bridge/aggregate_rebinding.rs)
compare independent wrapping-value oracles with reference and real native execution
after reversing YIR declarations. They cover widths 1/3/7, both induction directions,
old snapshots, nested/mixed joins, selected field order, skipped invalid arithmetic,
reached overwritten-result traps, preflight and forged layout/projection rejection.
[Source admission tests](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/aggregate_calls_tests.rs)
also exercise 65-field discovery without widening native limits and reject nominal,
scope and constant/parameter-write drift. The [budget probes](../../tools/nuisc/tests/native_application_bridge/helper_entries.rs)
check exact 16-entry/eight-trip work, both branch selections and each insufficient budget.
The [ordinary native entry tests](../../tools/nuisc/tests/control_flow_syntax_native/aggregate_rebinding.rs)
and [lifecycle fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_rebinding_loops.ns)
cover executable entry and build/cache/source-free standalone restoration respectively.

### Outer Flat-I64 Carries

A nonempty, nongeneric record with only exact i64 fields may now cross the loop
backedge when initialized by a mutable local declaration before the loop. Parameters
and constants remain read-only; copying them to a mutable local grants authority to
that local only. Outer bool bindings use the typed word rules below; mixed/nested/resource
records remain outside this slice. Literal counted nesting follows the scoped
rules below. Buffer admission is unchanged.

The iteration normalizer reuses the private flat-i64 transport plan. One-field
records, multiple records and scalar companions use layout-derived slot ranges,
not a finite arity table. Return slots follow lexical first-write order and each
record's declared field order; helper parameters may be captured in a different order.
Each carried argument must supply exactly one same-nominal-type seed. Reconstruction
accepts only complete ordered projections from that one result, not calls, arithmetic,
duplicate/missing fields or a same-shaped substitute type. A record is not a break flag.

The existing scoped-i64-carries contract receives flattened seeds and backedge words;
there is no new loop opcode, provider path or public ABI. Zero-trip loops return the
initialized seeds. Final projections construct new nominal value nodes, preserving
pre-loop snapshots. RHS evaluation and branch joins still follow source order, and
the no-forward-sibling-read rule is retained: an own update can read its old value,
but a sibling carry read requires the earlier update to be available. Fallible
record fields stay inside the selected iteration, including overwritten results.

[Source admission](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/aggregate_carries_tests.rs)
checks widths 1/3/7/65 independently of native slot limits, lexical authority and
nominal/order drift. [Projection tests](../../tools/nuisc/src/lowering/scoped_loop_lowering/scalar_carries_tests.rs)
reject forged reconstruction and seed layouts. [Native probes](../../tools/nuisc/tests/native_application_bridge/aggregate_carries.rs)
compare every record field and snapshot with reference and independent wrapping
oracles, observe checked operands and iteration counts, and check allocation/drop
balance after successful callbacks. They cover callee/inline/prefix/suffix placement,
both directions, zero trips, invalid preflight, overwritten traps and metadata drift.
The shared [field-order probes](../../tools/nuisc/tests/native_application_bridge/aggregate_rebinding.rs)
also run with records carried across iterations. Ordinary native-entry and both
independent native-session work budgets retain their existing policies.

The [aggregate-carries fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_carries_loops.ns)
combines zero-trip and repeated record carries with typed lifecycle callbacks,
checked arithmetic and existing multi-state break/continue. It passes build/cache
and source-free standalone restoration. This is bounded CPU value-state evidence,
not resource-state execution, native GPU dispatch or an allocation-free loop claim.

### Outer Bool Carries

A bool initialized by a mutable local declaration before a counted loop can now
cross its backedge. A single bool, multiple bools, i64 companions and flat-i64 record
companions share the existing private transport. This does not admit mixed-field
records, parameter/constant mutation, Buffer writes or resource state.

The iteration normalizer encodes each bool seed with `CastBoolToI64`, receives an
exact i64 private parameter and decodes it to a fresh bool binding inside the same
helper. Returns encode bools as canonical 0/1 words by construction. Even a singleton
bool uses a one-slot flat return; output projections explicitly decode that slot.
There is no new loop opcode, callback ABI, extra helper-entry boundary or backend
special case. Generated parameter names avoid user and iteration-local bindings.

Seed/projection admission requires the exact conversion around a named bool seed
and its matching result slot, not a raw bool operand, constant or arbitrary call.
Zero trips retain the initialized value; loop exit creates fresh value nodes so
pre-loop snapshots remain unchanged. Slot order follows first writes independently
of capture order. The existing own-old-value and no-forward-sibling-read rules still
apply. Enclosing branch capture traverses the inserted conversions without granting
source-level casts new catalog authority.

[Source admission](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/bool_carries_tests.rs)
checks mutable seeds, name collisions, constants/parameters, type changes and
forward sibling reads. [Projection tests](../../tools/nuisc/src/lowering/scoped_loop_lowering/scalar_carries_tests.rs)
reject missing, duplicate, raw or mistyped seeds and malformed decodes.
[Native probes](../../tools/nuisc/tests/native_application_bridge/bool_carries.rs)
compare independent value/snapshot oracles against reference and real native
execution after reversing YIR declarations. They cover 1/3/7 bool carries mixed
with scalar/record state, callee/inline/prefix/suffix placement, both loop directions,
zero trips, lazy checked operands, overwritten-result traps, full preflight and
successful-path allocation/drop balance. A separate singleton bool test retains
the typed callback state. Forged raw-bool seeds, layouts and cast metadata reject.

[Budget probes](../../tools/nuisc/tests/native_application_bridge/helper_entries.rs)
retain exact 16-entry/eight-trip composition and independently insufficient entry
and loop budgets, without refunds, resets or callback output publication.
[Ordinary native-entry tests](../../tools/nuisc/tests/control_flow_syntax_native/bool_carries.rs)
cover singleton and mixed-value loops without importing native-session budgets.
The [bool-carries fixture](../../tools/nuisc/tests/native_application_bridge/bool_carries_loops.ns)
combines typed lifecycle, record state, checked arithmetic and existing break/continue
helpers through build/cache/source-free standalone restoration. These are CPU
value-state checks, not new GPU/resource execution or allocation-free-loop evidence.

### Literal Nested Counted Loops

Literal `while` bodies can now contain counted child loops using the same scoped
function and private value-transport contracts as loop-bearing helper calls. Each
loop retains a single i64 induction step and invariant atom bound/stride. Leading
and trailing placements use the distinct source-timing rules below.
Child bounds may use an already-available parent index or iteration-local value,
including triangular loops. Child indices may be initialized per parent trip or
persist as seeded outer mutable locals. Counter-only children retain the existing
metadata loop; children with effects use scoped helpers even for a single bool guard.
There is no new loop opcode, public callback ABI or backend-specific normalization.

Write discovery includes descendants, so changing an ancestor induction, bound or
stride is rejected even under an untaken branch. Child admission preserves the
parent's unavailable siblings and resets child-written availability for each child
backedge. Own-old-value and source-ordered writes remain valid; an inner loop cannot
launder a future sibling read. Parameters and constants remain read-only, and child
declarations never escape. Branches and child loops share a source nesting cap of
32, independently of expression and native call-graph limits.

Every selected invocation performs its own complete induction preflight before its
body, including invocations reached after earlier parent iterations. Zero trips and
unselected children do no body work. Both work counters remain callback-owned:
each selected loop reserves its full trip count, while helpers debit actual entries.
Neither counter resets at nesting boundaries; insufficient budgets trap without
publishing callback output. Ordinary native entry does not inherit session budgets.

[Source checks](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/literal_loops_tests.rs)
cover lexical scope, descendant writes, forward reads, exact types and mixed
branch/loop depth. [Native oracles](../../tools/nuisc/tests/native_application_bridge/literal_loops.rs)
compare real execution and reference results after reversing YIR declarations,
covering two-to-four total loop levels, both directions, rectangular/triangular
bounds, bool/flat-record snapshots, wrapping values, selected/overwritten arithmetic
traps, skipped invalid children and late child preflight failure. Volatile probes
observe actual iterations and checked-call operands; successful paths balance
aggregate allocation/drop. [Budget tests](../../tools/nuisc/tests/native_application_bridge/literal_loops_budget.rs)
cover exact eight-trip/16-entry composition, independent exhaustion, counter-only
children, persistent child indices and single-bool guards. [Ordinary entry](../../tools/nuisc/tests/control_flow_syntax_native/literal_loops.rs)
retains local and persistent child indices. The [literal-loops fixture](../../tools/nuisc/tests/native_application_bridge/literal_loops.ns)
composes typed lifecycle, checked arithmetic and existing exit-bearing helpers through
build/cache/source-free standalone restoration.

Guarded exits in these leading-step bodies now use the contract below. Resource and
mixed/nested payloads, Buffer authority, allocation elimination, native preemption
and device execution are not widened.

### Leading-Step Value Loop Exits

Pure-value counted loops now admit direct and guarded `break`/`continue`, including
literal child loops. Each loop owns its exit scope: a child exit cannot terminate
its parent or suppress the parent's suffix. Prefix writes remain source ordered;
the selected exit skips the rest of its own iteration without evaluating skipped
arguments or checked arithmetic. The strict single leading i64 step remains required;
an extra induction write or a statement after a direct exit in the same arm rejects.

The [normalizer](../../tools/nuisc/src/lowering/buffer_loop_outline/control_flow.rs)
reuses existing flag guards, with a separate leading-step mode. It does not require
another step before `continue`. On `break`, the shared driver's tail step is suppressed,
but the source step has already run. A [private recovery carry](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/exits.rs)
holds that advanced value and restores the visible induction after the driver exits.
Its entry seed preserves zero-trip identity; fresh names reserve future source
bindings as well. The canonical break flag stays last and is seeded with zero,
using existing `scoped_call_i64_carries_break` admission. No new opcode, public ABI,
backend rule or Buffer exit behavior is introduced.

Ordinary `main` loops still use the allocation-free flow path when the existing
shared flow parser admits them; `main` is not a registered native-session callback.
Callable value loops use the scoped contract, and complex ordinary-entry loops can
use that same fallback. Only the admitted native-session helper path now removes
per-return allocation; the ordinary LLVM fallback remains separate.

Generated suffix guards also count toward the value-profile depth limit of 32.
Source write checks retain readonly parameters, constants, protected headers and
forward-sibling rejection; normalization does not grant write authority. Full nonwrapping
induction preflight remains mandatory even for immediate break. Each selected loop
reserves its full bound without refund, while the separate helper counter charges
only actual entries. Neither failure publishes callback output.

[Source checks](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/exits_tests.rs)
cover local control, canonical transport, name collisions and rejected writes/depth.
[Native/reference oracles](../../tools/nuisc/tests/native_application_bridge/value_loop_exits.rs)
check both directions, zero trips, bool/record snapshots, parent/child exits, skipped
and overwritten arithmetic failures, late preflight failures, volatile operand and
iteration traces, and successful-path allocation/drop balance.
[Exact budget probes](../../tools/nuisc/tests/native_application_bridge/value_loop_exit_budgets.rs)
check full reservations, actual entry debits and independent exhaustion without refund.
[Ordinary entry](../../tools/nuisc/tests/control_flow_syntax_native/value_loop_exits.rs)
retains advanced indices without acquiring native-session budgets. The
[value-loop-exits fixture](../../tools/nuisc/tests/native_application_bridge/value_loop_exits.ns)
combines these controls with typed lifecycle state through build/cache/tamper rejection
and standalone restoration after source deletion.

### Trailing-Step Value Loops

Pure-value loops also admit a single trailing i64 induction step. The shared
[induction parser](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/induction.rs)
identifies the source placement before validation and outlining. The iteration
helper observes the pre-step index; only a successful driver backedge advances it.
`break` therefore retains the pre-step index and needs no advanced-index recovery.
Even a break-only body uses the existing canonical projected flag transport.
Leading and trailing child loops can mix; each child's exit and recovery remain local.

A trailing `continue` must be immediately preceded by the identical explicit
induction step in its own arm. Normalization removes only that duplicate, allowing
the driver to execute the one source-equivalent step. An unstepped continue, a
different step, a second induction mutation or effects between step and continue
remain rejected by this bounded value profile. This is not a language-wide ban on
continue: the profile cannot certify those bodies with its counted preflight.
Generated suffix guards retain the 32-level bound. Header/write authority and
source-order checks still apply to untaken branches and descendant loops.

Value-admitted functions bypass the separate Buffer outliner, preventing duplicate
rewriting of tail-step break bodies. The ordinary entry retains existing counted
and flow fast paths, including prefixed step temporaries; callable value functions
use scoped helpers without new opcodes, target-specific rules or callback ABI.
Full induction preflight and reservation still apply to immediate exits. Break
does not refund unentered iterations, including when a following loop requests work;
continue does not invent another iteration or helper debit. Native flat helper
returns and typed callback State now use value transport without aggregate heap storage.

[Source tests](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/trailing_tests.rs)
cover pre-step effects, exact continue steps, flag-only break, header authority,
forward reads, mixed exits and generated guard depth.
[Native/reference probes](../../tools/nuisc/tests/native_application_bridge/trailing_value_loops.rs)
check both directions, nonunit strides, mixed child placements, bool/record snapshots,
checked operand order, skipped/overwritten failures and full/late preflight.
Successful executions balance aggregate allocation/drop.
[Budget probes](../../tools/nuisc/tests/native_application_bridge/trailing_value_loop_budgets.rs)
check independent exhaustion, repeated invocation, flag-only control and no refunds.
[Ordinary-entry regressions](../../tools/nuisc/tests/control_flow_syntax_native/trailing_value_loops.rs)
check break indices and mixed scopes. The
[trailing-value-loops fixture](../../tools/nuisc/tests/native_application_bridge/trailing_value_loops.ns)
adds build/cache/tamper rejection and source-free standalone restoration.

### Returns From Counted Value Loops

Counted pure-value helpers now admit `return` inside leading- or trailing-step
loops, including child loops and conditional arms. Return types remain exact i64,
bool or nonempty nominal flat-i64 records. This is not admission for resource
returns, mixed/nested records, arbitrary loops or the separate Buffer profile.

[Return normalization](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/returns.rs)
seeds a private typed payload and canonical pending word before the source body.
The return expression executes at its source location before publishing that word.
Each containing loop then exits through its existing scoped-break contract; an
outer function guard returns the saved payload. A child return exits the entire
function, unlike an ordinary child break. Skipped suffixes, later child preflights
and trailing steps are not evaluated. Leading-step indices are already advanced;
zero trips leave the pending bit clear and retain the original fallback return.

Admission validates the same normalized body that outlining consumes, preserving
initialized-local write authority, exact nominal types and source-ordered reads.
Generated names reserve parameters, future bindings and references. The frontend
rejects unknown source names; normalization must not repair malformed NIR by
accidentally binding one to a generated slot. Source nesting and
generated suffix guards remain bounded; a direct return must terminate its source
arm (apart from the canonical trailing induction step). Wrong/void return values,
effects, writes to readonly/header state and hidden extra induction writes fail
closed. No new native opcode, callback ABI, resource authority or global state is
introduced.

Entered loops still preflight their complete bound and reserve all trips. Returns
do not refund work, and all entered helpers, including unselected guard helpers,
consume the same callback-owned entry budget. A failing return expression or
budget exhaustion leaves callback output unpublished; this is a process trap,
not a recoverable callback error or a whole-program memory-safety claim.

[Source admission tests](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/returns_tests.rs),
[native/reference oracles](../../tools/nuisc/tests/native_application_bridge/counted_returns.rs)
and [ordinary-entry tests](../../tools/nuisc/tests/control_flow_syntax_native/counted_returns.rs)
cover exact types, parent/child propagation, both directions and step placements,
snapshots, zero trips, lazy failure and full induction preflight. Successful native
paths balance aggregate allocations/drops.
[Budget probes](../../tools/nuisc/tests/native_application_bridge/counted_return_budgets.rs)
check repeated callbacks, shared reservations, helper exhaustion and unchanged
output sentinels for division by zero and signed overflow. The
[counted-returns fixture](../../tools/nuisc/tests/native_application_bridge/counted_returns.ns)
adds build/cache, tamper rejection and source-free standalone restoration.

Private continuation/guard expansion still consumes the unchanged 64-function
native graph budget. An initial fixture combining the previous deeply branched
body with the new returns exceeded that limit. The dedicated counted-returns
fixture removed redundant control rather than raising it; the full composition
is now restored by the ready-value optimization below.

### Ready-Value Control Elision

The [scalar-control outliner](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs)
omits private helpers for integer/bool literals and already-initialized variables
of the admitted scalar or flat-i64 record type. A terminal ready-value return needs
no continuation; a branch returning such a value, or an empty arm forwarding it,
needs no guard helper. The condition is still evaluated exactly once, including
when both arms return the same value. Admission still rejects unknown variables,
type/layout drift and unsupported effects before this transformation.

Calls, arithmetic, projections and record constructors are not ready atoms. They
stay behind their original guards. Multi-use nontrivial suffixes remain shared, preserving
linear helper growth instead of duplicating source computation. Unused source
calls and fallible predicates still execute. Ready-record selection reuses values;
helper allocation removal is handled by the separate native value transport below.

The [full control-composition fixture](../../tools/nuisc/tests/native_application_bridge/control_composition.ns)
combines counted returns with the original nested, branched, typed-carry workload.
It previously failed the 64-function limit; ready-value elision first reduced it
to 57 reachable native functions, terminal elision reduced it to 54, and statement
suffix folding below now reduces it to 51.
[The regression](../../tools/nuisc/tests/native_application_bridge/control_composition.rs)
requires at most 51 and compares native execution with reference lifecycle states;
the frontdoor suite also exercises build/cache/tamper rejection and source-free
standalone restoration for this unshortened source.

[Exact entry probes](../../tools/nuisc/tests/native_application_bridge/control_elision_budgets.rs)
check one-time/fallible predicates, equal arms, argument-before-callee ordering,
typed bool/record snapshots and untouched callback output on exhaustion. Eliminated
synthetic functions no longer consume entries; every remaining actual entry still
does. The loop reservation policy, both default budgets, graph/node/depth limits,
public ABI and native opcodes are unchanged. This is a size/correctness improvement,
not a measured runtime-speed or peak-memory result.

### Single-Use Terminal Continuations

A suffix consisting of one `return expression` can now lose its private forwarding
function when exactly one generated continuation use reaches it. Calls, arithmetic,
projections and flat-record constructors still execute behind the selected branch's
leading guard, not before the condition or as speculative `select` operands.
Argument and field evaluation order, old snapshots and reached unused calls remain
unchanged. No source call is removed by this transformation.

The outliner counts uses iteratively, saturating after the second use. An explicit
return consumes no inherited continuation. Terminal branch fallthroughs contribute
separate uses, but a nonempty suffix is emitted once regardless of how many runtime
paths reach it. This models generated code rather than runtime path counts. Two or
more uses still share a continuation; nontrivial expressions are not duplicated and
the existing linear helper-growth property is retained. Multi-statement suffixes
use the additional scope-preserving rule below.

[Unit regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_tests.rs)
cover typed terminals, nested fallthroughs, shared suffixes and a 32-return chain
with only one copy of the final calculation.
[Native budget probes](../../tools/nuisc/tests/native_application_bridge/terminal_continuation_budgets.rs)
check bool/record results, source-ordered fields, lazy failure, unused prefixes,
shared suffixes and unchanged output sentinels on exhaustion. The
[ordinary CLI regression](../../tools/nuisc/tests/control_flow_syntax_native/terminal_continuations.rs)
covers typed results and selected arithmetic traps without callback counters.
Terminal folding alone reached 54 reachable native functions instead of 57. Remaining
entries, including neutral guards, still cost one; loop reservations are unchanged.
There is no graph-limit, public ABI, opcode or aggregate-allocation policy change.

### Single-Use Statement Continuations

A multi-statement suffix with one generated continuation use can now move into
that use with disjoint bindings or the scope-aware local hygiene below. The outliner moves
the owned statement sequence rather than copying it across runtime paths. It uses
the same iterative boundary walk as terminal-use counting: an existing nonempty
inner suffix is emitted once, while terminal fallthroughs contribute separate uses.
Shared inner suffixes remain shared, and all moved work stays after the selected
guard. Reached unused calls, arguments, predicates and source ordering are retained.

Branch-local declarations do not escape in source. The initial optimization kept a
separate function when an arm binding collided with a suffix binding.
Scope-colliding single-use continuations now use explicit lexical renaming, rather than merging
unrelated bindings. Multiple uses still retain one shared body.

The unit return-chain regression drops from 63 to 32 private helpers while retaining
one copy of the final calculation. The full control-composition fixture
now emits 51 reachable native functions, down from 54; its native and frontdoor
regressions require at most 51, including byte-identical restored LLVM. Exact entry
probes check typed bool/record snapshots, unused fallible calls, shared inner work,
argument-before-callee entry and unchanged output sentinels. The unused-result
fixture now needs eight entries rather than nine, and still fails with seven.
Ordinary CLI tests cover the same scope, typed-value and lazy-failure boundaries.

The optimization removes forwarding functions only. Remaining real entries still
cost one; selected loops keep full reservations. Native graph/node/depth bounds,
public ABI, aggregate allocation and process-trap semantics are unchanged. These
are code-size and correctness results, not a runtime-speed or peak-memory claim.

### Hygienic Continuation Bindings

The [local-hygiene pass](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_hygiene.rs)
renames only arm-local declarations that collide with the moved suffix. Each
binding receives a fresh private identity reserved against all source parameters
and declarations, including future locals. Statement and expression walks are
iterative; each branch and loop inherits its own scope snapshot. Binding
initializers retain their original scope before a new identity is introduced.
Only already-admitted scalar bodies reach this pass; it does not grant general
source shadowing or make out-of-scope references valid.

Existing bindings are updates, not new declarations: loop writes keep the same
identity as their seed, and outer bindings are never renamed merely because both
the branch and suffix update them. Normalized bool-word conversions and flat-record
projections follow variable references; field labels, nominal types, function
symbols and statement order do not change. Private scoped iteration functions
retain their own parameter names and receive the renamed caller values normally.

The same-name record/scalar probe now needs five entries instead of six, with an
untouched output sentinel when four entries are insufficient. Native probes also
cover renamed local i64/bool/record loop carries, unrenamed outer carries, zero-trip
skipping, entry exhaustion and independent full-loop reservations. Unit and ordinary
CLI tests preserve sibling constants of different types, early returns, unused
fallible work, matching variable/field/function names and future private-like names.

The full composition now includes a record prefix named like its subsequent scalar
binding. Its 51-function bound, typed lifecycle totals, cache validation and exact
source-free LLVM restoration remain required. Shared multi-use bodies, admission
limits, callback ABI, lazy failure and source evaluation order are unchanged.
The subsequent per-return flat-i64 aggregate allocation change is confined to the
native helper transport below; it does not imply measured runtime speed.

### Native Flat Value Transport

Admitted flat-i64 helper returns use private LLVM `[N x i64]` values for 1..64
fields, including scoped iteration results. The earlier nested-loop probe fell
from 13 allocations to 1; typed callback transport removed the last allocation.
Selection happens after native admission, with separate helper and callback checks.
The [value-return contract](nuis-native-scalar-value-returns-v1.md) records layout,
snapshot and ordinary-ABI boundaries. This is not measured runtime speed.

### Native Callback Value Transport

Registered roots use the shared bounded scalar schema and publish all result slots
only after the callback returns. Exact nominal/kind checks, bit-preserving packing,
unaligned overlap, invalid-input sentinels and zero aggregate allocation remain
covered by the [callback probes](../../tools/nuisc/tests/native_application_bridge/callback_values.rs).
See the [value-return contract](nuis-native-scalar-value-returns-v1.md) for details.

### Native Typed Helper Returns

Ordinary native helpers now separately admit nonempty mixed/nested scalar records,
with exact call/result layouts and 1..64 scalar leaves. Source-flattened arguments,
helper graph admission and scoped flat-i64 carry schemas remain separate checks.
The [typed helper probes](../../tools/nuisc/tests/native_application_bridge/typed_helper_values.rs)
cover nested relay calls, callback-as-helper calls and independent snapshots;
[guard probes](../../tools/nuisc/tests/native_application_bridge/typed_helper_guards.rs)
cover lazy division, shared entry exhaustion and unchanged output sentinels.
This does not widen guarded local rebinding, resource values or provider callbacks.

### Reference State Normalization

The host binds a recursive shape from registered field paths and scalar kinds,
then validates all declared callback layouts when present, including nested nominal
identity. Once a callback completes, named fields are matched and reordered into
the registered layout before state publication. Duplicate/missing/extra fields,
wrong scalar kinds, wrong nesting and known nominal mismatches remain errors.
Invalid results retain the last accepted state; event failure permits one cleanup,
and close failure remains terminal. Normalization never retries callback effects.
NaN payloads and signed-zero bits are preserved without numeric conversion.

Metadata-free hand-authored YIR keeps its signature-only nested type boundary;
no missing nominal identity is invented. Reference states retain support beyond
the native 64-slot limit, while field-path nesting is bounded to 64 levels.
This is host-side value normalization, not allocation-free reference execution.
Constructor lowering also chains effectful field expressions in source order,
independently of declared storage order. It gates each emitted field subgraph's
roots, including nested call arguments, after the previous field completes.
Inline and outlined helper regressions check nested reverse constructors under
reordered YIR nodes/functions/bodies.
[Lifecycle regressions](../../tools/nuisc/tests/native_application_bridge/reference_state.rs)
cover nominal drift, rejected returned state, close-once and fuel exhaustion.

### Guarded Local Values

Effectful parents can now contain matching local i64/bool/flat-i64 selections,
including nested choices. The compiler extracts only admitted pure value arms
into private typed helpers, then applies the existing guarded scalar normalizer.
The outer predicate executes once; only ready captured values cross the boundary.
Unselected division, remainder and calls do not execute. Reached unused arithmetic
still fails, and same-name rebinding reads the previous value before replacement.
Single-sided choices keep an outer seed or discard a branch-local result after dead-arm pruning.
Effectful arms and resource-bearing values remain outside this extraction; the
speculative-failure rejection is unchanged. This repairs the image showcase's
`window_event` local selection without changing its Nuis source or provider ABI.

### Loop-Bearing Iteration Calls

The value catalog uses a dependency queue rather than a fixed two-pass profile.
Only a successfully validated callee releases its callers; invalid leaves, unknown
names and recursive components remain excluded. Source-only regression covers a
2048-level acyclic loop-helper chain in both declaration orders. This tests catalog
discovery, not native acceptance at that depth: generated helper nodes still count
toward the separate native function, node and call-depth budgets. Buffer's catalog
continues to exclude loop-bearing and aggregate helpers.
Dependency collection also uses explicit stacks for statements and expressions,
because collection now precedes body admission. A synthetic deep-structure test
checks this traversal separately, without claiming arbitrary source-depth admission.

Calls preserve exact i64/bool/flat-i64 types and lexical snapshots. Arguments execute
in source order before entering the callee. Each selected invocation preflights its
own full induction before entering its loop body, including when the result is
discarded. A skipped branch or zero-trip caller does not enter the callee or check its
induction. A later invocation may fail after earlier caller iterations have completed;
those earlier operations are not rolled back. Reached checked arithmetic remains a
process trap, not a catchable session error.

Ordinary native-entry regressions check result and arithmetic-failure preservation,
not this profile's bounded induction policy. General CLI loops do not acquire the
native-session trip limit simply by calling a loop-bearing helper. Both native test
runners use deadlines to prevent a bad negative fixture from hanging the test suite.

[Loop-call execution regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_loop_calls.rs)
compare native and reference results against an independent nested-loop oracle, with
real callee/iteration traces, success-path allocation/drop balance and selected-path
traps. They cover both directions, multiple call layers, scalar/record results,
logical/branch skipping, changing child strides and argument failure before child
preflight. The [loop-call lifecycle fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_loop_calls_loops.ns)
scans divisors in a helper called from another loop and crosses build/run-artifact,
cache reuse and standalone restoration.

Per-invocation induction limits alone are not a whole-callback work budget. The
additional reservation policy below bounds their sum across selected synchronous
calls, without broadening the iteration-helper source catalog. In particular,
guarded-break helpers still have ordinary-call evidence, not admission as arbitrary
iteration callees.

### Shared Callback Loop-Work Reservations

Each newly emitted exported callback allocates a fresh stack-owned i64 counter.
Lowering forwards its pointer as a private synchronous helper parameter, including
scoped iteration calls and transitive wrappers. Helpers never reset it. The exported
four-argument scalar-slot ABI, YIR function slots and shared transport contract are
unchanged. No global or thread-local counter is introduced, and the stack context
cannot be captured by a generated deferred-task invoker. This profile still excludes
async tasks and provider effects.

`emit_registered` uses `DEFAULT_LOOP_WORK_LIMIT = 1_048_576` per callback invocation.
The producer API `emit_registered_with_loop_work_limit` bakes an explicit u64 limit
into the generated LLVM for direct bridge consumers; there is no CLI budget flag.
The formal native-session package profile verifies the current default policy,
so custom-limit LLVM is not interchangeable with that profile's checkpoint.
A zero loop limit permits loop-free,
unselected and zero-trip paths, subject to the separate function-entry limit below.
Each open, event and close invocation owns a new
budget, rather than sharing one for the whole application session.

For each selected loop, the existing finite/non-wrapping induction check and
65,536 per-loop limit run first. The loop then reserves its **complete induction
trip count** before any body work. Both constant and runtime induction use the same
unsigned compare-before-subtract operation. Argument evaluation still precedes
callee admission; invalid induction does not debit the budget. Early `break` and
`continue` do not refund reservations. A skipped child or zero-trip loop consumes
nothing; repeated child calls reserve repeatedly against the caller's remaining
counter. This conservative policy can reject work whose actual early-exit iteration
count would fit. It is not instruction-by-instruction fuel.

Exhaustion executes a native process trap before the rejected loop body. Earlier
completed operations are not rolled back, but the failing callback does not publish
its output state. The host may retain previously accepted states. A trap is not a
catchable callback error, reference-fuel exhaustion or cooperative cancellation.
Reservation bounds alone do not bound loop-free function fanout. The independent
helper-entry policy below addresses that gap, not total node execution, aggregate
memory, elapsed time, FFI/device work or preemption.

[Native reservation probes](../../tools/nuisc/tests/native_application_bridge/loop_work.rs)
observe real debits, argument order, body counts, process traps and untouched output
sentinels. They cover exact/insufficient/zero/u64-max budgets, constant/dynamic loops,
ordinary and scoped calls, early exits, lifecycle resets and a reentrant transport
probe. Reentry is test instrumentation, not recursive Nuis admission or a thread
stress certification. The [loop-work fixture](../../tools/nuisc/tests/native_application_bridge/loop_work.ns)
and [frontdoor regression](../../tools/nuis/tests/native_session_workflow/loop_work.rs)
execute the default 1,048,576 reservation boundary through build/run-artifact, cache
reuse and standalone materialization after deleting the source project files.
Budget exhaustion on an event does not publish an additional state or completion.

The budget is embedded in the LLVM already bound to the artifact identity. Older
compiled artifacts are not retroactively budgeted; rebuild them to adopt this
producer policy. Existing executable instructions do not change, while the current
artifact verifier rejects stale or rehashed budget-changed LLVM checkpoints instead
of silently injecting a counter. Compile-cache restoration revalidates the checkpoint
and rebuilds mismatches. General CLI LLVM emission does not gain a hidden budget parameter
or the native-session loop limit. Its invariant add/multiply carry regression also
guards the ordinary chain-emitter fix found while constructing these probes.
That repair lowers existing `add_invariant`/`mul_invariant` YIR payloads; the native
entry regression uses literal invariants. Direct source variable-invariant carry
normalization and admission of richer carry payloads in this native-session profile
remain separate boundaries.

### Shared Callback Helper-Entry Accounting

Shared callback-wide helper-entry accounting for loop-free fanout now accompanies
loop reservations. Each exported invocation owns a second stack i64 counter, with
`DEFAULT_HELPER_ENTRY_LIMIT = 1_048_576`. The private context travels through the same
ordinary and scoped synchronous call paths; there is no global/TLS state or reset
inside a helper. The external four-argument callback ABI and YIR parameter slots
remain unchanged. Task invokers cannot capture either stack counter.

Every **actual admitted YIR function entry** consumes one unit before parameter
conversion or body work. This includes lifecycle roots, user helpers, iteration
helpers, guarded branches and continuations. An outlined guard that immediately
returns a neutral result still costs one entry. This is not a count of source calls,
LLVM instructions or post-inlining machine calls, and outlining changes may change
the charge. Calls not entered consume no entries. Caller-side argument evaluation
has already happened; argument helpers themselves consume entries in source order.
Returned or unused results do not refund entries. Reentrant exported invocations
receive independent counters, while nested synchronous calls share their caller's.

The producer API `emit_registered_with_work_limits` accepts independent u64 loop
and entry limits. Existing `emit_registered_with_loop_work_limit` retains the default
entry limit. Zero entry allowance rejects even a loop-free root, unlike zero loop
allowance. Invalid shape and noncanonical transport still return statuses 1/2 before
root entry, even with zero budgets. Unsigned compare-before-subtract prevents wrap.
No CLI override is added; the formal package still verifies the current defaults.

Exhaustion traps before the rejected function body and publishes no callback output.
Earlier argument evaluation and completed calls are not rolled back. Loop induction
admission remains independent and runs after entry to its containing function;
invalid induction does not debit loop reservations. A rejected child function has
not yet reserved its own loops. Neither budget promises native cancellation,
elapsed-time bounds, peak-memory bounds or instruction/node-level accounting.

The shared scalar-control outliner now preserves conditional calls in its admitted
i64/bool/flat-i64 catalog even without loops or checked arithmetic. Pure calls may
expand into substantial work: evaluating both arms before a `select` is not an
acceptable replacement for selecting which call to execute. Existing guarded
continuations keep argument work and callees behind the source decision. Their own
entries remain charged, including neutral guard returns; this does not expand the
catalog to arbitrary types, effects or control flow. Ordinary CLI emission receives
this control-flow correction, but not the native-session counters or limits.

[Entry probes](../../tools/nuisc/tests/native_application_bridge/helper_entries.rs)
exercise exact/insufficient/zero/u64-max limits, root-only and ignored-result calls,
argument ordering, skipped business calls, neutral guards, scoped iterations,
independent induction/loop debits and lifecycle reset. The reentrant loop-work probe
now checks isolation of both counters. The [frontdoor fanout regression](../../tools/nuis/tests/native_session_workflow/helper_entries.rs)
uses an acyclic, loop-free binary call tree: depth-18 work succeeds, while depth-19
work exceeds the shared default once lifecycle/wrapper entries are included. Build,
cache reuse and standalone materialization retain both the successful states and
failure-before-publication behavior. Source-predicate probes exclude budget checks
by operand provenance, not by removing all unsigned comparisons or weakening the
independent short-circuit oracle. Rehashing modified entry limits does not bypass
the bound LLVM checkpoint verifier. These are CPU proofs, not provider scheduling
or thread-stress certification.

### Metadata And Closure Limits

Native conditional chains reuse the existing shared metadata parser and LLVM
emitter, but admit only bounded trees of pure comparisons and nonfallible add/multiply/keep
branches. Every seed and every predicate RHS requires exact i64, even on zero trips
or an unreachable condition leaf. Native admission limits each condition to
127 tree nodes and depth 32, with at most 64 carries and a total argument bound.
An iterative shape scan precedes the native entry's recursive parser; hidden
payloads, forward state and arbitrary ignored `always` tokens reject. Both the
canonical `always` encoding and the compiler's induction-seed placeholder remain
supported. No opcode or ABI is added. The reference CPU routes these chains through
the existing cooperative scalar driver; this is not native preemption.
General YIR verification runs before native admission. Its shared conditional
parser independently rejects depth above 256, so native-only bounds cannot leave
the earlier verifier exposed. These are parser/profile budgets, not limits copied
into the source value catalog or commitments to a stable ABI.

The acyclic catalog propagates loop presence from callees to callers, retaining
guarded lowering even without division/remainder. The reached branch performs
preflight before the first iteration; an unselected branch performs neither.
Inline arms and shared suffixes preserve the resulting induction and carry values, while
source-prefix work still runs before a later branch. Carry arithmetic wraps; scalar
i64/i32 add/subtract/multiply and i64 negation in the reference CPU now explicitly
wrap too, instead of depending on Rust debug overflow checks. Division/remainder
remain checked, and induction still requires finite, non-wrapping progress.
This is not full aggregate
control flow, a whole-language effect proof or a new runtime loop dispatcher.

Native flat neutral returns use the same value transport as selected returns.
The helper entry is still charged; this is not zero-cost branching. Ordinary LLVM
retains owned storage; native callback State uses typed values. Trap cleanup remains unpromised.

Native failure terminates the process, not a catchable callback result or fuel
error. It cannot promise cleanup, rollback or a returned state. The reference
session instead reports an arithmetic error, preserves its last accepted state
and retains failure through close. These are distinct failure mechanisms.

## Counted Loops

The profile consumes existing `cpu.loop_while_i64`, `cpu.loop_while_i64_chain`
and `cpu.loop_while_scalar_chain` nodes. For direct constant i64 start, limit and
add/sub step nodes, admission proves termination in wide arithmetic before
emission. Runtime i64 inputs instead receive an O(1) LLVM preflight at the reached
loop site, before the first iteration. The ordinary loop emitter stays unchanged;
general LLVM inspection and reference execution do not acquire this native policy.

Both routes support `eq/ne/lt/le/gt/ge`. Zero-trip loops preserve their seeds,
including inactive zero steps. Active zero steps, wrong directions, unreachable
non-wrapping equality exits, induction overflow and more than 65536 trips reject.
Constants reject during emission; invalid dynamic values execute `llvm.trap`.
That process failure is not an ABI status, catchable callback error or reference
fuel exhaustion, and cannot promise cleanup or recover the process. Earlier
callback work is not rolled back. A prior guarded return bypasses the preflight.

The preflight uses unsigned i64 quotient/remainder with a safe nonzero divisor,
plus i128 distance, step and final-counter arithmetic. It handles `i64::MIN`
subtraction and distances across the entire signed range without signed overflow
or division-by-zero UB. No i128 division runtime helper, loop interpretation or
new opcode is introduced. Constant proofs require no dynamic guard.

Carries have exact i64 seeds. Add/multiply may read the old/new counter, any
previous-iteration carry, or an already-updated sibling carry. Future sibling
reads, implicit float/bool/i32 conversion and payload-bearing carry operations
are not admitted. LLVM reuses the ordinary loop lowering; carry arithmetic wraps
as i64 and no per-iteration owned aggregate is allocated for this flat chain.
Scalar helper calls may surround loops, including in nested helpers, or become
the admitted scoped body action described below. Other conditional/effectful
loop forms remain separate work.

The CPU module reuses its cooperative registered driver for plain i64 chains,
publishes `LoopState` only at completion and shares reference execution fuel.
No CPU loop dispatch is added to the generic executor or application host. This
per-loop native bound is not whole-callback fuel, native preemption or a wall-time
guarantee; native budgeted calls still reject before entry.

### Scoped Calls

The existing `cpu.loop_while_i64_effect` opcode is admitted only when its action
is `cpu.scoped_call`, `cpu.scoped_call_i64_carry`, `cpu.scoped_call_i64_carries`
or `cpu.scoped_call_i64_carries_break`.
This is not blanket admission
of effect metadata. The first form discards a scalar return; the second consumes
the shared `parse_scoped_i64_carry` contract with one named i64 seed and exactly
one `$carry` operand. Neither form introduces an interpreter or new runtime ABI.

Each body target must be a scalar-returning helper or the checked multi-carry
helper described below, in the same admitted closure.
It participates in cycle, depth, function-count and total-node checks, including
when a loop is statically zero-trip or its result is discarded. Hidden printing,
resource access, unknown/drifted signatures and cross-function value captures
reject before emission. The ordinary loop emitter and CPU registered execution
driver are reused, not copied into the application host.

`$current` is the counter before its step; `$carry` is the preceding return or
initial seed. Both require exact i64 helper parameters. Named captures support
`bool/i32/i64/f32/f64` without coercion, including source lowering of a single-i64
carry with i32/float captures. Calls complete, update the carry, then step the
counter. Zero-trip loops preserve the seed and do not invoke the helper. The
native induction preflight runs before the first invocation; invalid values trap
the process. Pure scalar carry transport adds no owned aggregate per iteration.

Multi-carry calls use `parse_scoped_i64_carries`, not a new opcode or ABI. Each
`$owned_struct_carry:N:seed` binds exactly one named i64 seed to the corresponding
flat `carryN:i64` field. The declared helper result must be owned and match the
action's nominal type, field order and kinds; parameters remain exact scalar
values. At most 64 slots are allowed, within the existing 64-parameter bound.
Operand order need not equal field order. Scalar i32/float captures are also
supported by the source multi-carry path, without widening carried state kinds.

The helper sees the previous iteration's carries and pre-step counter. Its own
ordered updates produce one returned aggregate; native flat leaves are extracted
from an LLVM value before stepping or invoking it again, with no per-iteration
heap owner. Ordinary LLVM still unpacks/drops owned storage. Zero-trip loops
preserve every seed. Typed callback State now also avoids aggregate heap storage;
this is not a claim of whole-program allocation freedom or memory safety.

The explicit source shape is a scoped helper call, ordered scalar projections
and a counted induction step. Source guarded breaks can instead use the private
normalization described below. Unrestricted aggregate calls, move/copy/resource
captures and arbitrary conditional loop bodies are not admitted.

### Guarded Break

`scoped_call_i64_carries_break` reuses the same parser, typed layout, bounded helper
closure and aggregate loop emitter. The last i64 slot is private control: its seed
must be zero even on zero trips, and each returned value must be 0 (advance) or 1
(break). A control-only layout is valid and the control slot counts toward the
64-slot/parameter bounds. It is not an arbitrary user-data or boolean carry.

The returned aggregate is unpacked (and owned storage released on ordinary LLVM),
then the control value is checked
before committing any returned carries. A valid return commits every carry; 1 exits
with the current pre-step counter, while 0 reaches the ordinary induction step.
An invalid seed/return traps the process and never publishes callback output.
This is not a catchable callback status or reference-fuel error.

Native admission still requires the **entire induction sequence**, ignoring any
possible early break, to be finite, non-wrapping and within 65536 iterations. An
immediate break does not excuse an active zero step, overflow or excessive bound.
Constant proofs and reached-loop runtime preflight are unchanged.

Pure scalar source loops now reuse the existing effect-loop normalizer for strict
ascending/descending unit steps and i64/bool helper composition. Break-only loops
use one private bit instead of two complementary flags. This avoids an unnecessary
aggregate branch call while preserving selected-path suffix evaluation and nested
loop scope. The compiler still requires generated control provenance; a user
aggregate followed by `if signal == 1 { break; }` is not automatically trusted.
YIR scoped captures retain all five exact scalar kinds, independently of this more
restricted automatic source normalizer.

Branches that update multiple values, including a user carry plus the break bit,
and mixed break/continue normalization can generate ordinary aggregate helper
calls. The checked flat-i64 subset below now admits these calls. A continue still
requires its matching explicit unit step; step-before-break remains unsupported.

### Flat Helper Returns

Ordinary `cpu.call_owned_struct` and scoped multi-carry calls share one native
return-layout validator. The helper must return an owned nominal aggregate with
1..64 uniquely named, ordered i64 fields. Ordinary helpers use their declared field
names; only scoped multi-carry payloads require `carry0` through `carryN`, checked
by the separate loop contract. The call, declared result,
terminal return and any explicit guarded-return layout must agree. Actual return
leaves and all scalar input values retain exact types; there is no implicit
bool/i32/float conversion, resource input or nested aggregate admission.

This is a structural contract, not an allowlist of generated helper names or
precombined arities. Ordinary and mixed scoped edges share the existing acyclic
closure and size/depth limits, including discarded and guard-bypassed calls.
Unknown targets, foreign-lane values, hidden effects and signature drift reject.
Native lowering extracts a flat LLVM value; generic LLVM still unpacks and drops
each temporary owned return immediately. No interpreter, host dispatcher or new
YIR opcode is introduced; only the native lowering-private return ABI changes.

Multi-state guarded break and explicit-step continue now preserve source-ordered
updates, selected-path suffix evaluation and child-loop exit scope. A guarded
return bypasses later dynamic induction preflight; reaching an excessive bound
still traps the process. These checks do not add callback fuel/preemption or
resource-bearing returns. Native flat helpers and callback State no longer allocate
on each return; general owned-return paths still allocate.

## Call ABI

Each export has the same internal native ABI:

```text
i32 bridge(ptr args, i64 argc, ptr out, i64 outc)
```

`args` and `out` contain native-endian 64-bit slots. Counts are exact signature
lengths, not buffer capacities. State output follows the declared layout's
depth-first field order. Event/close inputs start with those state slots, followed
by their additional arguments. An open callback with zero arguments accepts a null
input pointer. Output must be nonnull because empty state is not admitted.

| Scalar | Slot Representation |
| --- | --- |
| `bool` | Exactly 0 or 1. |
| `i32` | Sign-extended to 64 bits. |
| `i64` | Its 64-bit representation. |
| `f32` | IEEE bits in the low 32 bits; upper bits must be zero. |
| `f64` | Its 64-bit IEEE representation. |

Status 0 means success, 1 means invalid counts/pointers, and 2 means noncanonical
input encoding. Counts, null pointers and all scalar encodings are checked before
entering the callback or writing output. All input slots are loaded before output
publication, so in-place and partial-overlap state updates are supported. The bridge
extracts the returned LLVM slot value before publishing; there is no temporary
aggregate heap owner to require, read or drop on this native-only path.

This is a trusted, statically linked internal host boundary, not a new Nuis cffi
authorization or pointer sandbox. Nonnull pointers must actually reference the
claimed readable/writable memory. The bridge cannot validate arbitrary addresses.
Native traps terminate execution; they are not caught, rolled back, retried or
converted into reference-executor fuel exhaustion.

## Explicit Host

`yir-pack-aot <module.yir> <output-dir> --native-session <registration-id>` now
builds a scalar-session executable on 64-bit macOS/Linux hosts. This selection is
exclusive with `--headless` and frame-scale/window options. The default image and
headless provider hosts are unchanged. The corresponding `nuis build` profile is
described below; neither route is the Nsld self-contained image path.

The packer statically links a callback object, an LLVM process entry and the
existing runtime. The entry consumes a `NativeSessionDescriptorV1` emitted beside
the callbacks: version `u64`, source pointer/`u64` length, registration-ID
pointer/`u64` length, layout pointer/`u64` length, then open/event/close pointers.
There is no dynamic library lookup. The source and descriptor are bounded to 8 MiB
of YIR, 256 ID bytes and 64 KiB of layout respectively at host ingress.

Before open, the host compares the complete parsed YIR graph embedded in the
entry against the graph bound into the callback object, including nodes, edges,
function tables, registrations and lanes. Whitespace differences may parse equally;
declaration reordering is not normalized during identity admission. A newly compiled
reordered object works, but mixing it with a different entry rejects. Matching
names or arity is insufficient. This detects stale/mixed artifacts, not malicious
native-code tampering: trusted static production/linking still establishes that
the descriptor and function pointers belong together. It is not a signature scheme.

`ApplicationSession::open_native_registered` consumes a thread-local
`NativeSessionBindings` instead of constructing a `FunctionSession` interpreter.
It reuses the existing state machine: invalid caller arguments do not enter a
callback or consume close; callback failure faults the session, retains the last
accepted state, and permits one explicit close. Failed close is not retried. Cleanup
success cannot erase an event failure. Drop does not run a Nuis callback. Native
output is decoded in separate storage before state publication. Native budgeted
event/close calls reject before callback entry rather than pretending reference
fuel can preempt native instructions.

Scalar slot encoding and nested state reconstruction now live in `yir-core`.
The runtime does not depend on LLVM or repeat the CPU operation admission table.
The unsafe static-binding constructor is a compiler/host trust boundary, not new
Nuis `unsafe` syntax or authorization for arbitrary C ABI calls. Existing Rust
host implementation and aggregate allocation compatibility routines remain.

The native process accepts `--application-session`, `--open-args`, repeatable
`--event-args`, and `--close-args`. Arguments are comma-separated, typed according
to the registered signature; booleans are `true`/`false`, integers and floats are
parsed as their declared kinds. An empty string supplies zero arguments. All calls
are checked before open, with at most 64 events and 64 slots per callback. The
host reports accepted states and `native_session_completed=1` only after successful
explicit close. It offers no provider transport, cancellation, rollback, retry,
wall-time preemption or invented per-node reference trace.

For the nested scalar fixture compiled to `session.yir`:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p yir-pack-aot -- session.yir build/native --native-session counter
build/native/session --application-session counter --open-args '10,-17,true,1.5,-2.25' --event-args '3,false' --close-args '5'
```

## Build And Launch

The project manifest may select `packaging_mode = "native-session-aot-bundle:counter"`,
or the build command may explicitly override it. The registration suffix is part
of the build manifest, compiled artifact, envelope and cache identity. There is
no implicit first-registration selection and no runtime switch to another native
registration in the same artifact.

```sh
nuis build --packaging-mode native-session-aot-bundle:counter path/to/project build/native
nuis run-artifact build/native --native-session counter --open-args '10,-17,true,1.5,-2.25' --event-args '3,false' --close-args '5'
nuis verify-artifact build/native/nuis.compiled.artifact
nuis materialize-artifact build/native/nuis.compiled.artifact build/restored
nuis run-artifact build/restored --native-session counter --open-args '10,-17,true,1.5,-2.25' --close-args '5'
```

`check`/`workflow` honor manifest selection and report a real LLVM checkpoint.
The selected callbacks are lowered directly; an unsupported callback fails the
build rather than falling back to a verified-YIR-only or whole-main path.

The `nuis-native-session-build-inputs-v1` carrier uses the
`native-scalar-llvm-v1` checkpoint. It embeds source, tokens, AST, NIR, YIR,
compiler handoff, selected LLVM and bundle declarations. Present documentation,
import, Galaxy and resolution-lock metadata is also carried and relocated.
Each input has a unique hash row and a canonical output filename, with per-input
bounds and a shared 64 MiB limit. Native YIR retains the host's 8 MiB bound.
Verification regenerates the selected callback LLVM exactly and checks its
registration, scalar layout and static-host declarations. Manifest verification
also compares the executable bytes with the compiled artifact image. These are
integrity/consistency checks, not native-code authentication or a signature scheme.

Standalone verification and `materialize-artifact` do not require the original
build directory. Materialization verifies the restored manifest before granting
executable permissions on Unix. Informational producer paths in bundle text are
not runtime dependencies. The launcher checks the resolved binary identity and
uses the same typed script parser as the packaged entry before open. Unlike the
provider `--application-session` option, `--native-session` admits all five scalar
kinds, prepares no provider, and has no interpreter or Nsld-runner fallback.
Cancellation, window, frame-export and provider options cannot be combined with
this script. Nonzero native process exit remains failure.

## Evidence

The [native regression](../../tools/nuisc/tests/native_application_bridge.rs) uses
a [pure Nuis fixture](../../tools/nuisc/tests/native_application_bridge/main.ns).
A static LLVM test driver invokes the exported bridges in one executable, passing
actual returned state into subsequent calls. It does not interpret YIR. Existing
host object emission and runtime support are used; no new C/Objective-C shim is added.

The regression compares open, three separate events (including an early return)
and close against `ApplicationSession` reference execution. It covers nested state,
all five scalar kinds, finite floats, signed zero, NaN payload bits, in-place calls,
reordered declarations and unrelated-main exclusion. Invalid arguments preserve
the output; an entry counter confirms they never call the native helper.
Separate checks cover zero-argument native open, the 64/65-slot boundary, resource
and layout drift, parameter/lane drift, external initialization dependencies and
unsupported side effects. A malformed unused field operation must fail strict
native materialization instead of silently disappearing.

The [helper fixture](../../tools/nuisc/tests/native_application_bridge/helpers.ns)
retains actual typed YIR/LLVM calls after compilation, not just inline expansions.
Six more native executions cover nested and shared helpers, source early returns,
all five scalar kinds, NaN/signed-zero transport and reversed declarations.
[Closure regressions](../../tools/nuisc/tests/native_application_bridge/helpers.rs)
reject direct/mutual recursion, unknown or drifted signatures, implicit scalar
coercions, foreign-lane/global dependencies, hidden side effects and unmaterialized
helper values. They test accepted/rejected function-count, depth and body-size
boundaries, plus the whole-closure node limit. Unselected recursion stays excluded.

The [loop fixture](../../tools/nuisc/tests/native_application_bridge/loops.ns)
retains five counted loops through source lowering. Six native executions compare
all scalar slots with the reference for ascending, descending, zero-trip, ordered
multi-carry and product loops composed through helpers, including reversed
declarations and float bit transport. [Loop regressions](../../tools/nuisc/tests/native_application_bridge/loops.rs)
reject malformed constant induction, excessive trips, forward carries, float
seeds and hidden helper effects. Budgeted reference exhaustion preserves the
accepted application state and cannot become success through cleanup. Independent small-step simulation checks the
termination proof, including extreme signed inputs and the exact trip bound.

The [dynamic loop fixture](../../tools/nuisc/tests/native_application_bridge/dynamic_loops.ns)
adds runtime start/limit/step parameters from callback values, descending and
inactive zero-step chains. [Dynamic parity tests](../../tools/nuisc/tests/native_application_bridge/dynamic_loops.rs)
perform six additional native runs with exact typed state and reversed declarations,
and retain general-lowering policy isolation and shared bound/carry dependencies.
The native frontdoor/cache/standalone regression also covered this fixture before
advancing to scoped loop calls.
[Guard regressions](../../tools/nuisc/tests/native_application_bridge/dynamic_loop_guard.rs)
run 1680 parameter cases in twelve native executables across six comparisons and
add/sub. Volatile inputs prevent constant-only evaluation. A test-only rejection
sentinel permits comparison with independent checked-step simulation; iteration
counters prove rejected/zero-trip/skipped paths never enter the body. Six separate
executions invoke the real registered export, retain the unmodified trap and
require process failure without a returned ABI status. Boolean induction drift
rejects instead of silently coercing to i64.
Every guard-probe process has a deadline so a regression cannot hang the test suite.

The [scoped-loop fixture](../../tools/nuisc/tests/native_application_bridge/scoped_loops.ns)
retains four scoped loops, shared/nested helpers, a zero-argument discarded call,
single-i64 carry returns and dynamic induction. Six native/reference executions
retain exact session slots and reversed-declaration parity.
[Admission regressions](../../tools/nuisc/tests/native_application_bridge/scoped_admission.rs)
check scoped-only and mixed direct/scoped cycles, exact depth/function boundaries,
unknown/drifted targets, malformed captures, hidden effects and caller-lane isolation.
[Execution regressions](../../tools/nuisc/tests/native_application_bridge/scoped_execution.rs)
observe the actual helper arguments in 36 callback cases and 84 invocations across
two native binaries, with old-counter/old-carry ordering, wrapping i64 carries,
all five scalar kinds, signed zero and NaN payloads. Three more binaries retain
real traps and prove invalid induction enters no helper or publication path.
Reference fuel exhaustion retains the accepted state and cleanup cannot clear
the failure. This is not native fuel/preemption evidence.

The [multi-carry fixture](../../tools/nuisc/tests/native_application_bridge/multi_loops.ns)
adds shared aggregate-returning helpers with two carries, ascending/descending
loops and zero-trip seeds. Six more native/reference runs retain exact session
slots and reversed-declaration parity. The
[multi-carry execution tests](../../tools/nuisc/tests/native_application_bridge/multi_execution.rs)
run 144 callback cases and 288 actual helper invocations across six native
executables with 2/3/7 carries, reversed operand order, source-ordered updates,
wrapping carried arithmetic, zero/one trips and all five scalar capture kinds.
Test-only wrappers delegate to the real aggregate allocator/drop functions,
checking no live heap owner before the next helper and exactly one allocation/drop
per successful export. Three additional binaries retain real preflight traps.
[Multi-carry admission tests](../../tools/nuisc/tests/native_application_bridge/multi_admission.rs)
reject layout/ownership/seed/parameter drift, recursive zero-trip edges, hidden
effects, foreign lanes and attempts to enable general aggregate calls. They also
guard against silently discarded outer-state updates in counted-loop fallback.

The [guarded-break fixture](../../tools/nuisc/tests/native_application_bridge/break_loops.ns)
adds source-normalized two-carry and control-only loops. Six native/reference runs
retain typed state and reversed-declaration parity. The
[break execution tests](../../tools/nuisc/tests/native_application_bridge/break_execution.rs)
exercise 180 callback cases in six native binaries with 2/3/7 slots, early/middle/
last/no break, zero trips, both directions, reversed operands/declarations, wrapping
user carries and exact five-scalar captures. Real allocation/drop counters now
observe only callback State, not the exiting iteration. Eleven process-trap runs
cover nonzero seeds (including zero trips), invalid returned control and invalid
induction even when the helper would immediately break. Helper-entry probes flush
their output before execution so a trap cannot hide buffered evidence.
[Source tests](../../tools/nuisc/tests/native_application_bridge/break_source.rs)
compare native, reference and independent expected states in 40 cases across eight
binaries: nested/flat loops, both directions, pre-step exit counters, skipped suffix
updates, multi-state guarded break and matching explicit-step continue.
Shared negative checks retain
layout/kind/closure restrictions and reject unmodeled source updates or forged
control provenance. Reference budget exhaustion retains the last accepted state;
native budget/preemption remains unimplemented.

The [multi-state branch fixture](../../tools/nuisc/tests/native_application_bridge/branch_loops.ns)
adds six typed native/reference lifecycle runs with guarded updates and mixed
break/continue. [Nested aggregate execution](../../tools/nuisc/tests/native_application_bridge/aggregate_execution.rs)
uses six native binaries for 144 callback cases and 576 inner helper calls, with
2/3/7 slots, reversed capture/declaration order, wrapping updates, signed-zero/NaN
bits and one real allocation/drop per callback, including early returns. Entry
probes verify that no heap temporary remains live before the next inner call.
[Aggregate admission](../../tools/nuisc/tests/native_application_bridge/aggregate_admission.rs)
rejects malformed layouts, result/parameter/return-field drift, hidden effects,
foreign captures and recursive calls, while shared unit tests exercise slot bounds.
[Guard execution](../../tools/nuisc/tests/native_application_bridge/aggregate_guards.rs)
proves that an early return skips a later excessive-loop preflight, while actually
entering that loop traps without publishing callback state.

The [division/remainder tests](../../tools/nuisc/tests/native_application_bridge/division_execution.rs)
compare 1840 callbacks across eight native binaries with reference execution and
an independent i128 oracle. Inputs cover signed extremes, signs, zero, scalar and
flat aggregate returns, unused results and unused arguments. Twenty invalid
dynamic cases and four invalid literal cases trap; four unselected literal cases
return safely. Flushed leaf-entry probes prove actual evaluation order, including
failing prefix work before an unselected branch. Every process run is bounded.
[Admission tests](../../tools/nuisc/tests/native_application_bridge/division_admission.rs)
reject either/both operand kind drift, malformed arity and non-i64 typed opcodes,
and admit guarded flat-value branches without speculating their arguments. The
[division fixture](../../tools/nuisc/tests/native_application_bridge/division_loops.ns)
adds six typed lifecycle native/reference runs with multi-state loop exits and
reordered declarations; it also passes the real build/cache/standalone workflow.

The [aggregate arithmetic regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_division.rs)
add 3220 callbacks across fourteen native binaries, with the same reference and
independent i128 oracles. They cover early/two-arm/nested branches, record captures,
shared suffixes, unused results/arguments and registered State fields whose names
are not loop carry names. Thirty-two dynamic and four literal invalid cases trap;
four unselected literal cases return safely. The probes delegate to the real
allocator/drop functions, require balanced release after each accepted callback
and before leaf entry, and verify that allocations really occurred. They do not
promise cleanup after a trap. The former counted-loop rejection now has guarded
lowering evidence. Source tests separately prove 32 guards produce linear
helper growth and that effects/cycles do not enter the value catalog.
The [aggregate-division fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_division_loops.ns)
adds six typed lifecycle runs and standalone restoration, using declared
quotient/remainder fields alongside the existing bounded loop exits.

The [counted aggregate regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_counted.rs)
add 2064 callbacks across sixteen binaries for helper calls, inline arms, prefix
work and shared suffixes, with ascending/descending dynamic induction and checked
division/remainder. Independent checked-step simulation and reference execution
agree with actual result slots, iteration counters and balanced real allocation/drop.
Twenty-four skipped invalid inputs and two loop-only success cases bring this
slice to 2090 accepted/skipped callbacks. Twenty-six real process traps cover zero
step, wrong direction, induction overflow, excessive trips and invalid arithmetic;
loop preflight failures report zero iterations and no arithmetic-leaf invocation.
An unselected inclusive-overflow loop succeeds without any division in its source,
and the exact 65536-trip case succeeds. Reference fuel is not a substitute for
native preflight and is not used to certify preflight rejection. The former
loop-carried rejection now has guarded chained-loop lowering evidence.
The [aggregate-counted fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_counted_loops.ns)
adds six typed lifecycle runs and the same build/cache/standalone restoration
workflow, with a counted helper called inside an aggregate arithmetic branch.

The [carried aggregate regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_carried.rs)
add 2752 callbacks across sixteen binaries for the same four branch shapes, both
induction directions, checked division/remainder and ordered add/multiply carries.
All result fields agree with reference execution and an independent i128 wrapping
oracle. Variable 1/7-carry cases, skipped invalid inputs and the exact trip-limit
case bring this slice to 2849 accepted/skipped callbacks. Twenty-seven real process
traps cover preflight failure before any update, reached zero/overflow arithmetic,
unconditional prefix work and loop-only guards. Ascending overflow and descending
underflow execute zero iterations. A shared
[execution probe](../../tools/nuisc/tests/native_application_bridge/aggregate_loop_probe.rs)
observes result slots, actual iterations and balanced real allocation/drop;
it keeps the actual process trap and never substitutes reference fuel for preflight.
The [aggregate-carried fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_carried_loops.ns)
adds six typed lifecycle runs and build/cache/standalone restoration with the
existing break/continue paths. Source policy tests retain exact i64/local-seed
requirements and invariant headers; nested carry-update acceptance is now covered
separately by the scoped-iteration regressions below.
Extreme seeds also exposed and fixed host-debug-dependent scalar integer overflow
in the reference CPU. Dedicated CPU tests keep wrapping arithmetic separate from
checked division/remainder errors.

The [conditional aggregate regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_conditional.rs)
add 1152 callbacks across sixteen binaries for all four branch positions, both
directions, checked division/remainder and conditional add/multiply/keep updates.
Another twelve binaries cover all six comparisons, reversed operands and 1/7-carry
widths. With skipped invalid calls and the exact trip limit, this slice covers
1357 accepted/skipped callbacks and 27 real process traps. The same independent
i128 oracle and shared execution probe compare every returned slot, actual
iterations, leaf operands and real allocation/drop balance. Preflight rejects
before the first update; selected arithmetic failure is still a process trap.
Exact-kind drift, including zero-trip inputs, missing operands, hidden branches,
source predicate/arm restrictions and cooperative reference-fuel failure have
dedicated regressions. Shared comparison normalization now accepts invariant-on-left
comparisons without changing which iteration's state is read.
The [aggregate-conditional fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_conditional_loops.ns)
adds six typed lifecycle runs and the eighth build/cache/standalone-restoration
fixture, retaining earlier break/continue and checked arithmetic evidence.

The [one-sided aggregate regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_one_sided.rs)
reuse that independent oracle and execution probe for omitted `else`, empty
`else`, empty `then`, and mixed conditional/linear carry sequences. Thirty-nine
native binaries cover 2502 accepted/skipped callbacks and nine real process traps.
Every carry keeps its own prior value when its update is skipped, including with
extreme seeds, earlier updated siblings, all six comparisons, reversed operands
and 1/3/7-carry widths. Dynamic strides, zero trips, the exact 65536-trip limit,
overflowing induction and selected-path arithmetic retain their existing checks.
The [aggregate-one-sided fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_one_sided_loops.ns)
adds six typed lifecycle runs; a reference-fuel failure preserves accepted state
and cleanup without pretending to preempt native execution. The ninth frontdoor
fixture verifies build/cache isolation, tamper rejection and standalone restoration.

The [compound aggregate regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_compound.rs)
add 1216 accepted/skipped callbacks and nine real process traps across 31 native
binaries. In addition to state slots, iterations, leaf operands and allocation
balance, a [comparison probe](../../tools/nuisc/tests/native_application_bridge/predicate_probe.rs)
counts comparisons in the emitted loop body with volatile observations. Counts
match an independent short-circuit oracle across `&&`, `||`, mixed precedence,
explicit grouping, variable widths, wrapping and selected/skipped guards.
The exact trip limit and invalid induction still distinguish preflight from
reference fuel. Nested RHS type drift rejects before execution. A 10000-leaf
malformed condition is rejected during general verification rather than relying
on later native admission. Generic backend regressions retain i64/f32/f64 scalar
selection without admitting float carries into this native profile.
The [aggregate-compound fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_compound_loops.ns)
adds six typed lifecycle runs and reference-budget failure with accepted-state
retention and cleanup. The tenth frontdoor fixture also passes build/cache
isolation, tamper rejection and standalone restoration of the compiled artifact.

The [nested aggregate regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_nested.rs)
compare native execution, registered reference execution and an independent i128
wrapping/checked-induction oracle. They cover 1/3/7 carries, both induction directions,
callee/inline/prefix/suffix positions, three distinct update leaves, implicit keep,
zero trips and signed extremes. Volatile probes check actual iteration counts,
source comparison counts and aggregate allocation/drop balance. Packed boolean
normalization is excluded by operand provenance, not by discarding integer `==` or
`!=` comparisons. Separate process-trap cases reject zero/negative strides,
overflowing/excessive induction and reached division by zero; skipped calls retain
state. Source admission tests reject invalid leaves and over-deep guard trees.
The retained [original-decision oracle](../../tools/nuisc/tests/native_application_bridge/aggregate_nested_tree.rs)
and its [execution regressions](../../tools/nuisc/tests/native_application_bridge/aggregate_nested_oracle.rs)
also cover short-circuit paths, empty arms, own-value retention, exact induction
limits and reference-fuel cleanup without depending on the old two-value collapse.
The [aggregate-nested fixture](../../tools/nuisc/tests/native_application_bridge/aggregate_nested_loops.ns)
adds the eleventh frontdoor build/cache/relocation case with the existing typed
lifecycle, checked arithmetic and multi-state break/continue paths.

The [production-host regression](../../tools/nuisc/tests/native_application_host.rs)
compiles the [basic scalar callback fixture](../../tools/nuisc/tests/native_application_bridge/main.ns)
and invokes the real packer for normal and reversed YIR
declarations. Actual statically linked host calls match reference states. Invalid
later arguments reject before open. A generated-object event failure preserves the
last accepted state, runs cleanup and exits unsuccessfully without reference
fallback. Mixing an entry with a different YIR graph rejects before any callback.
Separate [host policy tests](../../crates/yir-runtime-host/src/native_application_session/tests.rs)
use fault-injecting test exports to cover malformed output, failed/duplicate close,
open failure, Drop, descriptor drift, argument validation and fuel rejection.
These test doubles are policy evidence, not additional lowering proofs.

The [frontdoor regression](../../tools/nuis/tests/native_session_workflow.rs) builds
the five-scalar multi-carry, guarded-break, multi-state branch, checked-division,
aggregate-division, aggregate-counted, aggregate-carried, aggregate-conditional
and aggregate-one-sided/aggregate-compound/aggregate-nested/
aggregate-sequences/aggregate-temporaries/aggregate-checked/aggregate-calls/aggregate-local-values/aggregate-loop-calls/bool-rebinding/aggregate-rebinding
fixtures with two registrations,
runs typed events and close,
rejects wrong arguments and changed binary/YIR/LLVM/bundle/metadata before open,
and switches registrations through a shared cache/output directory. It removes
the original source/manifest and output, verifies the standalone compiled artifact, materializes it
elsewhere and runs the restored executable with identical states. Separate
carrier tests reject rehashed LLVM and bundle drift, schema/profile downgrades,
duplicate fields and missing hash rows.

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --test owned_cleanup_return -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_host -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib native_ -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --test checkpoint_workflow -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib aot_application_bundle -j 1 -- --test-threads=1
```

The earlier native evidence was recorded on Apple Silicon. The nested-carry
extension additionally has Linux x86-64 native/reference execution checks; retain
its runner, toolchain, source identity and test logs with the acceptance record.
Neither platform's CPU results certify Windows execution or device-provider parity.

Shared loop-work integration on 2026-09-19 was checked on macOS aarch64:
141 LLVM unit tests, all 144 native bridge regressions, 27 native-entry tests,
48 Buffer-outliner unit tests, four compound-loop regressions and four bound-artifact
unit tests passed. The seven loop-work regressions were rerun after adding constant
zero-trip and invalid-induction-before-debit probes. All eighteen frontdoor
build/cache/standalone restoration fixtures passed, including exact/default budget
exhaustion and lifecycle resets. All 26 tensor tests passed. The live CLI reported
1221 drift checks with zero
failures and clean coverage, hierarchy and lineage. These are targeted CPU integration
checks, not a full-workspace run or a new Linux/GPU certification. Native-entry tests
preserve checked failures but do not claim the native-session induction or reservation
policy. Reentrant transport isolation is not concurrent-thread stress evidence.

The subsequent helper-entry integration on the same host passed 141 LLVM unit
tests, 71 targeted compiler unit tests (outlining, optimization, artifact binding
and portable document paths), 31 ordinary native control-flow tests, all nineteen
frontdoor fixtures and 26 tensor tests. The first 150-case bridge run passed 136
and exposed fourteen source-comparison probe mismatches: private budget checks
were being counted as source predicates. After the provenance-only probe repair,
all 33 affected/additional checks passed, including the fourteen failures, entry
and loop budgets, reentry and the new probe regression. Together these runs cover
151 bridge cases; this was not a second full-suite run after the test-only repair.
The live tensor reported 1232 drift checks, zero failures and clean coverage,
hierarchy and lineage. No new Linux/GPU certification or whole-workspace test run
is claimed. The broad session coordinate remains active at 86.

Iteration-local bool rebinding was subsequently checked on macOS aarch64: all
156 native bridge cases, 29 default native-entry cases, four compound-loop cases,
89 Buffer-loop cases, one owned-cleanup case, 73 targeted compiler unit tests,
141 LLVM unit tests and 26 tensor tests passed. Four selected frontdoor cases
(basic scalar, bool rebinding, loop-work and helper-entry budgets) passed build,
cache and standalone restoration; this was not a full rerun of all twenty frontdoor
cases. The live tensor reported 1242 drift checks, zero failures and clean coverage,
hierarchy and lineage. The old i64 guard-check anchor was updated to the explicit
typed parameter predicate, not removed. The session coordinate remains active at
86; these CPU checks do not certify Linux/GPU execution or the whole workspace.

Iteration-local flat rebinding was checked on 2026-09-20 on macOS aarch64: all
162 native bridge cases, 31 default native-entry cases, 89 Buffer-loop cases,
one owned-cleanup case, four compound-loop cases, 74 targeted compiler unit tests,
141 LLVM unit tests and 26 tensor tests passed. Seven selected frontdoor cases
(basic scalar, flat-local values, loop-bearing calls, bool/flat rebinding and both
work budgets) passed build/cache/source-free standalone restoration. This was not
a full rerun of all twenty-one frontdoor cases or the workspace. The live tensor
reported 1252 drift checks, zero failures and clean coverage, hierarchy and lineage.
At that local-rebinding checkpoint the broad session coordinate remained active at
86, with outer aggregate carries next. These CPU correctness/cleanup checks are not performance measurements,
formal memory-safety proofs or new Linux/GPU certification.

Outer flat-i64 carry integration was checked on 2026-09-20 on macOS aarch64:
all 168 native bridge cases, 33 ordinary native-entry cases, 89 Buffer-loop cases,
four compound-loop cases and one owned-cleanup case passed. The final revision
also passed 80 targeted compiler unit tests, 141 LLVM unit tests and 26 tensor
tests. Eight selected frontdoor cases (basic scalar, flat-local values, loop-bearing
calls, bool/flat rebinding, outer flat carries and both work budgets) passed build,
cache and source-free standalone restoration. This was not a full rerun of all
twenty-two frontdoor cases or the workspace. The fresh tensor CLI reported 1263
drift checks with zero failures and clean coverage, hierarchy and lineage.
At that checkpoint the broad session coordinate remained active at 86, with outer
bool carries next.
These are CPU correctness and successful-path cleanup checks, not performance
measurements, formal memory-safety proofs or new Linux/GPU certification.

Outer bool carry integration was checked on 2026-09-20 on macOS aarch64:
all 174 native bridge cases, 35 ordinary native-entry cases, 89 Buffer-loop cases,
four compound-loop cases and one owned-cleanup case passed. The final compiler
revision passed 84 targeted unit tests, 141 LLVM unit tests and 26 tensor tests;
seven affected bool/budget cases were rerun after test-helper cleanup and passed.
Nine selected frontdoor cases (basic scalar, flat-local values, loop-bearing calls,
bool/flat rebinding, outer bool/flat carries and both work budgets) passed build,
cache, tamper rejection and source-free standalone restoration. This was not a full
rerun of all twenty-three frontdoor cases or the workspace. The freshly built tensor
reported 1275 drift checks, zero failures and clean coverage, hierarchy and lineage.
At that checkpoint the broad session coordinate remained active at 86, with literal
nested counted loops next. These CPU correctness and successful-path cleanup checks do not certify
performance, formal memory safety or new Linux/GPU execution.

Literal nested counted-loop integration was checked on 2026-09-20 on macOS aarch64:
all 180 native bridge cases, 37 ordinary native-entry cases, 89 Buffer-loop cases,
four compound-loop cases and one owned-cleanup case passed. The compiler passed
87 targeted unit tests and the tensor passed 26 unit tests. Three affected budget
cases were rerun after adding the persistent-child-index check and passed.
Seven selected frontdoor cases (basic scalar, loop-bearing calls, outer bool/flat
carries, literal nesting and both work budgets) passed build, cache, tamper rejection
and source-free standalone restoration. This was not a full rerun of all twenty-four
frontdoor cases or the workspace. The fresh tensor CLI reported 1287 drift checks,
zero failures and clean coverage, hierarchy and lineage. At that checkpoint the broad
session coordinate remained active at 86, with guarded exits in literal nested value loops next. These
checks do not establish performance, formal memory safety or new Linux/GPU execution.

Leading-step value-loop exit integration was checked on 2026-09-20 on macOS aarch64:
528 lowering unit tests, all 186 native bridge cases, 39 ordinary native-entry cases,
89 Buffer-loop cases, four compound-flow cases and one owned-cleanup case passed.
Six affected exit cases were rerun after strengthening early-parent child-preflight
skipping and following-loop no-refund checks, and passed. Existing ordinary-entry
flow instruction assertions remain intact. All 26 tensor unit tests passed.
Six selected frontdoor cases (basic scalar, outer bool carries, literal nesting,
value-loop exits and both work budgets) passed build, cache isolation, tamper rejection
and source-free standalone restoration. This was not a full rerun of all twenty-five
frontdoor cases or the workspace. The fresh tensor CLI reported 1300 drift checks,
zero failures and clean coverage, hierarchy and lineage. The broad session coordinate
remained active at 86, with trailing-step counted value loops next. These are CPU
correctness and successful-path cleanup checks, not performance measurements, formal
memory-safety proofs or new Linux/GPU certification.

Trailing-step value-loop integration was checked on 2026-09-20 on macOS aarch64:
531 lowering unit tests, 41 ordinary native-entry cases, 89 Buffer-loop cases,
four compound-flow cases and one owned-cleanup case passed. All 191 native bridge
cases were exercised. The full sweep passed 189; two old source/positional assertions
were updated and passed focused reruns: extra pure carries now require independent
native/reference lifecycle totals, effectful unsupported bodies still reject, and
kind corruption now locates its bool capture by declared parameter type rather than
assuming it is the last operand. The 18 affected cases passed across focused reruns,
including stronger nonunit continue and early-parent child-preflight skipping probes.
All 26 tensor unit tests passed. Nine selected frontdoor cases (basic scalar,
guarded break, compound carries, outer bool carries, literal nesting, leading/trailing
value exits and both work counters) passed build, cache isolation, tamper rejection
and standalone restoration after source deletion. This was not a full rerun of all
twenty-six frontdoor cases or the workspace. The freshly built tensor reported 1312
drift checks, zero failures and clean coverage, hierarchy and lineage. The broad
session coordinate stays active at 86, with early-returning counted value helpers
next. These are CPU correctness and successful-path cleanup checks, not new
performance measurements, formal memory-safety proofs or device certification.

Counted-return integration was completed on 2026-09-23 on macOS aarch64. This
change passed 535 lowering unit tests, 23 selected native-bridge regressions,
89 Buffer-loop tests, 43 ordinary control-flow tests, four compound-loop tests,
one owned-cleanup return test and 26 tensor unit tests. After the final private-name
hardening, all 535 lowering tests, the five new native-bridge tests and two new
ordinary-entry tests were rerun successfully. Three selected frontdoor cases
(counted returns, trailing-step loops and leading-step value exits) passed
build/cache, tamper rejection and source-free standalone restoration. The earlier
interrupted frontdoor run was not counted as completion; these three cases were
rerun together to obtain a complete result. This was not a full rerun of the
196-case bridge suite, all 27 frontdoor cases or the workspace. The live tensor
reported 1322 drift checks with zero failures, clean coverage/hierarchy/lineage,
and the session coordinate remains active at 86. Private control-helper expansion
is next; no graph limit, public ABI or opcode was widened. These are CPU-path
proofs, not new Linux GPU, Windows, peak-memory or performance certification.

Ready-value control elision was checked on 2026-09-23 on macOS aarch64.
All 538 lowering unit tests, 141 LLVM unit tests, 43 ordinary control-flow tests,
89 Buffer-loop tests, four compound-loop tests, one owned-cleanup test and 26 tensor
tests passed. Across focused runs, 42 native-bridge cases passed, including exact
entry accounting, ready bool/record snapshots, fallible aggregate cleanup, counted
returns and unchanged graph/depth/node rejection. Old helper-count expectations
were recalibrated to actual emitted entries; insufficient-budget rejection remains
required. The restored full composition emits 57 reachable native functions.
Five frontdoor cases (basic scalar, counted returns, full control composition,
loop-work reservations and helper-entry fanout) passed build/cache, tamper rejection
and source-free standalone restoration. This was not a full rerun of the 200-case
bridge suite, all 28 frontdoor cases or the workspace. The fresh tensor reported
1331 drift checks with zero failures and clean coverage/hierarchy/lineage. The
session coordinate remains active at 86, with nontrivial terminal continuations
next. These are CPU correctness and artifact-closure checks, not new Linux/GPU,
Windows, peak-memory, formal safety or runtime-performance certification.

Single-use terminal continuation folding was checked on 2026-09-23 on macOS
aarch64. All 541 lowering unit tests, 45 selected native-bridge cases, 44 ordinary
control-flow tests, 89 Buffer-loop tests, four compound-loop tests, one owned-cleanup
test, 141 LLVM unit tests and 26 tensor tests passed. The focused bridge run includes
typed terminals, field/argument ordering, shared suffixes, unused calls, exact work
debits and unchanged graph/depth/node rejection. Five frontdoor cases (basic scalar,
counted returns, full composition and both work budgets) passed build/cache, tamper
rejection and source-free standalone restoration. The full-composition frontdoor
also requires at most 54 native functions and byte-identical LLVM after restoration.
The live tensor reported 1338 drift checks, zero failures and clean coverage,
hierarchy and lineage; the session coordinate remains active at 86. Single-use
multi-statement continuations are next. This was not a full rerun of all 203 bridge
cases, all 28 frontdoor cases or the workspace, and does not add Linux/GPU, Windows,
formal safety, peak-memory or runtime-performance certification.

Single-use multi-statement continuation folding was checked on 2026-09-23 on
macOS aarch64. All 544 lowering unit tests, 49 selected native-bridge cases,
89 Buffer-loop cases, 45 ordinary control-flow cases, four compound-loop cases,
one owned-cleanup case, 141 LLVM unit tests and 26 tensor tests passed. The Buffer
sweep exposed an old assertion requiring an eliminated continuation; it now requires
no such forwarder while retaining guards, independent results and reversed-node
execution. The complete 89-case suite passed again after that assertion update.
The focused bridge run checks multi-statement typed results, unused fallible calls,
argument ordering, same-name record/scalar scope isolation, shared inner suffixes,
exact entry exhaustion and unchanged loop reservations and graph/depth/node bounds.
Five frontdoor cases (basic scalar, counted returns, full composition and both work
budgets) passed build/cache, tamper rejection and source-free standalone restoration.
The full composition emits 51 reachable native functions, and both its native and
frontdoor regressions require at most 51; restored LLVM remains byte-identical.
The fresh tensor reported 1344 drift checks, zero failures and clean coverage,
hierarchy and lineage. The session coordinate remains active at 86, with
scope-colliding single-use continuations next. This was not a full rerun of all
207 bridge cases, all 28 frontdoor cases or the workspace, and does not add
Linux/GPU, Windows, formal safety, peak-memory or runtime-performance certification.

Scope-aware continuation hygiene was checked on 2026-09-23 on macOS aarch64.
All 546 lowering unit tests, 51 selected native-bridge cases, 89 Buffer-loop cases,
46 ordinary control-flow cases, four compound-loop cases, one owned-cleanup case,
141 LLVM unit tests and 26 tensor tests passed. The two new loop-budget cases
passed again after strengthening zero-trip bool/record seed checks. Native traces
cover same-name record/scalar bindings, renamed local and unrenamed outer loop
updates, private bool conversions, exact entry exhaustion and independent loop
reservations. Unit and CLI tests preserve sibling constants, future private-like
names, field/function symbols, early returns and reached unused fallible calls.
Five frontdoor cases (basic scalar, counted returns, expanded full composition and
both work budgets) passed build/cache, tamper rejection and source-free standalone
restoration. The full composition now includes a same-name record/scalar prefix;
its 51-function ceiling and byte-identical restored LLVM remain verified.
The fresh tensor reported 1351 drift checks, zero failures and clean coverage,
hierarchy and lineage. The session coordinate remains active at 86, with per-return
flat-i64 aggregate allocation next. This was not a full rerun of all 209 bridge
cases, all 28 frontdoor cases or the workspace, and adds no Linux/GPU, Windows,
formal safety, peak-memory or runtime-performance certification.

Native flat value transport was checked on 2026-09-23 on macOS aarch64.
All 145 LLVM unit tests, 89 Buffer-loop cases, 46 ordinary control-flow cases,
four compound-loop cases, one owned-cleanup case and 26 tensor tests passed.
The full 210-case native-bridge scan passed 209 and exposed one outdated rejection
of an i64 return inside a counted loop. That test now rejects a wrong return kind;
a new positive case checks independent lifecycle totals instead. The six-case
value-return/break rerun passed, including the repaired rejection and new case;
the bridge suite now contains 211 cases. It was not rerun in full a second time.

Six selected frontdoor cases cover multi-carry, fallible aggregates, counted
returns, full control composition and both shared work budgets. Five passed in
the combined run; the multi-carry case passed after updating its old pointer-word
call assertion to require the new two-field value return. Build/cache, tamper
rejection, project/source deletion and standalone materialization/execution remain
covered. The full composition retains its 51-function ceiling and byte-identical
restored LLVM, with explicit value-call/extraction assertions.

At that checkpoint the fresh tensor reported 1358 drift checks, zero failures and
clean coverage, hierarchy and lineage. The session coordinate remained active at
86, with final callback-state allocation next. The probes counted one real State
allocation/drop per successful callback in those fixtures, rather than one per
helper/iteration return. This is not a whole-workspace or all-28-frontdoor run,
nor fresh Linux/GPU, Windows, formal-safety, peak-memory or runtime-speed evidence.

Native callback value transport was checked on 2026-09-23 on macOS aarch64.
All 147 LLVM unit tests, the complete 212-case native-bridge suite, 89 Buffer-loop
tests, 46 ordinary control-flow tests, four compound-loop tests, one owned-cleanup
test and 26 tensor tests passed. The zero-allocation regression was first observed
failing on the old final-State path, then passed after the private return change.
The new 1/6/64-slot matrix executes 216 successful and 228 rejected callbacks,
checking complete backing buffers, boundary bytes, exact float bits and entry
counters. Separate 1/2/7/64-field probes also call a registered flat callback as a
helper and retain independent snapshots without aggregate allocation/drop.

Six selected frontdoor cases (multi-carry, fallible aggregates, counted returns,
full composition and both work budgets) passed build/cache, tamper rejection and
source-free standalone restoration. The full composition retains its 51-function
ceiling and byte-identical restored LLVM; compiled wrappers now require typed
value returns and no aggregate runtime calls. At that checkpoint the tensor reported
1363 drift checks, zero failures and clean coverage/hierarchy/lineage. The session
coordinate remained active at 86, with the reproduced reference constructor-order
gap next. That historical rejection is superseded by Reference State Normalization
above. This was not a full workspace/all-28-frontdoor
run and adds no Linux/GPU, Windows, formal-safety, peak-memory or speed certification.

Reference state normalization was checked on 2026-09-23 on macOS aarch64.
The prior inverse-constructor rejection was reproduced before the fix. Afterward,
84 runtime-host unit tests, 11 application-session and 52 provider-session tests,
546 lowering unit tests, 140 ordinary Buffer/control/cleanup regressions and 20
selected native-bridge tests passed. The latter include six reference lifecycle
regressions and the 1/6/64-slot native probe with reference open/event/close parity.
Nested argument effects were separately observed running early, then fixed by
ordering emitted field roots; inline and outlined source-order tests now pass.
Two selected CLI workflows (multi-carry and full composition) passed cache,
tamper and source-free artifact restoration checks. All 28 tensor-related tests
passed; that checkpoint reported 1368 drift checks, zero failures and clean coverage,
hierarchy and lineage. The session coordinate remains active at 86.
At that checkpoint the expanded image-session target had one passing and three
failing tests at `window_event`, also reproduced without the constructor-order
change. Guarded Local Values above now repairs that separate lowering gap.
Neither checkpoint certifies fresh Linux/GPU, Windows, formal safety or speed.

Guarded local values were checked on 2026-09-23 on macOS aarch64.
All 555 selected compiler-unit cases, 143 ordinary Buffer/control/cleanup cases,
five image-session cases and 20 selected native-bridge cases passed.
Two native CLI workflows retained cache, tamper and source-free restoration proof.
A separate headless build/run-artifact rerun verified two exact 76800-byte Metal
frames, live/replay equality, write/break counts and binary/YIR tamper rejection.
The 28 tensor tests passed; 1373 drift checks report zero failures with clean
coverage, hierarchy and lineage. Progress remains active at 86. This is not a full
workspace/native-bridge run or fresh Linux/CUDA, Windows or performance evidence.

Typed mixed/nested helper returns were checked on 2026-09-23 on macOS aarch64.
All 236 YIR-core/LLVM-lowering unit tests and 116 selected native-bridge regressions
passed, alongside three guarded-local-value native and five reference image/window
session cases. Three CLI workflows passed, including mixed/nested State-returning helpers,
cache reuse, tamper checks and byte-identical source-free LLVM restoration.
The 28 tensor tests passed; 1381 drift checks report zero failures with clean
coverage, hierarchy and lineage. The session coordinate remains active at 86.
At that checkpoint, the mixed/nested fallible-local-rebinding rejection was
reproduced separately. Guarded typed local values below now repair that boundary.
Neither checkpoint is a full workspace, fresh GPU/provider, cross-target or performance run.

The preceding guarded-local checkpoint on 2026-09-23 passed 125 compiler-unit,
42 native-bridge, four ordinary native, five image/window and three CLI regressions.
Its 1/6/63-slot probes covered snapshots, exact bits, zero allocations/drops,
unused traps, five-entry budgets and unchanged output sentinels. The CLI skipped
its zero-divisor arm; ten typed cases were rerun after bounding neutral expansion.
All 28 tensor tests and 1386 drift checks passed. That checkpoint exposed the
64-leaf mixed capture plus predicate boundary now repaired by private capture transport.

Private capture projection was checked on 2026-09-23 on macOS aarch64: 207 compiler/registry-unit,
45 native-bridge, eight ordinary native and five reference image/window cases passed.
Five CLI workflows passed, including exact-64-slot and sparse build/cache/tamper/source-free restoration.
All 28 tensor tests passed; 1400 drift checks report zero failures with clean coverage,
hierarchy and lineage. Sparse 64-i64 state now uses four private selection arguments.
Progress remains active at 86; the native 64-argument check is unchanged. This is not
fresh GPU/Linux, full-workspace or performance evidence.

## Next Boundary

Propagate field demand through immutable private capture aliases, preserving snapshots.
Sparse captures now project fields bottom-up before private boolean packing; user signatures
and scoped-action metadata stay unchanged. Whole 64-i64 records plus a predicate still exceed the bound.
Keep mixed/nested local values separate from the flat scoped-loop carry profile.
Keep guarded local values, selected-only failures and reference image/window regressions.
Retain reference named-field normalization, nominal/kind/malformed-state rejection,
source effect order and inverse-constructor native/reference parity. Retain typed
slot publication, input/output overlap, failure sentinels and allocation-free callback
and typed scalar helper return transport, native graph bounds, selected-path evaluation and
shared entry accounting. Retain the 51-function full-composition regression, single-use terminal
and statement evaluation, scope-aware local hygiene, unused calls, shared multi-use
suffixes, and the counted-return
source timing and propagation proof without changing the separate Buffer exit profile.
Retain leading/trailing step timing, leading-step break recovery, loop-local continue, each selected inner
invocation's preflight and both shared work counters. Keep resource
payloads and mixed/nested loop carries separate. Retain outer bool
carry seeds and explicit typed backedge conversion, outer flat-i64 carries,
zero-trip seeds, exact nominal reconstruction,
source-ordered backedge projections and local aggregate and bool rebinding,
shared helper-entry accounting, loop-work reservations
and independent per-invocation induction preflight; do not mistake these policies
for bounded total node work, memory or scheduling latency. Preserve
flat-i64 snapshots, exact nominal layouts, field/argument order and allocation cleanup,
temporary initialization, lexical visibility and stored snapshots.
Retain induction preflight, invariant header
inputs, exact layouts, source order and selected-path failure semantics. Keep
checked iteration expressions, flat-local and loop-call values, flat-helper, counted/carried/conditional/one-sided/compound/nested/sequence/temporary-aggregate
and scoped-break frontdoor/relocation regressions. Ordinary and resource-bearing
aggregate allocation remain separate optimization boundaries.
Whole-callback native scheduling limits, general loops, Buffer callbacks, resource
state, provider dispatch and ordinary image-host selection still need separate
implementation and evidence. The bridge alone does not impose call order; the
shared application session host does. Existing embedded-YIR image and guarded-loop
proofs remain required.
