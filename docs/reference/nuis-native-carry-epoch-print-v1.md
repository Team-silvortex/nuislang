# Native Carry Epoch Print V1

This bounded CPU follow-up continues the [computed-print checkpoint](nuis-native-computed-iteration-print-v1.md)
on Git beta-0.17.7 at 7eb90ed4 without changing the version or native print policy.

## One Print Read Lens

An iteration's exact-i64 mutable carry can now be observed by a pure computed
print before its first update. The [epoch certificate](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs)
intersects initialized entry scope, source write authority and the iteration's
update set, retaining exact non-reference `i64` bindings only. It grants neither
expression purity nor writes. Each missing ordered read must still name one of
those seeds and retain the exact type at the print site.

The [temporary validator](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs)
first applies the existing pure-i64 print admission, then lends certified reads
to a temporary copy-on-write validation view for that one print. It does not
change the real availability set. Existing expression checks still run against
the view; no alternate expression renderer or speculative value is introduced.
Already-ready prints need no clone. This is compiler validation storage, not a
new allocation in generated application code.

Ordinary conditions, assignments, temporary initializers and sibling reads keep
their previous ordered-read rules. Parameters/constants do not acquire write
authority, unknown/future names do not become initialized, and bool/i32/float/
record/reference carries do not acquire a pre-update print grant. Pure and buffer
profiles remain separate. Branches retain their lexical scope and the same
entry certificate; updated values continue through the existing source-order
renderer. No print permission is exported as general readiness.

Child loops deliberately receive an empty certificate. They retain existing
ready/self-read behavior but cannot borrow a parent's pre-update print grant or
create an unproved child epoch. Complete pure-call admission and the shared
4,096-work/64-depth print-root budget are unchanged. Failed admission renders no
new source/helper plan.

## Source Order Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/carry_epoch_print.ns)
uses a trailing induction step and a mutable `current` initialized from State.
Each iteration optionally prints `observe(current, divisor - index)`, then prints
the old value, updates `current`, and prints the new value. A final checked
`finish` precedes State publication. Leading/trailing and branch print variants
also execute in the four new [source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch_tests.rs).

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/carry_epoch_print_cases.rs)
models scalar words and source prints directly rather than reading compiler
output or reusing a rendered plan. Healthy State progresses 64 -> 72 -> -2.
Additional rows cover zero/negative trips, skipped poisoned print arguments,
negative deltas and MIN/MAX divisors. Its six failure rows trap in the first,
second or third selected print argument, or in final checked work after zero or
multiple iterations. Later failures retain earlier prints without publishing
partially updated State. Observer markers distinguish real `observe`/`finish`
entries from the source's own output.

[Native/reference execution](../../tools/nuisc/tests/native_application_bridge/carry_epoch_print.rs)
checks normal and reversed YIR order, reached callee counts, exact flushed native
fault prefixes, canary-guarded output and last accepted State. A faulted reference
session rejects further events and completion remains failed. Work/entry limits
remain 4/64, and this witness has at most 32 functions. The existing native i64
policy is reused unchanged; the bridge adds no aggregate allocation/drop.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/carry_epoch_print.rs)
checks fresh/cache and two independent source-free images, each with healthy
sessions before and after all six failures. Source, manifest and project cache
are absent during restoration. Binary/container, host-object, envelope, lifecycle
and packaging identities remain fixed. Native probes pin full flushed prefixes;
CLI checks trap status, State and any flushed bytes without assuming a trapping
process flushes stdout.

## Validation

Bounded local M2 CPU validation passed 321 distinct tests, including seven new
tests. Repeated focused runs are counted once:

- Compiler: 193 focused/adjacent checks in 58.79 seconds. Four new controls cover
  entry certification, single-print-only availability, wrong scopes/types, child
  isolation and leading/trailing guarded source order. Existing pure temporary,
  ordered carry, word conversion, literal child-loop, iteration-effect and record
  join controls remain green.
- Native/reference: 47 checks in 152.97 seconds, including two new witnesses,
  prior computed-print/record families and six legacy literal-policy checks.
  The new witness runs all six failure rows in normal and reversed YIR order.
- Producer policy: three LLVM build-policy checks and three packaging-option
  checks, reusing the existing i64-v1 identity without widening legacy v1.
- CLI delivery: one new check in 38.15 seconds. Fresh/cache and two independent
  source-free images each run healthy sessions before and after all six failures.
- Borrowed-buffer integration: 17 checks in 38.36 seconds; host failure state:
  three; repository maintenance: 24 in 25.453 seconds; host absolute-path policy:
  one; file-line limits: two; status tensor: 27 in 6.61 seconds.

The final CLI-test compilation passed without warnings and the ordinary CLI build
passed in 0.02 seconds after delivery finished. Formatting, whitespace, UTF-8
across 5,773 files, 4,427 local documentation links and locked offline metadata
for 29 packages passed. All 3,770 drift checks and 25,571 required patterns match.

The actual rebuilt CLI matches all six source fields. Complete predecessor
hashes are verified, historical commands remain inert, coverage/hierarchy/lineage
are clean, the next task card is ready and the coordinate remains active/99.
The preceding receipt's aggregate was corrected from 301 to 299 to match its
listed test groups; its passing results and six-field predecessor values did
not change.

These are targeted local CPU checks, not a full workspace run or a performance
benchmark. Earlier CLI delivery families, the independent/shared storage matrix,
GPU/Linux execution and physical float/performance tests were not rerun here.
Their receipts remain historical evidence rather than new carry-epoch results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next prove a fresh child-loop epoch for its own initialized exact-i64
carry, without inheriting an unavailable ancestor's grant or leaking readiness
to surrounding work. Keep the empty child certificate until that proof exists.
Record-field pre-update reads, hidden logical print roots, general effects,
I/O/async/resource capture, outer writes, stable ABI, full GPU/Linux parity and
self-hosting remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- print_epoch:: iteration_prints:: iteration_effects:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- carry_epoch_print:: computed_iteration_print:: literal_print_policy:: literal_print_execution:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- carry_epoch_print --test-threads=1 --nocapture
```

The tensor retains the adjacent validation chain and quoted inert history.

## Predecessor Lineage

All six complete computed-print fields are retained by SHA-256. Historical
commands remain inert, and historical results are not counted as newly rerun.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `c4f1f9a7aa549b6e5fab4626e30cceb5271cee81a0476a187a95ba9d158c58b2` |
| `next_step` | `bcfe82f5e7680198787a6a18629603b2e19ab1c45c08f168edc748d79d3e8a9e` |
| `blocker` | `2b8e56d5304a2383ff32c32900044462293667ae24992f0fbbde4057227b7130` |
| `next_action` | `f47e853b5fde8a699a3442c519f356115c5c08912b992dbe839758fedace22ac` |
| `validation_command` | `6b0189917fa22e5b7c6cbff93c044ed99375245ad8453eed13d0a4d10e4230ed` |
| `expected_artifact` | `26cf2aad79e86d4592715682de939888486831695dc5829b3443a1dee76dfcfe` |
