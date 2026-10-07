# Native Paired Effect-Result Joins v1

This bounded CPU follow-up extends the
[exit-only region proof](nuis-native-exit-only-regions-v1.md).
It is worktree development after `585eb409` (`beta-0.15.9`), not a release
or acceptance of complete heterogeneous application closure.

## Paired Result Authority

An existing typed `let/const` if-expression may now export its fresh exact owned
bool/i64 result from fully continuing effect-region selection arms:

```nuis
mod cpu Main {
    @noinline fn event(outer: bool, gate: bool, left: i64, right: i64) -> i64 {
        print(99);
        if outer {
            let selected: i64 = if gate {
                let local = 100 / left;
                print(local);
                local
            } else {
                let local = 100 / right;
                print(local);
                local
            };
            print(selected);
            return selected;
        }
        print(77);
        return 19;
    }
    fn main() -> i64 {
        let result = event(true, true, 2, 0);
        print(result);
        return 0;
    }
}
```

The selected division runs once; the unselected zero divisor is never checked.
Successful output is `99`, `50`, `50`, `50`, in that order. The independent
source return remains necessary: joining a value does not establish an exit.

The existing frontend represents this initializer as an if statement whose two
arms end in same-name, same-kind result bindings. Both original child scopes and
types prove admission before the new private join is prepared. The destination
must be absent from the parent scope; both result kinds must be exactly equal
owned bool/i64. Only that paired result is exported, never the child environments
or their other alias facts. Ordinary branch-local bindings do not gain new source
visibility. Let/const mismatches, rebinding and resource/borrow results reject.

Both result-producing arms must be free of actual source exits, at every nested
level. Source exits before or after the initializer in the surrounding region
remain independently supported. Exiting result-producing arms need a separate
presence/continuation proof and are not enabled by this extension. The subsequent
[partial-result proof](nuis-native-partial-effect-result-joins-v1.md) separately
admits source returns when each arm still reaches its paired result on every
continuing path. The [one-sided result proof](nuis-native-one-sided-effect-result-joins-v1.md)
separately admits one continuing result opposite a wholly exiting arm.

All computations, checked unused bindings and prints stay staged at their source
positions, under saved parent/child live paths. A join reads only already-staged
typed scalar snapshots or total atoms. Its pure scalar helper uses the saved bool
selection condition and current live mask; it cannot replay an initializer or
condition. Private false/zero seeds on inactive paths are not source exit signals.
Actual prints remain in the parent. Nested and chained result selections compose
without capturing resources or modifying original producer/helper definitions.

Complete original and expanded prefix/tail ledgers remain shared: 32 statements,
4096 expression nodes, depth below 64 and 32 logical edges. Both outer arms prove
admission before installation; failed preparation remains atomic. No new NIR/YIR
instruction, public ABI, transport limit, Nustar dependency, ownership exemption
or GLM exemption is introduced.

The regression also fixes frontend publication of explicitly typed `const`
if/match-expression results. Only the validated final result enters the outer
binding environment, after branch lowering succeeds. Child locals, initializing
destination self-reads, mismatched result types and missing required annotations
remain rejected. No general let/match or binding-rule redesign is claimed.

## Reproduction

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_effect_joins const_control_expression_results --test-threads=1
```

Six ordinary join units, one default-source AOT unit and one frontend const unit
cover this extension. An independent imperative evaluator covers 192 core
sources: three condition kinds, bool/i64 results, flat/nested selections,
terminal/early-exit surroundings and eight inputs. Each semantic source executes
in normal and reversed graph order. Smoke, const, zero-result, selected unused
check, chained join, private-name hygiene and pre-initialization false/zero exits
bring this to 203 distinct semantic sources and
406 ordinary/reversed executions. Once-only trace probes reuse core sources
rather than adding cases.

Tests independently retain exact scalar helper captures, unchanged callees,
valid generated NIR, idempotence, source scope/result/resource/effect vetoes,
atomic rollback and the shared statement 32/33 boundary. General child-scope
export and early-exiting result arms are directly mutated NIR vetoes rather
than silently accepting frontend-invalid programs. This records the 149-case
all-continuing checkpoint; the partial-result follow-up independently admits
matching valid source exits, retaining wrong-exit-type atomic vetoes.

Thirteen default-source AOT variants cover nested selection, const and bool/i64
results, a genuine zero return, parent continuation and skipped checks, including
real false/zero exits before result initialization suppressing every later call.
Three selected traps require native trap signals. Successful runs require exact
stdout and normal completion; neither partial stdout nor helper traces at a
trap are certified.

## Acceptance Receipt

Paired effect-result join verification: the 2026-10-06 follow-up passes
149 distinct selected tests:

* 54 return-control/frontend units, including all eight extension units
* 58 native bridge tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

Focused tests, the reinforced 54-case rerun and post-receipt tensor reruns overlap
the selection and do not add distinct tests. The reinforced rerun adds probes
for actual false/zero exits before initialization to the existing join and AOT
units. Production lowering is unchanged during these broader regressions.

Static drift passes 2019 definitions/13792 literal patterns. All 45 changed/new
file caps, formatting, whitespace, 4773 UTF-8 files and 3121 local documentation
links pass. The actual rebuilt CLI reports `active/99`, 2019/2019 live drift
checks and clean coverage, hierarchy and task-card lineage. Complete original
evidence, task, boundary, action, artifact and validation-command history remain
intact.

The prerequisite remains `active/99`. The preceding 180-case exit-only receipt,
284-case internal-exit receipt, 2108-case continuing-region receipt and earlier
receipts are historical, not fresh counts for this extension. Complete earlier
tensor evidence and task history remain intact.

This is local macOS aarch64 CPU work. No fresh Linux/GPU run, full-workspace
acceptance, complete heterogeneous application, measured speedup or formal
memory-safety certification is claimed. The historical three blocked Unix-socket
tests and eleven strict-Clippy findings remain pending, not rerun here.
