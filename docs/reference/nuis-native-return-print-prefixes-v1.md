# Native Return Print Prefixes v1

This is a bounded CPU lowering proof after `98ca713e` (`beta-0.15.8`), not a new
public protocol or general effectful-helper contract. It follows the
[computed logical tree proof](nuis-native-computed-logical-gates-v1.md).

## Admitted Source

An otherwise admitted top-level conditional scalar return arm may begin with
one or more prints of an integer literal or an already-bound, exact owned `i64`.
The remainder must independently satisfy the existing pure return proof.
Then-only, else-only and both-arm prefixes are supported. An opposite print-only
continuation arm does not acquire source-return authority.

```nuis
if outer {
    print(88);
    print(stamp);
    return helper(produce(left)) && (gate || helper(produce(right)));
}
```

The outer entry may be an already-bound bool, an admitted pure computed predicate,
or a bounded direct logical tree. Return tails retain their independent direct,
fresh binding, complete-tree, partial-tree, suffix and continuation admission.
An actual return of zero or false is not confused with absence of a source exit.

## Effect Ordering

The complete original entry is bound once in the parent before any branch print.
Each print uses that saved value, or its inverse for the else arm. The existing
pure return computation then uses the same saved bool. Prints never enter the
pure helper catalog, and variables used only by prints are not helper captures.

One-sided atom prints use the existing `cpu.guard_print` instruction. Its node
joins the statement effect chain, so previous parent effects precede the print
and later return calculations follow it independently of node-array order.
At this atom-only checkpoint, fallible or call-backed print arguments were
outside the proof. The subsequent [computed argument proof](nuis-native-computed-return-prints-v1.md)
admits bounded pure work inside selected helpers, never eager work outside a
guard. No new instruction, ABI, GLM/ownership exception,
provider dependency or native-word limit is introduced.

## Bounded Admission

Both the original complete arm and the expanded pure suffix plan reserve the
leading print statements and their atom nodes before recursive typing or cloning.
The existing shared limits remain 32 statements per original/expanded arm,
4096 expression nodes, depth below 64 and 32 logical edges. Common-tail expansion
cannot multiply a fresh budget for each branch.

Rejection leaves source bodies and helper definitions unchanged. Prefixes after
pure locals, internal leaf prints, computed print arguments, effectful callees,
loops, rebindings, unknown names and new borrowed/resource/aggregate captures
remain outside this proof. These are residual boundaries, not claims that all
such language constructs are forbidden or permanently unsupported.
Computed arguments now have the independent follow-up above; the atom receipt
below remains historical and does not retroactively include that work.

## Reproduction

Run from the repository root with serial, nonincremental builds:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_effects --test-threads=1
```

The nine focused tests include seven reference/structure/admission/budget tests
and two default-source AOT tests. The independent source oracle exercises 1345
runtime source cases and 2690 ordinary/reversed YIR executions. Reversal changes
nodes, edges, functions and every function's body-node array. Successful cases
compare exact print order, main result and helper-call count; failing cases check
selected arithmetic failure. Sixteen AOT variants check actual host execution,
including four selected traps, early exits, current print atoms and zero returns.
The native failure harness checks failure signals, not buffered stdout on traps.

Original/expanded 32/33-statement, shared 4096/4097-node, 32/33-edge and
63/64-depth boundaries have separate probes. Eight former leading-print rejection
fixtures are retained as positive evidence; internal/callee effects still veto.

## Acceptance Receipt

The 2026-10-05 worktree follow-up passes 2072 distinct selected tests:

* 1971 compiler-unit cases across the broad sweep and two corrected-fixture reruns
* Seven separately selected native AOT unit tests
* 58 native bridge tests
* 28 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy case

The first broad sweep passed 1969 cases and failed two obsolete expectations
that outer leading prints must be rejected. Both original fixtures now remain
positive evidence, with internal-effect vetoes retained, and both reruns pass.
Only test fixtures changed after that sweep; production lowering did not.
Overlapping focused tests and reruns are not added again to the selected total.
The nine new focused tests pass, including both actual AOT tests. Historical
logical-tree acceptance remains a separate checkpoint.

Static drift passes 1940 definitions/13142 patterns. Formatting, whitespace,
4721 UTF-8 text files, 3068 local documentation links and all 27 changed/new file
caps pass. The rebuilt CLI reports `active/95`, 1942/1942 live drift checks and
clean coverage, hierarchy and task-card lineage with the historical evidence
prefix retained. This is not completed application closure. Post-receipt reruns
retain the same 28 tensor tests and do not add new acceptance cases. The earlier
three blocked Unix socket tests and eleven strict-Clippy findings remain pending
and uncounted.

## Remaining Work

This checkpoint's coordinate remains historical `active/95`; the follow-up is
`active/96`, not application closure. Interleaved effects,
internal-leaf effects, selected fallible print arguments and general resource/
FFI effects need separate effect-region proofs; bounded pure computed arguments
are superseded only by the matching follow-up proof. Logical gates hidden in ordinary
leaf arguments and changing capture transports remain independent work.
This CPU proof alone does not establish a self-contained heterogeneous application,
fresh Linux/GPU execution, formal memory-safety certification or a measured speedup.
