# Native Effect-Region Exits v1

This bounded CPU follow-up extends the
[continuing-region checkpoint](nuis-native-continuing-effect-regions-v1.md).
It is worktree development after `585eb409` (`beta-0.15.9`), not a release
or acceptance of complete heterogeneous application closure.

This receipt is a historical checkpoint. The subsequent
[exit-only region proof](nuis-native-exit-only-regions-v1.md) removes the tail
source-return requirement below only after independently proven internal exits
and wholly continuing pure tails. Its fresh acceptance is recorded separately.

## Selected Internal Returns

An admitted effect region may contain internal exact owned bool/i64 returns
beside continuing paths, before/between selected initializers and integer prints:

```nuis
if outer {
    print(88);
    let ready = gate && helper(produce(value));
    if helper(produce(value)) {
        let local = observe(right);
        print(local);
        if nested { print(44); return false; }
        print(55);
    } else {
        let local = observe(tail);
        print(local);
    }
    let ignored = observe(tail);
    print(89);
    return ready;
}
```

The selected return expression runs once at its source position in a pure
guarded scalar value helper. Its saved scalar and its saved selected-path bool
are distinct: returning false or zero is an exit, not a continuation seed.
The exiting child contributes an inactive continuation; sibling continuation
paths merge through fresh compiler-owned bool snapshots. Subsequent conditions,
initializers, unused checks and print arguments consume only the continued path.
Actual prints remain parent effects and generated helpers remain pure.

Parent return publication consumes only the saved exit bool and saved value.
The pure-tail helper uses the saved arm continuation and the existing private
typed exit/value signal. It wraps normalized original source tails, never scalar
seeds synthesized by another return proof. Exited paths cannot execute checked
or call-backed suffix work. Multiple exits retain original first-exit identity;
inactive siblings cannot override it. Conditions and initializers are not replayed.

## Source Authority And Bounds

Original lexical scopes and exact declared function result types are validated
before rewriting aliases or introducing private snapshots. Children retain
separate scopes and cannot export bindings to siblings or their parent. Source
code after an unconditional exit, or after an internal branch whose two arms
both exit, does not gain normalization authority. Both complete outer arms
must prove admission before any real names/helpers/definitions are installed.

The unchanged shared ledgers charge the complete original arm and then its
source effect prefix plus expanded pure tail: 32 statements, 4096 expression
nodes, depth below 64 and 32 logical edges. Internal return values now have
direct root authority, but ordinary print/call/comparison leaves do not. Removed
aliases and pure-tail fanout still count. Compiler-owned continuation wiring
does not duplicate source work or reset child budgets.

This route still requires a tail source return admitted by the existing pure
tail proof, as well as the existing independently proven checked/call work.
An exit-only effect region without such a tail remains excluded, even if a
child returns. Rebinding, loops, resource/borrow/FFI transport, general effectful
callees and branch-local value joins also remain excluded. The proof adds no
NIR/YIR instruction, Nustar dependency, transport-limit increase, public ABI
change, ownership exemption or GLM exemption.

## Reproduction

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_effect_exits --test-threads=1
```

Six ordinary tests and two default-source AOT tests cover the new route. An
independent imperative evaluator provides results, successful print ordering
and bool-helper counts for 900 core sources: three condition kinds, five
return-value modes, two nesting depths, three outer arm shapes and ten inputs.
Four further sources cover private-name hygiene, a later competing exit,
an earlier parent exit and an unused checked binding before an internal exit.
Eight further sources check composition with real false/zero partial-tail exits
and selected continuation checks. The combined 912 distinct sources have 1824 ordinary/reversed semantic
executions; duplicate smoke probes are not additional distinct sources.
Successful traces independently check that post-return `observe` calls stop.
Error probes require a zero-divisor error, not a partial failure trace.

Structural tests check pure exact-scalar helper captures, unchanged original
callees, saved exit/live/continuation identities, NIR verification and
idempotence. Invalid result types, void/resource/missing/logical-leaf returns,
unreachable suffixes and, at this checkpoint, exit-only regions must fail admission
atomically. The later exit-only proof replaces that blanket veto with positive
execution and a retained no-source-exit veto.
Budget tests cover statement 32/33, shared logical 32/33, node 4096/4097 and
depth 63/64 limits. Previously blanket-rejected legal internal exits become
executed positive cases; their replacements retain wrong-result-type vetoes.

Twenty-five default-source AOT variants cover nested bool/i64/short-circuit and
checked returns, skipped paths, false/zero exit identity, first versus later
exits, earlier parent exit suppression and partial-tail composition. Seven selected traps require native
trap signals. Successful native runs require exact stdout and normal completion;
partial stdout and helper traces at a trap are not certified.

## Acceptance Receipt

Effect-region exit verification: the 2026-10-06 follow-up passes 284 distinct
selected tests:

* 189 return-control units: 149 ordinary units and 40 native AOT units
* 58 native bridge tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

The return-control acceptance consists of one fresh 187-case regression and
two subsequently added partial-tail composition probes. The first ten-test
focused batch, duplicate smoke probes and post-receipt tensor checks add no
distinct cases. The final native acceptance includes all twenty-five new AOT
variants and seven selected traps. Production lowering is unchanged after the
first focused acceptance; later changes extend tests, guards and documentation.

Static drift passes 2000 literal definitions/13654 patterns. All 30 changed/new
file caps, formatting, whitespace, 4762 UTF-8 files and 3109 local documentation
links pass. The actual rebuilt CLI reports `active/99`, 2000/2000 live drift
checks and clean coverage, hierarchy and task-card lineage. Original evidence
and task prefixes and complete preceding history remain intact.

The prerequisite remains `active/99`, not completed application closure. The
previous 2108-case continuing-region receipt and all earlier checkpoint receipts
are historical, not fresh results for this extension. Complete existing tensor
evidence and task history are retained.

This is local macOS aarch64 CPU work. No fresh Linux/GPU run, full-workspace
acceptance, complete heterogeneous application, measured speedup or formal
memory-safety certification is claimed. The historical three blocked Unix-socket
tests and eleven strict-Clippy findings are pending, not rerun here.
