# Native Effect-Join Captures v1

This bounded CPU optimization follows the
[one-sided result proof](nuis-native-one-sided-effect-result-joins-v1.md).
It is worktree development after `dff1bdbc` (`beta-0.16.0`), not a release,
measured speedup or complete heterogeneous-application acceptance.

## Capture Boundary

A single-value effect-result helper does not select between two results. Its
captures now come only from the validated result atom. A literal zero/false
requires only the live parameter; a staged scalar snapshot requires live plus
that snapshot. Compared with the preceding installation, each eligible helper
and its call lose one unused bool parameter/argument.

The saved source condition is still required to be an exact ready owned bool
before join preparation can publish a private binding. Dropping a helper
parameter does not delete, reorder or replay source condition execution. The
branch plan, short-circuit work, exit operands, unused selected checks and
source/expanded budgets are unchanged.

If the atom itself reads the saved selection condition, that variable remains a
data capture. This is dependency collection, not removal by name. The live
parameter is always retained, including for literal false/zero: only the merged
source-live mask authorizes result presence after the continuing arm executes.
An inactive seed neither supplies a source result nor creates an exit.

Two-value helpers still capture the saved selector and each distinct atom input,
including when both atoms are identical. Shared inputs are deduplicated. This
follow-up does not claim identical-pair elimination; that requires its own proof.
This describes the 167-test checkpoint. The subsequent
[equal-atom proof](nuis-native-equal-effect-result-joins-v1.md) separately removes
selection for equal validated atoms, without deleting either source arm.
There is no new public ABI, NIR/YIR instruction, Nustar coupling, child-scope
export, resource transport, ownership exception or GLM exception.

## Reproduction

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_join_captures conditional_return_one_sided_effect_joins conditional_return_partial_effect_joins conditional_return_effect_joins if_expressions:: match_expressions:: --test-threads=1
```

Four new join-level units cover sixteen single-value combinations of bool/i64,
let/const, orientation and literal/staged atoms; four condition-as-data/live-name
cases; twelve paired literal/shared/distinct-atom cases; and six missing,
non-bool or borrowed-condition vetoes. Prepared names and ready scopes remain
unchanged on those vetoes. Installed call arguments, parameter order, exact
result type, pure helper reads and the live-guarded inactive seed are checked.

A fifth new unit inspects twelve actual source-lowered helpers across atom,
computed and logical conditions, bool/i64 results and both orientations, with
nested complete exits and partial continuations. Each helper has exactly live
plus its staged result, with no unused saved selector; generated NIR verifies
and outlining is idempotent. These sources reuse the preceding semantic matrix,
not twelve additional execution cases.

The selected regression suite retains ordinary/reversed YIR execution and
default-source native AOT for one-sided, partial and fully paired joins. Native
success requires exact stdout and normal completion; selected traps require the
existing trap contract, not certification of partial stdout at failure.

## Acceptance Receipt

Effect-join capture verification: the 2026-10-07 follow-up passes 167 distinct
selected tests:

* 72 return-control/frontend units, including all five new capture units
* 58 native bridge tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

The initial focused five-unit run overlaps this selection and adds no distinct
cases. Post-receipt tensor reruns likewise do not add cases. The preceding
1042-test one-sided receipt and all earlier receipts remain historical; this
optimization does not claim a fresh full-frontend run.

Static drift passes 2051 definitions/14005 literal patterns. All twelve
changed/new file caps, formatting, whitespace, 4732 recognized UTF-8 text files
and 4671 local Markdown links pass. The actual rebuilt CLI reports `active/99`,
2051/2051 live drift checks and clean coverage, hierarchy and task-card lineage.
All six complete prior field histories are retained and checked by SHA-256,
alongside the original committed-history checks.

The selected prerequisite remains `active/99`. The next candidate is identical
paired atoms: prove any selector reduction independently while preserving both
source arms' selected work, original condition staging and result-presence masks.
No fresh Linux/GPU run, full-workspace acceptance, measured speedup or formal
memory-safety certification is claimed. Historical three blocked Unix-socket
tests and eleven strict-Clippy findings remain pending, not rerun here.
