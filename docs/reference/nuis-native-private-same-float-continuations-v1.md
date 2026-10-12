# Native Private Same-Width Float Continuations V1

This is the frozen 358-test same-width receipt. The later
[cross-width float proof](nuis-native-private-cross-float-continuations-v1.md)
separately admits f32/f64 and f64/f32 without rewriting this checkpoint.

The persistent application-session CPU path carries one exact floating outward
ancestor beside one independently initialized private floating child through
bounded terminal mixed selections and sequential continuations. This extends
the [float/i64 checkpoint](nuis-native-private-float-i64-continuations-v1.md)
without admitting cross-width floating pairs or a public tuple ABI.

## Single Outward Target

The separate numeric-pair predicate admits exact unqualified f32/f32 and f64/f64
in addition to the previously proved orientations. Independently typed carrier
installation is not an admission rule. The enclosing source helper is still
**bool-returning**, and only the owner reaches its enclosing return prefix.

One delegated/prefix-owned outward target, readonly entering values, unsealed
prefix-defined private children, both source write orders, current/retained
versions and lexical child scope remain mandatory. Third/unrelated entering
writes, seals, qualifiers/resources, hidden ordinary logical work, child leaks
and partial exits reject the complete candidate atomically. Cross-width floats
and i32 numeric-pair orientations remain rejected, not silently converted.

## Exact Private Transport

The closed compiler-private record has exact independent fields:

```text
owner: f32 / child: f32
owner: f64 / child: f64
```

Inactive helpers return two width-specific positive zeros. Both typed
projections read the same selected record. No sign/payload bit is borrowed,
numeric cast/packing, source replay, new owned allocation/drop, public return
or common effect permission is introduced.

The original whole-forest limits remain 64 nodes, 256 expressions, 32 logical
edges, 16 statements per body/suffix, depth eight and 31 captures plus a gate.
Both widths check the seven-node/seven-expression/zero-logical-edge boundary,
adjacent failure, 16/17 statements, 31/32 captures and depth 7/8.

## Source And Probes

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/return_same_float_private_pair.ns)
accepts two independent floating seeds. Child sign changes affect later owner
reads; both write orders, nested selections, a shared continuation, seals,
unused work and caller continuation remain real source computations.
The seed state is fourteen words, not the earlier thirteen-word state.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_same_float_oracle.rs)
crosses twelve raw words for each field with both gates: 576 cases per width.
Each field independently covers signed zeros, smallest/largest subnormals,
finite values, infinities and quiet/signaling NaN payloads.
An actual pair snapshot may differ from the final owner; final bool equality
alone cannot prove the correct current fields were carried.

Native-only callee-entry probes bitcast both actual after-work arguments and
the actual final answer-work owner. No production probe or oracle-manufactured
output replaces original source work. Normal/reversed function inventories
retain calls, joins, callback-state forwarding and real traps.

The shared native test driver derives input/output bounds and canary offsets
from actual state/open/event widths. Existing one-seed thirteen-word cases
keep their old layout, while new two-seed fourteen-word cases use the same
bounded harness. Accepted callback outputs feed the next callback through
volatile loads/stores; independently calculated oracle states are never
substituted for the actual previous result.

Late native faults carry actual floating child word 96, created by negating a
negative subnormal input. Field outputs may equal stage markers, so failure
prefixes use original event positions. A failing argument traps before callee
entry and cannot print that callee's observations or later events.

Reference/CLI acceptance uses four asymmetric finite/signed-zero seed pairs
crossed with both gates. Full IEEE payload evidence belongs to native raw-word
probes, not decimal parsing or reference state formatting.

## Executed Acceptance

The 2026-10-12 serial scoped cohort has **358 distinct tests**:
273 compiler unit/artifact tests, 54 native bridge tests, one CLI test spanning
both widths, three host failure-policy tests and 27 tensor tests. Sixteen tests
are new. Repaired/repeated runs and the complete 343-test predecessor are
not added to the current count. The new same-width CLI matrix replaces, rather
than counts as rerunning, the predecessor's float/i64 CLI receipt.

The new matrix runs 2304 successful native processes and
36 fatal native processes across both widths and normal/reversed inventories.
It proves 6984 accepted callbacks, independent actual IEEE pair words,
final owners, current reads, source order and width-driven canaries.
Nine original proof stages per width trap before callee entry, including
marker-valued late failures. Reference cleanup does not make native process
traps recoverable.

Eight CLI images, four per width, cover fresh build, cache reuse and two
successive source-free restores. They prove 72 CLI event traps and 744 accepted
callbacks, including healthy reentry after every failure. Only the last accepted
open/event state is published; later event/close requests and completion remain
absent after fatal events. Executable, native objects, packaging, envelope and
lifecycle identity stay unchanged. Each image and restoration input container
remain byte-identical after use; diagnostic-bearing fresh/cache container bytes
need not match.

The prior i32/i64/float-ancestor, dual-i64, both integer/float orientations,
five-child-type event failure/budget and helper-entry native regressions remain
included. The rebuilt ordinary CLI reports 2614 clean drift checks / 18190
required patterns and clean coverage/hierarchy/task-card lineage. It checks
six source-equal fields and six complete predecessor hashes.
The selected cell stays `active/99`, not broad application-session closure.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_same_float_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The tensor retains all six complete 343-test predecessor fields after their
float-i64 history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `8cc61ff52a5255fc49ce8bf4422e1a124dcc67cd1bc4130db70ff2aa11f1031d` |
| `next_step` | `5fe2aa2e637db85e1c6184f965df74af9ef44c31e16a53d66290ce85eb9fcf2c` |
| `blocker` | `e460c4fa9920fe7818600f8bca465609789cb7bebf807a6ccf141ffb04adbbe6` |
| `next_action` | `4c46e94c1daff8db035a63e81b6fb03e736f66c27255ee3fbaff49a927267ab5` |
| `validation_command` | `1a68a250ef7aafbd07d4055b438f50f7963cfd058940f8681a888c46c4d05a47` |
| `expected_artifact` | `75666fea0dc83f177c19f4d51be61d56514671f6df59c2205d9ca6aedcc730af` |

## Remaining Boundary

The next independent proof is exact cross-width floating owner/private child
transport: f32/f64 and f64/f32. Preserve two independent original widths,
current-value reads, source order, seals, one outward target, readonly entering
values and original budgets. Do not derive numeric conversion, resource or
public ABI authority from the existing typed carrier.

This is a macOS CPU receipt, not new GPU/Linux/Windows execution, measured
performance, formal memory safety, self-hosting, full-workspace or strict-Clippy
acceptance. Broader application closure and public ABI stabilization remain open.
