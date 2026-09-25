# Beta 0.15.2 Patch

Date: 2026-09-25. Source version: `beta-0.15.2`, following `4257ebb6`
(`beta-0.15.1`) and minor baseline `05951bef` (`beta-0.15.0`). Git history
identifies the exact revision. Cargo package versions, protocol version identifiers,
public function signatures and capability scores are not bumped for this release.
The [minor snapshot](nuis-beta-0.15.0-snapshot.md) and
[preceding patch](nuis-beta-0.15.1-patch.md) retain their historical evidence.

## Included Changes

Private record capture projection now handles field-only fallthrough joins and
loop-written copy families through exact nominal input reconstruction. Existing
assignments, complete initializer work, per-trip snapshots and backedges remain
intact. Whole uses, computed callers and unproven types veto the candidate rewrite.

Scoped flat-i64 carry inputs support complete and partial field maps in declared
state-slot order, independently of helper parameter order. Initial record types
must be proven before flattening. Duplicate mappings, computed mutable-field
operands and mismatched or unknown origins are rejected; bool and break-control
inputs retain their existing identities.

The shared carry contract optionally transports complete initial state with
`$carry_seeds N seed0 ... seedN-1`, separately from iteration operands. Dependency
and GLM reads include unpassed seeds but exclude transport metadata. Registered
CPU execution and ordinary/native LLVM preserve every state slot and zero-trip
value, while loading only mapped operands for calls. Legacy complete-map payloads
remain supported; older consumers reject the new prefix rather than misreading it.
State width, helper arity and execution budgets retain their independent bounds.

Compiler-generated scoped helpers now automatically project partially read
flat-i64 record inputs. Each scoped caller must prove the original complete seed
map and exact reconstruction. Eligible parameter snapshots separate input demand
from later record updates without renaming local control, induction, bool-word or
break identities. Changes to signatures and callers are transactional. At least
one field per carried record remains an argument; complete state storage and
initializers are never removed merely because a field is not passed to the helper.

The [value-return and capture contract](../reference/nuis-native-scalar-value-returns-v1.md)
records the wire form, source proof, execution boundaries and dated validation
checkpoints. The [mainline map](../current-mainline-map.md), README and development
tensor reflect the new scope without claiming general aggregate/resource support.

## Validation Evidence

The final implementation checkpoint on macOS aarch64 passed 764 selected tests:

* 662 compiler-lowering unit tests
* 50 selected native application bridge tests
* eight ordinary native execution tests
* five reference image/window tests
* thirteen CLI build/cache/restoration workflows
* 26 development-tensor tests

Overlapping reruns are excluded. The direct NIR projection regression failed
before implementation and passed afterward. Source/reference cases retain
3/7/64 state slots with three iteration arguments. The native lifecycle fixture
retains all five state slots while reducing six arguments to five. Ordinary native
execution returns 154 and preserves selected iteration, zero-trip initializer and
second-trip constructor failures. CLI cases retain real native execution, cache
reuse, tamper rejection and source-free restoration with identical LLVM and state;
the generated-loop workflow also checks the reduced LLVM signature.

The preceding independent-seed checkpoint passed 1086 selected tests, including
90 core-contract, 96 registered-CPU and 149 LLVM-lowering tests. It is a separate,
overlapping checkpoint, not an extra 1086 tests to add to the final total. Both
checkpoints and intermediate record-join checks remain in the reference document.

The fresh CLI reports 1459 clean drift checks with clean coverage, hierarchy and
task lineage. These are selected local results, not a full-workspace run, fresh
Linux/GPU acceptance, formal proof or measured speedup. The
[validation checklist](nuis-beta-0.15.0-release-checklist.md) lists broader follow-up
suites; those commands are not blanket claims of already executed evidence.

Release preparation additionally passed all 20 maintenance-script tests and the
host-absolute-path policy test, and reran all 26 tensor tests successfully. Local
documentation links, UTF-8, staged-source formatting and staged-file line limits
also passed. These checks are recorded separately from the implementation total;
remote CI success must be verified for the pushed commit rather than inferred here.

## Remaining Work

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session` at `active/86`.
The next task is to remove entirely unconsumed flat-record inputs from generated
scoped helper signatures without losing initial state, initializer work or exit
semantics. Unresolved joins, nested-loop writes, whole-record escapes, sparse state
storage and mixed/nested carries retain separate admission boundaries. Resource
state, provider dispatch, self-contained application packaging and compiler
self-hosting remain separate work; this patch is not an ABI freeze or a claim of
complete native application closure.
