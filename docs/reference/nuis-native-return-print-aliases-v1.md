# Native Return Print Aliases v1

This bounded CPU worktree follow-up to the
[computed print-argument proof](nuis-native-computed-return-prints-v1.md) follows
`98ca713e` (`beta-0.15.8`). It does not establish general effectful helpers,
native application closure or a new release.

Its atom-only boundary and acceptance receipt remain historical. The later
[staged initializer proof](nuis-native-staged-return-effects-v1.md) separately
admits matching selected checked/call-backed scalar bindings, without relaxing
this proof into general initializer or effect-region authority.

## Admitted Source

Fresh exact owned `bool/i64` atom copies and literal bindings may occur before
or between top-level prints in an otherwise admitted conditional return arm:

```nuis
if outer {
    let local_stamp: i64 = stamp;
    print(88);
    const stamp_alias: i64 = local_stamp;
    print(100 / stamp_alias);
    let left_alias = left;
    let gate_alias: bool = gate;
    print(89);
    return helper(produce(left_alias)) && (gate_alias || helper(produce(right)));
}
```

Only `Int`, `Bool` and previously available scalar `Var` initializers qualify
in this interleaved prefix. Checked arithmetic, calls, casts, comparisons,
record/resource creation and rebinding initializers need separate proof.
Existing independently admitted pure tails retain their own local computations.
This is a bounded outlining rule, not a restriction on all language bindings.

The prefix ends at the last top-level print. Each original initializer and print
is validated in source order, using the original catalog and exact declared or
inferred types. The original tail is also validated with the extended lexical
scope before any alias is removed. Duplicate names, parent shadows, later tail
shadows, forward/self references, use-before-definition and unreachable suffixes
cannot become valid through substitution. Sibling arms have independent scopes.

## Nonexpanding Normalization

Each alias resolves directly to one original parent atom or literal, including
chains of copies. Both arms must validate before cloning normalized bodies.
An iterative single-map rewrite replaces uses with that one atom and removes
only the proven total copies. It neither expands expression trees nor evaluates
a call, check or resource operation early. No branch-local alias escapes or
becomes a new helper input.

The normalized body goes through the existing independent print/return proof.
The parent entry is saved once; each computed print remains selected, and its
actual print stays in the parent. Return-tail eligibility, source exits and
partial readiness remain independent: print-only work grants no tail eligibility.
No new YIR instruction, helper catalog authority, ABI, GLM/ownership exception,
Nustar dependency or native-word limit is introduced.

## Original Budgets And Atomic Veto

All original prefix statements, including unused aliases that disappear, and
their initializer nodes remain charged alongside complete print arguments and
expanded pure tails. Elision cannot buy admission budget. The unchanged limits
are 32 statements, 4096 expression nodes, depth below 64 and the tail's separate
32 logical edges. Print arguments remain ordinary roots, without logical-leaf
permission. Original preflight precedes recursive typing or expression cloning.

The normalizer changes no module, helper list or generated-name set. Both
original arms and the final normalized print/tail plan must succeed before
installation. Any invalid second arm leaves the valid first arm unchanged.
Internal prints, intervening control flow, computed prefix initializers,
resource/borrow/FFI regions and wider changing captures remain unproven.

## Reproduction

Growing this registration exposed a Rust query-depth failure in the tensor
CLI's deeply chained check iterator. The iteration-check registry now uses a
flat static slice array and `flatten`, preserving group/check order without
raising the recursion limit or introducing dynamic dispatch. A separate tensor
regression checks ordered complete groups and unique IDs; the old caller-record
and caller-spill registration guards retain equivalent flat-table checks.

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_print_aliases conditional_return_computed_prints conditional_return_effects --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor_native_iteration_registration --test-threads=1
```

Seven new focused tests cover six runtime/structure/admission/budget tests and
one default-source AOT test. The independent source oracle covers 616 cases
and 1232 ordinary/reversed YIR executions, with additional literal/sibling-scope
and current-parent-rebinding probes. Existing reversal also changes edges,
functions and each function's body-node array. Tests inspect original-only
captures, pure helpers, unchanged callees, idempotence and whole-module atomic
veto. Malformed NIR probes test the lowering defense directly, independently of
the front end's earlier rejection. Statement and node probes retain original
32/33, expanded 32/33 and complete-expression 4096/4097 boundaries.

Sixteen AOT variants cover selected copies and computations, skipped zero
divisors, entry/argument/tail failures, partial/continuation behavior and real
zero returns. Seven selected traps are checked through native failure signals;
buffered stdout at a trap is not certified. Prior print-prefix and computed
print-argument tests retain their own historical receipts.

## Acceptance Receipt

Interleaved alias verification: the 2026-10-05 follow-up passes
2087 distinct selected tests:

* 1982 compiler-unit tests in the broad frontend/lowering/optimizer sweep
* Ten separately selected native AOT unit tests
* 58 native bridge tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

All 23 combined print-prefix/computed-print/alias focused tests pass. Five native
print/alias tests from that run and five separately rerun prior native tests
make the ten native units above; ordinary focused tests already counted in the
broad sweep are not added again. Initial negative-fixture construction was
rejected by the front end before exercising lowering; corrected malformed NIR
probes passed before the broad sweep. Production lowering did not change after
that sweep. The first tensor build failed at iterator type depth before tests
ran; only the repaired flat-registry run contributes its 29 tests.

Static drift passes 1959 definitions/13272 patterns. All 44 changed/new file
caps, formatting, whitespace, 4736 UTF-8 files and 3083 local documentation links
pass. The rebuilt CLI reports `active/97`, 1961/1961 live drift checks and clean
coverage, hierarchy and task-card lineage. Original evidence prefixes and
historical receipts remain intact. Post-receipt tensor reruns are not extra
acceptance cases. The application prerequisite remains active, not complete.
The historical computed print-argument checkpoint remains 2079 distinct selected
tests at `active/96`; it is not relabeled as this follow-up's receipt.

This proof does not establish full-workspace acceptance, fresh Linux/GPU
execution, self-contained heterogeneous application closure, a measured speedup
or formal memory-safety certification. Three historical blocked Unix-socket
tests and eleven historical strict-Clippy findings remain pending and uncounted.
