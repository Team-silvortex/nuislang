# Native One-Sided Effect-Result Joins v1

This bounded CPU follow-up extends the
[partial paired-result proof](nuis-native-partial-effect-result-joins-v1.md).
It is worktree development after `585eb409` (`beta-0.15.9`), not a release
or complete heterogeneous-application acceptance.

## Source Exits And Expression Results

An initializer may now have one continuing result arm opposite a wholly exiting
arm. The continuing arm must reach a fresh final exact owned bool/i64 binding;
the exiting arm must independently prove an actual enclosing-function return on
every path. The enclosing return type need not equal the initializer result type:

```nuis
if outer {
    let selected: bool = if gate {
        print(31);
        return 0;
    } else {
        let local = 100 / value;
        print(local);
        local > 0
    };
    if selected { print(11); return 11; }
    print(19);
    return 19;
}
print(77);
return 19;
```

Both orientations, typed let/const destinations, nested wholly exiting arms and
continuing arms with partial early exits use the same proof. The parser now
represents control-expression tails as expression statements, rather than
synthetic returns. Explicit `return` remains an enclosing-function exit through
terminal rewriting, including if/if-let, match and else-if expression branches.
Function and lambda tails retain their existing return representation.

Root typed let if/match initializers, like const initializers, publish their
destination only after branch lowering succeeds. Other child locals remain
lexical, and initializing destinations cannot be read as forward values. This
does not claim a general binding/rebinding fix or a new public bottom/never type.
Parsing preserves exit operands; the effect-region validator independently
checks their enclosing result type before admission or alias substitution.

## Presence And Installation

Prepared join values are optional per source arm. A wholly exiting arm contributes
no result atom, not a made-up zero/false result. If both arms continue, the existing
paired same-name/same-kind/exact-type proof still applies. If neither continues,
no result join is prepared. An arm that merely lacks a result, still continues,
uses a sibling local, or contains statements after an all-path exit grants no
result presence.

All source conditions, checked operands and calls execute at their selected
source positions. The installed single-value helper reads only the continuing
arm's staged atom, guarded by the merged source-live mask. Two-value helpers
still select between their saved atoms. Inactive helper seeds are inert private
storage and cannot create source exits or make a result present.

Source exits independently retain their saved path and enclosing-function value.
They suppress subsequent joined-result use, checks, prints and parent continuation.
A genuine zero/false result remains distinct from a zero source return. Selected
unused checks before an exit or result still execute and may trap. Conditions and
exit operands are not replayed; unselected checked work cannot kill a continuing
path. Chained one-sided joins use the same continuation authority.

Both outer arms validate before installation, preserving atomic rollback. The
original and expanded shared ledgers remain 32 statements, 4096 expression nodes,
depth below 64 and 32 logical edges. No new NIR/YIR instruction, public ABI,
Nustar dependency, resource transport, ownership exemption or GLM exemption is
introduced. General child-scope export, rebindings, loops, resources/borrow/FFI
transport and general effectful helpers remain outside this proof.

## Reproduction

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_one_sided_effect_joins if_expressions:: match_expressions:: --test-threads=1
```

Four ordinary units and one native AOT unit cover the lowering extension. Three
frontend units cover explicit exits/tails, initializer scope publication and
generic result-tail hints without non-tail context leakage.
An independent source-control evaluator covers 384 core sources: three condition
kinds, bool/i64 results, both orientations, flat/nested complete exits, ordinary
or partially exiting continuing arms and eight inputs. Four const zero-result
sources, one private-name hygiene source, two selected-unused-check sources and
one chained one-sided join bring this to 392 distinct semantic sources and
784 ordinary/reversed executions. Once-only traces reuse core sources.

Tests check pure exact scalar captures, unchanged original callees, generated
NIR validity and idempotence. Missing results, a still-continuing opposite arm,
unreachable suffixes, wrong source exit types, forward reads, borrowed results,
invalid exit arity, two fully exiting arms followed by result use and effectful
callees reject atomically. Statement 32/33 probes include unused aliases in the
exiting arm; fitting a budget does not confer result or source-type authority.

Thirteen default-source AOT variants cover both orientations, nested exits,
partial continuations, bool/i64 results, selected traps, true zero/false results
and const initialization. Three selected traps require native trap signals.
Successful runs require exact stdout and normal completion; neither partial stdout
nor once-only traces at a trap are certified.

## Acceptance Receipt

One-sided effect-result join verification: the 2026-10-07 follow-up passes
1042 distinct selected tests:

* 947 return-control/frontend units: 912 frontend and 35 return-control units,
  including all eight extension units
* 58 native bridge tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

The first 34-unit focused
run passes 33 units, including the matrix and native AOT unit. One frontend test
incorrectly assumed parsing performs return-type validation; the correction tests
preservation of the actual exit operand, with wrong-type rejection separately
covered by atomic lowering vetoes.

The expanded 916-unit frontend/extension selection initially finds six actual
consumer regressions: generic expected-type/constraint propagation, implicit
return inference and higher-order lambda result hints. Result-context routing,
AST tail inference and lambda-tail expected types repair them, while preserving
generic result-call argument hoists and explicit source returns. A subsequent
916-unit run passes all units; final checks also include the additional targeted
generic-tail/non-tail test. The final 947-unit return/frontend selection passes.
Duplicate focused and post-receipt tensor reruns do not add distinct cases.

Static drift passes 2044 definitions/13960 literal patterns. All 68 changed/new
file caps, formatting, whitespace, 4729 recognized UTF-8 text files and 4666 local
Markdown links pass. The actual rebuilt CLI reports `active/99`, 2044/2044 live
drift checks and clean coverage, hierarchy and task-card lineage. Six complete
previous field histories are retained byte-for-byte and checked by SHA-256,
in addition to the original committed history checks.

The prerequisite remains `active/99`. The preceding 154-case partial paired-result
receipt, 149-case paired-result receipt, 180-case exit-only receipt and earlier
receipts remain historical. Complete earlier tensor evidence and task history
are retained. The next independent optimization boundary is unnecessary saved
selection captures in single-value joins, without weakening masks or atom proofs.

This is local macOS aarch64 CPU work. No fresh Linux/GPU run, full-workspace
acceptance, complete heterogeneous application, measured speedup or formal
memory-safety certification is claimed. Historical three blocked Unix-socket
tests and eleven strict-Clippy findings remain pending, not rerun here.
