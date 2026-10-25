# Native Computed Iteration Print V1

This bounded CPU follow-up continues the [origin-join checkpoint](nuis-native-origin-record-join-v1.md)
on Git beta-0.17.7 at 7eb90ed4 without changing the version.

## Completed Pure Values

The isolated native iteration-call route now admits `print(value)` when the
original expression has exact type `i64` and every called dependency has already
passed complete pure-body admission. An effectful helper returning `i64` does
not become a pure print argument. The iteration signature catalog stores this
distinction explicitly rather than inferring purity from a signature or name.
Print-only dependencies participate in discovery and reachability; missing names,
cycles and unadmitted transitive dependencies fail closed.

The [print-value owner](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_prints.rs)
uses the existing ordinary-expression syntax proof. A shared 4,096-work/64-depth
budget covers all print roots across branches and loops before normalization,
then rechecks the admitted body. It grants no hidden logical-expression root,
reference/resource, bool, i32 or float printing authority. Existing validation
also retains lexical types and the loop's ordered-read checks. A carry read
before its permitted update does not gain authority merely by being printed.

The original print statement remains in place. Scoped branch captures and
hygienic renaming include its argument; the value is evaluated only under its
original guard, after preceding record fields and effects. Argument failure
precedes the print itself, later work and pending State publication. Pure-value
and borrowed-buffer profiles still reject these effects; the old literal-only
validation mode remains literal-only.

## Separate Native Grant

The [new LLVM policy](../../crates/yir-lower-llvm/src/native_session/i64_prints.rs)
is explicit selected-site authority, not an automatic upgrade of literal prints.
`I64PrintPolicy` shares the bounded site, admitted closure and dependency-order
proofs. Strict native lowering still requires exact i64 operands and bool guards.
No other effect, foreign shim or application-specific backend case is added.

The persisted producer mode is `i64-v1`, with contract
`nuis-native-i64-print-build-policy-v1`, a separate `native_session_i64_print_bound`
claim and explicit `--native-i64-print-policy` packaging option. Canonical UTF-8
site encoding, uniqueness and work/entry limits remain checked. Option/token kind
mismatches are rejected. Old `v1`, `LiteralPrintPolicy`, literal bound claims and
the old packaging option retain their previous meaning and reject computed
operands. The shared integer runtime is enabled only for an admitted print mode.

This is a scoped opt-in API extension, not a stable ABI promise. Nuis frontend
purity admission and raw-YIR backend effect/order admission remain separate.

## Application Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/computed_iteration_print.ns)
replaces the preceding fixture's unused checked post-work with
`print(local_after(post_div))`. This runs after the complete certified record
join and source print 83, but before print 84 and final return. Selected lazy
fields, opposite field orders, thirteen pure-callee counts and loop exits remain.

The [oracle](../../tools/nuisc/tests/native_application_bridge/computed_iteration_print_cases.rs)
reuses the independent predecessor evaluator. That evaluator reaches marker 84
only after checked post-work succeeds, so the facade inserts independently
checked `1 / post_div` immediately before it. It derives no expected value from
compiler output. Argument failure emits neither quotient nor marker 84; later
failure retains the already printed quotient. Healthy MIN/MAX divisors produce
zero and divisor -1 produces -1.

Thirty-two core failures and 36 native/reference failures retain isolated later
traps and skipped poison. [Native/reference tests](../../tools/nuisc/tests/native_application_bridge/computed_iteration_print.rs)
check normal/reversed YIR, exact flushed fault prefixes, canary-guarded output,
State 64 -> 72 -> -2, faulted completion and rejection of subsequent events.
Limits remain 4 work, 64 entries and at most 64 functions. Additional controls
reject missing/unknown sites, wrong operand kinds and unordered work, and pin the
separate producer identity against the legacy literal policy.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/computed_iteration_print.rs)
checks fresh/cache and two independent source-free images. Each runs healthy
sessions before and after 32 core failures, preserving binary/container,
host-object, envelope, lifecycle and packaging identities. Native probes pin the
full flushed fault prefix; CLI pins State, trap status and any flushed prefix,
since a trapping process need not flush stdout. Four extra native probes are not
CLI cases.

## Validation

Bounded local M2 CPU validation passed 299 distinct tests, including fifteen new
tests. Repeated focused runs are counted once:

- Compiler: 173 focused/adjacent checks in 58.11 seconds. Eight new controls cover
  completed pure dependencies, exact numeric types, shared expression budgets,
  legacy profile isolation, current inputs, complete branch print sequences and
  ready-value prints in existing return/parent variants. Prior declaration,
  record-origin, logical-tree, join, exit and capture controls remain green.
- Native/reference: 45 checks in 143.63 seconds, including four new application
  and policy controls, six legacy literal-policy/execution checks and the prior
  record/join families. The final full focused run passed after repairing a
  malformed wrong-kind test; that test now pins the backend scalar-kind error.
- Producer policy: three LLVM build-policy checks and three packager option
  checks, including one new test in each group. Canonical identities, malformed
  tokens and explicit option/token-kind separation remain checked.
- CLI delivery: one new check in 310.78 seconds. Fresh/cache and two independent
  source-free images each run healthy sessions before and after 32 core failures.
- Borrowed-buffer integration: 17 checks in 42.68 seconds; host failure state:
  three; repository maintenance: 24 in 18.969 seconds; host absolute-path policy:
  one; file-line limits: two; status tensor: 27 in 6.36 seconds.

The shared oracle's two native-only observation imports are explicitly scoped as
optional for its CLI consumer. A final CLI-test compilation passed without the
earlier unused-import warning. The ordinary CLI build passed in 0.06 seconds after
delivery finished. Formatting, whitespace, UTF-8 across 5,765 files, 4,413 local
documentation links and locked offline metadata for 29 packages passed.

All 3,760 drift checks and 25,518 required patterns match. The actual rebuilt CLI
matches all six source fields; complete predecessor hashes are verified and
historical commands remain inert. Coverage, hierarchy and lineage are clean,
the next task card is ready, and the coordinate remains active/99.

These are targeted local CPU checks, not a full workspace run or a performance
benchmark. Earlier CLI delivery families, the independent/shared storage matrix,
GPU/Linux execution and physical float/performance tests were not rerun here.
Their receipts remain historical evidence, not new computed-print results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next is an independently proved current-epoch read for a computed
print that observes a mutable carry before its update. The later
[carry-epoch proof](nuis-native-carry-epoch-print-v1.md) adds that bounded read lens
for exact-i64 entry carries, without granting child epochs. At this checkpoint,
keep the existing
ordered-read rejection until that proof preserves guards, loop backedges,
capture identity and failure-before-publication. This does not authorize outer
writes, loop-local const normalization, hidden logical print roots, general
effects/I/O/async/resource capture, stable ABI, full GPU/Linux parity or self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_prints:: iteration_effects:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- computed_iteration_print:: literal_print_policy:: literal_print_execution:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- native_session::build_policy --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-pack-aot --bin yir-pack-aot --locked -j1 -- host_application_script::tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- computed_iteration_print --test-threads=1 --nocapture
```

The tensor retains the complete adjacent validation chain and quoted inert history.

## Predecessor Lineage

All six complete origin-join fields are preserved by SHA-256. Historical commands
remain inert rather than being executed again.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `2776e4c0187c50fe29ec25ba218b5757348137d0166a29857f7cf9d463ba7da4` |
| `next_step` | `3df3f294caec7482b7b0873067c35efad0fbfbdaf140dd429a45bbae552abdbf` |
| `blocker` | `e9b61f11736ac3fab7b66e5328ea897c3b92aa2d89444e0dd835182ab2970771` |
| `next_action` | `1110ca945fb6db245fa0e74e63c4d508c92edc873634637e3c70ceeb1c88f3dc` |
| `validation_command` | `429088fa954517f7d3a737066d856bce7c91aa64d2a0afe0b536de6a2652f2e7` |
| `expected_artifact` | `50691433633eb8a526c2c770ece64ad8f852a558f9b2b63ea21876efd3a19b4e` |
