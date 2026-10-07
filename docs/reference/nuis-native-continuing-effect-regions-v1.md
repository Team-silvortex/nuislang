# Native Continuing Effect Regions v1

This bounded CPU proof follows the
[logical staged initializer checkpoint](nuis-native-logical-staged-initializers-v1.md)
included in `585eb409` (`beta-0.15.9`). It is subsequent worktree development,
not a release or completed native heterogeneous application closure.

The subsequent [internal-exit proof](nuis-native-effect-region-exits-v1.md)
extends selected regions with bounded scalar returns and continued-path guards.
The continuing-only scope and acceptance below remain a historical checkpoint;
the exit extension has its own source authority, boundaries and fresh receipt.

## Selected Continuing Regions

A bounded fully continuing internal `if` may now occur before or between
selected scalar initializers and integer prints in a conditional return arm:

```nuis
if outer {
    let ready = gate && helper(produce(value));
    print(88);
    if (ready || gate) && helper(produce(right)) {
        let sample = observe(right);
        print(sample);
        if nested { print(100 / tail); } else { print(55); }
        const copy: i64 = sample;
        print(copy);
    } else {
        let sample = observe(tail);
        print(sample);
    }
    let after = observe(tail);
    print(after);
    return ready;
}
```

The outer entry is validated against the original parent scope and saved once.
Each internal condition is independently validated as an exact owned bool,
including admitted direct logical roots, before alias substitution. A pure
selected value helper evaluates that condition once at its original position.
Unselected paths receive an inert false seed. Two compiler-owned path snapshots
combine the saved condition with the saved parent path; neither child can run
when its parent is unselected. Descendants consume saved bool paths rather than
replaying source conditions.

Selected bool/i64 bindings retain exact types, original order, unused checks
and once-only snapshots. Actual prints remain parent effects; generated helpers
remain pure. Both child scopes start independently from their original parent
scope. Their local names and alias facts do not join the parent or sibling.
The return tail is validated against its original scope before substitution.
Both complete arms must prove admission before real names, helpers or parent
statements are installed; failed admission leaves the module unchanged.

## Authority And Bounds

The prefix ends after the last top-level statement containing any descendant
print, so a region with only nested prints is covered. The discovery walk is
iterative and bounded before recursive typing or cloning. The same root ledger
charges the complete source prefix/children/tail and then the source prefix
with the expanded pure tail: 32 statements, 4096 expression nodes, depth below
64 and 32 logical edges. Child regions never receive fresh independent budgets;
removed aliases and tail fanout still count. Ordinary print roots do not gain
logical authority. The earlier ordinary print-only reservation is unchanged.

A real source return is still required. Independently proven selected binding
or condition work may support the pure return tail, including a checked
condition whose branches are empty. Total-only work or checked print arguments
alone do not grant return eligibility. Source captures remain exact owned
bool/i64 values; record/resource/borrow/FFI captures and effectful callees gain
no new authority. An admitted outer-entry record operand remains parent-local;
internal conditions cannot use that proof to gain a private record capture.

At this historical checkpoint the internal effect region could not contain early returns, rebinding, loops,
resource operations or arbitrary effects. Branch-local value joins need their
own proof. Existing complete/partial pure return-tail routes remain separate.
This change adds no NIR/YIR instruction, Nustar dependency, transport-limit
increase, public ABI change, GLM exemption or ownership exemption.

## Reproduction

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_control_regions --test-threads=1
```

Seven new tests contain six semantics/structure/admission/budget tests and one
default-source AOT test. An independent imperative evaluator supplies exact
successful results, print order and helper counts for 432 core cases: three
condition kinds, two nesting modes, three return/continuation modes, three arm
shapes and eight inputs. Positions before, between or without top-level prints
rotate across inputs; this is not a full position cross-product. Nine further
sources cover condition-only work, empty branches, unused checks, computed
outer entries and dependent post-region bindings; a tenth covers private-name
collisions. The combined 442 sources have 884 ordinary/reversed semantic executions.
Successful traces separately check once-only conditions and selected `observe`
calls. Error probes require a zero-divisor error, not a partial failure trace.

Structural tests check pure generated helpers, exact scalar captures, saved
paths, unchanged original callees, original child scopes, idempotence and atomic
vetoes. Budget tests cover original/expanded statement 32/33, shared and
expanded-tail logical 32/33, complete node 4096/4097 and depth 63/64 boundaries.
An older continuing-child veto became an independently executed positive
example; its replacement retained the internal early-return veto at this checkpoint.

Sixteen default-source AOT variants cover selected and skipped nested work,
condition-only checks, unused checks, dependent bindings, continuation, early
exit and a true zero return. Four selected failures require native trap signals;
partial stdout or helper traces are not certified at a trap. Successful native
runs require exact stdout and normal completion.

## Acceptance Receipt

Continuing control-region verification: the 2026-10-06 follow-up passes
2108 distinct selected tests:

* 2000 compiler units in one fresh wide frontend/lowering/optimizer sweep
* Thirteen separately counted native AOT units
* 58 native bridge tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

The initial 49-case focused batch passed all thirteen native units and exposed
two obsolete assertions that rejected newly admitted continuing children.
Those ordinary tests independently executed selected/unselected positive
cases and retained internal early-return vetoes at this checkpoint. All 36 ordinary cases passed in
the corrected wide sweep; production lowering is unchanged after the native
acceptance. Three corrected boundary tests also pass a focused rerun. That
rerun, the initial seven-test run and post-receipt checks add no distinct cases.

Static drift passes 1991 literal definitions/13579 patterns. All 25 changed/new
file caps, formatting, whitespace, 4757 UTF-8 files and 3103 local documentation
links pass. The actual rebuilt CLI reports `active/99`, 1991/1991 live drift
checks and clean coverage, hierarchy and task-card lineage. Original evidence
and task prefixes and complete preceding history remain intact.

The prerequisite remains `active/99`, not completed application closure. The
preceding logical checkpoint is historical: 2101 distinct selected tests,
1981/1981 live drift checks and `active/99`. Earlier 2094/2087/2079 receipts
remain separate, not fresh results for this proof.

This is local macOS aarch64 CPU evidence. No full-workspace acceptance, fresh
Linux/GPU execution, complete heterogeneous application, measured speedup or
formal memory-safety certification is claimed. The historical three blocked
Unix-socket tests and eleven strict-Clippy findings are pending, not rerun here.
