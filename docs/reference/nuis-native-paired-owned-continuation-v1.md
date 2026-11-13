# Native Paired Owned Continuation V1

This beta-0.18.9 worktree follow-up extends the
[empty-else continuation proof](nuis-native-owned-continuation-v1.md) to explicit
nonempty else arms. The resource coordinate stays active/95; its evidence and
next action advance, not the frame/window or release milestones.

The subsequent [resource scalar-shadowing proof](nuis-native-resource-scalar-shadowing-v1.md)
admits explicit i64 branch-local shadows. This receipt retains the preceding
capture restriction and validation counts as historical evidence.

## Source And Ownership

Both arms may contain an early owned Ok or Err return. If neither selected inner
return executes, the surviving arm transfers the original incoming Bytes into
one shared continuation containing the function's remaining statements. Branch
locals retain their own scope-exit cleanup. Identically named locals in different
arms do not become shared captures.

The source outliner uses the existing operations:

```text
guard_call_owned_struct_return condition then_helper layout captures owners
tail_call_owned_struct_return else_helper layout captures owners
```

The taken guard ends the caller, so only its untaken edge reaches the else helper.
There is no second evaluation or negation of the source condition. Each surviving
arm ends in a terminal call to the same continuation. A terminal arm does not call
that suffix; a fully terminal pair needs no continuation or fabricated return.
Nested pairs may mix terminal and continuing arms under the existing bounds.

No public callback ABI, YIR opcode, runtime dispatcher or provider grant is added.
The existing typed callee/result, GLM, lifetime, exact live-owner partition and
independent helper-consumption checks still apply. LLVM forwards the helper's
packed Result unchanged, including allocation failure. No pending Result,
payload repacking, copied suffix tree or machine tail-call guarantee is used.
The root-inclusive depth-8, 32-function and per-function resource/node budgets
remain unchanged; paired branches and shared continuations count toward them.

## Native Evidence

The paired regression executes 120 events across six paths: two early owned
returns in each arm and one surviving edge from each arm. Original snapshots,
branch-local snapshots, two incoming Bytes owners, complete scalar/pixel State
and one suffix counter are checked against an independent expected-state model.
Each completed callback leaves zero tracked live blobs. YIR roundtrip and graph
inspection prove that the two surviving calls share one suffix body.

Further cases check:

- A Buffer-reading predicate whose selected arm changes the read location,
  without retesting the condition or running the other arm.
- Branch-local allocations with no incoming Bytes owner and no invented transfer.
- Nested mixed terminal/continuing pairs and a complete pair without a trailing
  source return.
- Err from either arm and faulted suffix allocation: accepted State stays intact,
  resources are released and successful close cannot erase the prior failure.
- Poisoned arithmetic in an unselected arm and in a suffix skipped by early return.
- Captured scalar rebinding, consumed/renamed owners, missing lifetime edges and
  missing owner trailers on either join edge are rejected.
- Three sequential pairs fit the existing closure depth; four exceed it rather
  than bypassing the limit through shared suffixes.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo/README.md)
now has both heartbeat and paint early-return/fallthrough paths. Five events
produce red then green pixels, four black pixels, and scalar State `[3, 2, 5, 2]`.
The delivery regression checks that result through cache reuse, source-free NSB
relocation and detached binary launch with empty PATH.

## Validation

Run from the repository root, with one build job and incremental caches disabled:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor_ --test-threads=1
```

Local validation for this worktree follow-up:

- All 63 native resource-session tests pass, including nine paired-arm cases;
  both file-line-budget checks pass.
- All five resource-session delivery tests pass. The shared-continuation case
  was rerun after the final depth-budget check ordering change and passes.
- Both ordinary record-return bridge regressions and all 50 tensor tests pass.
- All 43 repository-maintenance tests pass. Workspace architecture, UTF-8,
  documentation links and the host-absolute-path policy checks pass.
- The tensor reports 4565 drift checks, zero failures and `clean`. The resource
  coordinate remains `active/95`; the other 36 coordinates match their pre-change
  content hash.

## Remaining Boundaries

Captured binding replacement remains rejected. The next slice is hygienic
branch-local scalar shadowing, so the continuation receives the original outer
binding rather than an arm-local replacement. This is not permission for mutable
resource phi joins or changing owner identity across a surviving edge.

General resource loops/helpers, arbitrary State layouts, resize, cancellation,
provider/device completion and native ns-nova windows remain separate. Static
host finalization still uses clang and the allocation compatibility ABI; host
transport still copies. No full-workspace, fresh remote Linux/GPU, performance,
pure Nsld finalization or all-Nuis runtime claim is made.
