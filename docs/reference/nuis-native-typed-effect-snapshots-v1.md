# Native Typed Effect-Region Snapshots v1

This bounded CPU follow-up extends the
[equal-atom join proof](nuis-native-equal-effect-result-joins-v1.md) in the
`beta-0.16.*` worktree after `dff1bdbc`. It is not a release or a complete
heterogeneous-application acceptance.

## Separate Data Profile

Effect-region initializer, internal condition and actual-exit captures now admit
exact owned `i32/f32/f64` data alongside bool/i64. Fresh final result bindings on
continuing arms can join these data kinds too. Original initializer types and
annotations validate before aliases, rewritten ready types validate afterward,
and both continuing source destinations validate before publication.

Path, selection, continuation and live masks remain exact bool. References,
optional/generic scalar lookalikes, aggregates and resources are not admitted by
this data profile. It adds no public ABI, YIR instruction, transport exemption,
Nustar dependency or ownership/GLM exception.

Guarded typed helpers reuse the existing pure-value neutral initializer contract:
i32 uses an explicit narrow integer zero, f32/f64 use their own positive-zero
literal kinds, and bool/i64 retain their previous seeds. Inactive seeds never
establish source-result presence or authorize a return. Float atoms retain exact
NIR literal identity, not numeric equality: distinct positive/negative zero atoms
still require selection. Converted i32 literals remain independently staged
snapshots; equal-looking conversions do not become equal ready atoms.

The enclosing return and legacy tail/capture profiles still admit only bool/i64.
Typed data must be consumed within the selected effect region before a legacy
tail captures its resulting bool/i64 snapshot. This first extension does not
admit typed return values, direct typed tail captures, computed float printing,
arbitrary effectful callees or general child-local export. Existing source and
expanded statement/node/depth/logical ledgers and atomic vetoes are unchanged.
The later [typed selected-return handoff](nuis-native-typed-effect-returns-v1.md)
adds enclosing exact scalar returns only for admitted selected regions. That
separate proof does not retroactively widen this first-extension receipt or
ordinary computed-print authority.

## Reproduction

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_typed_snapshots --test-threads=1
```

Six new units cover the exact owned profile, source semantics, helper captures
and seeds, lexical/type/effect/budget vetoes, signed NIR atom identity and native
AOT. The independent oracle covers 288 distinct core sources: three data kinds,
computed/shared/literal results, let/const, ordinary/partial continuations and
eight inputs, each executed in ordinary and reversed YIR order (576 executions).
Eighteen core sources also undergo helper NIR/idempotence inspection and 36
additional trace executions, checking once-only calls and actual typed arguments
for both selected arms. These observations reuse core sources.

Selected unused checks still trap, unselected zero-divisor work stays inactive,
and actual source exits suppress later typed work and parent continuation. Wrong
annotations, borrowed/resource kinds, non-bool conditions, unreachable suffixes,
effectful callees and direct typed legacy-tail captures veto without modifying
the module. A 0..32 alias sweep charges both original and expanded source ledgers.

Eighteen default-source native AOT variants cover both selections, inactive outer
paths, zero-valued data and partial source exits; three selected traps require
native trap signals. Successful runs require exact stdout and normal completion.
The native stdout oracle does not numerically inspect the unused typed parameter
of the pure `inspect` fixture; typed argument values are checked by YIR traces
and exact helper/call structure. No native float bit-pattern certificate or
partial stdout/once-only trace certificate at traps is claimed.

## Acceptance Receipt

Typed effect-snapshot acceptance: The 2026-10-07 follow-up passes 179 distinct
selected tests:

* 84 return-control/frontend units, including all six new typed snapshot units
* 58 native bridge tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

Focused/latest-source and post-receipt reruns overlap this selection and add no
distinct cases. The preceding 173-test equal-atom receipt remains historical,
not added to this follow-up's count. Static drift passes 2067 definitions/14122
literal patterns. All 34 changed/new file caps, formatting, whitespace, 4745
recognized UTF-8 text files and 4700 local Markdown links pass. The rebuilt CLI
reports `active/99`, passes 2067/2067 drift checks and reports clean coverage,
hierarchy and lineage. All six complete preceding equal-atom field histories are
preserved and SHA-256 checked against their worktree checkpoint.

Initial test compilation exposed private sibling-fixture/profile access. The
fixture now owns its inputs and profile tests live with the private implementation;
production visibility was not widened. Initial source cases incorrectly mixed an
i32 operand with an i64 literal; the fixture now explicitly narrows its increment.
Duplicate reduced input records and a const/let NIR pattern assumption were also
corrected rather than increasing the source-count claim or changing production
semantics to fit the tests.

Source unary `-0.0` at this checkpoint lowered to subtraction from positive
zero. The signed atom test therefore constructed exact NIR atoms; source-level
negative-zero preservation was a separately tracked frontend gap, not acceptance
here. The subsequent [literal negation repair](nuis-native-float-literal-negation-v1.md)
closes that literal gap and updates the signed-atom test to start from source.
Its native float bit-pattern certificate is scoped to its own fixtures, not
retroactively to the unused `inspect` argument in this historical receipt.
The selected application prerequisite remains `active/99`, not closure.
No fresh Linux/GPU run, full-workspace acceptance, measured speedup or formal
memory-safety certification is claimed. Historical socket/strict-Clippy findings
remain pending and were not rerun here.
