# FFI Pointer Safety Boundary

This file records the current `FFI` pointer and text boundary as it exists in
the alpha mainline.

The short rule is:

`source-level FFI stays handle-first; AOT may lower selected internal values to ptr; raw pointer syntax is not open yet`

## Why This Exists

The repository now has three related but different concepts:

* `ref Node` / `ref Buffer` inside `nuis`
* host-facing integer handles such as text handles and file handles
* LLVM/AOT `ptr` values used to call dynamic host functions

These must not be collapsed into one idea.

If the language exposes raw pointers too early, `GLM`, ownership, async
transfer, and host ABI rules all become coupled at once. The current contract is
therefore deliberately staged.

## Current Source-Level Contract

Current `.ns` source still does not expose a general raw pointer type.

Stable source-facing FFI should be read as:

* scalar `i64` / `i32` host values
* host-owned handles represented as integer values
* `Text`-like data flowing through text handles or compiler-generated text
  lift/lookup helpers
* the narrow `ref Buffer` host bridge used for buffer-backed read/write
  surfaces

Host FFI is also registered through `nustar`.

The current CPU package does not merely say "`c` exists"; it now carries a
small FFI allowlist in `abi_capabilities`. The compiler checks `extern`
declarations against that allowlist before lowering them.

Current initial allowlist shape:

* legacy integer/handle facades: `i64(*)`
* i32 scalar probes: `i32(*)`
* buffer bridge families: `i64(ref_Buffer+*)` and `i32(ref_Buffer+*)`
* registered symbol contracts: `ffi_symbol:<symbol>=<signature>`
* hash-registered symbol contracts: `ffi_symbol_hash:<symbol>=fnv1a64:<hex>`

The signature families remain intentionally coarse as a staging guard. When a
symbol has a `ffi_symbol:` entry, that symbol is checked against its registered
signature first and cannot fall back to the wider family allowlist.

The alpha mainline also keeps the source-facing `host_*` facade set used by
`std` and curated examples in exact `ffi_symbol:` registration. This includes
the current CLI/text/filesystem/process/diagnostic/result/network facade
symbols. The broad `i64(*)` family remains only as a compatibility staging
surface for experiments, not as the intended security boundary for official
host facades.

`libc` is a separate registered ABI surface rather than another name for the
project-owned `c` host facade set. The initial libc allowlist is deliberately
tiny: `getpid() -> i32`, `usleep(i32) -> i32`, `puts(String) -> i32`,
`strlen(String) -> i64`, `write(i32, String, i64) -> i64`, and
`close(i32) -> i32`, plus `read(i32, ref_Buffer, i64) -> i64`. The text and
buffer bridges are still not raw
pointer escapes: source code passes a Nuis `String`, and lowering exposes the
backing C string pointer only inside the registered call boundary. That keeps
system C calls explicit and auditable while the wider C FFI nustar grows.

The hash form uses the canonical input:

`nuis-ffi-symbol-v1|<abi>|<symbol>|<signature>`

For example, `c|host_hashed_curve|i64(i64)` is registered as
`ffi_symbol_hash:host_hashed_curve=fnv1a64:38ca92f356fcb551`.

In `nustar` manifest strings, multi-argument `ffi_symbol:` signatures can use
the same comma-separated form as source-facing signatures, for example
`i64(i64,i64)`. Older `+`-separated manifest signatures such as
`i64(i64+i64)` remain accepted as a compatibility alias.

AOT bundle manifests mirror the same contract with:

* `host_ffi_symbols=<symbol>@<abi>:<signature>;...`
* `host_ffi_symbol_hashes=<symbol>:fnv1a64:<hex>;...`

The packer verifies these two lines against each other before writing the
bundle manifest, so ABI drift, signature drift, and hash drift are caught at
pack time.

The packer also compares those manifest lines against its host FFI registry
view. A symbol must be registered by `(abi, symbol)` and must match either the
registered signature or the registered signature hash. This keeps `C ABI` as a
declared capability instead of an implicit escape hatch, even in generated AOT
bundles.

By default, the AOT packer loads the host FFI registry view from the CPU
`nustar` manifest and then adds only its own built-in shim symbols. This keeps
official `std`/CLI/network facade registration anchored in one manifest source
instead of duplicating it in the packer. Generated bundle manifests always
record `host_ffi_registry_source` so fallback behavior is visible during
debugging; bundles without host FFI use `host_ffi_registry_source=none`.
Bundles also record `host_ffi_registry_lines` and
`host_ffi_registry_symbols` to make the loaded registry size auditable.
`host_ffi_registry_abis` records the ABI set visible through that registry, and
`host_ffi_registry_hash` fingerprints the canonical sorted registry lines so
registry drift can be detected without diffing the whole manifest. Bundles also
record `host_ffi_used_symbols` and `host_ffi_used_abis` to summarize the
bundle's actual host FFI footprint. `host_ffi_footprint_hash` hashes the
canonical symbol/signature list and per-symbol hash list so two bundles can be
compared for host FFI drift without diffing every entry.

The narrow buffer bridge means:

* an extern parameter may be declared as `ref Buffer` for current buffer
  transport surfaces
* frontend lowering turns that parameter into `HostBufferHandle(...)`
* this is not a promise that arbitrary `ref T` is a stable host ABI value
* borrowed and owned pointer authority still belongs to the internal memory
  model and verifier

Not currently source-stable:

* `ptr<T>` / raw pointer types
* pointer arithmetic
* arbitrary `ref T` host ABI parameters
* host ABI pointer returns
* generalized external authority contracts for raw host memory

## Current AOT / LLVM Contract

The LLVM bridge may lower selected YIR producers to host ABI `ptr` when calling
dynamic extern symbols.

Current dynamic extern parameter inference is conservative:

* `cpu.text` / `TextHandle` producers can pass a `ptr`
* pointer producers such as `alloc_buffer`, `alloc_node`, `borrow`,
  `move_ptr`, `load_next`, and `null` can pass a `ptr`
* `extern_call_i32` / `const_i32` / `call_i32` producers can pass `i32`
* everything else defaults to `i64`

This is an AOT bridge implementation detail, not a source-language raw pointer
feature.

The old built-in host stubs continue to use their existing integer-handle ABI
so current `std` facades are not silently reinterpreted as raw-pointer APIs.

The important architectural rule is:

`C ABI is a registered host-FFI capability, not an implicit compiler escape hatch`

Current regression anchors:

* [tests.rs](/Users/Shared/chroot/dev/nuislang/crates/yir-lower-llvm/src/tests.rs)
* [ffi_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/ffi_compile.rs)

## Current String Boundary

String-like FFI currently has two levels:

* source-facing handle APIs, such as `host_text_len(handle)` and
  `stdout_write(text_handle)`
* AOT-internal text pointer lowering, where `cpu.text` is lifted through
  `nuis_host_text_lift(ptr)` and can still carry a `TextHandle { ptr, handle }`
  inside LLVM lowering

That means text can reach dynamic host externs as a `ptr` in the LLVM bridge,
but ordinary source should still treat text as a managed host/runtime surface,
not as mutable raw memory.

## Current Pointer Boundary

Internal `ref` values already participate in real ownership-sensitive
compiler behavior.

That behavior is anchored by:

* [address-surface-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/address-surface-contract.md)
* [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)

At the host edge, however, `ref` is still not a universal ABI promise.

Current safe reading:

* internal `ref` is real
* `ref Buffer` at FFI is a narrow transport bridge
* LLVM `ptr` lowering is allowed only as an AOT bridge step
* source-visible raw pointer APIs need an explicit future safety surface

## Future External Authority Gate

`nuis` may not need a Rust-style `unsafe` keyword at all. The stronger alpha
direction is that native code is only promised to be valid inside the `nuis`
execution and memory model; calls outside that model must pass through explicit
registered capabilities.

Before raw pointer syntax or generalized pointer FFI is opened, the language
therefore needs at least:

* an explicit external-authority marker for raw host pointer APIs
* a source-visible distinction between owned, borrowed, and raw host pointers
* GLM facts for host calls that consume, lend, mutate, or retain pointer values
* verifier rules for pointer escape across async/task/thread boundaries
* AOT/linker agreement on pointer ownership and lifetime after the call

Until those exist, new FFI work should prefer:

* managed handles
* explicit buffer bridge helpers
* compiler-owned text lifting
* small dynamic extern probes with regression tests

Short future rule:

`raw pointer FFI should be introduced as a registered external capability, not as a convenient spelling`
