# `nuis` `alpha-0.10.*` Mainline Entry

This file is now the predecessor short entry point for the `alpha-0.10.*`
line.

It does not replace the `alpha-0.8.*` binary-linking convergence entry, the
`alpha-0.7.*` std/tooling smoke entry, the `alpha-0.6.*` Nsld entry, or the
`alpha-0.4.*` hardening baseline. Those remain predecessor and baseline
context. This file records the line where the binary route moves from "before
alpha-0.10" target language into the active alpha-0.10 closure window.

Short rule:

`alpha-0.10.*` is where nuis started treating the minimal executable artifact
loop as the main integration gate: not final self-hosting, not a finished
self-owned linker, but an honest route from project inputs through Nsld-owned
metadata to a runnable or explicitly blocked executable boundary.

## Current Line Shape

Read the current line as:

* `alpha-0.4.*` hardening baseline still applies
* `alpha-0.6.*` introduced the named Nsld linker frontdoor
* `alpha-0.7.*` made std-backed tooling examples the default smoke surface
* `alpha-0.8.*` made binary-linking convergence the default toolchain pressure
* `alpha-0.10.*` makes executable-artifact closure the active integration gate
* current docs should say `alpha-0.13.*` for present-tense work and link this
  file as predecessor executable-artifact context

## Current Front Doors

Start here:

1. [../current-mainline-map.md](../../docs/current-mainline-map.md)
2. [../reference/nsld-linker-frontdoor.md](../../docs/reference/nsld-linker-frontdoor.md)
3. [../reference/nsld-binary-assembly-gap-map.md](../../docs/reference/nsld-binary-assembly-gap-map.md)
4. [../reference/nuis-native-artifact-workflow.md](../../docs/reference/nuis-native-artifact-workflow.md)
5. [../reference/nuis-binary-format-protocol.md](../../docs/reference/nuis-binary-format-protocol.md)
6. [../reference/nustar-multi-backend-artifact-contract.md](../../docs/reference/nustar-multi-backend-artifact-contract.md)
7. [nuis-alpha-0.8-mainline-entry.md](nuis-alpha-0.8-mainline-entry.md)
8. [nuis-alpha-0.7-mainline-entry.md](nuis-alpha-0.7-mainline-entry.md)
9. [nuis-alpha-0.6-mainline-entry.md](nuis-alpha-0.6-mainline-entry.md)
10. [nuis-alpha-0.4-system-inventory.md](nuis-alpha-0.4-system-inventory.md)
11. [nuis-alpha-0.4-mainline-hardening-plan.md](nuis-alpha-0.4-mainline-hardening-plan.md)

## Main Target During `alpha-0.10.*`

The mainline target is not a complete self-owned production linker yet.

The target is the first minimal executable-artifact closure:

```text
project
  -> nuisc/NIR/YIR/AOT artifact
  -> Nsld object image or compatibility object
  -> Nsld container + payload
  -> closure snapshot
  -> final-stage plan
  -> executable writer input
  -> runnable host-assisted artifact or explicit blocked executable artifact
  -> verify/check
```

The first acceptable route may still use a host-compatible wrapper or platform
object shell. That is fine as long as the contract remains Nsld-owned:

* no hidden dynamic shortcut
* no linker special case for one Nustar domain
* no untracked C-world bypass
* deterministic artifact-chain ordering
* deterministic section/data ordering
* stable final-stage readiness/check diagnostics
* documented blockers when real execution cannot happen

## What Alpha-0.10 Should Optimize

Prefer work in this order:

1. make existing Nsld plans consume verified artifact/container/closure state
2. keep object-image, container, closure, and final-stage hashes stable
3. make writer-input verification mandatory before any final executable emit
4. keep host-assisted finalization separate from future self-owned finalization
5. keep Mach-O, ELF, and PE target concepts behind target/backend traits rather
   than hardcoding one platform path
6. expose blocked executable artifacts as first-class checked outputs
7. route std/tooling examples through the same artifact chain rather than
   inventing side channels
8. keep shader/kernel/network/CFFI outputs registered through Nustar contracts
   instead of coupling Nsld to domain-specific logic

## Current Weakest Links Toward Bootstrap

The weakest link is no longer "does the language have enough surface syntax?"
It is the back half of the native compilation loop.

Read the current pressure order as:

1. finish the Nsld final-executable boundary so the current driver can move
   from verified writer/final-stage inputs to an explicit final output, even if
   the first output is still host-assisted
2. keep lowering honest under larger examples, especially when std, CFFI,
   shader, kernel, network, and project imports meet in one build route
3. make std capable enough for real CLI/tooling programs without creating
   hidden host-only side channels
4. keep package/import/module behavior stable enough that future self-hosted
   compiler pieces can depend on it without special bootstrap rules
5. grow diagnostics, debug metadata, and blocked-artifact reports so the
   toolchain can explain why a build cannot yet become runnable

This is the alpha-0.10 bridge into the longer self-hosting roadmap: executable
closure first, richer self-use later. The route can stay incremental, but it
should not blur whether a step is a verified Nsld-owned artifact, a host-assisted
compatibility layer, or a future self-owned linker/runtime responsibility.

## What Already Exists From The Predecessor Lines

The current repository already has enough structure to make this closure work
incrementally:

* `nuis` and `nuisc` frontdoors over project manifests, checks, build routes,
  artifact inspection, and release checks
* NIR/YIR generation, verification, LLVM lowering, and AOT artifact emission
* Nustar registration for `cpu`, `data`, `shader`, `kernel`, `network`, and
  CFFI/host-compatibility boundaries
* Nsld object/container/closure/final-stage planning and diagnostics
* YIR-level debugger metadata direction through Nsdb
* std-backed tooling examples that can pressure the real build/link route
* PixelMagic and WitSage source assets that can become heterogeneous workload
  pressure once shader/kernel closure is stronger

## What Should Not Be Claimed Yet

`alpha-0.10.*` should not claim:

* final self-hosting
* final std API stability
* final self-owned linker implementation
* final unified heterogeneous executable binary
* final GPU/NPU backend maturity
* beta-level public stability

Safe wording:

* say `executable-artifact closure`, `minimal runnable route`, or
  `host-assisted finalization` when that is the current truth
* say `self-owned final linker` only for Nsld-owned contracts or future work
  until the final executable emitter is real
* say `blocked executable artifact` when the pipeline can prove the boundary but
  cannot yet produce a runnable host artifact
* say `predecessor executable-artifact closure line` for `alpha-0.10.*`
