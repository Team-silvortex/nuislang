# FFI Pointer Safety Boundary

This file records the current `FFI` pointer and text boundary as it exists in
the current mainline.

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
* one exact-whitelisted owned `ref Buffer` return whose runtime-header length
  and registered destructor enter the normal GLM ownership pipeline
* one exact-whitelisted owned `ref String` return whose UTF-8 validity, byte
  length, read-only access, and registered destructor are checked end to end

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

The current mainline also keeps the source-facing `host_*` facade set used by
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

## Hash-Bound Memory Authority

An exact symbol signature can now carry a second, independent memory-authority
hash. Its canonical input is:

`nuis-ffi-memory-v1|<abi>|<symbol>|<signature_hash>|<descriptor>`

The manifest registration is:

`<abi>:<symbol>@<signature_hash>@<capability_hash>=<descriptor>`

The descriptor has one canonical field order:

`kind=<kind>,slot=<slot>,length=<policy>,mutability=<policy>,lifetime=<policy>,destructor=<authority>`

The first three admitted capability shapes are deliberately closed sets:

* `borrowed_utf8` must target a real `String` argument, use
  `length=nul_terminated`, `mutability=read_only`, `lifetime=call`, and
  `destructor=none`
* `owned_return_buffer` must target a `ref_Buffer` return, use
  `length=runtime_header`, `mutability=unique`, `lifetime=owned`, and name an
  exact registered destructor with signature `i64(ref_Buffer)`
* `owned_return_utf8` must target a `ref_String` return, use
  `length=runtime_header`, `mutability=read_only`, `lifetime=owned`, and name an
  exact registered destructor with signature `i64(ref_String)`

Validation requires the ABI to be declared, the symbol to have an exact
`ffi_symbol:` signature, both hashes to match, every kind/slot pair to be
unique, and any destructor registration to match exactly. A coarse `ffi:*`
family or a hash-only symbol entry cannot grant memory authority.

`official.cffi` currently uses the borrowed form for `libc.puts` argument 0,
`libc.strlen` argument 0, `libc.write` argument 1,
`c.host_text_line_count` argument 0, and `c.host_text_word_count` argument 0.
Those are real source/compiler paths, not placeholder registrations. A
non-reference `String` extern parameter without the matching capability is
rejected before lowering.

Owned return-buffer execution is now open for three deliberately narrow linear
paths. `host_owned_buffer_make(i64) -> ref Buffer` is admitted only through its
exact manifest capability. Lowering emits a dedicated Res-producing operation
with `nuis-ffi-owned-buffer-v1` metadata, recomputes the producer, capability,
and destructor hashes, recovers length from the registered runtime header, and
carries the exact destructor beside the LLVM pointer. A post-lowering graph
gate allows direct buffer access followed by either one same-function
`free(...)`, one registered conditional owner transfer, or one synchronous
helper return. Conditional transfer accepts two direct external producers only
when ABI, destructor symbol, and destructor signature hash are identical. The
helper path uses `nuis-ffi-owned-buffer-function-transfer-v1`: YIR and the
helper signature retain ABI/destructor/hash identity statically, while the
runtime return payload is exactly LLVM `{ ptr, i64 }`. One non-recursive
synchronous helper may now receive that result from another helper and move it
directly into its own result. Both function boundaries retain the same static
identity, the intermediate helper cannot release the owner, and only the entry
caller may perform the final exact-once `free(...)`. Native smoke executes both
conditional directions, direct helper calls, and the nested helper transfer.
Deeper source chains normalize lower calls by inlining, while loop, task/async,
recursive runtime transfer, a second helper-to-helper runtime hop,
secondary-extern, and mixed heap/external-owner escapes remain closed.

Owned UTF-8 uses a separate, intentionally smaller lane.
`host_owned_utf8_make(i64) -> ref String` is admitted only through its exact
manifest capability and lowers to `cpu.extern_call_owned_utf8` with
`nuis-ffi-owned-utf8-v1` metadata. Before source-visible reads, the native
runtime checks the allocation header, terminator, byte-length bound, and UTF-8
encoding. `owned_utf8_len(...)` and `owned_utf8_byte_at(...)` provide bounded
read-only access; mutation is rejected. The post-lowering gate requires all
reads and one exact registered `free(...)` to remain in the producer function,
with the release ordered after every read. Missing or duplicate cleanup,
helper/branch transfer, recursion, loops, tasks, and async escape fail closed.
The native example reads a multibyte value, releases it once, observes zero
live owned UTF-8 allocations, and exits `0`.

This is not generalized pointer-return support. It is one owned `ref Buffer`
contract with bounded transfer plus one direct-scope, read-only owned
`ref String` contract. A valid manifest still cannot authorize arbitrary
`ref T`, raw `ptr<T>`, retained borrows, recursive or unbounded runtime return
boundaries, or task-carried host memory.

In `nustar` manifest strings, multi-argument `ffi_symbol:` signatures can use
the same comma-separated form as source-facing signatures, for example
`i64(i64,i64)`. Older `+`-separated manifest signatures such as
`i64(i64+i64)` remain accepted as a compatibility alias.

AOT bundle manifests mirror the same contract with:

* `host_ffi_symbols=<symbol>@<abi>:<signature>;...`
* `host_ffi_symbol_hashes=<symbol>:fnv1a64:<hex>;...`

Project metadata mirrors the same policy earlier in
`nuis.project.host_ffi.txt`. Each extern row keeps the source-facing
`signature=fn <name>(...) -> <type>` for humans, and also records
`signature_pattern=<ret>(<args>)`, `signature_hash=fnv1a64:<hex>`, and
`policy=signature-whitelist-required` for tools. It additionally records
`memory_capability_count` and the canonical `memory_capabilities` list for each
row. This is the project-level audit trail used by heterogeneous proxy tests
before the final AOT bundle is packed. `nuisc verify-build-manifest` checks the
signature hash, capability count, identity, hash, shape, and destructor linkage
before reporting the build manifest as verified.

The linker-facing plan consumes the same project host FFI index as structured
data. Its `host_ffi` footprint contains the original index path, symbol and
policy counts, parsed entries, ABI groups, and validation summaries. The
top-level validation exposes `valid`, `link_allowed`, `issues`, and `notes`.
`link_allowed` is intentionally conservative: issues such as duplicate
`abi+symbol+signature` whitelist entries, policy drift, or count drift block
linking; notes such as multiple whitelisted signatures for the same
`abi+symbol` stay visible without blocking. ABI groups repeat this validation
locally and expose memory-capability counts and canonical descriptors, so
`nsld` can reason about each host ABI lane without reparsing the text index.

This is still not a dynamic C escape hatch. The linker consumes a static,
manifest-verified whitelist footprint. It does not invent new C ABI authority,
does not special-case libc, and does not bypass the registered `nustar`
capability model.

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
* an exact `owned_return_buffer` capability may return `ref Buffer`, but only
  through runtime-header length recovery and one registered destructor
* one synchronous helper may transfer that exact owner to its entry caller
  through `{ ptr, i64 }`; ABI/destructor/hash identity remains static metadata
* an exact `owned_return_utf8` capability may return read-only `ref String`, but
  it remains in one direct function scope and must use its registered destructor

Not currently source-stable:

* `ptr<T>` / raw pointer types
* pointer arithmetic
* arbitrary `ref T` host ABI parameters
* unregistered or generalized host ABI pointer returns
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

The owned return lanes are stricter than that inference. Each dedicated YIR op
calls its registered producer as `ptr`, validates the runtime-header length,
records `ptr+len+destructor`, and lowers `cpu.free` to that exact destructor
rather than libc `free`. Owned UTF-8 additionally invokes the fixed runtime
validator before reads and lowers byte access through checked `i8` loads.

This is an AOT bridge implementation detail, not a source-language raw pointer
feature.

The old built-in host stubs continue to use their existing integer-handle ABI
so current `std` facades are not silently reinterpreted as raw-pointer APIs.

The important architectural rule is:

`C ABI is a registered host-FFI capability, not an implicit compiler escape hatch`

Current regression anchors:

* [tests.rs](../../crates/yir-lower-llvm/src/tests.rs)
* [ffi_owned_utf8_compile.rs](../../tools/nuisc/tests/ffi_owned_utf8_compile.rs)
* [ffi_smoke.rs](../../tools/nuis/tests/ffi_smoke.rs)
* [ffi_compile.rs](../../tools/nuisc/tests/ffi_compile.rs)
* [registry_host_ffi_tests.rs](../../tools/nuisc/src/registry_host_ffi_tests.rs)
* [pipeline_ffi_owned_buffer.rs](../../tools/nuisc/src/pipeline_ffi_owned_buffer.rs)
* [ffi_smoke.rs](../../tools/nuis/tests/ffi_smoke.rs)
* [lib_tests_execution.rs](../../tools/nuisc/src/lib_tests_execution.rs)
* [owned_return_buffer_select_demo.ns](../../examples/ns/ffi/owned_return_buffer_select_demo.ns)

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

* [address-surface-contract.md](address-surface-contract.md)
* [nir-memory-model.md](nir-memory-model.md)
* [cpu-task-glm-contract.md](cpu-task-glm-contract.md)

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
