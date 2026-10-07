# Native Partial Effect-Result Joins v1

This bounded CPU follow-up extends the
[all-continuing paired result proof](nuis-native-effect-result-joins-v1.md).
It is worktree development after `585eb409` (`beta-0.15.9`), not a release
or complete heterogeneous-application acceptance.

## Continuing Result Presence

Result-producing if-expression arms may now contain actual source returns,
provided each arm still has a continuing path reaching its paired final result:

```nuis
if outer {
    let selected: i64 = if gate {
        if stop_left {
            print(31);
            return 0;
        }
        let local = 100 / left;
        print(local);
        local
    } else {
        if stop_right {
            print(37);
            return 0;
        }
        let local = 100 / right;
        print(local);
        local
    };
    print(selected);
    return selected;
}
print(77);
return 19;
```

The same source-level typed let/const initializer shape is retained. Both
original arms must reach final same-name, same-kind fresh exact owned bool/i64
bindings on every continuing path. Preparation still rejects source statements
after unconditional/all-path exits. Original lexical/result typing precedes
alias substitution, and a forward result read in an exit cannot use the private
join slot as source authority. Other child locals never become parent facts.

Each actual source return independently proves the enclosing function's exact
owned bool/i64 return type, which may differ from an inner joined result type.
Its saved exit path/value is retained. After executing both guarded arms, the
existing installation mechanism merges their continuing live masks; only this
mask authorizes the following join. The helper reads the saved selection bool
and staged atoms, without replaying conditions, initializers or exit operands.
Inactive false/zero join seeds cannot create source exits or result presence.

On an exiting path, later local computations, unused checks, nested/chained
joins, prints and parent continuation remain suppressed. On a continuing path,
the paired value flows into later computations, prints or actual returns.
Selected checks before a return still trap before that return; checked return
operands execute once. Work in unselected exiting arms cannot kill the selected
continuation. Nested partial joins compose through the same saved live masks.

Both original outer arms prove admission before installation, retaining atomic
rollback. Original and expanded ledgers stay shared: 32 statements, 4096 nodes,
depth below 64 and 32 logical edges. No new NIR/YIR instruction, public ABI,
transport limit, Nustar dependency, ownership exemption or GLM exemption is added.

General child-scope export,
rebinding, loops, resources/borrow/FFI transport and general effectful helpers
remain outside this extension. The
[one-sided result proof](nuis-native-one-sided-effect-result-joins-v1.md) separately
admits a continuing result arm opposite a wholly exiting arm, with explicit
source returns preserved rather than rewritten into initialization values.

## Reproduction

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_partial_effect_joins --test-threads=1
```

Four ordinary units and one native AOT unit cover the extension. An independent
imperative evaluator covers 480 core sources: three selection-condition kinds,
bool/i64 results, flat/nested partial joins, returns before/after checked values,
terminal/continuing outer tails and ten inputs. Atom/computed source exit values
rotate with the inputs. Each source executes with normal and reversed graph
order. Four const sources, two genuine zero-result sources, one unused-check
source, one chained partial join and one private-name hygiene source bring this
to 489 distinct semantic sources and 978 ordinary/reversed executions. Additional
once-only traces reuse core sources rather than adding cases.

Tests retain pure exact scalar captures, unchanged original callees, valid
generated NIR, idempotence, missing/unpaired/unreachable result vetoes,
source exit type and forward-read vetoes, borrow/effect/let-const vetoes and
atomic rollback. Boundary probes retain statement 32/33 and complete exit-operand
node 4096/4097 preflight. Invalid-arity node probes reject atomically at both
widths: fitting the budget never grants source type authority.

Thirteen default-source AOT variants cover const and bool/i64 results, nested
partial joins, actual false/zero exits, genuine continued zero results, parent
continuation and selected checked work. Three selected traps require native trap
signals. Successful runs require exact stdout and normal completion; neither
partial stdout nor helper traces at a trap are certified.

## Acceptance Receipt

Partial effect-result join verification: the 2026-10-06 follow-up passes
154 distinct selected tests:

* 59 return-control/frontend units, including all five extension units
* 58 native bridge tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

The first five-unit run passes four units, including the 480-source matrix and
13 native variants. One supplemental zero-result expectation is corrected: a
positive divisor can still produce a zero truncated quotient, so the source
`positive(local)` must inspect the quotient, not the divisor's sign. The corrected
five-unit rerun and full 59-case return/frontend selection pass. Production
lowering is unchanged by the oracle correction and broader regressions.
Duplicate focused and post-receipt tensor reruns do not add distinct tests.

Static drift passes 2028 definitions/13859 literal patterns. All 50 changed/new
file caps, formatting, whitespace, 4778 UTF-8 files and 3127 local documentation
links pass. The actual rebuilt CLI reports `active/99`, 2028/2028 live drift
checks and clean coverage, hierarchy and task-card lineage. All original field
history and complete preceding evidence/task history remain intact.

The prerequisite remains `active/99`. The preceding 149-case paired-result
receipt, 180-case exit-only receipt and all earlier receipts are historical,
not fresh counts for this extension. Complete earlier tensor evidence and task
history remain intact.

This is local macOS aarch64 CPU work. No fresh Linux/GPU run, full-workspace
acceptance, complete heterogeneous application, measured speedup or formal
memory-safety certification is claimed. Historical three blocked Unix-socket
tests and eleven strict-Clippy findings remain pending, not rerun here.
