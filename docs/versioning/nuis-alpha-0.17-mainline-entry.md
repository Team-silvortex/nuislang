# `nuis` `alpha-0.17.*` Mainline Entry

This file is the current short entry point for the `alpha-0.17.*` line.

Do not confuse this line with the historical pre-alpha `0.17.0` documents.
Files named `nuis-0.17.0-*` describe an earlier compiler phase. Files named
`nuis-alpha-0.17-*` describe the current alpha mainline.

The direct predecessor is:

* [nuis-alpha-0.16-mainline-entry.md](nuis-alpha-0.16-mainline-entry.md)

Earlier executable, linking, std, and hardening anchors remain useful as
predecessor context:

* [nuis-alpha-0.10-mainline-entry.md](nuis-alpha-0.10-mainline-entry.md)
* [nuis-alpha-0.8-mainline-entry.md](nuis-alpha-0.8-mainline-entry.md)
* [nuis-alpha-0.7-mainline-entry.md](nuis-alpha-0.7-mainline-entry.md)
* [nuis-alpha-0.6-mainline-entry.md](nuis-alpha-0.6-mainline-entry.md)
* [nuis-alpha-0.4-system-inventory.md](nuis-alpha-0.4-system-inventory.md)

Short rule:

`alpha-0.17.*` turns the registered heterogeneous worker boundary into the
active integration gate: Nuis owns lifecycle and authorization, Nustars own
backend-specific execution, and Nsdb coordinates graph/session evidence
without hardcoding a finite backend combination.

## Current Line Shape

Read the current progression as:

* `alpha-0.4.*` established the hardening baseline
* `alpha-0.6.*` introduced the named Nsld linker frontdoor
* `alpha-0.7.*` made std-backed tooling examples the default smoke surface
* `alpha-0.8.*` made binary-linking convergence the toolchain pressure
* `alpha-0.10.*` made executable-artifact closure the integration gate
* `alpha-0.16.*` made the development tensor the default steering surface
* `alpha-0.17.*` makes registered heterogeneous worker execution the weakest
  closure boundary to cross

Current docs should use `alpha-0.17.*` for present-tense work. Older alpha
entries are predecessor or baseline context rather than competing current
routes.

## Canonical Reading Order

1. [../current-mainline-map.md](../../docs/current-mainline-map.md)
2. [../reference/nuis-development-tensor.md](../../docs/reference/nuis-development-tensor.md)
3. [../reference/nustar-multi-backend-artifact-contract.md](../../docs/reference/nustar-multi-backend-artifact-contract.md)
4. [../reference/nsld-linker-frontdoor.md](../../docs/reference/nsld-linker-frontdoor.md)
5. [../reference/nsld-binary-assembly-gap-map.md](../../docs/reference/nsld-binary-assembly-gap-map.md)
6. [../reference/nuis-native-artifact-workflow.md](../../docs/reference/nuis-native-artifact-workflow.md)
7. [../reference/toolchain-galaxy-core-boundary.md](../../docs/reference/toolchain-galaxy-core-boundary.md)
8. [../reference/cffi-von-neumann-domain-contract.md](../../docs/reference/cffi-von-neumann-domain-contract.md)
9. [nuis-alpha-0.16-mainline-entry.md](nuis-alpha-0.16-mainline-entry.md)

## Current Connected Spine

```text
nuis source / nuis.toml
  -> nuis frontdoor
  -> nuisc
  -> NIR
  -> YIR + GLM / clock / domain verification
  -> registered Nustar lowering and artifacts
  -> Nsld object / container / closure / final-output planning
  -> run-artifact and Nsdb trace evidence
  -> development tensor handoff
```

The heterogeneous execution path now adds:

```text
provider graph request
  -> registered provider / adapter / operation identity
  -> cached Nuis AOT worker image
  -> persistent Unix worker lease
  -> Nuis-owned ingress and lifecycle dispatch
  -> worker-issued, status-bound operation permit
  -> registered Metal / CoreML execution boundary
  -> output carrier, comparison, trace, and graph-close evidence
```

## Verified Truth Entering Alpha-0.17

The repository currently verifies:

* one persistent worker process per provider adapter/session
* PID-bound worker handshake and ordered request sequence
* post-spawn carrier descriptor transfer through `SCM_RIGHTS`
* one semantic role per transferred descriptor
* binary-safe, length-bound, hash-bound request and response payloads
* open-ended provider, adapter, and operation registration identities
* a Nuis-authored lifecycle and request-ingress loop
* a reply protocol, `NUISPWUR7`, carrying the positive Nuis invocation status,
  request identity without body echo, bounded adapter metadata, and a
  worker-owned output carrier role, mode, length, and hash
* a worker error receipt, `NUISPWUE1`, preserving failure stage and status
* fail-closed rejection of non-positive worker dispatch status
* operation permit evidence in final native-output summaries
* `nuis-provider-execution-capsule-v1` registration binding provider, adapter,
  operation token, and ordered input/output carrier roles
* stable capsule id/token evidence carried through worker authorization into
  final native-output summaries
* capsule token and input/output role counts validated by the persistent Nuis
  worker through eight-dependency `data.provider_request_ingress`
* a registry-derived capsule invoker called by Nuis before reply
* independent parent-side verification of the returned output descriptor
* return-producing effectful `if` lowering through the open
  `compare_call_result` host-call guard protocol, with the provider worker
  proving unselected reply calls remain unexecuted
* four ordered CoreML requests and one Metal request in the official
  heterogeneous smoke route
* fan-in descriptors, cross-provider transfer evidence, output comparison,
  graph-close release, and development-tensor drift checks

This is real integration evidence, but it is not yet the final heterogeneous
binary closure.

## Main Target During Alpha-0.17

The provider-neutral execution capsule contract, its Nuis/YIR ingress, concrete
Metal/CoreML invocation below the persistent worker, graph-scoped adapter image
cache, compact bounded adapter control record, hash-bound control carrier, and
registration-negotiated descriptor capacity now exist. The v3 handshake
separates 31 semantic input slots from one transport-control slot and a
registered eight-output budget; a provider-neutral regression proves
three-input fan-in and two-output fan-out. Ordered output bindings now reach the
provider request, worker lease, and session: all reply descriptors are retained
and verified, and every output role receives its own session handle and GLM
ownership token. The highest-value next boundary is publishing all of those
outputs into graph ownership rather than only the compatibility primary output.

The worker should consume:

* registered provider identity
* registered adapter identity
* registered operation token
* ordered input carrier roles
* declared output carrier roles
* YIR / GLM / clock metadata needed by the invocation

The worker should return:

* execution status from the registered capsule
* output carrier identity and integrity evidence
* backend-neutral lifecycle timing evidence
* enough trace metadata for Nsdb replay and inspection

Nsdb should remain the graph/session coordinator. It should not become the
owner of Metal, CoreML, CUDA, ROCm, Vulkan, network, or future hardware policy.

Short rule:

`registration chooses and authorizes the capsule; the Nuis worker owns the
generic process-adapter invocation; the Nustar owns backend meaning; Nsdb
verifies and records the returned descriptor`

## Native-Language Ownership

Until self-hosting, Rust remains the compiler/tooling implementation host.
Outside that bootstrap exception:

* lifecycle loops belong in Nuis
* validation and dispatch policy belong in Nuis/YIR contracts
* backend choice belongs in registration
* C, C++, and Objective-C remain thin generated or registered ABI adapters
* host ABI adapters must not own scheduling, memory policy, or Nustar dispatch

Platform SDK calls may still require host-language shims. That is an ABI fact,
not permission to move Nuis control flow into the shim.

## Pressure Order

Prefer work in this order:

1. execute a registered provider-neutral capsule inside the Nuis worker
2. return and verify output-carrier receipts from that worker boundary
3. connect worker execution evidence to Nsld final heterogeneous payload layout
4. keep PixelMagic and WitSage as real shader/kernel pressure tests
5. keep std sufficient for CLI, IO, filesystem, text, task, benchmark, and
   network programs without hiding host-only policy
6. stabilize package/import behavior needed by future compiler self-use
7. keep Nsdb replay/debug metadata aligned with YIR-level execution
8. update the development tensor after every completed closure step

## What Should Not Be Claimed Yet

`alpha-0.17.*` should not claim:

* final self-hosting
* final std API stability
* final production-grade Nsld replacement for every system linker path
* final unified heterogeneous executable format
* production-complete GPU/NPU portability
* a mature Ns Nova application framework
* beta-level compatibility guarantees

Safe wording:

* `registered heterogeneous worker boundary`
* `worker-authorized operation permit`
* `worker-invoked provider-neutral execution capsule`
* `NUISPWUR7 ordered multi-output result-carrier receipt`
* `ordered hash-bound worker process-adapter launch for Metal and large/chained/fan-in CoreML`
* `real Metal/CoreML provider smoke`
* `binary-linking convergence`
* `executable-artifact closure`
* `Nuis-owned lifecycle with thin host ABI adapters`
* `graph-scoped registration-bound provider adapter image cache`
* `one compiled CoreML adapter image reused by chained and fan-in requests`
* `nuis-provider-worker-adapter-control-v1 compact bounded control metadata`
* `oversized provider control rejected before sendmsg`
* `hash-bound control.adapter carrier excluded from capsule input counts`
* `NUISPWUH3 registration-negotiated input/control/output descriptor capacity`
* `provider-neutral three-input fan-in plus one control descriptor`
* `provider-neutral two-output fan-out with independent length/hash/mode evidence`

## Exit Evidence

Alpha-0.17 should leave behind:

* a versioned execution-capsule contract
* at least one registered capsule invoked by the persistent Nuis worker
* a verified output-carrier receipt returned across the worker protocol
* no compiler-side finite list of allowed backend combinations
* PixelMagic/WitSage evidence consuming the same capsule vocabulary
* one hash-verified CoreML model adapter executing beneath the Nuis worker
* one source/framework/target/contract-bound CoreML adapter image reused across
  all four official CoreML requests and removed at graph close
* Nsld and Nsdb metadata that can identify the executed capsule and outputs
* development-tensor evidence naming the next weakest verified cell

The line is successful if another backend can register the same contract
without rewriting compiler control flow.
