# Nuis Compiler Data Model

`nuis-compiler-data-model-v5` is the current compiler-owned data boundary
written in Nuis itself. Its machine-readable contract is
[nuis-compiler-data-model-v5.toml](nuis-compiler-data-model-v5.toml). The
complete-token-pagination predecessor remains frozen in
[nuis-compiler-data-model-v4.toml](nuis-compiler-data-model-v4.toml), the
materialized-token predecessor remains frozen in
[nuis-compiler-data-model-v3.toml](nuis-compiler-data-model-v3.toml), the
deterministic vector paging proof remains frozen in
[nuis-compiler-data-model-v2.toml](nuis-compiler-data-model-v2.toml), and the
original four-slot proof remains frozen in
[nuis-compiler-data-model-v1.toml](nuis-compiler-data-model-v1.toml).

The implementation is split across `StdLanguageCore`, `StdCompilerData`,
`StdCompilerTokenEmit`, `StdCompilerTokens`, and `StdCompilerProjection`, all
inside bootstrap subset v8. It proves that a compiler component can own text,
token records, canonical token serialization, structural page state, paths,
source identity, allocation indices, maps, and diagnostics without borrowing
Rust, C, libc, FFI, or host-language layouts.

This is a bootstrap contract, not the final collection API for every Nuis
program.

## V5 Surface

V5 retains the v4 compiler foundation:

- `CompilerVector` stores up to sixteen `i64` values in four ordered pages.
- `CompilerText` owns canonical UTF-8 bytes, scalar length, and a deterministic
  content hash.
- `CompilerPath`, `CompilerMap`, `CompilerArena`, `CompilerSourceSpan`, and
  `CompilerDiagnostic` retain stable-index and fail-closed behavior.
- Generic `Option<T>` and `Result<T, E>` remain the absence and error channels.

V5 replaces the original four-slot map proof with a deterministic columnar
map suitable for bootstrap symbol tables:

- `CompilerMap` stores keys and values in two shape-checked
  `CompilerVector` columns, for a bounded capacity of sixteen entries.
- New keys append in stable insertion order. Replacing an existing key updates
  its value at the same index without changing length or order.
- Lookup and replacement use bounded Nuis recursion rather than host
  collections or host iteration state.
- `nuis-compiler-map-ordered-identity-v1` binds length and every ordered
  key/value pair with a canonical non-negative modular fold.
- A seventeenth distinct key returns capacity error `1`; malformed column
  shapes return error `3` before lookup, mutation, or identity production.

V3 added the materialized token boundary:

- `CompilerTokenStore` holds four records in deterministic columnar vectors.
- Each record owns kind, payload start, payload length, and numeric value.
- `CompilerTokenBuffer` owns up to 128 bytes as at most nineteen arithmetic
  little-endian seven-byte `i64` words across two deterministic vectors. This
  is a protocol-defined packing order, not host endianness or host layout.
- Text payloads are produced only from valid Unicode scalars and canonical
  UTF-8 encoding.
- `compiler_token_store_begin`, `push_scalar`, and `finish` form an explicit
  record lifecycle.
- `compiler_token_store_get` reconstructs an owned `CompilerTokenRecord`.
- `compiler_token_store_emit` reconstructs canonical `nuis-token-stream-v1`
  bytes without host token semantics.
- `StdCompilerTokenEmit` owns `CompilerTokenMaterializer`, which drives the
  standalone bounded `StdCompilerTokens` DFA into a fresh owned store and
  stops mutating state after its first complete four-record page.
- `CompilerDecimalState` and the emitter cover the complete signed `i64`
  domain without negating `i64::MIN`.

V4 adds complete-stream token pagination without pretending every record fits
inside the materialized window:

- `nuis-compiler-token-pagination-v1` divides the exact verified token payload
  into contiguous 128-byte pages; only the terminal page may be shorter.
- Page boundaries may cross records. Long strings and documentation comments
  therefore require no oversized token store or record truncation.
- Every page binds ordinal, byte start, byte count, cumulative completed-record
  count, page hash, and cumulative chain identity.
- Five exact subset-v8 scalar exports let Nuis compute the page hash and chain;
  the host adapter only transports bytes and counters.
- `nuis-artifact` independently decodes the complete stream and recomputes
  every page and chain identity before candidate production succeeds.
- The original four-record canonical page and identity `164749511446` remain
  as a stronger materialization/re-emission compatibility proof.

Open records enforce structural invariants but may be semantically incomplete.
`finish` owns final kind-specific checks such as required word text, symbol
Unicode scalar validity, and arrow numeric shape. Completed stores must satisfy
both structural and semantic invariants. This separation prevents a semantic
error from being misreported as malformed storage.

Error code `1` means capacity exhaustion, `2` means invalid state or index, and
`3` means invalid input or shape. The packed-word fields and public `*_raw`
emitter helpers are transitional bootstrap implementation surfaces; neither is
a stable std layout or host ABI.

## Executable Evidence

The pure Nuis project
`examples/projects/tooling/bootstrap_compiler_data_model_demo` materializes four
records:

```text
word: ns
integer: -12
symbol: +
arrow
```

It re-emits the exact 59-byte stream:

```text
nuis-token-stream-v1
word	6e73
integer	-12
symbol	43
arrow
```

The contract independently pins emitted hash `2002147233`. Boundary checks
cover record and byte lookup, closed-record mutation, empty-word completion,
surrogate-symbol completion, open-store emission, fifth-record capacity,
legacy vector capacity, malformed paging, map absence, arena exhaustion, and
UTF-8 scalar rejection.

The same executable builds an eight-entry symbol map across two pages, updates
key `13` in place, and pins ordered identity `415394959`. It then fills all
sixteen entries, rejects a seventeenth distinct key with error `1`, and rejects
a malformed column shape with error `3`. These checks execute in the native
binary through ordinary Nuis calls.

The same executable also constructs the first complete materialized page from the real
candidate token stream: `use`, `cpu`, `StdLanguageCore`, and semicolon. It owns
21 payload bytes and canonically emits 91 bytes with independently pinned hash
`1277127995`. The `StdCompilerTokenEmit` materializer drives the standalone
`StdCompilerTokens` DFA over those exact bytes into a fresh
`CompilerTokenStore`, and the canonical emitter reproduces the same length
and hash. Production additionally packs the actual handoff prefix through the
twenty-one-export subset-v8 scalar ABI, and the artifact layer independently verifies page
identity `164749511446`. This proves an owned decode/re-emit round trip across
the fifth and sixth former 16-byte boundaries on a real compiler prefix rather
than only a synthetic capacity test. Candidate production v9 additionally
pages the complete real token stream, including records longer than one page,
and binds its terminal page hash and ordered chain identity.

The acceptance chain is:

```text
nuisc bootstrap-check
  -> normal semantic / NIR / YIR / LLVM pipeline
  -> nuis bootstrap-build
  -> host-native executable
  -> deterministic exit score 130
```

The production path contains no deferred LLVM lowering. Nested aggregate
parameters use the generic scalar-leaf direct-call ABI, and structural
`cpu.guard_return` now branches and returns through the owned aggregate ABI.
Guarded variant returns carry the same function-level owned layout as ordinary
returns, including recursively materialized unit-enum payloads, so all return
paths share one aggregate shape.
Because arbitrary aggregate loop-carried values remain incomplete, the page
materializers use eight deterministic sixteen-byte chunks and the scalar
exports validate raw aggregate state explicitly. The native regression also
pins open-record versus completed-record and partial structural-line boundaries.

`StdCompilerProjection` consumes the first AST and NIR structural pages,
serializes an opaque eight-lane scanner cursor, and resumes both into a second
page over this foundation. It does not claim that the bounded v5 model parses a
complete source file or stores an unbounded page sequence.

## Honest Boundary

V5 proves owned token materialization, canonical re-emission, complete
token-stream pagination, and deterministic multi-page map behavior, but it is not
yet an unbounded compiler heap:

- Token storage is limited to four records and 64 payload bytes.
- The canonical bootstrap emitter is limited to four records, 64 input payload
  bytes, and 128 output bytes.
- `CompilerVector` remains `i64`-specific and bounded to sixteen values.
- `CompilerMap` remains `i64`-specific and bounded to sixteen entries.
- `CompilerArena` allocates indices but does not own general object storage.
- Generic helper specialization still needs defining-module provenance.
- Arbitrary aggregate loop-carried state remains a lowering gap; fixed chunks
  avoid pretending otherwise.

The readiness gate is therefore not yet `stable/100`. The next mainline step
is compiler-arena-owned object storage while preserving stable map indices.
Broader structural paging and aggregate
backedge lowering follow without weakening
canonical UTF-8, stable indices, fail-closed errors, host-layout independence,
or the frozen v8 import ceiling.
