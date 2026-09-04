# Nuis Compiler Data Model

`nuis-compiler-data-model-v11` is the current compiler-owned data boundary
written in Nuis itself. Its machine-readable contract is
[nuis-compiler-data-model-v11.toml](nuis-compiler-data-model-v11.toml). The
chunked aggregate predecessor remains frozen in
[nuis-compiler-data-model-v10.toml](nuis-compiler-data-model-v10.toml), and the
registered aggregate predecessor remains frozen in
[nuis-compiler-data-model-v9.toml](nuis-compiler-data-model-v9.toml). The
paged typed-text predecessor remains frozen in
[nuis-compiler-data-model-v8.toml](nuis-compiler-data-model-v8.toml), the
single-vector typed-text predecessor remains frozen in
[nuis-compiler-data-model-v7.toml](nuis-compiler-data-model-v7.toml), the
stable arena-envelope predecessor remains frozen in
[nuis-compiler-data-model-v6.toml](nuis-compiler-data-model-v6.toml), the
deterministic-map predecessor remains frozen in
[nuis-compiler-data-model-v5.toml](nuis-compiler-data-model-v5.toml), the
complete-token-pagination predecessor remains frozen in
[nuis-compiler-data-model-v4.toml](nuis-compiler-data-model-v4.toml), the
materialized-token predecessor remains frozen in
[nuis-compiler-data-model-v3.toml](nuis-compiler-data-model-v3.toml), the
deterministic vector paging proof remains frozen in
[nuis-compiler-data-model-v2.toml](nuis-compiler-data-model-v2.toml), and the
original four-slot proof remains frozen in
[nuis-compiler-data-model-v1.toml](nuis-compiler-data-model-v1.toml).

The implementation is split across `StdLanguageCore`, `StdCompilerData`,
`StdCompilerPayload`, `StdCompilerPayloadRegistry`, `StdCompilerTokenEmit`,
`StdCompilerTokens`, and `StdCompilerProjection`, all inside bootstrap subset
v8. It proves that a
compiler component can own text, paged payload bytes, token records, canonical
token serialization, structural page state, paths, source identity,
allocation indices, maps, and diagnostics without borrowing Rust, C, libc,
FFI, or host-language layouts.

This is a bootstrap contract, not the final collection API for every Nuis
program.

## V11 Surface

V11 closes the bounded complete-arena call boundary without widening subset v8:

- `compiler_aggregate_arena_forward` captures the exact registry and complete
  arena identities, then passes both owned aggregates through
  `compiler_aggregate_arena_forward_checked` and revalidates them on return.
- The native fixture forwards the complete three-object, 44-byte, three-page
  v10 arena before projecting its chunked value. Stable indices, page bytes,
  registry identity `1593840720`, and complete identity `551151124` remain
  unchanged.
- Forwarding that same arena with the valid but incomplete v9 registry fails
  with code `3`. Re-reading the input under its correct registry produces the
  unchanged identity, so failure remains atomic.
- Both boundaries use ordinary Nuis owned-aggregate calls. No host collection,
  pointer identity, FFI, new import, new data type, or scalar export is added.

## V10 Surface

V10 closes the first individual-payload paging gap without changing the v9
registry, arena, or scalar-export contracts:

- `CompilerChunkedPayload` owns one canonical `CompilerPayloadBuffer` and a
  page-derived shape identity. It may span all eight logical pages instead of
  returning through the sixteen-value `CompilerVector` compatibility path.
- `compiler_payload_copy_buffer_range` validates source shape, range, target
  shape, and capacity before applying eight fixed sixteen-byte Nuis chunks.
  Store and projection expose no host collection, pointer, loop state, or
  partial mutation.
- Kind `3`, schema `103`, stores the 24-byte reference payload
  `nuis-compiler-payload-v1`. Its page identities are `1042165038` and
  `957262363`; the typed identity is `94500080`.
- The extended three-kind registry has identity `1593840720`, while the frozen
  two-kind v9 registry remains `1630830726`. Text, source span, and chunked
  payloads occupy 44 aggregate bytes and three pages with complete identity
  `551151124`.
- Forged typed identity and full object capacity fail before returning a new
  arena. Native execution remains `130`, and the subset still has exactly
  twenty-one scalar exports.

## Preserved V9 Surface

V9 turns the v8 page store into a registered aggregate boundary without
changing the frozen language subset or scalar export ABI:

- `CompilerPayloadRegistry` is a stable insertion-order `CompilerMap` from a
  positive kind to a compact schema and fixed/variable-length descriptor.
  Duplicate registration fails with code `4`; registry identity is
  `1630830726` for the reference text and source-span kinds.
- `CompilerAggregateArena` composes the unchanged stable-index
  `CompilerArena` envelope with one shared `CompilerPayloadBuffer`. Generic
  storage validates registration and length before copying bytes and appending
  the envelope, so failures leave the input identity unchanged.
- Kind `1`, schema `101`, projects canonical `CompilerText`. Kind `2`, schema
  `102`, projects `CompilerSourceSpan` as three canonical little-endian u31
  fields with fixed length 12 and reference shape identity `1383365918`.
- The reference store places eight-byte `nuislang` before the 12-byte span.
  The span crosses the page boundary; page identities are `934788601` and
  `1229397900`, envelope identity is `1109161393`, and complete identity is
  `1274791798`.

V9 adds one approved std import and two registered data names, but no source
capability or scalar export. The subset remains v8 with exactly twenty-one
exports, and the frozen v8 contracts remain unchanged.

V8 adds the first typed payload column that crosses the old sixteen-byte
boundary:

- `CompilerPayloadBuffer` is the payload-facing alias for the canonical owned
  packed-byte buffer already proven by the token path. It stores at most 128
  bytes without making host endianness or aggregate layout part of identity.
- `nuis-compiler-payload-pages-v1` views that contiguous storage as up to eight
  logical sixteen-byte pages. Page ordinal and offset select one byte; only the
  terminal page may be short.
- Every page identity binds ordinal, byte start, byte length, and all bytes.
  The reference pages have lengths `16` and `2` and identities `712007164` and
  `132664649`.
- `CompilerPagedTextArena` preserves the v7 `CompilerArena` envelope while
  replacing its single-vector payload column with the paged buffer. UTF-8
  validation, projection, and ordered identity cross page boundaries in Nuis.
- The reference store contains `nuislang`, U+03BB, and `nuislang` as 18 bytes.
  Its third record begins at byte 10 and crosses the sixteen-byte boundary;
  envelope identity is `784615130` and complete identity is `322532187`.

The underlying arithmetic packing is a storage implementation, not token
semantics and not a host ABI. V8 adds one approved std import and two approved
data names, but no source-language capability or scalar export; the frozen
bootstrap ceiling remains twenty-one exports.

V9 preserves the v8 surface, which retains the v7 compiler foundation:

V7 retains the v6 compiler foundation:

- `CompilerVector` stores up to sixteen `i64` values in four ordered pages.
- `CompilerText` owns canonical UTF-8 bytes, scalar length, and a deterministic
  content hash.
- `CompilerPath`, `CompilerMap`, `CompilerArena`, `CompilerSourceSpan`, and
  `CompilerDiagnostic` retain stable-index and fail-closed behavior.
- Generic `Option<T>` and `Result<T, E>` remain the absence and error channels.

V7 adds the first typed aggregate payload over stable arena indices:

- `CompilerTextArena` composes an unchanged `CompilerArena` envelope with one
  contiguous compiler-owned `CompilerVector` UTF-8 payload column.
- Kind `1` identifies an owned `CompilerText`; its three fields bind payload
  start, byte length, and Unicode scalar length without storing a host pointer.
- Store validates source vector shape, canonical UTF-8, scalar count, and text
  hash before copying bytes and appending the envelope. Immutable value returns
  make capacity or validation failure atomic to the caller.
- Projection validates the complete store, copies the selected range into a
  fresh vector, recomputes its hash, and returns `Result<CompilerText, i64>`.
  Invalid indices use code `2`; malformed shape, kind, UTF-8, or hash uses `3`.
- `nuis-compiler-text-arena-ordered-identity-v1` binds the existing envelope
  identity, total payload length, and every payload byte in stable order.

The v7 typed store intentionally remains frozen as a bounded compatibility
proof in which all text payloads share one sixteen-byte vector. V8 layers a
new type beside it rather than changing its shape or identity.

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

V6 turns the arena's stable indices into owned compiler object storage:

- `CompilerArena` stores `kind`, `field0`, `field1`, and `field2` in four
  shape-checked `CompilerVector` columns for up to sixteen objects.
- A successful store appends all four fields atomically and returns the same
  index forever; `last_index` is always object count minus one.
- Kind `0` is reserved for the compatibility allocator. Negative kinds,
  oversized capacities, malformed columns, and identity requests fail closed.
- `compiler_arena_object_value` projects one checked index/slot and returns
  `Option.None` for an absent index or invalid slot.
- `nuis-compiler-arena-ordered-identity-v1` binds object count and every slot
  in stable index order using Nuis recursion and canonical signed folding.

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

It also stores two compiler objects with indices `0` and `1`, reads positive
and negative fields back through checked projections, and pins ordered arena
identity `1064756829`. Full capacity, negative kind, invalid index/slot,
oversized capacity, malformed columns, and malformed identity all fail before
observable mutation.

The preserved v7 path stores `nuislang` and U+03BB (two-byte UTF-8 `cebb`) at typed
indices `0` and `1`. Their ten payload bytes rebuild owned texts with hashes
`1135407074` and `53387`; the unchanged envelope binds identity `1856301942`
and the complete typed store binds identity `1643761726`. Object exhaustion,
payload exhaustion, invalid index, wrong kind, malformed UTF-8, and forged text
hashes fail with exact codes while the pre-failure identity remains unchanged.

The v8 path then appends `nuislang` a second time through
`CompilerPagedTextArena`. Its 18-byte payload occupies two logical pages, and
the third record crosses the boundary at byte 16. Native checks project that
record into a fresh `CompilerText`, verify page lengths and boundary bytes,
pin both page identities plus complete identity `322532187`, and reject an
absent page or object with exact error `2`.

The v9 path explicitly registers text and source-span schemas, stores both at
stable indices in one aggregate arena, and projects fresh owned values through
typed getters. Its 20-byte payload crosses the first logical page inside the
source-span record. Native checks pin registry, span, envelope, page, and
complete identities, reject duplicate registration with code `4`, reject
wrong kind and absent index with code `2`, reject malformed fixed-length bytes
with code `3`, and prove the failed store leaves identity unchanged.

The v10 extension registers kind `3` beside those frozen entries and stores
`nuis-compiler-payload-v1` as one 24-byte `CompilerChunkedPayload`. The record
starts at aggregate byte 20, spans the second and third logical pages, and
projects into a fresh packed buffer with the same `94500080` typed identity.
The extended registry identity is `1593840720`, envelope identity is
`1520342505`, page identities are `934788601`, `1001962162`, and `1407376619`,
and complete aggregate identity is `551151124`. A forged identity returns code
`3`; a fourth object returns capacity code `1`; both leave the input identity
unchanged.

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
than only a synthetic capacity test. Candidate production v11 additionally
pages the complete real token stream, binds its terminal page hash and ordered
chain identity, and attests compact structured NIR records plus their
producer-neutral v2 selection beside the data model without making host layout
part of their identity.

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
page over this foundation. It does not claim that the bounded v11 model parses a
complete source file or stores an unbounded page sequence.

## Honest Boundary

V11 proves owned token materialization, canonical re-emission, complete token
pagination, deterministic multi-page maps, bounded arena object storage, and
registered typed text, source-span, and 24-byte chunked projection across
logical payload pages. It also proves that the complete registered arena and
its registry survive two nested owned helper boundaries without identity drift,
but it is not
yet an unbounded compiler heap:

- Token storage is limited to four records and 64 payload bytes.
- The canonical bootstrap emitter is limited to four records, 64 input payload
  bytes, and 128 output bytes.
- `CompilerVector` remains `i64`-specific and bounded to sixteen values.
- `CompilerMap` remains `i64`-specific and bounded to sixteen entries.
- `CompilerArena` remains a sixteen-object, four-`i64` envelope.
- One `CompilerText` remains limited to sixteen bytes, while the v11 aggregate
  payload column is bounded to 128 bytes across eight logical pages.
- Registration is generic, but typed codecs currently cover only
  `CompilerText`, `CompilerSourceSpan`, and canonical chunked bytes.
- Generic helper specialization still needs defining-module provenance in the
  general case; the concrete compiler arena path now preserves it explicitly.
- Arbitrary aggregate loop-carried state remains a lowering gap; fixed chunks
  avoid pretending otherwise.

The bounded self-hosting readiness gate is now `stable/100`; this is not an
unbounded heap or replacement authorization. Future growth can add broader
structural paging and aggregate loop backedges without weakening
canonical UTF-8, stable indices, fail-closed errors, host-layout independence,
or the frozen v8 import ceiling.
