# Nuis Bootstrap Language Subset

This reference freezes the first compiler-authoring source boundary as
`nuis-bootstrap-language-subset-v3`. Its machine-readable inventory is
[nuis-bootstrap-language-subset-v3.toml](nuis-bootstrap-language-subset-v3.toml),
and the executable policy lives in `nuis-semantics`.

This is not a promise to keep the public Nuis language frozen. It is a narrow,
versioned dependency ceiling for the first Nuis-written compiler components.

## Frontdoor

Validate a source file or project through the installed compiler:

```bash
nuisc bootstrap-check compiler.ns
nuisc bootstrap-check --json compiler-project
```

The repository-local form is:

```bash
cargo run -q -p nuisc -- bootstrap-check tests/fixtures/bootstrap/accepted/compiler_scanner.ns
```

An accepted report means two independent conditions hold:

1. Every project-local AST module obeys the v3 allowlist.
2. The same input crosses the normal semantic, NIR, YIR, and LLVM emission
   checks without an error.

Policy rejection skips the semantic pipeline and exits nonzero. A source that
obeys the allowlist but fails normal compilation also exits nonzero with
`semantic_pipeline=failed`. JSON reports preserve the same fail-closed result.

## Frozen Surface

V3 permits only `mod cpu` compiler modules. Local CPU modules may import each
other; external imports are limited to `CorePrelude`, `StdLanguageCore`,
`StdCompilerData`, `StdCompilerTokenEmit`, `StdCompilerTokens`,
`StdCompilerProjection`, and
`StdTextContracts`.

The executable type allowlist contains:

- `bool`, `i64`, and transitional `text` scalars
- types declared by project-local bootstrap modules
- local unbounded generic parameters
- `Option`, `Result`, and the reserved compiler data-model names recorded in
  the TOML contract

The structural surface includes constants, aliases, structs, enums, ordinary
functions, unbounded generics, local mutation, destructuring, direct and method
calls, struct literals, field access, integer/Boolean operators, `if`, `match`,
`while`, `break`, `continue`, `return`, and `?` propagation.

The source spellings `loop { ... }`, `else if`, and `if let` are also accepted.
They normalize before subset validation to `while true`, nested `if`, and a
two-arm `match` respectively, so subset v3 gains no new AST capability or
policy bypass.

Documentation attributes are generally the only metadata dependency. Twelve
exact all-`i64` `@export` signatures are additionally reserved for the stage1
candidate's scalar fold and token-decoder ABI. Function name, symbol name,
parameter count, and return type must all match the machine-readable allowlist;
every other export remains `NBS004`. Diagnostics must be returned as data
rather than printed by a compiler component. V3 adds only the registered
`StdCompilerData`/`StdCompilerTokenEmit` import pair and its
`CompilerDecimalState`, `CompilerTokenBuffer`, `CompilerTokenRecord`, and
`CompilerTokenStore` owned types. It preserves the v2 scalar export ABI and
does not admit a new language capability class.

## Deliberate Exclusions

V3 rejects these capabilities even when the wider language supports them:

- shader, kernel, network, data, CFFI, and other non-CPU domains
- arbitrary library imports, extern declarations, and FFI interfaces
- arbitrary attributes or exports, traits, impl dispatch, bounded generics,
  lambdas, and dynamic invocation
- async functions, `await`, tests, and benchmarks
- floating-point values and types
- references, nullable address types, raw dereference, and low-level memory
  intrinsics
- heterogeneous unit instantiation and host/runtime effect intrinsics

These are sequencing decisions, not claims that the features are inherently
unsuitable for compilers. A later subset version may admit a capability only
after its ownership, determinism, data-model, and differential-stage behavior
has executable evidence.

## Diagnostics

Stable codes `NBS001` through `NBS017` identify the frozen rejection classes.
The TOML contract maps every code to one capability, while reports include the
module identity, AST path, and explanatory message. AST paths are structural
rather than byte offsets so this first protocol does not depend on a host
parser's span representation.

## Fixtures

The accepted scanner fixture is
`tests/fixtures/bootstrap/accepted/compiler_scanner.ns`. It exercises generic
`Result`, enums, structs, `?`, `match`, loops, mutation, direct calls, and
integer control flow before crossing the full semantic pipeline.

Rejected fixtures under `tests/fixtures/bootstrap/rejected` independently pin
async effects, FFI/address behavior, non-CPU domains, float/lambda behavior,
unapproved imports, and host effects. Adding a newly permitted capability
requires updating the contract, executable policy, fixtures, tests, readiness
evidence, and development tensor together.
