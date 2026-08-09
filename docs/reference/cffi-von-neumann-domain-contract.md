# CFFI / Von Neumann Domain Contract

This note records a long-range architecture rule that already matters to
current `beta-0.1.0` linker, FFI, standard-library, and heterogeneous-worker
work.

## Short Rule

`C and the classic von-Neumann host stack are a compatibility domain inside Nuis, not the ontology of Nuis`

Nuis may use C ABI, libc, host operating systems, native object files, and
platform executable shells during bootstrap. Those pieces are practical and
important. They should still enter the Nuis world through an explicit domain
contract rather than silently becoming the default machine model.

## Domain Shape

The C / host compatibility world should be modeled as a first-class registered
execution domain, comparable in status to other `nustar` capability domains:

* `cpu`
* `shader`
* `kernel`
* `network`
* `data fabric`
* `cffi` / host compatibility

This does not mean C is unimportant. It means C is too powerful and too
historically central to be left as an implicit escape hatch.

The domain should be able to describe:

* ABI family and platform shell facts such as Mach-O, ELF, PE/COFF, and C ABI
* allowed symbol signatures through whitelist and hash-signature registration
* pointer, buffer, ownership, and lifetime boundary policy
* memory effects visible to GLM and YIR verification
* lifecycle hooks used to schedule host-compat work deterministically
* native-object or wrapper payloads admitted by Nsld into a Nuis container
* diagnostics explaining which host facts are compatibility facts, not core
  language semantics

## Linker Rule

Nsld should not treat native object files as the core linker substrate.

Traditional object files and platform executable shells are compatibility and
finalization artifacts. When they are needed, they should be represented as
host-compat payloads inside the Nuis binary/link graph, with lifecycle metadata
and validation attached.

Current naming anchor:

```text
on_cffi_native_object
```

That hook represents the current narrow lane where verified native-object
payloads can be admitted into the container as CFFI / host-compat work.

Future Nsld work should preserve this direction:

* Nuis-owned link graph first
* deterministic lifecycle and clock metadata first
* verified domain payloads first
* host-native object or executable wrappers as selected outputs, not inputs
  that define the model

## Safety Rule

The CFFI / host domain should stay conservative by default.

For current work, prefer:

* explicit host-symbol declarations
* hash-registered ABI signatures
* narrow pointer and string boundary forms
* wrapper policy owned by the CFFI domain
* no linker-only special cases that bypass YIR/GLM metadata

If a future host call needs broader pointer or ownership behavior, the domain
contract should grow first. The implementation should not silently widen raw
host access because a platform ABI happens to allow it.

## Performance Posture

The CFFI / host-compat layer is allowed to have real overhead during bootstrap.

That overhead is not a failure by itself. It is the price of keeping the C
world explicit, whitelistable, inspectable, and replaceable while Nuis is still
building its own linker, binary format, runtime contracts, and future OS path.

Planning posture:

* near-term bootstrap: expect host-compat calls and wrapper paths to be slower than
  equivalent hand-written C/C++ when they cross the CFFI boundary frequently
* medium-term Nuis OS: move more compatibility work below the process-level
  wrapper into kernel/runtime-owned compatibility services
* long-term hardware-paradigm work: let heterogeneous scheduling, data-fabric
  placement, shader/kernel/NPU participation, and lifecycle-aware packaging
  compete on workloads where a classic CPU/C++ pipeline is not the best shape

Target language should stay honest:

* compatibility overhead should be benchmarked, not guessed into invisibility
* a rough early 20% class slowdown against tight native C/C++ baselines can be
  acceptable if the boundary is explicit and removable
* an OS-level compatibility lane should aim to push common host-boundary costs
  toward a lower 5-10% class range where the workload shape allows it
* no percentage is a contract until measured by checked-in benchmarks

Short rule:

`pay the compatibility tax only where the architecture can later itemize, move, or remove it`

## Hardware-Paradigm Rule

The long-range interpretation is intentionally broader than "FFI binding."

Nuis should be able to treat the classic C / von-Neumann stack as one hardware
and execution paradigm among several. Today that paradigm maps to existing CPUs,
operating systems, process loaders, libc, and host toolchains. Tomorrow it may
be one compatibility fabric inside a Nuis-native OS or a heterogeneous machine.

This is similar to the current status of `data fabric`: the project may not
have independent hardware for it yet, but the architecture should still reserve
the concept so future hardware or runtime support can grow into it.

Short rule:

`host compatibility is a domain adapter; it is not the root of the universe`

## Current Beta Reading

The repository now has a registered `official.cffi` Nustar and a strict source
boundary. Checked-in Nuis declarations using `extern` live under `mod cffi`;
placing an extern declaration in `mod cpu` is rejected rather than treated as a
legacy escape hatch. The CFFI manifest owns the accepted ABI profiles,
host-symbol surfaces, exact signatures, and signature hashes.

```nuis
mod cffi Main {
  extern "c" fn host_stdout_write(text_handle: i64) -> i64;

  fn main() -> i64 {
    return host_stdout_write(71);
  }
}
```

This is still a bootstrap bridge, not a claim that the repository has a
finished self-owned linker or native Nuis OS loader. `cffi.yir.lowering.v1`
currently delegates host object-code production to the registered CPU/LLVM
provider. A CFFI project therefore reports both `official.cffi` and
`official.cpu`: CFFI owns source admission and ABI policy, while CPU owns the
current host machine-code realization.

The first concrete memory-authority protocol is also active. Borrowed UTF-8
arguments are bound to their exact ABI, symbol, signature hash, argument slot,
NUL-terminated length policy, read-only mutability, call lifetime, and absent
destructor by a second `nuis-ffi-memory-v1` hash. Five official libc/text
entries exercise this route through compilation, project metadata, and Nsld
link-plan validation. Drift in signature, capability hash, slot, length,
mutability, lifetime, count, or destructor fails before lowering or linking.

An owned return-buffer descriptor can already prove a `ref_Buffer` return and
an exact registered `i64(ref_Buffer)` destructor. It does not yet grant source
execution authority: lowering remains closed until YIR and GLM can own the
returned length-bearing allocation and prove exact-once transfer and cleanup.
This is the intended domain rule in practice: registration may describe the
next safe capability without silently opening a host pointer escape.

Current constraints remain:

* standard-library host facades should stay narrow and explicit
* CFFI signatures must remain exact-whitelist and hash registered
* Nsld native-object lanes should carry lifecycle metadata
* YIR and GLM should remain the semantic review layer
* host-native formats should remain useful wrappers, not the internal truth

Related references:

* [nsld-linker-frontdoor.md](nsld-linker-frontdoor.md)
* [nsld-binary-assembly-gap-map.md](nsld-binary-assembly-gap-map.md)
* [ffi-pointer-safety-boundary.md](ffi-pointer-safety-boundary.md)
* [host-read-bridge.md](host-read-bridge.md)
* [std-host-io-layering-contract.md](std-host-io-layering-contract.md)
* [../versioning/nuis-long-range-heterogeneous-os-roadmap.md](../versioning/nuis-long-range-heterogeneous-os-roadmap.md)
