# Native Exit-Only Regions v1

This bounded CPU follow-up extends the
[internal effect-region exit proof](nuis-native-effect-region-exits-v1.md).
It is worktree development after `585eb409` (`beta-0.15.9`), not a release
or acceptance of complete heterogeneous application closure.

## Internal Source Authority

An effect region with a validated internal exact owned bool/i64 source exit
may now have an empty or wholly continuing pure tail. A tail source return is
not required on this separate route:

```nuis
if outer {
    print(88);
    let ready = gate && helper(produce(value));
    if ready {
        print(44);
        return false;
    }
    let ignored = observe(tail);
    print(89);
    let final_check = observe(tail);
}
print(77);
return false;
```

The ordinary pure-return proof still requires its own actual source return.
The new fallback requires a private continuation slot minted only after exact
internal source exits have proved admission. Both original tails must independently
prove bounded exact scalar captures, purity and continuation on all paths.
A tail containing any source return cannot bypass the ordinary return proof.
The existing selected checked/call work gate is retained; prints alone, including
checked print arguments, do not establish work eligibility.

The existing saved path/value mechanism publishes internal returns. False/zero
source returns remain exits. Empty tails and other continuing leaves use a
private typed non-exited signal; false/zero continuation seeds cannot return
from the parent. Continued paths retain selected unused checks and resume the
parent suffix. Exiting paths suppress all later checks, print arguments and
effects. A prefix whose two children exit is admitted only with no unreachable
source suffix inside that arm.

Original lexical scopes are validated before aliases/private snapshots. Child
bindings cannot leak to siblings or parents. Both complete outer arms prove
admission before installation, with atomic rollback for an invalid second arm.
Original and expanded prefix/tail ledgers remain shared: 32 statements, 4096
expression nodes, depth below 64 and 32 logical edges. No seed, removed alias
or fanout resets a source budget. No new NIR/YIR instruction, Nustar dependency,
public ABI, transport limit, ownership exemption or GLM exemption is introduced.

Rebinding, loops, general effectful callees, resource/borrow/FFI transport and
branch-local value joins remain outside this bounded proof. The subsequent
[paired result-join proof](nuis-native-effect-result-joins-v1.md) separately admits
fresh owned bool/i64 if-expression results from fully continuing selection arms;
other child locals and exiting result arms remain outside that extension.

## Reproduction

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_exit_only --test-threads=1
```

Six ordinary units and one default-source AOT unit cover the extension.
The independent imperative evaluator covers 900 core sources: three condition
kinds, five result modes, two nesting depths, three outer-arm shapes and ten
inputs, rotating empty, checked and branch tails. Every source runs with normal
and reversed graph ordering. Supplemental probes cover all-exiting prefixes,
private-name hygiene, continuation traps and skipped traps, pure scalar helper
captures, unchanged original callees, once-only observations and idempotence.
Eight additional tail-shape smoke sources, three all-exiting/hygiene sources,
two continuation-check sources and three checked-tail-only work sources bring
the extension to 916 distinct semantic sources and 1832 ordinary/reversed runs.
Four overlapping smoke sources do not add distinct cases. Tail-only checked
work can prove eligibility; total and print-only-work regions still cannot.
Three legal legacy signal/suffix sources also become executed positives, with
wrong-result-type replacements retaining their atomic vetoes. This makes 919
distinct new semantic sources and 1838 ordinary/reversed runs across the extension
and migrated probes. Invalid source scope is checked both by the frontend and
by directly mutated NIR at the lowering boundary.
Atomic vetoes retain invalid result types, forward/child-local captures,
no-source-exit regions and failed tail returns. Boundary tests cover empty-tail
statement 32/33 and expression 4096/4097, alongside existing shared-ledger tests.

Seventeen default-source AOT variants cover first exits, continued parent
fallbacks, all-exiting prefixes, short-circuit selection and false/zero identity.
Five selected traps require native trap signals. Successful runs require exact
stdout and normal completion. Neither partial stdout nor helper traces at a
trap are certified.

## Acceptance Receipt

Exit-only region verification: the 2026-10-06 follow-up passes 180 distinct
selected tests:

* 85 return-control units: 69 ordinary and 16 native AOT units
* 58 native bridge tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

The initial 85-case return regression passes 82 cases. Three boundary probes
are repaired: one separates frontend scope rejection from directly mutated NIR
atomicity, and two migrate formerly blanket-rejected legal sources to executed
positives while retaining wrong-result-type vetoes. All four repair/eligibility
probes pass; the eligibility rerun overlaps an already counted case. Production
lowering is unchanged after the first broad regression. Duplicate smoke and
post-receipt tensor reruns do not add distinct tests.

Static drift passes 2008 definitions/13714 literal patterns. All 37 changed/new
file caps, formatting, whitespace, 4767 UTF-8 files and 3115 local documentation
links pass. The actual rebuilt CLI reports `active/99`, 2008/2008 live drift
checks and clean coverage, hierarchy and task-card lineage. Original evidence
and task prefixes and complete preceding history remain intact.

The prerequisite remains `active/99`. The preceding 284-case internal-exit
receipt, 2108-case continuing-region receipt and all earlier receipts are
historical, not fresh counts for this extension. Complete earlier tensor
evidence and task history remain intact.

This is local macOS aarch64 CPU work. No fresh Linux/GPU run, full-workspace
acceptance, complete heterogeneous application, measured speedup or formal
memory-safety certification is claimed. The historical three blocked Unix-socket
tests and eleven strict-Clippy findings remain pending, not rerun here.
