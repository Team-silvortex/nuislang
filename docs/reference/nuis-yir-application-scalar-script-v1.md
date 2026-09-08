# Headless Application Scalar Script

Contract: `nuis-yir-application-scalar-script-v1`.

This is a bounded, non-window ingress profile over `ApplicationEventPump`.
It selects an embedded application registration and drives its Nuis open/event/close
callbacks. It neither uses `WindowSession` nor implements application state,
image algorithms, provider selection or device cleanup in a new host adapter.

## Packaging And Input

`yir-pack-aot <module.yir> <output-dir> [frame-scale] --headless` requires at
least one registered application session. The output contains an executable,
embedded YIR and the existing statically linked host runtime. Its generated C
source is only a process-entry bridge to `nuis_application_script_main`.
No AppKit/Objective-C host or prerendered fallback frame is selected.
This profile does not emit or advertise unused CPU LLVM IR/shims, nor claim
AppKit's affinity worker. Native lowering remains a separate path; an unbound
scalar parameter there now produces a function-context diagnostic, not a panic.

The bundle declares:

```text
cpu_host_binary_mode=embedded_yir_headless
runtime_bootstrap_mode=embedded_yir_session
application_script_contract=nuis-yir-application-scalar-script-v1
application_provider_drain_contract=nuis-yir-provider-session-drain-v1
```

The process accepts:

- `--application-session ID`: required embedded registration ID.
- `--open-args I64,...`: required; an empty value means no arguments.
- Repeated `--event-args I64,...`: zero to 64 ordered event deliveries.
- `--close-args I64,...`: explicitly call close, then await provider Finish.
- Alternatively, `--cancel-after-events`: abandon without invoking close.
- `--drain-provider`: only with cancellation; observe explicit provider drain.

Each argument list has at most 16 values. The whole script has one 180-second
deadline, not a fresh budget per callback. IDs are 1..128 UTF-8 bytes; argv entries
are bounded to 512 bytes. Duplicate, unknown, excess, malformed and contradictory
options fail before session startup. Callback signatures remain validated by the
registered application contract. This profile supports i64 ingress, not arbitrary
strings, pointer arguments, interactive stdin or a general scripting language.

## Termination And Ownership

There is only one outstanding operation and one consumed reply at a time.
The host observes accepted states and trace frames without retaining a history.
RGBA frames report byte count and content hash; they are observations, not a
success artifact or authority to reuse resources.

Close yields success only after lifecycle validation and provider Finish.
Cancellation occurs only after the preceding replies passed validation, drops
the pump and waits on its independent retirement ticket. It does not synthesize
Nuis close, a terminal application outcome or parent delivery. Host-only retirement
is not provider retirement. The headless and AppKit adapters use the same safe
receipt-to-exit classifier: an affirmed drain without a latched fault maps to 130,
not success. Replay-only, failed, missing or mismatched drain evidence maps to 1.
The supervising provider launch policy must also validate its typed Drained
terminal; neither exit130 nor EOF proves retirement.

A callback error or timeout aborts the script before its requested terminal
operation. There is no automatic cleanup, retry, reconnect, budget reset or Finish
fallback. Already admitted effects are not rolled back. An in-flight worker can
outlive a timeout; the bounded host wait does not preempt FFI or device work.

Exactly one nonempty provider source must be explicit:
`NUIS_YIR_PROVIDER_DISPATCH_SOCKET` for the current Unix IPC adapter, or
`NUIS_YIR_PROVIDER_RESULT_STREAM` for identity-bound replay. Neither source,
both sources, or an empty path fails. No reference-device fallback is selected.

## Reproduce

Run from the repository root with one Cargo job. These tests exercise a compiled
headless process with a protocol peer, without needing GPU hardware:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-pack-aot --test headless_session -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --test provider_application_session script:: -j 1
```

The separate real M2 Metal test builds the Nuis image showcase, packages its
registered callbacks without AppKit, verifies both GPU frames' exact RGBA hashes,
normal Finish, one/two-frame typed drain, worker removal and preservation of prior
success evidence. The same binary rejects replay-only drain:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis headless_metal_session_uses_shared_lifecycle_without_appkit -j 1
```

For a manual replay, first build the
[image showcase](../../examples/projects/domains/ns_nova_image_showcase/README.md)
and complete its `--window-events 32` route, producing two identity-matched frames.
Then package the same YIR and replay those callbacks without a window:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p yir-pack-aot -- build/ns-nova-image/ns_nova_image_showcase.yir build/ns-nova-headless --headless
env -u NUIS_YIR_PROVIDER_DISPATCH_SOCKET NUIS_YIR_PROVIDER_RESULT_STREAM=build/ns-nova-image/nuis.runtime.provider-result-stream.toml \
  build/ns-nova-headless/ns_nova_image_showcase --application-session window \
  --open-args 160,120 --event-args 0,0 --event-args 1,32 --close-args 1,0
```

## Current Boundary

The headless host profile is exposed by the packager. Ordinary `nuis build` and
`nuis run-artifact` do not yet select or admit this profile; do not relabel an
existing window bundle to bypass capability/binary validation. Their explicit
profile selection and shared launch-policy integration are the next task.

This is embedded-YIR execution, not fully native CPU callback lowering or a
self-contained Nsld application image. The registered live provider remains an
external execution scope. macOS and Linux have host build branches; the recorded
real hardware evidence is M2 Metal, not Linux GPU or Windows transport certification.
Parent cancellation, arbitrary resource reuse and recovery remain separate work.

Related contracts: [application session](nuis-yir-application-session-v1.md),
[cancellation](nuis-yir-application-cancellation-v1.md),
[provider drain](nuis-yir-provider-session-drain-v1.md).
