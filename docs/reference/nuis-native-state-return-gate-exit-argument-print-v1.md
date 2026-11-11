# Native State Return Gate Exit Argument Print V1

Audit date: 2026-11-11. Checkout baseline: beta-0.18.7, `a270a99e`.
This worktree follow-up extends the
[ordered-chain witness](nuis-native-state-branch-chain-exit-argument-print-v1.md).
It creates no version bump, commit, tag or release.

## Return-Controlled Calls

The [Nuis source](../../tools/nuisc/tests/native_application_bridge/state_return_gate_exit_argument_print.ns)
binds the first checked, effectful helper's actual return to `next`. Only a
nonzero `next` enters the second helper. A zero return skips that helper and
decodes the return locally. Both paths retain the original selected branch's
helper family and publish the same complete two-field State as the predecessor.

The bit-32 return tag distinguishes original control, first return and decoded
control. Alternate bit-40 and sign-bit native source contrasts exercise both
helper families at zero, wrapping arithmetic, changed intermediate signs and
nonzero full-width values whose low 32 bits are zero. The predicate must consume
the actual full-width return, not the original input or decoded final value.

Open/event/close input widths remain 43/45/2; State remains two words. The base
child count two, 12-work ceiling and 192-entry ceiling are unchanged. No new
production lowering special case or wider effect authority is introduced.

## Skipped Work And Earlier Failures

A zero return suppresses all second-helper entry observations, prints, checks
and completion effects. Dedicated inputs poison its guard at offsets 16, 17,
48 and 49. Their success requires the call to be skipped at the first/later
child of the first/later active parent. A separate event starts with accepted
control `TAG - 2` but supplies an i64 MIN input decoy: the gate must use accepted
State and skip the poisoned call.

Skipping the second call cannot skip earlier work. Three collision rows use
that same zero-producing accepted control and poison the float-update check,
selected caller check or first helper check. Each must fail at its own exact
prefix before producing a usable return. Completed print effects are not rolled
back. Failed reference events and faulted close retain both accepted State words;
native traps still terminate the process, not return a recoverable pending State.

[Source tests](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_return_gate_exit_argument_print_tests.rs)
retain exact types, lexical scope, original field readiness and continue-step
rules, including rejection of a non-boolean return gate. Potentially reachable
effects still need explicit grants: the
[native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_return_gate_exit_argument_print.rs)
withhold each of twelve leaf print-site grants separately under both float widths
and declaration orders. Runtime skipping does not confer static authority.

## Independent Models And Delivery

The [transition model](../../tools/nuisc/tests/native_application_bridge/state_return_gate_exit_argument_print_cases.rs)
checks the tagged first return before selecting the second-helper guard or
emitting its observations. Existing selected-branch and ordered-chain profiles
remain independent. The chain's six native/reference tests are included in this
round's selected regression command.

The raw matrix has 21 healthy sequences and 24 fault sequences per width:
the previous sixteen healthy sequences plus five skip sequences, and the previous
21 failures plus three earlier-check collisions at the zero gate. Sign-bit source
contrasts explicitly reclassify a previously reached fault as healthy when the
changed return becomes zero, rather than assuming every fault row must still trap.

The [CLI driver](../../tools/nuis/tests/native_session_workflow/state_return_gate_exit_argument_print.rs)
selects nine finite sequences and fourteen fault sequences. Healthy sequences
run before and after faults on fresh builds, cache hits and two independent
source-free restorations in both widths: 256 process runs. This is a selected
delivery matrix, not all 24 native fault rows. Source, manifest and project cache
are removed before restoration. Complete State, stdout, completion and
binary/container/host-object/envelope/lifecycle/packaging identity are checked.
Direct native probes own exact trapping prefixes; buffered CLI stdout may be
shorter.

## Validation

Fresh selected validation on 2026-11-11:

| Selected Suite | Result |
| --- | --- |
| Return-gate source admission and refusals | 2 passed |
| Gate/chain native, reference, return, grant and budget controls | 13 passed |
| Fresh, cached and source-free CLI delivery | 1 passed; 256 process runs |
| Tensor selection, coverage, drift and complete predecessor lineage | 38 passed |
| Development and repository scripts | 40 passed |
| Repository line budgets and exception ratchet | 2 passed |
| Host absolute-path policy | 1 passed |

These are 97 selected test functions, including the Python tests. Repeated runs,
matrix rows and drift guards are not additional test functions. Native/reference
validation completed in 62.97 seconds; CLI delivery completed in 635.33 seconds,
including intended failures. This is selected host validation, not a full
workspace or CI acceptance.

The rebuilt ordinary CLI reports 4399 passing drift checks and zero failures.
All 4390 predecessor IDs retain their order, with nine new checks. All six
complete predecessor fields and hashes match source and runtime; coverage,
hierarchy and task lineage are clean. Historical commands pass syntax checks
and their quoted no-op suffix executes without replaying old jobs.

UTF-8 validation covers 6132 tracked files; documentation checks cover 4904 local
links. Formatting, diff whitespace and architecture checks pass. The dependency
graph remains at 29 packages, six domains and 74 normal/build edges, excluding
dev-only edges. No line-budget exception or runtime quota was increased.

## Scope And Next Gate

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next join the conditional-call result into a branch-local value and consume it
in later ordered work before returning, preserving definite initialization,
skipped effects, first-failure order and complete State publication.

This is bounded host CPU evidence, not Linux/GPU parity, unlimited call depth,
resource or I/O/async capture, stable ABI, general memory safety or self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_return_gate_exit --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_gate_exit state_branch_chain_exit --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_return_gate_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
python3 scripts/check-workspace-architecture.py
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete ordered-chain checkpoint fields and their SHA-256 hashes are
retained. Historical validation commands remain quoted no-op here-document
data. Syntax-check the full command through stdin and execute only its inert
history suffix, never the historical command chains.

| Complete predecessor field | SHA-256 |
| --- | --- |
| `evidence` | `3ebd4df21ed2de1c64ece468acea6c38884683d4cd39bae60a1b7b5353253ade` |
| `next_step` | `48d2ab8eb3ea6e105cc2be043337b399b231d77157b70b45e68e4cda20230e88` |
| `blocker` | `594e3206a23650659367c67d315d5e9e2fa13f72747175ccc0787c1e0934a0c5` |
| `next_action` | `a3ebd5014550f667708e9133e2c5ddc27d225e769dfc28d23fe7fa489391cd62` |
| `validation_command` | `254d26199082adb338cc1df730fa34a98519b4ee86b6bd7f39e11874d4cbf8dd` |
| `expected_artifact` | `4536c3ecd87f4b8698629c4de02b9cce8d1a678d52a78f9b0905f82a4f1da7ae` |
