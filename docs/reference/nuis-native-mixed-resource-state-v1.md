# Native Mixed Resource State V1

The later [checked-State follow-up](nuis-native-checked-resource-state-v1.md)
adds a separate Result contract for Nuis validation and guarded Err. This receipt
retains the original mixed-only contract, score and validation scope.

This beta-0.18.9 worktree follow-up extends the
[CPU canvas consumer](nuis-native-resource-frame-export-v1.md) with Nuis-owned
dimensions and control State. It adds a distinct contract, not a widened
interpretation of the existing Bytes-only or scalar/value-record ABI.

The resource tensor coordinate advances from active/65 to active/75 for this
bounded slice. The other 36 coordinates retain their content and scores;
interactive native frame dispatch remains early/10. This is not a percentage
of self-hosting or engine completion.

## State Contract

The [mixed canvas example](../../examples/projects/domains/native_mixed_canvas_session_demo)
declares `Canvas { width:i64, height:i64, events:i64, cursor:i64, pixels:Bytes }`.
Open computes height from the Buffer extent and an initial width request. Each
event writes the current pixel, increments the event count and advances the
cursor. Close returns the complete final State. Those operations are Nuis code;
the host neither implements a canvas state machine nor infers updates from events.

The project declaration is:

```toml
packaging_mode = "native-resource-session-aot-bundle:canvas"
resource_sessions = ["canvas open=canvas_open event=canvas_event close=canvas_close min=1 max=4096 state=Canvas"]
```

`state=Canvas` selects `nuis-native-mixed-resource-session-v1`. Canonical YIR adds
the nominal State type to the existing registration record. All three declared
returns must have exactly the same encoded layout. Layout details remain in YIR,
covered by the exact graph and LLVM checkpoint, rather than copied into an
engine-specific compiler schema.

Current admission is deliberately bounded:

* One flat owned record with 1..=16 i64 value fields and exactly one owned Bytes
  field. Names are unique; nested resources, floats, bools and multiple resources
  are not implicitly granted by this first mixed profile.
* Open receives `(borrowed ref Buffer, i64 argument)` and initializes all scalar
  State in Nuis. Event/close receive Buffer, each current scalar State field, then
  one i64 argument. Scalar order and names must match the declared record layout;
  flattened parameter names may have a record prefix.
* Fields use their declared order, not a hard-coded set of canvas offsets. The
  Bytes field may occur before, between or after scalars. Core contracts and
  lowering never recognize `Canvas`, `width`, `height`, `events` or ns-nova names.
* Existing straight-line resource operation, 256-node and eight-copy limits
  remain. Pure i64 records can be constructed/projected; an owned mixed record
  must be returned whole, without aliasing its Bytes owner through projections.
* Complete State is transported in separate scalar and resource regions. No raw
  pointer is smuggled into a scalar or pixel cell.

## Native Transport And Failure

Mixed callbacks use distinct `nuis_mixed_resource_<hex-id>_<role>_v1` symbols,
a distinct ABI tag and typed host binding. The shared static descriptor has
discriminator 2 for this entry; the old entry still requires discriminator 1.
The LLVM adapter validates counts, alignment, non-null pointers and the tag
before calling Nuis. The tag is a discriminator, not authentication.

The admitted Nuis return is lowered using the existing owned aggregate ABI.
Its exact flat layout and slot kinds are proven before emission. The adapter
extracts its Bytes owner, checks the entire buffer length, copies scalar and
resource outputs, and releases the returned aggregate and Bytes once. It uses
the existing allocation ABI, not new application C/ObjC code or a second custom
heap layout. This is still a trusted static-code boundary, not a decoder for
arbitrary untrusted native pointers or malformed heap allocations.

The host borrows private copies of both input regions and stages both outputs.
It publishes one complete State only after successful return. A failure after
partial output writes therefore preserves both previous scalars and pixels.
Explicit close after an event failure consumes that last accepted State, but
does not turn the failed session into success. Close remains at most once; Drop
does not run Nuis code. Native traps terminate the process, with no unwind or
invented cleanup guarantee.

The session completion sequence remains a synchronous CPU sequence, distinct
from the Nuis application's own event count and from any device fence.

## State-Driven Frame Export

Mixed images use:

```text
--resource-frame-fields width,height --resource-frame-output new-image.ppm
```

These names select two i64 fields from the checked State layout. Unknown/resource
fields, missing pairs, duplicate fields and fixed-dimension overrides are rejected
before open. The dimensions themselves are read only from successfully closed
Nuis State. They must be positive, within the 4096-pixel bound, and exactly cover
the accepted Buffer. Pixel encoding and exclusive output creation retain the
previous RGBA8/PPM contract. An invalid final view does not roll back completed
callbacks; it fails export without writing an image.

The optional presentation claim is
`native_resource_frame_contract=nuis-native-mixed-resource-frame-rgba8-v1`.
It must agree with the resource contract, so a Bytes-only frame capability cannot
be substituted for a mixed one. Project/YIR identity, export identity, LLVM
checkpoints, artifact hashes and source-free restoration remain independently
checked. Embedded YIR is identity/audit data, not the execution fallback.

## Validation

The native resource library tests cover 80 mixed-state updates, complete scalar
and pixel publication, full-width i64 values, one/four/16-field layouts, and
resource-first, resource-middle and resource-last records. They also cover
tag/count rejection, allocation failure, errors before and after output writes,
once-only close, and zero retained Bytes after ordinary
completion/error. Descriptor drift is rejected before a marker callback enters.
The original Bytes-only library cases remain regression gates.

The artifact workflow compares native State to reference-return records at every
step and checks all PPM bytes against a separate oracle. Seven events on six
pixels exercise native cursor wraparound. The same binary produces 3x2 and 2x3
images from different Nuis initialization requests, without host dimension
arguments. Invalid field selection, fixed-size override, final shape/color
failure, zero-width native trap, existing output, capability mismatch, cache,
source-free NSB relocation and binary-only launch with empty PATH are covered.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --test native_resource_session -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_resource_session_workflow -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --bin nuis -- dev_tensor mainline resource_scripts --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- project::application_sessions aot_native_session::tests --test-threads=1
```

Local results for this follow-up: 22 native resource tests, three complete
artifact workflows, 52 tensor/mainline/CLI tests, 12 runtime/frame tests and four
manifest/packaging tests passed. The last group also caught a stale test import
after the earlier session-profile split; it now uses the shared dispatch module.
Repository checks passed: 40 script tests, two file-budget tests, the
host-path policy, UTF-8 validation, workspace architecture and documentation
links. Live tensor drift is clean across 4535 checks; the other 36 coordinate
records retain their pre-change digest.

No fresh Linux/GPU execution or performance result is claimed by this local
macOS witness. Byte-owner counters do not constitute a process-wide leak proof.

## Remaining Boundaries

The mixed slice is not arbitrary records, native ns-nova windows, GPU/provider
dispatch, resizing, asynchronous ownership or device retirement. The runtime
host remains Rust and finalization still uses Clang plus the existing allocation
shim. Neither pure Nsld linking nor complete self-hosting is claimed.

Next, use this caller to add Nuis-side configuration validation and bounded
conditional mixed-State control, with typed failure and unchanged publication
rules. More resource layouts or helpers should follow concrete callers rather
than broadening admission without execution evidence.
