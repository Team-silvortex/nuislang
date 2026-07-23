# `nuis` `alpha-0.16.*` Mainline Entry

This file is the direct predecessor entry point for the `alpha-0.16.*` line.

For present-tense work, start with
[nuis-alpha-0.17-mainline-entry.md](nuis-alpha-0.17-mainline-entry.md).

It does not replace the `alpha-0.10.*` executable-artifact closure entry, the
`alpha-0.8.*` binary-linking convergence entry, the `alpha-0.7.*` std/tooling
smoke entry, the `alpha-0.6.*` Nsld entry, or the `alpha-0.4.*` hardening
baseline. Those remain predecessor and baseline context.

Short rule:

`alpha-0.16.*` is where nuis treats the development tensor as the default
mainline steering surface: close the weakest verified cell first, keep the
compiler/linker/std/heterogeneous surfaces on one route, and avoid adding new
islands unless they are immediately registered, tested, and explainable.

## Current Line Shape

Read the current line as:

* `alpha-0.4.*` hardening baseline still applies
* `alpha-0.6.*` introduced the named Nsld linker frontdoor
* `alpha-0.7.*` made std-backed tooling examples the default smoke surface
* `alpha-0.8.*` made binary-linking convergence the default toolchain pressure
* `alpha-0.10.*` made executable-artifact closure the active integration gate
* `alpha-0.16.*` makes tensor-guided closure hardening the current working mode
* current docs should say `alpha-0.17.*` for present-tense work and treat this
  file as the tensor-guided closure predecessor

## Current Front Doors

Start here:

1. [../current-mainline-map.md](../../docs/current-mainline-map.md)
2. [../reference/nuis-development-tensor.md](../../docs/reference/nuis-development-tensor.md)
3. [../reference/nsld-linker-frontdoor.md](../../docs/reference/nsld-linker-frontdoor.md)
4. [../reference/nsld-binary-assembly-gap-map.md](../../docs/reference/nsld-binary-assembly-gap-map.md)
5. [../reference/nuis-native-artifact-workflow.md](../../docs/reference/nuis-native-artifact-workflow.md)
6. [../reference/nustar-multi-backend-artifact-contract.md](../../docs/reference/nustar-multi-backend-artifact-contract.md)
7. [../reference/toolchain-galaxy-core-boundary.md](../../docs/reference/toolchain-galaxy-core-boundary.md)
8. [nuis-alpha-0.10-mainline-entry.md](nuis-alpha-0.10-mainline-entry.md)
9. [nuis-alpha-0.8-mainline-entry.md](nuis-alpha-0.8-mainline-entry.md)
10. [nuis-alpha-0.7-mainline-entry.md](nuis-alpha-0.7-mainline-entry.md)
11. [nuis-alpha-0.6-mainline-entry.md](nuis-alpha-0.6-mainline-entry.md)
12. [nuis-alpha-0.4-system-inventory.md](nuis-alpha-0.4-system-inventory.md)
13. [nuis-alpha-0.4-mainline-hardening-plan.md](nuis-alpha-0.4-mainline-hardening-plan.md)

## Main Target During `alpha-0.16.*`

The main target is not broad feature sprawl.

The target is a steadily tightening route:

```text
project
  -> frontdoor/project metadata
  -> nuisc/NIR/YIR/lowering
  -> std and official Galaxy contract consumption
  -> Nustar registered backend artifacts
  -> Nsld object/container/closure/final-stage metadata
  -> run-artifact / trace readiness diagnostics
  -> explicit runnable output or explicit blocked boundary
  -> development tensor evidence update
```

The important rule is that every useful feature should eventually show up in
the same evidence chain. If it cannot be checked, linked, inspected, traced, or
represented in the tensor, it is probably still a sketch rather than mainline
truth.

## Native-Language Ownership Rule

Until self-hosting, the compiler implementation may remain on its Rust host
base. Outside that explicit bootstrap exception, new std, runtime, worker,
scheduler, and official Galaxy control logic should be authored in Nuis
whenever the language and lowering can express it.

C, C++, Objective-C, and platform SDK languages may remain at registered ABI
boundaries where the host exposes no native Nuis interface. Those adapters must
stay generated or deliberately thin, policy-free, replaceable, and visible to
YIR/GLM contracts. They must not silently become the owners of lifecycle loops,
request protocols, scheduling, memory policy, or Nustar dispatch.

Short rule:

`compiler hosting is temporary; product control flow belongs to Nuis`

## What Alpha-0.16 Should Optimize

Prefer work in this order:

1. close the weakest development tensor cells before opening unrelated lanes
2. keep `nuis` frontdoor commands, `nuisc` lowering, Nsld reports, and std
   examples aligned on one build/run-artifact story
3. promote PixelMagic and WitSage shader/kernel demos from artifact smoke tests
   toward per-domain run-artifact trace records
4. keep CFFI and host compatibility as registered domains, not linker special
   cases
5. keep Mach-O, ELF, PE, shader, kernel, and network outputs behind target and
   backend contracts rather than finite hardcoded combinations
6. update docs/status/tensor evidence whenever a route becomes real
7. continue file/module splitting when implementation files start hiding
   unrelated concerns

## Current Weakest Links Toward Bootstrap

The weakest link is still the executable back half, but it is now more specific
than "we need a linker".

Read the current pressure order as:

1. turn Nsld final-stage metadata and run-artifact readiness into a reliable
   runnable or explicitly blocked product loop
2. make heterogeneous shader/kernel payloads participate in trace/debug
   evidence, not only build-side artifact checks
3. keep std broad enough for real CLI programs without smuggling in untracked
   host-only side channels
4. stabilize package/import/module behavior enough for future self-hosted
   compiler pieces
5. keep diagnostics and status tensor output precise enough that the next thin
   place is always visible

## What Should Not Be Claimed Yet

`alpha-0.16.*` should not claim:

* final self-hosting
* final std API stability
* final self-owned production linker
* final unified heterogeneous executable binary
* final GPU/NPU backend maturity
* beta-level public stability

Safe wording:

* say `tensor-guided closure hardening` for the current work mode
* say `executable-artifact closure` for the Nsld-owned build/run boundary
* say `run-artifact trace readiness` when metadata can explain the route but
  device execution is not yet final
* say `blocked executable artifact` when the pipeline can prove the boundary but
  cannot yet produce a runnable host artifact
* say `tensor-guided closure predecessor` for `alpha-0.16.*`
