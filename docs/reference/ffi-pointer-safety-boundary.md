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

The hash form uses the canonical input:

`nuis-ffi-symbol-v1|<abi>|<symbol>|<signature>`

For example, `c|host_hashed_curve|i64(i64)` is registered as
`ffi_symbol_hash:host_hashed_curve=fnv1a64:38ca92f356fcb551`.

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
* unsafe blocks or unsafe function contracts

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

## Future Unsafe Gate

Before raw pointer syntax or generalized pointer FFI is opened, the language
needs at least:

* an explicit unsafe marker for raw host pointer APIs
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

`unsafe pointer FFI should be introduced as a contract surface, not as a convenient spelling`
