# `nuis` `beta-0.10.*` Self-Hosting Entry

This file is the current-line anchor for the formal Nuis self-hosting migration
period. The first recorded checkpoint is Git commit `c496bb67`
(`beta-0.10.0`). Git history remains authoritative for later patch checkpoints;
independent Cargo package versions still do not encode the project release.

`beta-0.10.*` activates staged `stage0 -> stage1` compiler migration. It does
not claim that Nuis is already self-hosted, that stage0 can be removed, or that
the candidate compiler has final replacement authority.

Short rule:

`beta-0.10.* starts measured compiler ownership transfer while every incomplete gate remains visible and stage0 remains a rollback root`

## Canonical Reading Order

1. [../current-mainline-map.md](../current-mainline-map.md)
2. [../reference/nuis-self-hosting-readiness.md](../reference/nuis-self-hosting-readiness.md)
3. [../reference/nuis-development-tensor.md](../reference/nuis-development-tensor.md)
4. [../reference/nuis-compiler-candidate-fresh-source-capability.md](../reference/nuis-compiler-candidate-fresh-source-capability.md)
5. [../reference/nuis-compiler-candidate-successor.md](../reference/nuis-compiler-candidate-successor.md)
6. [../reference/nuis-compiler-component-transition-v2.toml](../reference/nuis-compiler-component-transition-v2.toml)
7. [../reference/nuis-compiler-component-reproducibility.md](../reference/nuis-compiler-component-reproducibility.md)
8. [nuis-beta-0.6.0-mainline-entry.md](nuis-beta-0.6.0-mainline-entry.md)
9. [README.md](README.md)

## Entry State

The executable readiness protocol is now
`nuis-self-hosting-readiness-v2`. Its current state is deliberately asymmetric:

* phase: `stage0-to-stage1-migration`
* phase status: `active`
* migration active: `true`
* final readiness: `false`
* closed gates: `1/5`
* weakest coordinate: `compiler-toolchain/bootstrap/stage0-stage1-driver`

Entering migration with four `usable/99` gates does not silently close them.
It changes task selection: bounded candidate-owned vertical slices now take
priority, while each remaining gate must still reach `stable/100` before final
replacement authority is considered.

## Proven Starting Point

The entry checkpoint already has more than a parser sketch:

* a frozen bootstrap language subset with twenty-one exact scalar exports;
* canonical source, token, AST, NIR, and YIR handoff evidence;
* a pure Nuis stage1 candidate and production-v11 proof;
* stage0/candidate `13/13` differential and two-clean-build reproducibility;
* separate attestation, replacement authorization, active-state, rollback,
  dispatch, preselection, and signed-successor boundaries;
* direct provider-free candidate front-end execution;
* one exact 56-byte fresh-source snapshot consumed by candidate-owned source,
  token, AST, NIR, and YIR state without a stage0 handoff.

That final slice is intentionally narrow. It proves that compiler ownership can
cross the source boundary without claiming a general parser or native object
producer.

## First Migration Target

The first active target is one vertical materialization path:

```text
canonical fresh Nuis source
  -> candidate-owned source/token/AST/NIR/YIR identity
  -> candidate-owned native object or equivalent Nsld input
  -> independent stage0 comparison
  -> path-free capability evidence
  -> no replacement or selection authority
```

The source snapshot should stay frozen until this path is deterministic,
reproducible, fail-closed, and independently comparable. Grammar breadth comes
after ownership depth for this step.

## Migration Discipline

Every self-hosting increment should preserve these rules:

1. transfer one explicit capability rather than relabeling a host operation;
2. keep stage0 and the last verified candidate immutable and independently
   addressable;
3. emit canonical, path-free evidence before granting any new authority;
4. compare candidate and stage0 semantics before selection changes;
5. reject partial persistence, environment leakage, provider drift, and stale
   predecessor identity;
6. update the development tensor after each completed slice;
7. widen the bootstrap language subset only through a new protocol version.

## What Remains Open

The migration period begins with these known gaps:

* candidate-owned NIR/YIR to native object materialization;
* general fresh-source grammar and serialized stage representations;
* wider compiler collections, nested aggregate forwarding, and loop backedges;
* a signed transition that replays and binds the fresh-source capability;
* remote fresh-source and native-materialization reproducibility evidence;
* repeated deterministic stage1 rebuilds sufficient for final selection;
* eventual stage2-equivalent ownership during the later gamma completion
  window.

## Honest Boundary

`beta-0.10.0` proves the project reached the planned migration starting line.
It is not a self-hosting completion release, a stable ABI promise, or permission
to remove Rust/LLVM/bootstrap compatibility code. Stage0 remains part of the
trusted rollback chain until later evidence explicitly supersedes it.
