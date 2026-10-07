# Native Equal Effect-Result Joins v1

This bounded CPU follow-up extends the
[single-value capture proof](nuis-native-effect-join-captures-v1.md).
It is worktree development after `dff1bdbc` (`beta-0.16.0`), not a release,
measured speedup or complete heterogeneous-application acceptance.

## Exact Atom Equality

Both continuing source arms still independently validate a fresh same-name,
same-kind, exact owned bool/i64 destination and a ready total result atom. The
saved selection condition still validates as an exact ready owned bool before
any private join binding is published. Equality is considered only after those
checks, not as permission to bypass the second arm.

When both atoms are structurally equal, the helper returns that atom directly
under the existing live guard. Equal literal results require only live; a shared
ready scalar snapshot requires live plus that snapshot. When the selector is not
itself result data, the helper and call lose one selection parameter/argument.
The internal result selection disappears for every equal pair. When the saved
condition is also result data, it remains a data capture. Live-name collisions
still use fresh private names.

Source-arm presence remains separate from equality: both prepared values remain
present, unlike a wholly exiting arm's absent value. The merged source-live mask
still authorizes use only after an actual continuing path executes. Two zero/false
inactive seeds do not establish equal source results. Distinct staged snapshots
keep the selector even when their source expressions or callees look identical,
or their runtime results happen to match.

The original branch plan is untouched. Conditions and short-circuit work still
execute once at their source positions, selected calls/checks/prints still run,
unused selected checks may still trap, and source exits suppress subsequent
result use and parent continuation. Both original arms and expanded work retain
the shared 32-statement, 4096-expression-node, depth-below-64 and 32-logical-edge
ledgers. There is no new public ABI, NIR/YIR instruction, Nustar coupling,
child-local export, resource transport, ownership exception or GLM exception.

## Reproduction

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_equal_effect_joins conditional_return_join_captures conditional_return_one_sided_effect_joins conditional_return_partial_effect_joins conditional_return_effect_joins if_expressions:: match_expressions:: --test-threads=1
```

Six new units cover two join-level proofs, three source/control proofs and one
default-source native AOT proof. Updated preceding capture tests now expect
selector-free equal literal/shared pairs while still requiring selectors for
distinct atoms; their earlier 167-test receipt remains historical.

An independent source oracle covers 384 core sources: three condition kinds,
bool/i64 results, literal/shared atoms, let/const destinations, ordinary/partial
continuations and eight inputs. Two nonseed literal/private-name sources, one
same-expression/distinct-snapshot source, one selected-unused-check trap and one
checked-condition trap bring this to 389 distinct semantic sources and 778
ordinary/reversed executions. Eight capture/once-only inspections reuse core
sources, not extra executions in this count.

Tests require pure exact capture/call sets, the live guard, unchanged source
callees, generated NIR validity, idempotence and once-only successful traces.
Second-arm missing/wrong/borrowed values, binding-kind/name mismatches, parent
destinations and unready shared atoms reject before private publication. Source
wrong types, borrowing, unreachable suffixes and effectful callees veto atomically.
A 0..32 unused-alias sweep checks complete source budget admission, including
32/33-statement boundaries at twenty/twenty-one aliases.

Eleven default-source AOT variants cover equal zero/false/shared results, both
selection arms, let/const, partial exits, unselected bad work, inactive outer
paths and selected traps. Three selected traps require native trap signals.
Successful runs require exact stdout and normal completion; neither
partial stdout nor once-only traces at a trap are certified.

## Acceptance Receipt

Equal-atom effect-result join verification: the 2026-10-07 follow-up passes 173
distinct selected tests:

* 78 return-control/frontend units, including all six new equal-atom units
* 58 native bridge tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

Focused/post-receipt reruns overlap this selection and add no distinct cases.
The preceding 167-test capture receipt and all earlier receipts remain historical;
this follow-up does not claim a fresh full-frontend run.

Static drift passes 2059 definitions/14066 literal patterns. All 24 changed/new
file caps, formatting, whitespace, 4739 recognized UTF-8 text files and 4694 local
Markdown links pass. Current entrypoints and the minor snapshot/checklist now
identify beta-0.16.0; older release/acceptance records remain historical. The
rebuilt CLI reports `active/99`, passes all 2059/2059 drift checks and reports clean
coverage, hierarchy and lineage. All six complete prior field histories are
preserved and SHA-256 checked against the preceding worktree checkpoint.

The initial eleven-unit focus passes ten units but finds an incorrect budget-test
prefix: the new source includes an extra result marker print, so its prefix ends
at three statements, not two. The test now asserts that layout and charges the
complete source. An intermediate compile rejects direct access to a private
prefix recognizer; the correction uses the existing visible preflight contract
without widening production visibility. The initial unused fixture import is
removed; the final 78-unit run compiles without that warning and passes all units.

The selected prerequisite remains `active/99`. The next boundary is typed scalar
snapshots beyond bool/i64: audit i32/f32/f64 data admission separately from bool
path masks, without broadening legacy profiles or using numeric equality to
replace source-presence proof. That support is not claimed here.
The later [typed effect-snapshot proof](nuis-native-typed-effect-snapshots-v1.md)
records the scoped data extension separately; this receipt remains historical.
No fresh Linux/GPU run, full-workspace acceptance, measured speedup or formal
memory-safety certification is claimed. Historical three blocked Unix-socket
tests and eleven strict-Clippy findings remain pending, not rerun here.
