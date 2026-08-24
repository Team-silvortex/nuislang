# Nuis Compiler Data Model

`nuis-compiler-data-model-v1` is the first compiler-owned data boundary written
in Nuis itself. Its machine-readable contract is
[nuis-compiler-data-model-v1.toml](nuis-compiler-data-model-v1.toml), and its
implementation lives in `StdLanguageCore` so it remains inside the frozen
bootstrap import ceiling.

This is a bounded proof contract, not the final collection API for all Nuis
programs. It exists to prove that a compiler component can own text,
collections, source identity, allocation indices, paths, and diagnostics
without borrowing Rust, C, libc, FFI, or host-language layouts.

## Surface

The v1 surface provides:

- `CompilerVector<T>` with four owned slots and checked push/get/set
- `CompilerText` with canonical UTF-8 scalar encoding, scalar length, and a
  deterministic content hash
- `CompilerMap` with four integer key/value entries and deterministic upsert
- `CompilerArena` with monotonic stable indices and explicit capacity
- `CompilerSourceSpan` as a validated half-open byte range
- `CompilerPath` as bounded interned segment identities
- `CompilerDiagnostic` combining stable code, severity, text, span, and path
- generic `Option<T>` plus the existing `Result<T, E>` error channel

Error code `1` means capacity exhaustion, `2` means invalid index, and `3`
means invalid input. The operations return data; they do not print or invoke a
host diagnostic service.

## Executable Evidence

The project
`examples/projects/tooling/bootstrap_compiler_data_model_demo` is a small
compiler-shaped component using only `StdLanguageCore`. It builds source text,
a token stream, a symbol map, arena node identities, a source span, a path,
and a diagnostic. It also proves that invalid vector lookup, missing map keys,
arena exhaustion, and surrogate scalar input fail closed.

The acceptance chain is:

```text
nuisc bootstrap-check
  -> normal semantic/NIR/YIR/LLVM pipeline
  -> nuis build
  -> host-native executable
  -> deterministic exit score 43
```

The emitted LLVM must contain no deferred lowering. A dedicated LLVM test also
pins repeated `Result.Ok` payload selection across nested tagged-union chains;
the tag and payload must be selected together.

`StdCompilerProjection` is a separate companion module over this owned data
foundation. Its first native candidate consumes stable scalar AST/NIR record
tags and carries projection state in Nuis-owned structures. It deliberately
does not claim that the bounded collection model can parse a complete source
file yet.

## Honest Boundary

Four slots are sufficient to pressure ownership, generics, enums, `?`, match
binding, nested structures, error behavior, and native lowering. They are not
enough for a realistic tokenizer or parser. The readiness gate is therefore
`usable/75`, not `stable/100`.

The next version must add deterministic page- or chunk-backed growth without
widening the frozen bootstrap subset. It must preserve canonical UTF-8,
stable arena indices, fail-closed errors, host-layout independence, and the
same bootstrap/build/run evidence before a tokenizer-sized component adopts
it.
