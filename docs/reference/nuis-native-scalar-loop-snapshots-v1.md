# Native Scalar Loop Snapshot Proof

This reference describes the current `beta-0.15.*` return-loop snapshot proof,
including the 2026-10-03 post-loop extension and capture normalization. It supplements
[native value returns](nuis-native-scalar-value-returns-v1.md) and the
[session bridge](nuis-native-scalar-session-bridge-v1.md); it does not replace
the historical `beta-0.15.6` release checkpoint.

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
artifact tests cover three variants, cache reuse, selected failure, tamper rejection
before open and identical LLVM/state after deleting source and the original build.
The unused-call fixture observes the result as an ordinary field read. Forwarding
that computed record's field into another call still protects its immutable
transport chain. A separate native/storage test keeps the complete record in a
single readonly `[64 x i64]` parameter and executes it without heap aggregation.
Together with its guard this is two typed arguments, not 65 scalar arguments;
record width, native argument count and carry width are different contracts.
This is bounded input-alias closure, not general local/call-result optimization or
a measured performance improvement.

## Validation

Run sequentially from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- return_invariant --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --test native_application_bridge -- typed_sparse_literal_returns --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_literal_snapshots --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- private_record_copies --test-threads=1 --quiet
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

## Remaining Boundaries

The persistent-session coordinate remains `active/86`. Further changing
backedges, local-version/opaque snapshots and call-backed wide private captures
need additional bounded proof, not a wider native limit. The near-boundary
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
