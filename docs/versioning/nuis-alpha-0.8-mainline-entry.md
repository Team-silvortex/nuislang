# `nuis` `alpha-0.8.*` Mainline Entry

This file is the current short entry point for the `alpha-0.8.*` line.

It does not replace the `alpha-0.7.*` std/tooling smoke entry, the
`alpha-0.6.*` Nsld entry, or the `alpha-0.4.*` hardening baseline. Those remain
predecessor and baseline context. This file records the line where current work
starts treating binary linking as the main convergence target.

Short rule:

`alpha-0.8.*` is where Nsld moves from a linker frontdoor with strong reports
toward the smallest real binary-linking loop that can run before `alpha-0.10.0`.

## Current Line Shape

Read the current line as:

* `alpha-0.4.*` hardening baseline still applies
* `alpha-0.6.*` introduced the named Nsld linker frontdoor
* `alpha-0.7.*` made std-backed tooling examples the default smoke surface
* `alpha-0.8.*` makes binary linking convergence the default toolchain pressure
* current docs should say `alpha-0.8.*` for present-tense work and link older
  alpha docs as predecessor/baseline context

## Current Front Doors

Start here:

1. [../current-mainline-map.md](../../docs/current-mainline-map.md)
2. [../reference/nsld-linker-frontdoor.md](../../docs/reference/nsld-linker-frontdoor.md)
3. [../reference/nsld-binary-assembly-gap-map.md](../../docs/reference/nsld-binary-assembly-gap-map.md)
4. [../reference/toolchain-galaxy-core-boundary.md](../../docs/reference/toolchain-galaxy-core-boundary.md)
5. [nuis-alpha-0.7-mainline-entry.md](nuis-alpha-0.7-mainline-entry.md)
6. [nuis-alpha-0.6-mainline-entry.md](nuis-alpha-0.6-mainline-entry.md)
7. [nuis-alpha-0.4-system-inventory.md](nuis-alpha-0.4-system-inventory.md)
8. [nuis-alpha-0.4-mainline-hardening-plan.md](nuis-alpha-0.4-mainline-hardening-plan.md)

## Main Target Before `alpha-0.10.0`

The mainline target is not a complete self-owned production linker yet.

The target is a minimal binary-linking loop:

```text
prepare
  -> object image / compatibility object
  -> container + payload
  -> closure snapshot
  -> final-stage plan
  -> final executable readiness
  -> smallest runnable binary or explicit blocked executable artifact
  -> verify/check
```

By `alpha-0.10.0`, the repository should be able to demonstrate one honest
route where the final stage is more than a report. The acceptable first route
may still use a host-compatible wrapper or platform object shell, but the route
must stay Nsld-owned in contract terms:

* no hidden dynamic shortcut
* no linker special case for one Nustar domain
* no untracked C-world bypass
* deterministic artifact-chain ordering
* stable final-stage readiness/check diagnostics
* documented blockers when real execution cannot happen

## What Alpha-0.8 Already Has

The current Nsld line has enough structure to start chasing the real run:

* artifact-chain diagnostics with stable stage ids and suggested commands
* object writer readiness and Mach-O arm64 compatibility object emission
* object byte/file/image dry-run reports
* relocation lowering rules and relocation record tables
* object-output verification against deterministic image bytes
* container and container payload emission
* closure snapshot emission and verification keyed by `linker_contract_hash`
* final-stage plan/readiness/blocked executable reports
* `check` integration for emitted artifacts and optional chain-tail diagnostics

This is why alpha-0.8 should spend less energy on new surfaces and more energy
on turning the existing Nsld chain into a runnable binary path.

## What To Do First

The next work should prefer this order:

1. keep `artifact-chain` suggestions aligned with real emit commands
2. keep final executable writer planning aligned with the exact writer steps and
   writer blockers
3. keep final executable writer input emit/verify aligned with the host-assisted
   command boundary
4. keep final executable emit consuming writer-input verification before any
   real host linker invocation
5. split host-compatible finalization from future self-owned Nsld finalization
   without hardcoding Mach-O as the only world
6. add the first minimal final-executable emitter, even if it is intentionally
   host-assisted
7. verify the emitted executable boundary with the same hash/check discipline
   used by object output and closure snapshots

## What Should Not Be Claimed Yet

`alpha-0.8.*` should not claim:

* final self-hosting
* final std API stability
* final self-owned linker implementation
* final unified heterogeneous executable binary
* final GPU/NPU backend maturity
* beta-level public stability

Safe wording:

* say `binary-linking convergence`, `minimal runnable route`, or
  `host-assisted finalization` when that is the current truth
* say `self-owned final linker` only for Nsld-owned contracts or future work
  until the executable emitter is real
* say `current line` for `alpha-0.8.*`
* say `predecessor` for `alpha-0.7.*` and `alpha-0.6.*`; say `baseline` for the
  still-relevant `alpha-0.4.*` hardening docs
