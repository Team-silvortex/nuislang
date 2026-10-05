# Native Computed Return Prints v1

This bounded CPU worktree proof follows the
[atom print-prefix checkpoint](nuis-native-return-print-prefixes-v1.md),
after `98ca713e` (`beta-0.15.8`). It is not a public effectful-helper protocol or a
claim of completed application closure.

## Admitted Source

An otherwise admitted conditional scalar return arm may start with integer
prints whose arguments are bounded pure computations of exact owned `i64`.
Checked arithmetic, completed pure calls and typed local record-field observations
are supported; ready integer atoms retain their existing no-helper path.

```nuis
if outer {
    print(88);
    print(observe(stamp));
    print(100 / stamp);
    return helper(produce(left)) && (gate || helper(produce(right)));
}
```

Then-only, else-only and both-arm prefixes compose with independently admitted
pure return tails, including partial/suffix/continuation exits and zero results.
The original parent entry is still computed once;
print-only computation grants no return-tail eligibility. The tail independently
satisfies the old source-exit and checked/call-work proof. This is not a
restriction on all ordinary language prints.

## Ordering And Isolation

Each computed print receives its own pure value helper. The helper takes the
saved selected/inverted entry plus only ready original scalar captures. Its
selected arm evaluates the complete original argument; the unselected arm
returns zero without evaluating argument calls or checks. Printing occurs only
in the parent, after the helper has completed, using the same saved entry.
The next print computation and return tail follow the prior print's statement
effect anchor. Identical calls are not coalesced and unselected work is not run.

Only exact owned bool/i64 parent atoms cross the helper boundary. Types and
purity come from the original helper catalog; generated signatures grant no new
catalog authority. Checked local record construction retains unused field work.
Print-only inputs do not become return-helper captures. There is no new YIR
instruction, ABI, GLM/ownership exception, Nustar coupling or native-word limit.

## Bounds And Rejection

Both original arms and expanded suffix plans include complete print argument
trees in the shared 32-statement/4096-node/depth-below-64 admission budget.
Print expressions are ordinary roots: logical gates hidden inside arguments
do not acquire the return tail's separate 32-edge logical-root permission.
Preflight precedes recursive type inference, capture collection or cloning.
Both arms must prepare successfully before helpers, names or source bodies
change. Capture qualifiers and exact current parent binding versions remain
mandatory; an invalid later argument cannot install an earlier helper.

Interleaved effects, internal prints, effectful callees, parent record/resource/
borrowed captures, logical argument embeddings and general FFI effects remain
separate work. The common one-sided print shortcut is still atom-only; it does
not eagerly compute a fallible argument before a guard.

The later [interleaved atom-alias proof](nuis-native-return-print-aliases-v1.md)
supersedes this boundary only for fresh total bool/i64 copies and literals
around top-level prints. Computed prefix initializers and general interleaved
effects remain separate; the receipt below belongs to this earlier checkpoint.

## Reproduction

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_computed_prints conditional_return_effects --test-threads=1
```

Seven new focused tests cover five runtime/structure/admission/budget tests and
two default-source AOT tests. The independent source oracle covers 576 cases
and 1152 ordinary/reversed YIR executions, including skipped and selected zero
divisors, partial tails, earlier exits and zero returns. Additional probes retain
each call with a parent print between calls, current rebinding, fresh generated
names and selected signed division/remainder overflow. Reversal changes nodes,
edges, functions and each function's body-node array. Structural tests inspect
pure helpers, minimal captures, original callee preservation, atomic rejection
and idempotence; node/depth probes retain 4096/4097 and 63/64 boundaries.

Nineteen AOT variants exercise real host binaries, including nine selected traps
(argument, entry, tail and signed-overflow failures). Successful outputs and
native failure signals are checked; buffered stdout on a trap is not certified.
The nine earlier atom-prefix tests remain separately identifiable and passing.

## Acceptance Receipt

Computed argument verification: the 2026-10-05 follow-up passes
2079 distinct selected tests:

* 1976 compiler-unit tests in the broad frontend/lowering/optimizer sweep
* Nine separately selected native AOT unit tests
* 58 native bridge tests
* 28 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

All seven new focused tests pass in the final rerun. Two fixtures were
strengthened after the broad sweep: a well-typed logical argument still vetoes,
and repeated identical calls remain distinct. The first combined native/focused
rerun passed 13 of 14 tests, including all nine native tests; the sole failure
was the test reader locating identical event text at its first occurrence.
The corrected reader records actual event indices. Production lowering logic
did not change after the broad sweep. Overlapping tests and reruns are not added
again to the selected total.

Static drift passes 1949 definitions/13210 patterns. All 34 changed/new file
caps, formatting, whitespace, 4728 UTF-8 text files and 3073 local documentation
links pass. The rebuilt CLI reports `active/96`, 1951/1951 live drift checks and
clean coverage, hierarchy and task-card lineage, with the original evidence
prefix and historical checkpoints retained. Post-receipt tensor reruns are not
extra acceptance cases. The application prerequisite remains active, not complete.
Historical atom-prefix acceptance remains 2072 distinct cases at `active/95`;
no old receipt is attributed to this follow-up.

This local CPU proof does not establish self-contained heterogeneous application
closure, fresh Linux/GPU execution, a measured speedup or formal memory-safety
certification. Three previously blocked Unix-socket tests and eleven historical
strict-Clippy findings remain pending and uncounted.
