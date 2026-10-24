# Native Return Ready Values V1

This local CPU follow-up removes the helper-count obstacle recorded by the
[conditional-field checkpoint](nuis-native-return-effect-storage-conditional-payload-v1.md).
It continues from Git beta-0.17.5 at c3e6a14f without changing the version.

This records the preceding ready-value checkpoint. The
[typed scalar join follow-up](nuis-native-typed-scalar-joins-v1.md) tracks its
next source-declaration gate and retains this checkpoint's complete lineage.

## Narrow Lowering Changes

[Ready-value analysis](../../tools/nuisc/src/lowering/buffer_loop_outline/ready_values.rs)
recognizes already available scalar values and literals, not arbitrary pure
expressions. Straight-line writes to existing scalar bindings become ordered
matching-value selects. Each unselected arm keeps the current binding. Later
writes observe preceding selected writes; both arms use the once-snapshotted
predicate. New locals, nominal records, resources, calls, projections, prints,
returns and arithmetic expressions do not qualify for this transformation.

[Logical-condition extraction](../../tools/nuisc/src/lowering/buffer_loop_outline/logical_conditions.rs)
also avoids a helper when its RHS contains only ready boolean values or
comparisons of same-typed scalar atoms. The left operand is still evaluated.
Calls, checked arithmetic and projections retain their existing guards, even
when a call is otherwise pure. Nested non-total left operands still undergo
guard extraction. This is not general speculative execution or effect erasure.

The representative full fixture goes from 75 to 63 NIR function definitions:
eight private helpers for ready flag updates and four for total logical RHS
work disappear. This is a module-definition count, not a claimed execution-time
speedup or a count of reachable functions alone. All four variants fit the
existing 64-function native admission profile. The two effectful field RHS
helpers remain, as do the original source call occurrences and effect ordering.

## Full Fixture

Compiler, native/reference and CLI delivery now use the same full conditional
fixture. The temporary return-focused source filtering and unavailable-mode
adapter are removed. Modes 2-5 and 7-9 again exercise local and parent break,
continue and return paths alongside first-parent and post-backedge payload
returns. No source branch is deleted to satisfy native admission.

The shared oracle still checks both snapshot epochs, live reads, raw
bool/i64/f32/f64 fields, all six lazy truth paths and 31 failure categories.
Failures before or after the desired field cannot publish partial State.
Previously emitted effects are not rolled back. Poisoned skipped work and
zero-width work remain separate controls.

The native profile limits remain 64 functions, 16,384 nodes and call depth 32.
Fixture loop-work/helper-entry budgets remain 18/192. Removing synthetic
bookkeeping calls reduce entry consumption. The earlier long callback now
succeeds with 160 entries and still traps with 100, before publishing State.
Source calls and loop reservations retain their checks. The existing synthetic 64/65
function boundary, recursion and depth controls are retained independently.

## Validation

Bounded local M2 CPU validation passed 227 distinct tests, including six new
compiler controls. Repeated tests are counted once:

- Compiler: 90 distinct checks, including all four full variants in the
  independent/shared and normal/reversed execution matrix. A former helper
  existence assertion now checks both elided ready writes and retained
  computed-bool word transport; all six temporary-scope controls pass.
- Native/reference: 39 checks cover full conditional variants, exact fault
  prefixes, reservation bounds, graph limits and adjacent entry-budget controls.
  The long-callback control now proves success at 160 and rejection at 100
  entries in normal and reversed YIR.
- Borrowed-buffer integration: 41 checks cover scalar control, branch scope,
  condition snapshots, break/continue, effect order and ownership rejection.
- CLI delivery: one check passed in 781.30 seconds. Fresh/cache and two
  source-free images each execute healthy sessions, all 31 failure categories
  and a further healthy session. Binary, host-object, envelope, lifecycle and
  packaging identities remain equal across cache/restoration.
- Host failure-state checks: three passed; repository maintenance: 24 passed;
  file-line limits: two passed; status tensor: 27 passed.

The ordinary CLI rebuild passed in 9.23 seconds. Formatting, whitespace, UTF-8
across 5,634 files, 4,163 local documentation links and locked offline metadata
for 29 packages passed. Drift validation matches all 3,588 checks and 24,573
required patterns. The rebuilt CLI matches all six source fields, with complete
predecessor hashes and inert historical commands verified. Coverage, hierarchy
and lineage are clean; the next task card is ready and the coordinate remains
active/99.

These are targeted local CPU checks. The full workspace, other historical
delivery families, GPU/Linux execution and physical performance were not rerun.

## Remaining Frontier

The status coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Typed conditional scalar bindings in effectful callees still require their own
admission/lowering proof. This change optimizes already admitted generated
control flow; it does not authorize arbitrary source-level scalar reassignment.
Resource/I/O/async captures, stable ABI, GPU/cross-platform parity, broad
heterogeneous closure and self-hosting remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- ready_values::tests effect_return_storage_conditional_payload --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- conditional_payload scoped_call_edges_share_recursion_depth_and_function_bounds --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- conditional_payload --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete fields from the preceding conditional-field checkpoint are
retained inside the status tensor with these SHA-256 hashes. Historical
validation commands remain inside a quoted inert shell here-document.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `b488d0e6078c5e2a8dc91bf0a54b3e477543d151803872858883e4874f3113f9` |
| `next_step` | `c48ff331d4a0baf6a40e787a87ca9317a0a17e591a2bd7ccff09219be3604564` |
| `blocker` | `4e61dd701f06887948a78a1f084d835011071829071de4b707f4644e6b453c37` |
| `next_action` | `54404f090401b2950485b1a077ac8b599d3779458d594c910c118c72b4d41cb9` |
| `validation_command` | `6bedcec4b33e1bbeb44625ca6337fdaaebcf351ed46ef6f7b8147a2b2e034856` |
| `expected_artifact` | `d1a1f175bcacd75f8b0b47a6d863c8fff213fa7c08087cec37a1e1562fcdb2c7` |

