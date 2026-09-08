# Nuis Software Manufacturing Architecture

## Standing And Current Scope

`nuis-software-manufacturing-model-v1` records an architectural direction, not
an implemented daemon, a new wire protocol, or a compatibility freeze.

**The OS hosts software execution; Nuis governs software manufacture.** The
toolchain is a software production system with its own resource, dependency,
build, cache, verification, and delivery contracts, not merely a collection of
commands whose semantics are defined by one host OS or package registry.

These are peer responsibility domains, not equal kernel privileges. The current
implementation still uses host processes, filesystems, security boundaries,
network services and device APIs. Independence means replaceable host adapters
and Nuis-owned production semantics, not bypassing host permissions or claiming
that Nuis already supplies an operating system. The eventual
[Nuis OS direction](../versioning/nuis-long-range-heterogeneous-os-roadmap.md)
does not change these boundaries.

Nuis-rc is the **machine-local package heap and compilation control plane**.
Projects are separately owned production jobs within that system, not isolated
cache islands. Machine-local scope covers registered Nuis projects and resources;
it does not authorize scanning/deleting unrelated repositories. The first trust
boundary remains one OS user; authenticated multi-user pooling is a later adapter.

Only the [heap/stack lifecycle policy](nuis-rc-cache-lifecycle.md) exists as a
read-only library today. Durable CAS storage, daemon operation, leases, remote
acquisition, physical deduplication and adaptive eviction are not implemented.
The existing [resolution-lock contract](galaxy-resolution-lock-contract.md)
continues to govern project-local locked compilation.

## Responsibility Map

| Owner | Production Responsibility |
| --- | --- |
| Project / Nuis front door | Declare inputs, policy and private lifetimes; submit and observe production jobs. |
| Nuis-rc | Coordinate inventory, logical roots, leases, storage budgets, materialization and collection through registered capabilities. |
| Galaxy resolver and trust policy | Select exact dependency closures, evaluate source authorization, and preserve portable lock identity. |
| Nuisc / Nustar | Compile and lower through shared YIR contracts and registered backend capabilities. |
| Verifiers | Check the relevant source, YIR, GLM, artifact and lineage contracts; future Vulpoya review remains independently owned. |
| Nsld / Nsbdr | Assemble core binary artifacts, then adapt verified outputs to host packaging and distribution formats. |
| OS / runtime hosts | Consume delivered software and provide execution services; Yalivia remains a separate future runtime collaborator. |

RC coordinates existing authorities instead of inventing a second resolver,
compiler, verifier, linker or application scheduler. A daemon is one service
adapter over versioned reusable capabilities, not the only public interface.
The same [core Galaxy boundary](toolchain-galaxy-core-boundary.md) must support
CLI, CI, future IDE, and self-hosted consumers without parsing terminal text.
Delivered applications do not require the development controller.

## Decentralized Acquisition

**Discovery != Identity != Integrity != Trust.**

| Concern | Question | Must Not Imply |
| --- | --- | --- |
| Discovery | Which candidate or location might supply this dependency? | An index, search result, mirror or LAN peer is authoritative. |
| Identity | Which exact material or recipe did the consumer select? | A mutable name, branch, URL or version label uniquely fixes bytes. |
| Integrity | Do the retrieved bytes match the pinned digest and canonical inventory? | Correctly hashed code is safe or its publisher is authorized. |
| Trust | Is this origin/signer/provenance permitted for this consumer and purpose? | Successful transport or prior use by another project grants authorization. |

Sources can include GitHub, GitLab, self-hosted Git, HTTP, configured LAN peers,
Team Silvortex indexes, enterprise indexes, or third-party indexes. These are
extensible discovery/acquisition providers, not a closed set of special cases.
A central index can be useful without being a prerequisite for the ecosystem.

Multiple indexes compose under explicit namespace/source and trust policies.
An unavailable private source must not silently become a same-name public
package, nor may provider order silently select between conflicting authorized
candidates. Reuse the existing bounded resolver and pinned closure; reject
ambiguity or request an explicit policy update rather than changing authority.

Acquisition may prefer local verified CAS, opted-in LAN peers, configured mirrors,
then origin. This is a configurable retrieval order, not a version-selection or
trust order. Every route must satisfy the same selected identity and current
consumer policy. A mirror change does not change content identity; a source
authorization change is not merely a mirror change. Mutable Git references must
resolve to an immutable revision plus verified content inventory before locking.

An index is a discovery service. It cannot replace the expected digest alongside
the payload to make a mismatch disappear. Trust checks still apply on cache hits
and locally shared objects, not only on downloads. Credentials and machine-local
access policy remain outside portable locks and shared content metadata.
Downloaded material does not acquire permission to execute build hooks or write
into the store; execution requires separate job/capability admission.

## Logical Instances And Physical CAS

Keep four records distinct:

* **Immutable blob:** content-addressed bytes within one authorized sharing
  domain, independent of which project currently holds them.
* **Object descriptor:** kind/schema, canonical file/tree inventory, dependency
  identities and interpretation contracts. Source snapshots, YIR, PTX, Metal,
  SPIR-V and other registered artifacts use the same storage substrate without
  erasing their different verifier and backend requirements.
* **Logical binding:** a project or public-heap instance, its ownership and
  retention obligations, referencing verified descriptors/blobs.
* **Build action:** a separately keyed recipe binding exact inputs, compiler and
  Nustar identities, options, target/ABI and all declared environment/tool inputs.
  It maps to verified output identities; an output content hash is not an action key.

Same source bytes do not imply reusable target code. Undeclared inputs, host
paths, secrets, device/SDK constraints, or nondeterministic actions must prevent
unjustified shared cache hits. Backend-specific validity remains with registered
Nustar contracts, not hard-coded RC knowledge of backend combinations. Verification
receipts must bind the object, contract and verifier identities they actually
checked; a cached success is not blanket validity under another verifier/policy.

Heap and stack specify **logical ownership**, not mandatory physical layouts.
A project can destroy its stack binding while another project retains an identical
read-only blob. Heap bindings may do the same. Physical collection follows every
remaining binding and lease, so ending one lifetime must not delete another's data.
Stack isolation does not require duplicate immutable bytes within the same
authorized sharing domain. Confidential or explicitly non-shareable objects still
need separate storage domains; identical hashes do not override that restriction.

Mutable working trees, writable outputs and project-private modifications are
not shared writable CAS objects. Snapshot them into verified immutable material
when authorized, using private writable copies/overlays for mutation. Never turn
physical deduplication into writable aliasing between projects.

## Retention, Residency And Collection

`hot`, `warm` and `cold` describe usage temperature. **Reclaimability is a
separate safety axis**, not simply the state after cold. Pins, active leases,
private lifetime ownership, reconstruction policy and uncertainty all constrain
what storage may retire regardless of heat or disk pressure.

Logical dependency reachability and physical residency also differ. A pinned
lock may remain valid while an explicitly reproducible/refetchable cache
materialization is absent. A future eviction operation can remove those resident
bytes only when no active reader, offline-retention requirement or private-owner
constraint forbids it. Preserve the exact descriptor and recovery recipe; reacquire
and reverify that same identity before use, or fail without substituting a version.
An origin's advertised availability is not an offline/recovery guarantee.

**Eviction** changes permitted residency; **GC** removes unreachable storage.
Neither operation is package update, source deletion, trust-policy relaxation or
destruction of the only required unrecoverable copy. Dynamic pressure and measured
rebuild/refetch cost rank safe choices; they do not manufacture deletion authority.
If protected material exceeds capacity, apply admission backpressure and report
the shortfall. A live daemon alone cannot guarantee bounded storage without
budgets, bounded journals, staging limits and explicit retention policy.

Leases prevent ordinary job exit from leaving permanent logical references, but
timer expiry is not proof that all readers stopped. Renewal/session generations,
fencing or confirmed owner termination, durable recovery and atomic root/lease/GC
admission must prevent resumed or paused readers from using collected content.
Uncertain ownership retains protection. Stale durable roots need an explicit
reconciliation policy, not an unbounded accumulation disguised as safety.

The current planner protects every reachable instance and accounts bytes per
logical object. It does not model physical blob deduplication, residency eviction,
cost/temperature ranking or live lease recovery. Those require a distinct physical
inventory and adapter acceptance tests; current byte totals are not disk savings.

## Acceptance Direction

1. Extend the durable inventory design to separate instance ownership, immutable
   blob identity, residency, exact action recipes and provenance/trust facts.
2. Demonstrate two indexes/mirrors providing identical content under independent
   consumer policies, rejecting dependency confusion and unauthorized cache hits.
3. Demonstrate separate stack/heap bindings sharing permitted immutable bytes,
   with no premature deletion after one binding or build lease is released.
4. Verify interrupted acquisition/publication, paused readers, daemon restart,
   corrupt metadata, missing volumes and concurrent sync/build/GC fail safely.
5. Add measured hot/warm/cold retention and cost-aware bounded eviction, proving
   exact rematerialization and honest logical versus physical byte accounting.

These are later acceptance gates, not evidence supplied by the current policy
tests. The [development tensor](nuis-development-tensor.md) records this direction
without raising implementation maturity or replacing the ns-nova mainline.

Short rule: **external acquisition may be decentralized; local Nuis production
must be verified, ordered and bounded.**
