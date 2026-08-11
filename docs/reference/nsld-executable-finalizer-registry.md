# Nsld Executable Finalizer Registry

Status: implemented early-beta contract, incomplete platform-writer parity.

## Purpose

`nuis-nsld-executable-finalizer-registry-v1` is the static boundary between
Nsld's object-format-neutral final executable pipeline and an OS-native
executable shell writer. The final-stage, writer, and emit paths select a
registered provider; they do not branch on Mach-O, ELF, or PE/COFF themselves.

This keeps the verified NSB image, lifecycle metadata, and deterministic link
plan as semantic authority. An OS executable is a compatibility shell around
that authority, not a replacement binary model.

## Selection Contract

Every selection uses a canonical target key:

```text
<machine-architecture>-<operating-system>-<object-format>
```

The current normalizations include:

* `arm64` to `aarch64`
* `darwin` and `apple-darwin` to `macos`
* `linux-gnu` and `gnu-linux` to `linux`
* `win32` and `win64` to `windows`
* `macho` to `mach-o`
* `coff`, `pe`, and `pe/coff` to `pe-coff`

Registrations may use `*` only for a target component they intentionally leave
open. The most specific match wins. Missing providers and equal-specificity
ambiguity fail closed, so registration order never becomes hidden authority.

The registry validates unique provider and target identities, verifies that a
`ready` provider owns an executor, and hashes its canonical registration set.
Dry-run JSON and the persisted host invoke plan expose:

* `finalizer_contract`
* `finalizer_registry_hash`
* `finalizer_registry_valid`
* `finalizer_target_key`
* `finalizer_provider_id`
* `finalizer_provider_status`
* `finalizer_execution_kind`

## Current Providers

The current static set is deliberately asymmetric:

| Provider | Target | Status |
| --- | --- | --- |
| `nsld.finalizer.mach-o.arm64.host-command-shell-v1` | `aarch64-macos-mach-o` | `ready` |
| `nsld.finalizer.mach-o.registered-v1` | `*-macos-mach-o` | `registered-not-implemented` |
| `nsld.finalizer.elf.registered-v1` | `*-linux-elf` | `registered-not-implemented` |
| `nsld.finalizer.pe-coff.registered-v1` | `*-windows-pe-coff` | `registered-not-implemented` |

The ready Mach-O arm64 provider owns both command planning and actual process
invocation. `final_executable_emit` no longer spawns a platform tool directly.
The existing policy and explicit-allow gates still apply before the provider
may run. A host-command provider must execute the exact driver path resolved
during the verified dry-run boundary; it may not repeat a `PATH` lookup after
admission.

This is a registration and decoupling milestone, not a claim that Nsld already
writes a complete Mach-O executable without host help. The ready provider still
uses the registered host command. ELF and PE/COFF remain visible, selectable,
and honestly blocked rather than silently falling through a generic linker.

## Extension Rule

A future provider must:

1. register one stable provider identity and canonical target pattern;
2. consume the format-independent command/request context;
3. preserve the verified NSB entry, lifecycle, clock, and data-order metadata;
4. return an error before output mutation when its target or relocation input
   is unsupported;
5. pass the registry conformance and finalizer CLI regressions.

The next native milestone is to replace the Mach-O arm64 host-command executor
with Nsld-owned relocation application and executable-shell byte emission
behind the same registry boundary. That work must not add Mach-O branches to
Nuisc, final-stage planning, or the generic emit frontdoor.

## Validation

```sh
CARGO_INCREMENTAL=0 cargo test -q -p nsld executable_finalizer
CARGO_INCREMENTAL=0 cargo test -q -p nsld final_executable_host
CARGO_INCREMENTAL=0 cargo test -q -p nsld --test host_finalizer_cli
```
