# Nsld Binary Assembly Gap Map

This note maps the current gap between the Nsld-owned deterministic container
pipeline and a truly runnable Nuis-owned heterogeneous executable.

## Current Assembly Chain

Nsld already has a deterministic preparation chain:

```text
link plan
  -> link inputs
  -> link units
  -> link bundle
  -> assemble plan
  -> section manifest
  -> object plan
  -> container plan
  -> container metadata
  -> container payload
```

`nsld prepare` can emit and verify this chain today. This is already useful
because it gives linker, cache, release, and debugger work one reproducible
artifact boundary.

## Current Artifact Meaning

The emitted `nuis.nsld.container` is a Nuis-owned binary-contract container,
not the final host executable.

It currently owns:

* deterministic section order
* section hashes and payload ranges
* loader-facing entry metadata
* loader symbol seeds
* relocation seeds
* external import records
* payload hash and container hash
* verification of metadata and payload consistency

It does not yet own:

* Mach-O, ELF, or PE object emission
* native relocation application
* final executable entrypoint generation
* Nuis lifecycle runtime bootstrapping
* heterogeneous dispatch at runtime

## Gap 1: Object Writer

Nsld needs a self-owned object writer layer.

Minimum first target:

* consume the section manifest and payload
* write a host object container for one platform
* preserve Nsld section identity in object metadata where possible
* keep Nsld hashes as source-of-truth verification metadata
* avoid special C-world shortcuts outside the external import contract

This layer should be deterministic and testable before it claims to replace the
host finalizer.

## Gap 2: Relocation Applier

The current relocation table is a loader-facing seed, not a native relocation
engine.

The next relocation layer needs:

* relocation kind registry
* section-relative offset model
* symbol resolution over loader symbols and hetero dispatch symbols
* deterministic unresolved-symbol diagnostics
* host ABI-specific lowering only behind the registered target profile

This should stay below the YIR contract and above the final platform object
format.

## Gap 3: Loader Bootstrap

The container records a lifecycle bootstrap symbol, but there is no finished
Nuis loader runtime yet.

The loader needs:

* one native entrypoint shim
* bootstrap of the Nuis lifecycle loop
* deterministic hook execution order
* access to container metadata and payload sections
* bridge points for clock protocol and GLM checks
* failure reporting that can be consumed by Nsdb later

This is where the container stops being just inspectable and starts becoming
runnable.

## Gap 4: Heterogeneous Dispatch Bridge

Shader, kernel, network, and future Nustar domains already contribute lowering
sidecars and link units. They still need a runtime dispatch bridge.

The bridge needs:

* per-domain dispatch table materialization
* backend target selection without hardcoding finite combinations
* capability-driven sidecar loading
* deterministic clock-edge handoff
* GLM/resource validation at the boundary
* fallback or host-proxy mode when real hardware backend is unavailable

The bridge should consume Nustar registrations, not compiler hardcoded domain
logic.

## Gap 5: Debug Metadata Section

Nsdb can inspect YIR metadata through the current manifest/link-plan route, but
the final executable needs a durable debug metadata section.

Minimum shape:

* YIR domain index
* clock edge table
* section-to-YIR mapping
* loader symbol map
* lowering sidecar references or embedded summaries
* GLM/debug state handles when ready

Native debuggers may still see the shell binary. Nsdb should own the Nuis
semantic view.

## Practical Next Milestone

The next useful milestone is not "replace the system linker immediately".

The current next milestone is now represented by:

```text
nsld prepare
  -> nsld object-plan
  -> nsld emit-object-plan
  -> nsld verify-object-plan
```

That gives the object writer a deterministic planning layer before bytes are
emitted. It mirrors the existing assemble/container path and keeps the project
moving without pretending the native executable story is finished. The current
`object-plan` is intentionally `plan-only`; native object bytes and relocation
application are still separate future layers.

The plan already assigns each Nsld section a writer-facing object section
record with a stable object section name, object section role, source section
id, source hash, and payload offset seed. The future byte writer should consume
that mapping instead of rediscovering object layout from the section manifest.

## Success Boundary

Nsld reaches the first real binary assembly milestone when:

* the object plan is derived from the verified container state
* every object section is traceable back to an Nsld section id
* object-plan hashes are stable
* unsupported native targets fail with structured diagnostics
* no domain-specific shortcut is hardcoded into the linker frontdoor

After that, byte emission and loader bootstrap can evolve against a stable
plan instead of a moving pile of ad hoc linker code.
