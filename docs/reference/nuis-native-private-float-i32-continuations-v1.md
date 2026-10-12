# Native Private Float I32 Continuations V1

This is the complete 430-test checkpoint. The subsequent
[three-field proof](nuis-native-private-three-field-continuations-v1.md) adds a
separate two-private-child certificate without rewriting this scoped acceptance.

The persistent application-session CPU path carries one exact f32 or f64
outward owner beside an independently initialized private i32 child through
bounded terminal mixed selections and sequential continuations. This follows
the complete 415-test predecessor,
[i32/float](nuis-native-private-i32-float-continuations-v1.md), without a public
tuple ABI, backend specialization or new common effect permission.

## Single Outward Target

The new exact unqualified float-owner/i32-child admission is separate from
generic record installation. The source helper still returns **bool**; only
its authorized floating owner reaches the enclosing return prefix.

One delegated/prefix-owned outward target, readonly entering values, current/
retained versions, both source write orders, private scope and seals remain
mandatory. Qualified/resource/unsigned fields, third or unrelated entering
writes, hidden ordinary logical work, child leaks and partial exits reject
atomically. Historical rejection tests now cover unsupported unsigned/resource
types or sealed bindings, rather than rejecting the newly proved orientation.

The recorded bounded numeric-pair admission now covers all sixteen exact
ordered combinations of i32, i64, f32 and f64. This statement concerns the
existing owner/private-child route, not arbitrary tuples, resources, casts or
general rebinding.

## Exact Private Transport

The closed compiler-private record retains independent original-width fields:

```text
owner: f32 or f64 / child: i32
```

The owner neutral has its original f32/f64 zero type. The child neutral is the
existing checked i32 cast of literal i64 zero. Distinct typed projections read
the same selected record. The early owned-field route takes precedence over
the existing bool/i32 codec even though the child is i32. No live floating/
integer conversion, payload-bit borrowing, source replay, child export, new
owned allocation/drop or public pair ABI is introduced.

The original whole-forest limits remain 64 nodes, 256 expressions, 32 logical
edges, 16 statements per body/suffix, depth eight and 31 captures plus a gate.
Both four-test width families retain the exact seven-node/seven-expression/
zero-logical-edge boundary, adjacent rejection, 16/17 statements, 31/32
captures, depth 7/8 and complete input/name atomicity.

AOT checks exact original-width layouts, floating helper results, guard-first
transitive Effect paths and default/empty-print/removed-Effect policy vetoes.
YIR floating/integer casts stay absent. Integer narrowing admits only bounded
literal zero/negative-one construction, width-safe i32 complement work and
existing canonical bool/i32 restoration with a proved producer. The actual
i64 proof parameter rejects as an arbitrary narrowing source.

The test-only LLVM check permits legacy auxiliary f32 widening solely into a
dead integer conversion. Synthetic live floating/integer consumers and f64-to-
f32 narrowing reject. Existing surrounding bool/i32 restoration and dead
bookkeeping remain valid; the whole LLVM unit is not claimed cast/codec-free.
These structural checks grant no compiler conversion or formal verification.

## Source And Probes

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/return_float_i32_private_pair.ns)
takes independent floating and i32 seeds, retained separately in a fourteen-word
state. Child work uses typed negative-one minus an i32 value, avoiding an
intermediate negation of the signed minimum. Current integer-child signs drive
floating-owner negation at the original IEEE width.

Both source write orders, nested current zero/negative-one updates, shared
continuation, seals, unused work and caller continuation remain real
computations. Pair snapshots can differ from the final owner. No input is
derived from the other; a correct final bool alone cannot prove the fields.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_float_i32_oracle.rs)
crosses twelve raw IEEE owner words with eleven i32 child seeds and both gates:
528 cases per width/inventory. It combines signed zeros, smallest/largest
subnormals, finite values, infinities and quiet/signaling NaN payloads with
independent integer extrema and diagnostic marker values. Imperative replay
uses raw sign-bit XOR and i32 complement, never the compiler carrier.

Reference healthy execution covers the full raw matrix; selected reference
failure/cleanup inputs are finite. Public f32 state text follows the existing
promoted f64 formatter, separately from shortest f32 input text. Raw state-word
checks remain unchanged, including a smallest-subnormal audit requiring word
one. Display precision is not numeric field widening.

Native entry probes observe actual original-width owner IEEE bits and the
sign-extended i32 child at after-work, then actual final-owner bits at answer-
work. f32 observations zero-extend only the original 32-bit word; f64 preserves
all 64 bits. Both normal/reversed function inventories retain actual source
computations, calls, joins, accepted-state forwarding and width-driven canaries.

Native late faults carry raw owner word 96 beside legal child -70002, starting
from independently initialized negative subnormal owner bits and child 70001.
The integer field deliberately matches a diagnostic sentinel. Full source-event
prefixes distinguish field values from markers, retain real SIGILL/SIGTRAP and
reject actual post-failure status/canary publication. Reference cleanup is not
native process recovery or rollback.

CLI acceptance uses six independent finite-float/integer pairs and both gates:
24 healthy cases per image. They include signed zeros, integer extrema and
both nested current zero/negative-one update orders. Finite late-fault owner
values and native raw-word-96 owner seeds are separate receipts. No arbitrary
IEEE NaN payload text roundtrip is claimed.

## Executed Acceptance

The 2026-10-12 serial scoped cohort has **430 distinct tests**:
315 compiler unit/artifact tests, 84 native bridge tests, one two-width CLI test,
three host failure-policy tests and 27 tensor tests. Sixteen tests are new.
Focused/repeated runs and the complete 415-test predecessor are
not added to the current count. The new reverse CLI replaces the predecessor's
forward i32/float CLI in this receipt; that old CLI is not claimed as rerun.

The new raw matrix runs 2112 successful native processes and
36 fatal native processes across both widths and normal/reversed inventories,
proving 6408 accepted callbacks. Actual pair/final-owner IEEE/integer words,
independent extrema/payloads, current reads, source order, state forwarding and
canaries remain checked. Nine original proof stages per width fail before
callee entry, including marker-valued late faults.

Eight CLI images cover fresh build, cache reuse and two successive source-free
restores per width. They prove 72 CLI event traps and 936 accepted callbacks,
including healthy reentry after every failure. Only the last accepted open/
event state is published; later event/close requests and completion remain
absent after fatal events. Executable, native objects, packaging, envelope and
lifecycle identity stay unchanged. Images and restoration inputs remain byte-
identical after use; diagnostic-bearing fresh/cache container bytes need not match.

The previous bounded scalar-pair, ancestor, private-event failure/budget and
helper-entry native regressions remain included. The rebuilt ordinary CLI
checks six source-equal fields and six complete predecessor hashes, with
2737 clean drift checks / 19228 required patterns and clean coverage/
hierarchy/task-card lineage. The selected cell stays `active/99`, not broad
application-session closure.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_float_i32_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

All six complete 415-test predecessor fields remain after their i32-float
history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `73a932a93948f53b55c4d38d1e8d8e001c92ce77a6d9a3370b9bab0ab05562d1` |
| `next_step` | `6e4401e692ef17c58b4680a1a0155aa9df40fe3c9f244ae02536a76daea84326` |
| `blocker` | `b42fecd1e0e82295b82582782c6f8e8f595a053e3acd8eb6e6dc5198e24ec139` |
| `next_action` | `09cc47683c623a9544885f6e19d24b0a5eecf4ff6a34924ddc02f2cc192fc255` |
| `validation_command` | `0ed3203baa03bb680b582005303163c69cfd14eb08c675753948bc9db58521c5` |
| `expected_artifact` | `ce47076c1c40a248f272c354feb82df3cab244f2108df2bda1992bc149c243ef` |

## Remaining Boundary

Two private children beside one authorized outward target need a separate
bounded proof. Begin with an i64 owner and independently initialized i32 and
f32/f64 children, but keep the carrier representation driven by independent
field types rather than a fixed fixture/backend combination. Do not infer
three-field write permission from the complete two-field scalar matrix.

Preserve private scope/seals, readonly entering values, original forest/
capture/depth/statement budgets, actual current/retained field observations,
source-order traps, accepted-state forwarding/canaries and finite-CLI fresh/
cache/two source-free publication identity.

This is a macOS CPU receipt, not new GPU/Linux/Windows execution, measured
performance, formal memory safety, completed self-hosting, full-workspace or
strict-Clippy acceptance. General resource transport, broader application
closure and public ABI stabilization remain open.
