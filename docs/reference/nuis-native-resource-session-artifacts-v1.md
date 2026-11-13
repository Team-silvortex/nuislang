# Native Resource Session Artifacts V1

Follow-up: [native CPU canvas export](nuis-native-resource-frame-export-v1.md)
adds an optional presentation capability and final accepted-State PPM consumer.
The artifact receipt and its active/60 checkpoint below remain historical;
the later [mixed-State proof](nuis-native-mixed-resource-state-v1.md) brings that
coordinate to active/75, without promoting window/GPU execution.

This beta-0.18.9 worktree follow-up connects the bounded
[Buffer/Bytes callback contract](nuis-native-resource-session-v1.md) to project
registration, canonical YIR, static AOT packaging, NSB verification, cache reuse
and launch without the original source checkout. The resource tensor coordinate
advances from active/35 to active/60. This is a scope label, not a measured
percentage of engineering time. Frame/application and all other coordinates
retain their scores; the selected prerequisite is still resource callbacks.

## Registration And Delivery

The project uses a separate strict manifest field, not scalar pointer slots:

```toml
name = "resource_example"
entry = "main.ns"
modules = ["main.ns"]
packaging_mode = "native-resource-session-aot-bundle:bytes"
resource_sessions = ["bytes open=start event=step close=stop min=1 max=4096"]
```

`start`, `step` and `stop` are distinct CPU helpers, each taking
`(buffer: ref Buffer, argument: i64)` and returning owned `Bytes`. The source
fixture is `tools/nuisc/tests/native_resource_session/main.ns`. Registration
preserves these roots even when ordinary `main` does not call them. All
registrations must bind valid signatures, including non-selected registrations.
The selected profile additionally passes the existing straight-line ownership
and instruction admission checks; registering a helper does not bypass them.

There are at most 64 registrations. Duplicate IDs/fields, unknown fields,
noncanonical bounds, missing helpers and incompatible signatures are errors.
Bounds are inclusive i64-cell counts, at most 4096; zero is allowed only when
declared. Each invocation retains its initial length. Bytes is a native-endian
cell snapshot, not UTF-8 or a portable byte serialization format.

YIR serializes seven fields in an explicit record:

```text
resource-session bytes nuis-native-resource-session-v1 start step stop 1 4096
```

Export names derive from the UTF-8 ID's lowercase hexadecimal bytes:
`nuis_resource_6279746573_open_v1`, followed by corresponding event/close names.
The shared core contract owns this spelling. The producer and host check the
same canonical registration and ordered exports; they do not independently
invent lookup conventions or load a dynamic callback plugin.

The NSB build manifest carries the `nuis-native-resource-session-build-inputs-v1`
schema and `native-resource-llvm-v1` checkpoint. Its bound inputs include canonical
YIR, emitted LLVM, bundle identity claims, the existing source-to-YIR compiler
handoff, and project metadata. Verification re-emits the selected bridge and
compares it exactly, checks each required claim occurs once, and rejects unknown
resource claims or scalar-profile claims. Cache identity includes the selected
packaging mode and registration-bearing project/YIR inputs.

This is consistency checking, **not authentication** of arbitrary native code.
The existing artifact hashes are not a trust signature. Static callback pointers
remain trusted build products under the unsafe binding contract; a matching name
does not make an untrusted native object safe.

## Native Launch

```sh
nuis build path/to/project path/to/output
nuis run-artifact path/to/output --resource-session bytes \
  --buffer-words 0,-17 --open-arg 10 --event-arg 3 --event-arg -2 --close-arg 5
```

The script requires one selector, initial Buffer, open argument and close
argument. It admits at most 256 event arguments. Values are canonical signed i64
integers. The complete script is validated before open, including late invalid
arguments. Resource scripts cannot combine scalar/reference selectors, window
options, provider drain or cancellation. `artifact-doctor` identifies the
resource launch route explicitly.

The generated LLVM process entry calls a statically linked resource host. Its
descriptor contains the contract-bearing compiled YIR, canonical registration,
ordered export names and three resolved callback pointers. Before entry the host
checks graph equality, registration, exports, nonmissing callbacks and script
admission. Callback execution is real machine code, not interpretation of that
embedded YIR. The host only publishes whole successful results and retains the
existing once-only close and sticky-failure rules.

The test fixture publishes `[10, -17]`, `[13, -17]`, `[11, -17]`, `[16, -17]`
at completion sequences 1 through 4. Additional untouched i64 minimum/maximum
cells are preserved exactly. Native traps remain process failures, not recoverable
callback errors or evidence of unwind cleanup.

## Source-Free Evidence

`tools/nuis/tests/native_resource_session_workflow.rs` covers:

* Fresh project build and exact registration/YIR/export checkpoint verification.
* Required/duplicate/unknown claims and registration/export/LLVM drift rejection,
  including semantic checks independent of outer artifact hashes.
* Rejection before open for wrong registration, invalid late arguments, profile
  mixing and modified binary, YIR, LLVM, source snapshot or project metadata.
* Switching to a second registration, rejecting the first selector, then restoring
  the first build from cache and obtaining the identical complete State sequence.
  Changing bounds under the same ID also invalidates the cached checkpoint.
* Copying only the native executable into an otherwise empty directory, deleting
  the original source and build directory, and running with empty `PATH`.
* Keeping only `nuis.compiled.artifact`, verifying and materializing it into a new
  directory, then launching with the same accepted State and restored checkpoints.

"Source-free" means no original project checkout, compiler invocation, runtime
compilation or sidecar callback library is needed at launch. It does **not** mean
the NSB has been stripped of its embedded audit source/YIR. The standalone native
executable also retains its static YIR identity descriptor. OS system libraries
remain normal host dependencies.

## Remaining Boundaries

The build currently uses Clang for static host finalization and the existing
owned-blob compatibility implementation. No new C or Objective-C application
replaces the Nuis callbacks, but this is not pure Nsld finalization or an all-Nuis
runtime. Packaging admits the current 64-bit macOS/Linux host; this receipt is a
local macOS run, not a fresh Linux/GPU parity result or Windows/cross-build proof.

Resource loops/helpers, mixed scalar/resource State, resize, cancellation,
provider/device completion and native frame integration remain open. A local
completion sequence is not a GPU fence. The next slice connects this admitted
packaged boundary to a bounded native frame caller through shared contracts,
without adding engine-specific compiler logic or weakening scalar rejection.

## Reproduce

Local validation passed: 14 resource callback/identity tests, the resource and
scalar build/cache/source-free workflow tests, 136 compiler manifest/checkpoint
regressions, 61 tensor/CLI checks, four packer option checks, four lowering
boundary checks, seven scalar host lifecycle checks, and 40 repository script
tests. A further 26 verifier graph/project/result regressions pass. File budgets,
UTF-8, relative documentation links and host-path policy also pass. These are
focused suites, not a full workspace or hardware sweep. Clippy reports no new
warnings in the checked core/lowering/runtime/packer packages; the pre-existing
`SessionExecution` large-enum warning remains.

The live tensor retains 37 coordinates and has 4527 clean drift checks. The other
36 complete cell records are unchanged; bootstrap-critical preparation stays at
21 coordinates / 100 average, not a self-hosting-completion claim.

Run from the repository root with the host LLVM/Clang toolchain:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- resource_registration_manifest --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-pack-aot --bin yir-pack-aot --locked -j1 -- host_application_script --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor mainline resource_scripts --test-threads=1
```
