# Beta 0.16 Snapshot

This minor-line anchor records the existing Git commit `dff1bdbc`
(`beta-0.16.0`, 2026-10-07). It is not another release, a compatibility freeze
or a completed self-hosting announcement. Cargo package and protocol versions
remain independent from the project release.

## Direction And Recorded Capability

The governing direction remains
[ns-nova application-led development](nuis-beta-0.11-application-led-mainline.md).
The application goal is `standard-library/ns-nova/interactive-image-workflow`;
its selected prerequisite is
`standard-library/ns-nova/persistent-application-session`, still `active/99`.
Foundation proofs do not by themselves certify complete application closure.

This baseline includes bounded CPU continuing effect regions, source exits,
exit-only continuations and fully paired, partially exiting and one-sided
bool/i64 result joins. Conditions and admitted checked/call-backed work are
saved once at their selected source positions. Actual prints stay in the parent,
source exits retain their identity, and merged live paths authorize result use.

Control-expression tails are expression statements; explicit `return` remains
an enclosing-function exit. Typed let/const results publish after branch
validation, with separate generic/lambda result-tail context and unchanged
non-tail call authority. Other child locals do not become parent-scope values.

The original and expanded work bounds remain 32 statements, 4096 expression
nodes, depth below 64 and 32 logical edges. Resource/borrow/FFI transport,
rebinding and general effectful helpers remain outside these bounded proofs.

## Evidence At The Baseline

The committed [one-sided result proof](../reference/nuis-native-one-sided-effect-result-joins-v1.md)
records 1042 distinct selected local macOS aarch64 tests: 947 return/frontend
units, 58 native bridge tests, 29 tensor tests, two source-free workflows,
five application-session tests and one host-path policy test. Its source matrix,
native AOT variants, initially found consumer regressions and subsequent repairs
retain their original scope and date. The accompanying 2044 drift definitions
are historical, not the current live drift count.

The earlier [beta-0.15 snapshot](nuis-beta-0.15.0-snapshot.md) records `05951bef`;
the [beta-0.15.9 patch](nuis-beta-0.15.9-patch.md) records `585eb409`. Neither is
rewritten into this checkpoint. Test totals from different receipts must not
be added into one acceptance claim.

The subsequent [single-value capture optimization](../reference/nuis-native-effect-join-captures-v1.md)
and [equal-atom follow-up](../reference/nuis-native-equal-effect-result-joins-v1.md)
are worktree development after `dff1bdbc`, with independent receipts. They do not
belong to the committed baseline's 1042-test count.
The [typed effect-snapshot extension](../reference/nuis-native-typed-effect-snapshots-v1.md)
is another independently tested worktree follow-up, not part of that baseline.
The [floating literal negation repair](../reference/nuis-native-float-literal-negation-v1.md)
subsequently closes its source signed-zero gap with a separate bit-pattern proof;
it is not folded into the committed baseline or preceding receipt.
The [nonliteral sign-negation follow-up](../reference/nuis-native-float-sign-negation-v1.md)
separately repairs variable/call zero signs and NaN payload transport without
widening enclosing return-tail profiles or the committed baseline receipt.
The [typed selected-return handoff](../reference/nuis-native-typed-effect-returns-v1.md)
then extends exact scalar enclosing returns for selected regions, with ordinary
AOT returned-word evidence and unchanged pure native session effect vetoes.
Its independent worktree receipt is not folded into the committed baseline.
The [explicit literal-print policy](../reference/nuis-native-literal-print-policy-v1.md)
subsequently connects constant i64 effects to registered native scalar callbacks,
with individual grants and dependency order proofs. Default pure admission and
the committed baseline receipt remain unchanged. Its
[explicit build policy follow-up](../reference/nuis-native-literal-print-build-policy-v1.md)
binds grants and work limits across cache, standalone restoration and verified
launch. This worktree transport receipt is not part of the committed baseline.
The [guarded scalar-call follow-up](../reference/nuis-native-effectful-scalar-selection-v1.md)
then repairs two-sided source call selections, argument laziness and effect/trap
order without changing default effect authority. The one-sided follow-up retains
existing scalar values across consecutive selected/skipped updates using the same
guard contract. Nested existing-scalar updates compose those guards without
hoisting descendant conditions. Bounded sequential leaves additionally keep
private scalar staging, current-version reads and unused effects behind guards.
Staging before a final child selection now preserves new-version captures and
post-prefix retention. A separate bounded scalar suffix proof now consumes the
child's merged target and private prefix staging without widening resource,
exit, effect-grant or multi-target authority.
Two sibling selections separated by bounded scalar staging now have their own
region proof and native fresh/cache/three source-free restoration evidence;
second predicates consume current target versions rather than ancestor captures.
The ordered-region worktree follow-up replaces all three specialized production
proofs with a shared walk, admitting adjacent/more/final children under unchanged
work/capture/depth, single-target and effect-authority limits. Historical receipts
remain separate from this newer acceptance.
The subsequent single-edge logical-child proof preserves owned-bool atom/literal
left gates and complete selected scalar RHS work within that region. Its receipt
is independent of the ordered-region receipt and the committed minor baseline.
The computed-left worktree extension adds once-only bounded nonlogical bool
calls/comparisons and checked arguments at original child sites, with left-only
effect discovery, unchanged guarded RHS work, source-free execution and selected
left/RHS/leaf traps. Nested logical operands remain a separate task.
The subsequent tree follow-up covers direct logical children under a shared
32-edge region budget and original expression/depth/capture limits, with linear
helper growth and separately guarded complete RHS subtrees. Hidden logical
leaves remain outside this proof. The subsequent fresh-bool staging extension
admits direct logical let/const initializer roots in selected scalar leaves and
prefix/middle/suffix stages, reusing shared predicate budgets and guards. It
retains private original declarations, actual merged versions and constant seals
across stages. The subsequent existing-bool update follow-up permits owned-bool
let roots in those same stages, retaining read-before-write captures and the sole
published target. Known outer logical constant writes reject conservatively;
single-statement logical child leaves and generalized resource/exit transport
remain separate.
These worktree receipts remain independent of
the committed baseline.

## Working Boundaries

Native scalar CPU closure is not a self-contained heterogeneous image or general
native GPU callback dispatch. Provider/device acceptance remains tied to its
tested hardware and workload. Compiler migration remains staged responsibility
transfer, not a completed Rust-free compiler. Nuis-rc CAS/lease/resident GC and
the rendering/control/ML/audio engine horizon remain separately tracked work.

This snapshot does not claim fresh Linux/GPU/Windows, full-workspace,
measured-performance or formal memory-safety acceptance. Historical socket and
strict-Clippy findings retain their own pending status.

## Reading Order

1. [Current mainline map](../current-mainline-map.md)
2. [Live task selection](../reference/nuis-development-tensor-mainline.md)
3. [One-sided baseline proof](../reference/nuis-native-one-sided-effect-result-joins-v1.md)
4. [Current capture optimization](../reference/nuis-native-effect-join-captures-v1.md)
5. [Current equal-atom proof](../reference/nuis-native-equal-effect-result-joins-v1.md)
6. [Beta-0.16 validation checklist](nuis-beta-0.16.0-release-checklist.md)

Run the live tensor from the repository root to select the next weak dependency;
use each proof's reproduction commands rather than replaying historical totals.
