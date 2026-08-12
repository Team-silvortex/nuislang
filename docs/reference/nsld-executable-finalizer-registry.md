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

Provider routes include the canonical target key and packaging mode. This lets
one platform keep a narrow internal route alongside a compatibility fallback
without relying on registration order.

The current static set is deliberately asymmetric:

| Provider | Target | Packaging | Input | Status |
| --- | --- | --- | --- | --- |
| `nsld.finalizer.mach-o.arm64.artifact-image-v1` | `aarch64-macos-mach-o` | `native-cpu-llvm` | compiled-artifact host image | `ready` |
| `nsld.finalizer.mach-o.arm64.host-command-shell-v1` | `aarch64-macos-mach-o` | any | native object | `ready` |
| `nsld.finalizer.mach-o.registered-v1` | `*-macos-mach-o` | any | native object | `registered-not-implemented` |
| `nsld.finalizer.elf.registered-v1` | `*-linux-elf` | any | native object | `registered-not-implemented` |
| `nsld.finalizer.pe-coff.registered-v1` | `*-windows-pe-coff` | any | native object | `registered-not-implemented` |

For `native-cpu-llvm`, the internal Mach-O provider parses the compiled
artifact, checks target and ABI identity, validates either a thin arm64
`MH_EXECUTE` image or the arm64 slice of a universal Mach-O, and atomically
materializes it with executable permissions. This path requires no host
process or environment gate, so Nsld no longer invokes clang a second time
after Nuisc has already produced the embedded host image.

The host-command provider remains a compatibility fallback. It owns both
command planning and actual process invocation, while
`final_executable_emit` stays provider-neutral. The policy and explicit-allow
gates still apply before that provider may run. It must execute the exact
driver path resolved during the verified dry-run boundary and may not repeat a
`PATH` lookup after admission.

This is not yet a pure Nsld linker claim. Nuisc currently asks the host
toolchain to produce the executable embedded in the compiled artifact; Nsld
validates and materializes that image but does not yet consume an LLVM
relocatable object and synthesize Mach-O load commands, symbols, and final
relocations itself. ELF and PE/COFF remain visible, selectable, and honestly
blocked rather than silently falling through a generic linker.

## Extension Rule

A future provider must:

1. register one stable provider identity and canonical target pattern;
2. consume the format-independent command/request context;
3. preserve the verified NSB entry, lifecycle, clock, and data-order metadata;
4. return an error before output mutation when its target or relocation input
   is unsupported;
5. pass the registry conformance and finalizer CLI regressions.

The next native milestone is to change the compiler handoff from a prelinked
host image to relocatable host/runtime objects, then add Nsld-owned Mach-O
relocation and executable-shell emission behind the same registry boundary.
That work must not add Mach-O branches to Nuisc, final-stage planning, or the
generic emit frontdoor.

## Validation

```sh
CARGO_INCREMENTAL=0 cargo test -q -j 1 -p nsld executable_finalizer
CARGO_INCREMENTAL=0 cargo test -q -j 1 -p nsld final_executable_host
CARGO_INCREMENTAL=0 cargo test -q -j 1 -p nsld --test host_finalizer_cli
```
