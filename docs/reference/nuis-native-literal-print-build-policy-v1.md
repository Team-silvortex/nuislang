# Native Literal Print Build Policy V1

This beta-0.16 worktree follow-up connects the
[explicit literal-print bridge](nuis-native-literal-print-policy-v1.md) to native
build, cache restoration, standalone materialization and verified launch. The
authorization is an explicit producer configuration, not a capability inferred
from source, a cached binary or a mutable runtime flag. Default pure packaging
keeps its existing admission and default work limits.

## Explicit Producer Identity

The opt-in packaging mode is
`native-session-policy-aot-bundle:<policy-token>:<registration-id>`.
It can be selected in `nuis.toml` or by `nuis build --packaging-mode MODE`.
The separate pure mode remains `native-session-aot-bundle:<registration-id>`.
Neither mode falls back to a reference executor after an admission failure.

`nuisc::aot::native_session::literal_print_packaging_mode(id, sites, loop_limit,
entry_limit)` constructs a canonical mode from an already reviewed node list.
`LiteralPrintBuildPolicy` lives beside the native LLVM bridge and is also consumed
directly by the AOT packer; the packer does not depend on the compiler frontdoor.
There is no automatic enumeration or approval of source print sites in production.

The portable token format is
`v1.<loop_work_limit>.<helper_entry_limit>[.<hex_utf8_site>...]`.
Limits are canonical decimal u64 values; zero is allowed. Site names are sorted
by their UTF-8 strings, distinct and nonempty, then encoded as lowercase hex.
There are at most 64 sites, 256 UTF-8 bytes per site and 33000 bytes per token.
Parsing rejects unsupported versions, missing or overflowing limits, leading
zeros, plus signs, unsorted or duplicate sites, uppercase hex, invalid UTF-8,
empty trailing fields and exceeded bounds. An empty list grants nothing.

For example, `v1.0.100.7072696e745f30` names only `print_0`, with loop-work limit
zero and helper-entry limit 100. It is not a reusable wildcard: a real selected
closure needs an explicit grant for every admitted print in every reached helper.
Registration IDs retain their existing bounds and UTF-8 support, including colons;
only the first colon after the token separates the ID.

## Build And Artifact Binding

Inspection, fresh compilation and AOT handoff all regenerate the selected LLVM
bridge from the exact verified YIR and explicit policy. The packer receives
`--native-session ID --native-literal-print-policy TOKEN` and independently repeats
admission. Missing, duplicate, malformed or non-native policy options reject.
The scalar callback ABI and native runtime descriptor are unchanged.
Authorized artifacts add only the existing i64 print ABI in the LLVM host entry
unit, delegating formatting to the host `printf`. No new C implementation is
introduced for this effect. Pure artifacts do not define the print adapter;
providing a runtime symbol is not a grant to emit additional effects.

Policy bundles carry exactly one of each claim:

* `native_session_policy_contract=nuis-native-literal-print-build-policy-v1`
* `native_session_policy=<canonical-token>`
* `native_session_loop_work_limit=<u64>`
* `native_session_helper_entry_limit=<u64>`
* `native_session_literal_print_bound=<u128>`

The bound is the admitted static site count times the helper-entry limit. It is
not an elapsed-time, memory or I/O guarantee. Missing, duplicate, inconsistent
and unknown reserved policy claims reject. Pure bundles reject policy claims
rather than importing their authority.

The policy profile uses `nuis-native-session-build-inputs-v2` and checkpoint
`native-scalar-literal-print-llvm-v1`. Pure native artifacts retain their v1 input
schema and `native-scalar-llvm-v1` checkpoint. Both bind source/token/AST/NIR/YIR
handoffs, LLVM and bundle bytes. Verification reparses canonical YIR and
regenerates exact LLVM using the mode's grants and limits. Changing a claim or
LLVM limit and recomputing the ordinary payload checksum does not bypass this
semantic check. Unknown, surplus, removed and non-print grants reject too.

The full mode is part of build identity. Project selections are included through
the manifest fingerprint; explicit overrides additionally bind the mode and
CPU target to the cache key. Cache restoration checks the verified artifact mode
against the current request. Different grants, limits or registration IDs cannot
reuse one another's authorized profile.

## Source Free Launch

Only the standalone compiled artifact is needed for materialization. Bound inputs
are checked before restoration; source and project manifests need not remain on
disk. Restored native launch checks the artifact, manifest, selected binary,
exact YIR/LLVM policy checkpoint and typed script before starting a process.
The command line selects a session and inputs, not new grants or larger limits.

Fresh counters remain local to each open/event/close callback. Successful returned
state feeds the next callback. Input rejection precedes source entry, prints and
state publication. Helper-entry exhaustion remains a fatal native trap, not a
catch, retry or interpreter fallback. Already completed prints are irreversible.

Payload checksums and exact regeneration establish internal consistency, not
publisher authentication or proof that arbitrary machine code matches LLVM.
A coherently rebuilt artifact with a different explicitly declared identity is
not covered by a stale-artifact rejection claim. Trusted delivery and executable
attestation remain separate contracts.

## Evidence And Remaining Scope

The shared Nuis selected-return fixture uses literal-print sites, explicit
per-site grants, branch-selected scalar helpers and carried i32 state. Its workflow
checks four branch/early-return paths, exact prints and four registered transitions
per script, cache reuse, changed limits, pure and missing/surplus-grant rejection,
malformed inputs and mismatched session launch. Source, project manifest and
project-local cache are removed before three repeated materializations;
each restores identical policy, LLVM and executable bytes before real native run.

Separate checkpoint tests rehash altered LLVM and bundle payloads and still
require rejection. Compact source branches that become compound print-return
instructions or selected-value prints remain rejected: the profile is not widened
to make such fixtures pass.
The [guarded scalar-selection follow-up](nuis-native-effectful-scalar-selection-v1.md)
repairs source-to-YIR order propagation for bounded two-sided effectful call
selections. Unsupported source shapes still reject rather than guessing execution
order; the build policy cannot repair or waive a missing dependency.

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- build_policy --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib -p yir-pack-aot --bin yir-pack-aot --locked -j1 -- aot_native_session aot_application_bundle host_application_script host_native_session --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- native_literal_print_build --test-threads=1 --quiet
```

This is a restricted scalar native producer/transport contract, not the default
application host or a self-contained heterogeneous image. Computed and typed
prints, string/resource effects, generalized application scheduling and native
GPU callback dispatch remain separate. Fresh multi-platform/device acceptance,
full-workspace acceptance, a measured speedup and a formal safety certificate
are not implied.

## Acceptance

Literal-print build policy acceptance: The 2026-10-07 selected acceptance passes
283 distinct tests across completed cohorts:

* 173 LLVM lowering tests, including ignored tests and two token boundary tests
* Ten compiler artifact/profile tests and three AOT packer tests
* 50 native bridge and ordinary-AOT regression tests
* Eight compiler selected-return and pure/effect-boundary tests
* 29 tensor tests and four native build/cache/materialization workflows
* Five application-session tests and one host-path policy test

The final two policy workflows additionally remove the project-local cache before
restoration and use shape-correct invalid input/session cases. Across four branch
paths, their successful fresh/cached/changed-limit/restored runs check 128 actual
registered open/event/close transitions. Three source-free restoration cycles
retain exact policy, LLVM, bundle and executable bytes. The one-entry profile
traps before the first print or state publication. Earlier linking failures were
repaired by the opt-in LLVM i64 host adapter, not bypassed or accepted as success.
Focused token/artifact/packer and tensor/workflow reruns overlap the selected
cohorts and add no distinct tests.

Static drift passes 2128 definitions/14502 literal patterns. All 109 changed/new
file caps, 4771 recognized UTF-8 files, 4737 local Markdown links and formatting/
whitespace checks pass. The rebuilt CLI confirms 2128/2128 drift checks and clean
coverage, hierarchy and lineage. All six complete preceding literal-print-policy
tensor fields are preserved and SHA-256 checked. The selected application cell
remained `active/99`, with source effectful-helper dependency order as its next
action at that checkpoint; the scalar-selection follow-up tracks the later repair.
This receipt does not extend the committed beta-0.16.0 baseline or add
historical receipts. Historical socket and strict-Clippy findings remain pending.
