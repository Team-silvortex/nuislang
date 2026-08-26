# Nuis Compiler Data Model

`nuis-compiler-data-model-v2` is the current compiler-owned data boundary
written in Nuis itself. Its machine-readable contract is
[nuis-compiler-data-model-v2.toml](nuis-compiler-data-model-v2.toml). The
bounded four-slot predecessor remains frozen in
[nuis-compiler-data-model-v1.toml](nuis-compiler-data-model-v1.toml).

The implementation lives in `StdLanguageCore`, inside the frozen bootstrap
import ceiling. It proves that a compiler component can own text, token and
path sequences, source identity, allocation indices, maps, and diagnostics
without borrowing Rust, C, libc, FFI, or host-language layouts.

This remains a bootstrap contract rather than the final collection API for all
Nuis programs.

## V2 Surface

V2 preserves the v1 import and `CompilerVector` type names while replacing the
single four-value body with four deterministic pages:

- `CompilerVector` stores up to sixteen `i64` values in four ordered pages.
- `CompilerVectorPage` stores four values and a checked local length.
- `compiler_vector_push`, `get`, and `set` validate the complete canonical
  shape before observing or changing a value.
- `CompilerText` uses the paged vector for canonical UTF-8 bytes, scalar
  length, and deterministic content hash.
- `CompilerPath` uses the same paging contract for interned segment identities.
- `CompilerMap` remains a deterministic four-entry integer map.
- `CompilerArena` provides monotonic stable indices with explicit capacity.
- `CompilerSourceSpan` is a validated half-open byte range.
- `CompilerDiagnostic` combines stable code, severity, text, span, and path.
- Generic `Option<T>` and `Result<T, E>` remain the error and absence channels.

Pages open in index order. A length of six, for example, requires a full first
page, a two-value second page, and empty third and fourth pages. Malformed
length/page combinations fail closed. The page fields are visible only because
the current bootstrap specializer needs cross-module aggregate access; they are
not stable std layout or host ABI.

Error code `1` means capacity exhaustion, `2` means invalid index, and `3`
means invalid input or malformed shape.

## Executable Evidence

The project
`examples/projects/tooling/bootstrap_compiler_data_model_demo` is a pure Nuis
compiler-shaped component using only `StdLanguageCore`. It builds eight bytes
of source text and six token values, so both cross a page boundary. It also
builds a symbol map, arena node identities, an eight-byte source span, a path,
and a diagnostic.

The boundary checks cover the sixth token, an out-of-range token read, all
sixteen capacity slots, a rejected seventeenth push, a missing map key, arena
exhaustion, a forged negative-length page shape, and surrogate scalar
rejection. The acceptance chain is:

```text
nuisc bootstrap-check
  -> normal semantic/NIR/YIR/LLVM pipeline
  -> nuis build
  -> host-native executable
  -> deterministic exit score 59
```

The emitted LLVM contains no deferred lowering. The v2 pressure fixture also
found a real NIR-to-YIR lexical-scope bug: a repeated match-arm name could
capture an earlier scrutinee during pure branch substitution. A dedicated
regression now requires substitution to stop at nested `let` and `const`
shadowing boundaries.

`StdCompilerProjection` remains a separate consumer over this data foundation.
It consumes stable scalar AST/NIR record tags and carries projection state in
Nuis-owned structures without claiming that the bounded v2 model can parse a
complete source file.

## Honest Boundary

V2 establishes deterministic page transitions and raises vector capacity from
four to sixteen values. It does not yet provide unbounded growth:

- `CompilerVector` is currently `i64`-specific because bytes, tokens, and
  interned identities are the first bootstrap workload.
- `CompilerMap` remains limited to four integer entries.
- `CompilerArena` allocates stable indices but does not yet own object storage.
- Generic nested-page specialization still needs defining-module provenance.
- Large pure aggregate expansion is correct but remains expensive to compile.

The readiness gate is therefore `usable/80`, not `stable/100`. The next model
should page the map and arena storage, generalize page specialization, and
reduce compile-time expansion while preserving v2 identities, canonical UTF-8,
fail-closed errors, host-layout independence, and the frozen bootstrap import
ceiling.
