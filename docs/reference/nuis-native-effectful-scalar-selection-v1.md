# Native Effectful Scalar Selection V1

This beta-0.16 worktree follow-up repairs source-to-YIR order propagation for
bounded scalar call selections. A prefix print, an effectful predicate, selected
argument calls, the selected callee and a suffix print retain their source order.
Branch-local work stays behind a guard rather than being eagerly evaluated before
a value select. Native print authority still comes only from the
[explicit build policy](nuis-native-literal-print-build-policy-v1.md).

## Source Boundary

The separate outliner handles top-level, two-sided selections with one matching
`let`, `const` or scalar `return` per arm. At least one arm must contain a call
not classified as a pure helper. Results and captured values use exact `bool`,
`i32`, `i64`, `f32` or `f64` types. Signature discovery does not approve a callee's
body, its effects or native capabilities; ordinary NIR verification and backend
admission remain authoritative. It also handles a top-level one-sided `let`
rebinding of an existing exact scalar. An empty `then` with a live `else` is
supported as well as a live `then` without an `else`.

Each expression proof is bounded to 256 nodes and fewer than 32 recursive levels.
An arm pair captures at most 31 existing scalar bindings plus one bool predicate.
The proof supports scalar literals, named values, exact synchronous call
signatures, supported scalar binary operations and i64/i32 conversions. Logical
`and`/`or`, resources, optional/reference/generic signatures and unsupported
expression forms do not gain this route. Pure selections keep their previous
normalization route. Nested existing-scalar updates use the separate bounded
proof below. Bounded sequential scalar bindings use the additional region proof.
Loop-local effects, nested fresh exports and nested returns/constants, general
multi-statement control flow and resource transport remain separate work.

For example:

```nuis
print(99);
let selected: i64 = if decide(gate) {
    yes(value, first(value, left_divisor), second(value, left_divisor))
} else {
    no(value, first(value, right_divisor), second(value, right_divisor))
};
print(80);
return selected;
```

The condition is evaluated once and stored. Two hygienic private arm helpers
receive that bool and only existing scalar captures. Each starts with the existing
`guard_return` contract and an exact typed zero for its inactive path. Original
calls, argument calls and arithmetic remain after the guard. Private names avoid
source functions and bindings; captures use the current binding version.

Both guard helpers are invoked in order. The inactive helper enters and returns
its neutral value, consuming its helper-entry budget, but does not evaluate the
original argument expressions or original callee. The selected result is then
chosen from already completed scalar values. This is not a new conditional-call
opcode, wildcard effect grant or scheduler/runtime capability.

## One-Sided Rebinding

An empty arm retains the pre-branch binding, not a typed zero. Normalization
synthesizes a scalar `Var` capture for that path and then applies the same bounded
two-arm proof. The retained value counts toward the existing 31-capture limit,
even when the active expression does not use it. A fresh branch-local name cannot
be exported by this route. Declared and inferred types must match the existing
binding exactly. No resource, async or effect capability is granted by retention.

```nuis
let saved: i64 = value;
let saved: i64 = saved + 1;
if decide(gate) { let saved: i64 = work(saved); }
if decide(second_gate) {} else { let saved: i64 = other(saved); }
return saved;
```

Each predicate runs once. The first inactive path retains `value + 1`; the second
captures the value after the first selection, including an actual first update.
The retained helper also consumes a helper entry and uses the unchanged leading
guard. Original active arguments and arithmetic remain behind their own guard.
This is bounded sequential scalar rebinding, not general nested control flow.

## Nested Existing-Scalar Updates

A separate recursive proof handles nested `if` trees that update the same existing
exact scalar on every nonempty leaf. Each arm has one nested selection, one `let`
update, or an empty path retaining that binding. It accepts either polarity and
paired leaf updates; the additional region proof below admits bounded sequential
binding leaves, not fresh exports or nested return/constant/resource admission. Fully pure trees keep
their previous normalization owner. Effectful descendant predicates are guarded
even when the final value computation is pure.

The proof has eight selection levels and 64 tree nodes. A shared 256-expression-node budget
covers conditions, leaf values and synthesized retained values. Each expression
retains the existing fewer-than-32-level limit. Every
selection keeps the 31-capture limit plus its stored bool gate; ancestor captures
include descendant condition inputs, not just final value inputs. Existing exact
sync signatures and native effect capability checks remain separate.

Nested plans are installed inside their selected ancestor helper, after its
leading `guard_return`. The inner predicate and its argument work are not
evaluated in the parent. Its own two private arm helpers are called only after
that predicate runs. This composition propagates ancestor control without a new
mask opcode, runtime ABI or backend-specific rule. An inactive outer wrapper
consumes an entry but does not enter its child subtree; each reached inner pair
still consumes both wrapper entries under the original budget.

The later scalar-control pass respects registered guard boundaries rather than
re-extracting their leading guard or relying on a private name prefix. A separate
regression uses a user-named registered boundary and an ordinary peer to verify
that protection follows metadata, while ordinary extraction still runs.

```nuis
let saved: i64 = value + 1;
if outer_condition(outer) {
    if inner_condition(inner) {
        if leaf_condition(leaf) { let saved = yes(saved); }
        else { let saved = no(saved); }
    }
} else {
    if other_condition(inner) {} else { let saved = no(saved); }
}
return saved;
```

## Sequential Scalar Regions

The separate region proof admits 2 through 16 sequential `let` statements as a
selection leaf. Its last statement must update one existing exact scalar. Earlier
statements may update that same target or create/rebind private scalar staging
names, but may not write a different outer binding. Each declared, inferred and
rebound type must match exactly. Signature discovery remains separate from effect
authority; at least one original call in the tree is not classified as pure.

RHS expressions are checked against the current local scope before installing
the LHS binding. Capture discovery removes names only after their first local
definition, so earlier reads still capture outer values and later reads use staged
versions. Private names never enter the parent scope or helper parameter list.
Original statements, including unused effectful calls, are installed in order
after the unchanged leading guard and return only the final target's scalar value.
They are not substituted into later expressions or hoisted out of the guard.

The eight-level/64-node tree proof shares its existing expression budget across
all conditions and staging RHS values; every staged statement consumes a node.
The existing 31-capture limit applies to external reads, including retention and
descendant conditions, not the number of private temporaries. Unsupported work
leaves the original NIR unchanged; this proof cannot grant an alternative eager path.

```nuis
let saved: i64 = value + 1;
if outer_condition(outer) {
    if inner_condition(inner) {
        let unused = probe(saved);
        let staged = first(saved, divisor);
        let saved = saved + staged;
        let staged = second(saved, divisor);
        let staged = yes(staged);
        let saved = saved + staged;
    }
}
return saved;
```

This leaf proof does not admit `const`, standalone print statements, mixed staging followed
by nested selections, loops, returns, resource/reference work, fresh outer exports
or writes to multiple outer bindings. The separate prefix-before-child proof below
adds only its independently checked shape; there is no eager fallback.

## Staging Before Child Selections

An additional proof admits 1 through 16 scalar `let` statements followed by one
final child `if`. It reuses the versioned staging proof, then proves the child in
the updated local scope with the same cumulative node/expression/depth budget.
Every child path still resolves to a single existing outer target of the original
exact scalar type. Other outer writes and fresh/private final exports reject.

The child captures the new local versions, including private staged values.
The ancestor captures only reads that occurred before their local definitions.
Private values are forwarded to child helpers inside the ancestor, never exported
to its parent. Each helper still has at most 31 scalar captures plus its bool gate;
forwarded private values count at the child level. Rebinding a different enclosing
binding, including a private value defined by an earlier ancestor, is not admitted.

The prefix is installed after the registered ancestor guard and before the child
predicate and helper calls. If the ancestor is inactive, no prefix or descendant
work runs. If the prefix runs but the child update is skipped, retention uses the
target's value after the prefix rather than its original ancestor value. The
inactive outer path independently retains the original pre-prefix target.

```nuis
let saved: i64 = value + 1;
if active_condition(enabled) {
    let staged = first(saved, prefix_divisor);
    let saved = saved + staged;
    let staged = second(saved);
    if child_condition(child, staged, saved) {
        let saved = leaf(saved, staged / leaf_divisor);
    }
}
return saved;
```

This composes with paired and either-polarity one-sided child selections, nested
guard trees and already proved sequential leaves. It adds no opcode, ABI,
backend-specific rule or effect grant. `const`, standalone effect statements,
returns, loops, resources, multiple outer results and suffix statements after the child
remain outside this staging-only proof. The separate continued-region proof below
admits its own bounded suffix shape without an eager fallback.

## Scalar Suffixes After Child Selections

A separate proof admits zero through 16 scalar prefix bindings, one child `if`
and 1 through 16 scalar suffix bindings. The child and the final suffix binding
must update the same single existing outer target with its original exact scalar
type. All other existing outer writes, fresh final exports, additional child
selections and non-`let` prefix/suffix statements reject. A child-private binding
cannot escape into the suffix; private prefix bindings remain local to this
selected outer region and can be rebound by the suffix.

Prefix, child and suffix share the existing 64-node/256-expression budget and
eight-selection depth limit. Each generated helper retains its 31-capture limit
plus bool gate, including forwarded private captures. Shape dispatch selects the
proof before consuming any budget; failed proofs do not partially rewrite NIR.
Suffix statements do not create new selection levels or reset work limits.

The installer emits the ancestor guard, prefix, complete child computation and
its actual merged target, then the suffix in source order. Suffix RHS reads use
that merged value, not another capture of its older ancestor version. An inactive
child still reaches the suffix with the retained post-prefix value. An inactive
outer region reaches neither its child nor its suffix and retains the original
target. Calls whose results are unused remain ordered work; a trap in selected
prefix, predicate, child argument or suffix prevents subsequent effects and state
publication but does not roll back completed host effects.

```nuis
let saved: i64 = value + 1;
if active_condition(enabled) {
    let staged = first(saved, prefix_divisor);
    let saved = saved + staged;
    let staged = second(saved);
    if child_condition(child, staged, saved) {
        let saved = leaf(saved, staged / leaf_divisor);
    }
    let unused = tail_probe(saved);
    let staged = finish(saved + staged / argument_divisor, suffix_divisor);
    let saved = publish(saved + staged);
}
return saved;
```

This adds no opcode, ABI, backend-specific rule, helper-entry allowance or effect
grant. Repeated sibling child selections are outside this one-child proof; the
separate proof below admits a bounded two-child shape. Multi-target regions,
resource work, loops and exits remain independently unproved rather than falling
back eagerly.

## Repeated Child Selections

A separate region proof admits exactly two immediate child `if` nodes updating
the same single existing outer target with its original exact scalar type.
The prefix contains zero through 16 scalar `let` statements. A mandatory
intermediate stage and final suffix each contain 1 through 16 scalar `let`
statements; the suffix ends by rebinding that target. Both children may be paired
or either-polarity one-sided selections, including already proved nested shapes.
All other existing outer writes, fresh final exports, child-private escapes,
non-`let` stages, adjacent children, third immediate children and missing suffixes
reject atomically. This is a proof boundary, not a fixed backend combination.

Each stage reads its current RHS bindings before installing the LHS. After each
child, only its merged target enters the enclosing region scope, never its private
bindings. Private enclosing staging can be rebound between children and forwarded
to the next child without becoming an outer export. The second predicate, its
arguments, its retained arm and the suffix all consume current versions. Skipping
the first child retains the post-prefix value, and skipping the second retains
the post-intermediate value; skipping the ancestor evaluates neither child nor
any region stage and retains the original outer value.

The installer emits the ancestor guard, prefix, first child and actual merge,
intermediate stage, second child and actual merge, then the suffix in source
order. Both predicates run once only when their region is reached. Original
calls and fallible argument computations stay behind the selected child guard.
The full proof shares 64 nodes and 256 expressions; sibling selections use the
same selection depth rather than adding artificial nesting. The eight-level
selection depth and per-helper 31 scalar captures plus bool gate remain intact.
No opcode, ABI, effect grant, entry allowance or resource/exit authority changes.

```nuis
let saved: i64 = value + 1;
if active_condition(enabled) {
    let staged = prefix(saved, prefix_divisor);
    let saved = saved + staged;
    if first_condition(first) {
        let saved = first_leaf(saved, saved / first_divisor);
    }
    let staged = middle(saved, middle_divisor);
    let saved = saved + staged;
    if second_condition(second, staged, saved, predicate_divisor) {
        let saved = second_leaf(saved, staged / second_divisor);
    }
    let staged = suffix(saved + staged, suffix_divisor);
    let saved = publish(saved + staged);
}
return saved;
```

General ordered sibling sequences, adjacent child nodes, final-child regions,
multiple published targets, resources, loops and exits remain separate work.

## Effect Order

Direct call emission now advances the statement-effect frontier, including
unused results and calls whose bodies may hide effects or traps. Call argument
roots are ordered after the preceding effect frontier; nested argument calls
remain left to right. Division and remainder also advance that frontier because
a native trap must not move across a preceding print or a following effect.
Inlined cleanup retains the existing protection against backedges to earlier
result values. Ordering is local to the function lane, not a cross-function
initialization dependency.

The old eager matching-binding fallback refuses effectful expressions outside
the guarded proof. The same-callee shortcut cannot eagerly select effectful
arguments. Unsupported cases reject rather than being serialized and incorrectly
executing both arms.

Native admission still checks explicit per-site literal grants and reachability
order against calls, effects, traps and returns. Removing order edges from a
compiled graph must fail admission. Pure packaging still rejects prints, and an
empty explicit policy grants nothing. Checkpoint verification regenerates LLVM
from the bound canonical YIR and exact policy; the outliner cannot waive it.

## Execution Fixture

The shared [Nuis fixture](../../tools/nuisc/tests/native_application_bridge/effectful_selected_calls.ns)
uses a printed predicate, two printed argument helpers with integer division,
different printed callees and a suffix print. With an inactive divisor of zero,
successful execution must show exactly `99`, `98`, `61`, `62`, the selected `70`
or `71`, and `80`, in that order. The original inactive callee and its arguments
must not run. With a selected divisor of zero, native execution must trap before
the second argument, callee, suffix or state publication. Host stdio can buffer
already completed prints on a fatal trap; this is not an output rollback contract.

The CLI workflow covers both branch directions, alternating event branches,
fresh builds, cache reuse, deletion of source/project manifest/project cache and
three standalone restoration cycles. It verifies the restored policy and exact
executable bytes before running again. Compiler tests separately check the
guard-to-argument-to-callee effect chain, explicit-policy rejection and source-free
checkpoint verification. NIR tests cover all five exact scalar zero kinds,
return/let/const destinations, private-name hygiene, unsupported signatures and
depth/node bounds. These are separate from publisher authentication and proof
that arbitrary machine code implements its declared LLVM.

The separate [rebinding fixture](../../tools/nuisc/tests/native_application_bridge/effectful_scalar_rebindings.ns)
adds consecutive same-name bindings and independently controlled one-sided
updates. For seed `10`, neither selected update preserves `11`, the first alone
produces `21`, the second alone produces `5`, and both produce `7`. Inactive
divisors are zero; both conditions still print exactly once. Its real CLI test
covers fresh/cache execution and three standalone restoration cycles after
deleting source, manifest and `.nuis`, checking 80 successful transitions. Two
selected zero-divisor cases must trap before suffix effects or publication.
Separate emitted-bridge tests cover both polarities for all five scalar kinds;
the new runtime fixture remains i64 rather than claiming native float coverage.

The [nested fixture](../../tools/nuisc/tests/native_application_bridge/nested_effectful_scalar_selections.ns)
adds three selection levels, separate printed conditions, a printed condition
argument, paired leaf callees and single-sided retention. All eight bool
combinations use zero divisors in inactive condition and argument paths. Selected
updates produce `21` or `5`; skipped updates retain the current `11`. Exact stdout
checks each reached condition once, excludes inactive predicate/argument effects
and preserves prefix-to-condition-to-callee-to-suffix order.

Its real CLI workflow checks 160 successful transitions across fresh/cache/three
source-free restored runs with exact policy and executable bytes. Six negative
scripts cover the outer predicate, each inner predicate, the third-level predicate
and each selected leaf's first division argument, requiring native traps before
suffix effects and publication. Already completed host prints are not rolled back.
Separate emitted-bridge tests cover nested polarities and all five scalar kinds;
the runtime fixture remains i64.

The [sequential region fixture](../../tools/nuisc/tests/native_application_bridge/sequential_effectful_scalar_regions.ns)
adds private staging, repeated target/staging rebindings and unused prefix calls
inside paired inner leaves. For seed `10`, the add path uses `11 -> 16 -> 32`, the
subtract path uses `11 -> 6 -> 0`, and an inactive outer path retains `11`. Both
divisors are zero when the outer region is inactive. Exact stdout checks prefix
`60`, first-stage `61`, later-stage `62`, selected `70`/`71` and suffix `80` order;
an inactive outer path also skips the inner predicate `98`.

Its CLI workflow checks fresh/cache/three source-free restored executions,
unchanged policy and executable bytes, and selected prefix/later-stage native
traps before suffix effects or publication. Exact-five-kind emitted-bridge checks
are separate from this i64 runtime fixture. The full acceptance receipt follows
the historical checkpoints below.

The [staged child fixture](../../tools/nuisc/tests/native_application_bridge/staged_effectful_scalar_selections.ns)
checks all eight enabled/outer/child combinations. With seed `10`, full skipping
retains `11`; the left prefix changes `11 -> 16` and its skipped child retains
`16`, while its selected child returns `32`. The right prefix changes `11 -> 6`
and its skipped child retains `6`, while its selected opposite-polarity child
returns `12`. Both predicates use newly staged data and the current target value.
Inactive prefix, condition and leaf divisors are zero. Exact stdout checks unused
prefix effects, each reached condition once and ordered selected work.

Its real CLI workflow checks 160 successful transitions across fresh/cache/three
source-free restored runs with exact policy and binary bytes. Six selected traps
cover prefix calls, child predicates and child arguments on both outer paths,
before suffix effects or publication. Completed host effects are not rolled back.
Separate emitted-bridge checks cover five exact scalar kinds and both branch
polarities; this runtime fixture remains i64.

The [continued child fixture](../../tools/nuisc/tests/native_application_bridge/continued_effectful_scalar_selections.ns)
uses the selected or retained child target in a later private staging calculation.
For seed `10`, a fully skipped outer subtree retains `11`. The left path reaches
`32` or retains `16` before its suffix, then publishes `64` or `40`. The right
path reaches `12` or retains `6`, then publishes `24` or `15`. Suffix argument
and callee divisors are independently poisoned on inactive paths. Exact stdout
checks unused suffix `65`, argument division, suffix callee `63`, final work `64`
and outer suffix `80` after the complete child computation.

Its real CLI workflow checks 160 successful transitions across fresh/cache/three
source-free restored executions with exact policy and executable bytes. Ten
selected trap scripts cover prefix, predicate, child argument, suffix argument
and suffix callee on both outer paths; suffix argument traps also follow a skipped
child. The final work and state publication must not run after a trap.
Fatal-signal checks do not require completed host prints to have been flushed;
success traces and YIR effect edges establish their order. Separate
typed emission checks cover all five scalar kinds, both child/outer polarities and
the optional prefix; this runtime fixture remains i64.

The [repeated child fixture](../../tools/nuisc/tests/native_application_bridge/repeated_effectful_scalar_selections.ns)
checks all 16 enabled/outer/first/second combinations. A fully skipped region
retains `11`. The left prefix produces `16`, the right prefix `6`; first-child
updates and retention feed a private middle binding and another target update.
The second printed predicate checks that the current target divided by two equals
that middle binding. Both child directions are inverted on the right path.
All inactive prefix, predicate, middle, leaf-argument and suffix divisors are
zero. Exact stdout and complete state checks distinguish every selected/skipped
combination and preserve the middle-to-second-predicate-to-suffix order.

Its CLI workflow checks 320 successful transitions across fresh/cache/three
source-free restored executions, checking exact policy and executable bytes after
deleting source, manifest and project cache. Twelve selected trap scripts cover
prefix, first argument, intermediate stage, second predicate, second argument
and suffix on both outer paths, before final work or publication. Completed host
prints need not flush on a fatal signal. Separate emitted-bridge checks cover all
five scalar kinds, optional prefixes and all eight outer/first/second polarities;
this runtime fixture remains i64.

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- repeated_effectful_scalar_selections native_repeated_effectful_scalar_selections continued_effectful_scalar_selections native_continued_effectful_scalar_selections staged_effectful_scalar_selections native_staged_effectful_scalar_selections sequential_effectful_scalar_regions native_sequential_effectful_scalar_regions effectful_selection effectful_scalar_rebindings native_effectful_selected_calls native_effectful_scalar_rebindings native_nested_effectful_scalar_selections scalar_control::tests aot_application_bundle --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- native_repeated_effectful_scalar_selections native_continued_effectful_scalar_selections native_staged_effectful_scalar_selections native_sequential_effectful_scalar_regions native_effectful_selected_calls native_effectful_scalar_rebindings native_nested_effectful_scalar_selections native_literal_print_build --test-threads=1 --quiet
```

This is a restricted scalar lowering repair, not general application effects,
GPU callback execution, new Linux/Windows acceptance, completed self-hosting,
measured performance improvement or a formal memory-safety certificate.

## Previous Selection Acceptance

Guarded scalar-selection acceptance: The 2026-10-07 final selected acceptance
passes 460 distinct tests across completed cohorts:

* 198 compiler/outliner/artifact/direct-call/branch/async/loop tests
* Eight typed selected-return and pure/effect-boundary compiler tests
* 173 LLVM lowering tests, including ignored tests
* 50 native bridge and ordinary-AOT regression tests
* Four real CLI policy/selection workflows and 27 tensor tests

The new selection workflow checks 80 successful registered open/event/close
transitions across fresh/cache/three restored runs, plus native division traps
on both selected paths. Together the four workflow tests check 208 successful
transitions. Source, project manifest and project-local cache are absent before
the selection restoration cycles. Default pure admission, missing grants and
removed effect-order edges continue to reject. Five exact scalar guard/callback
kinds are checked separately; the new CLI execution fixture uses i64 state.

Static checks pass 2141 drift definitions/14570 literal patterns, all 122
changed/new file caps, 4778 recognized UTF-8 files and 4746 local Markdown links.
At that checkpoint, the selected tensor cell remained `active/99`, preserving all
six complete preceding build-policy fields. Its next action was independently
proved one-sided and nested scalar selections, not relaxed admission or eager fallback. This
worktree receipt does not extend the committed beta-0.16.0 baseline or add
historical test counts. Historical socket and strict-Clippy findings remain
pending.

## Previous Rebinding Acceptance

One-sided scalar-rebinding acceptance: The 2026-10-07 completed selected cohorts
pass 467 distinct tests: 203 compiler/outliner/artifact/direct-call/branch/async/loop
units, eight typed selected-return/effect-boundary units, 173 LLVM tests including
ignored cases, 50 native bridge/ordinary-AOT regressions, six real CLI workflows
and 27 tensor tests. This replaces the current receipt rather than adding the
preceding 460-test checkpoint to the count.

The new fixture contributes 80 successful transitions across fresh/cache/three
restored executions; the six workflows together check 288 successful transitions.
Both selected update polarities also produce native division traps before suffix
effects or state publication. Restored policy and executable bytes are checked
after deleting source, manifest and project cache. Exact typed emission proof is
separate from the new i64 native fixture and does not claim float runtime coverage.

Static checks pass 2147 drift definitions/14609 patterns, 122 changed/new file
caps, 4779 recognized UTF-8 files and 4747 local Markdown links. The selected
tensor stayed `active/99`, preserving all six complete guarded-selection fields
with a rebuilt-CLI SHA-256 audit. The next task was independently proved nested
guard masks, including inactive inner predicates, not relaxed policy or broader
resource transport. Historical socket and strict-Clippy findings remain pending.

## Previous Nested Update Acceptance

Nested scalar-update acceptance: The 2026-10-07 final selected acceptance passes
488 distinct tests across completed cohorts: 222 compiler/outliner/artifact/
direct-call/branch/async/loop/scalar-control units, eight typed selected-return/
effect-boundary units, 173 LLVM tests including ignored cases, 50 native bridge/
ordinary-AOT regressions, eight real CLI workflows and 27 tensor tests. Compiler
and typed-return test-name lists were audited for zero overlap. Targeted reruns
and previous checkpoint counts are not added to this receipt.

The eight real workflows check 448 successful registered transitions. The new
nested fixture contributes 160 across fresh/cache/three source-free restored
executions, checking policy and executable bytes after deleting source, manifest
and project cache. Six selected predicate/leaf-boundary cases must trap before
suffix effects or publication. Unselected outer subtrees do not evaluate their
descendant predicates or leaf arguments. The new native fixture uses i64;
all-five-scalar-kind emitted-bridge proof remains separate from runtime coverage.

Static checks pass 2156 drift definitions/14671 patterns, all 126 changed/new file
caps, 4782 recognized UTF-8 files and 4748 local Markdown links. At that checkpoint,
the selected tensor cell remained `active/99`, with all six complete preceding
one-sided scalar-rebinding fields preserved and checked by SHA-256 through the rebuilt CLI.
Its next task was independently proved sequential statements inside guarded scalar
regions, including current-version capture, ordered effects, selected traps and
private staging, without relaxing admission or resource/exit boundaries.

This receipt covers bounded single-statement leaves updating existing scalars,
not general multi-statement control flow, new GPU/Linux/Windows execution,
publisher authentication, formal memory-safety proof, measured performance gains
or completed self-hosting. Historical socket and strict-Clippy findings remain
pending. It does not widen the committed beta-0.16.0 baseline.

## Previous Sequential Region Acceptance

Sequential scalar-region acceptance: The 2026-10-07 final selected acceptance
passes 496 distinct tests across completed cohorts: 228 compiler/outliner/artifact/
direct-call/branch/async/loop/scalar-control units, eight typed selected-return/
effect-boundary units, 173 LLVM tests including ignored cases, 50 native bridge/
ordinary-AOT regressions, ten real CLI workflows and 27 tensor tests. Compiler
and typed-return test-name lists have zero overlap. Targeted reruns and previous
checkpoint counts are not added to this receipt.

The ten workflows check 528 successful registered transitions. The new sequential
fixture contributes 80 across fresh/cache/three source-free restored executions,
checking exact policy and executable bytes. Four selected prefix/later-stage
division cases must trap before suffix effects or publication. Exact stdout
checks unused prefix calls, current-version argument reads and inactive outer
masking. Five exact scalar kinds have separate emitted-bridge evidence; the new
native execution fixture remains i64, not float runtime acceptance.

Static checks pass 2164 drift definitions/14729 patterns, all 130 changed/new file
caps, 4787 recognized UTF-8 files and 4749 local Markdown links. The selected
tensor stayed `active/99`, with all six complete preceding nested-update fields
preserved and checked by SHA-256 through the rebuilt CLI. Its next task was
independently proved scalar staging before nested guarded selections, including
new-version child captures and retention after an executed prefix.

This is bounded ordered scalar binding, not general mixed control flow, resource
transport, new GPU/Linux/Windows execution, publisher authentication, formal
memory-safety proof, measured performance gains or completed self-hosting.
Historical socket and strict-Clippy findings remain pending. The committed
beta-0.16.0 baseline is unchanged.

## Previous Staged Child Acceptance

Staged child-selection acceptance: The 2026-10-07 final selected acceptance passes
504 distinct tests across completed cohorts: 234 compiler/outliner/artifact/
direct-call/branch/async/loop/scalar-control units, eight typed selected-return/
effect-boundary units, 173 LLVM tests including ignored cases, 50 native bridge/
ordinary-AOT regressions, 12 real CLI workflows and 27 tensor tests. Compiler
and typed-return test-name lists have zero overlap. Targeted reruns and previous
checkpoint counts are not added to this receipt.

The 12 workflows check 688 successful registered transitions. The new staged
fixture contributes 160 across fresh/cache/three source-free restored executions,
checking exact policy and executable bytes after deleting source, manifest and
project cache. Six selected prefix/predicate/child-argument cases trap before
suffix effects or publication. Both child predicates read private staged values
and the current target version. An inactive outer subtree retains the original
target without running its prefix; an inactive child after a reached prefix
retains the updated target. Completed host effects are not rolled back.
Five exact scalar kinds have separate emitted-bridge evidence; the new native
runtime fixture remains i64, not float runtime acceptance.

Static checks pass 2172 registered drift checks, all 134 changed/new source,
test and Markdown file caps, 4846 recognized UTF-8 files and 3218 README/docs
local links. A supplementary whole-worktree Markdown audit passes 4750 local
targets. The rebuilt CLI confirms clean drift, coverage, hierarchy and task-card
lineage. At that checkpoint, the selected tensor cell remained `active/99`, with all six complete
preceding sequential-region fields preserved and checked by SHA-256. Its next
task was independently proved scalar suffixes after nested guarded selections,
including selected/post-prefix retained target versions and private staging,
not relaxed admission or broader resource/exit transport.

This is bounded scalar staging followed by a final child selection, not general
mixed control flow, new GPU/Linux/Windows execution, publisher authentication,
formal memory-safety proof, measured performance gains or completed self-hosting.
Historical socket and strict-Clippy findings remain pending. The committed
beta-0.16.0 baseline is unchanged.

## Previous Continued Child Acceptance

Continued child-selection acceptance: The 2026-10-07 final selected acceptance
passes 512 distinct tests across completed cohorts: 240 compiler/outliner/artifact/
direct-call/branch/async/loop/scalar-control units, eight typed selected-return/
effect-boundary units, 173 LLVM tests including ignored cases, 50 native bridge/
ordinary-AOT regressions, 14 real CLI workflows and 27 tensor tests. Compiler
and typed-return test-name lists have zero overlap. Targeted reruns and previous
checkpoint counts are not added to this receipt.

The 14 workflows check 848 successful registered transitions. The new continued
fixture contributes 160 across fresh/cache/three source-free restored executions,
checking exact policy and executable bytes after deleting source, manifest and
project cache. Ten selected prefix/predicate/child-argument/suffix-argument/
suffix-callee cases trap before final work or publication. An inactive outer
subtree reaches neither child nor suffix; a skipped child reaches the suffix
with the retained current target. Private prefix staging remains local and
current through its suffix rebinding. Success traces prove exact effect order;
fatal signals and no publication establish traps without requiring buffered
completed prints to have been flushed. Separate NIR merge-order and YIR Effect-edge checks place
suffix argument division after the child's merge and previous suffix call.
Five exact scalar kinds have separate emitted-bridge evidence with optional
prefixes and both polarities; the new native runtime fixture remains i64.

Static checks pass 2180 registered drift checks, all 138 changed/new source,
test and Markdown file caps, 4851 recognized UTF-8 files and 3219 README/docs
local links. A supplementary whole-worktree Markdown audit passes 4751 local
targets. The rebuilt CLI confirms clean drift, coverage, hierarchy and task-card
lineage. The selected tensor cell remains `active/99`, with all six complete
preceding staged-child fields preserved and checked by SHA-256. Its next task is
independently proved repeated scalar child selections inside one guarded region,
not relaxed admission or broader resource/exit authority.

This is bounded single-child scalar continuation, not general mixed control
flow, new GPU/Linux/Windows execution, publisher authentication, formal memory-
safety proof, measured performance gains or completed self-hosting. Historical
socket and strict-Clippy findings remain pending. The committed beta-0.16.0
baseline is unchanged.

## Repeated Child Acceptance

Repeated child-selection acceptance: The 2026-10-07 final selected acceptance
passes 520 distinct tests across completed cohorts: 246 compiler/outliner/artifact/
direct-call/branch/async/loop/scalar-control units, eight typed selected-return/
effect-boundary units, 173 LLVM tests including ignored cases, 50 native bridge/
ordinary-AOT regressions, 16 real CLI workflows and 27 tensor tests. Exact compiler
and typed-return test-name lists have zero overlap. Targeted reruns and previous
checkpoint counts are not added to this receipt.

The 16 workflows check 1168 successful registered transitions. The new repeated
fixture contributes 320 across fresh/cache/three source-free restored executions,
checking exact policy and executable bytes after deleting source, manifest and
project cache. Twelve selected prefix/first-argument/intermediate-stage/second-
predicate/second-argument/suffix cases trap before final work or publication.
All 16 enabled/outer/first/second combinations check complete state and exact
stdout, including private middle staging, current target reads and inactive
zero-divisor paths. Fatal signals do not require completed prints to flush.

NIR proof tests cover actual merges, exact scalar types, optional prefixes, all
selection polarities, hygiene, private child-scope vetoes and atomic rejection.
All stages and siblings share node/expression limits, selection depth and capture
limits. Separate NIR checks place intermediate/suffix work after actual merges;
YIR Effect edges independently order guards, predicates, helper calls and later
work. Default grants and removed-order graphs still veto. Five exact scalar kinds
have separate emitted-bridge evidence; this native runtime fixture remains i64.

Static checks pass 2188 registered drift checks, all 142 changed/new Rust/Markdown
file caps, 4856 recognized UTF-8 files and 3220 README/docs local links. A
supplementary whole-worktree Markdown audit passes 4752 local targets. The rebuilt
CLI confirms clean drift, coverage, hierarchy and task-card lineage. The selected
tensor cell remains `active/99`, with all six complete preceding continued-child
fields preserved and checked by SHA-256. Its next task is an independently proved
bounded ordered scalar-region walk covering adjacent/more/final children without
relaxing effect, resource, exit or multi-target authority.

This remains bounded two-child scalar continuation, not general mixed control
flow, new GPU/Linux/Windows execution, publisher authentication, formal memory-
safety proof, measured performance gains or completed self-hosting. Historical
socket and strict-Clippy findings remain pending. The committed beta-0.16.0
baseline is unchanged.
