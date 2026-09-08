# Nuis-Rc Heap/Stack Cache Lifecycle

## Status And Responsibility

`nuis-rc-cache-lifecycle-v1` establishes the hybrid development-resource model:
unmarked immutable packages use a shared **heap**; explicitly project-bound
packages use a private **stack**. These are ownership and lifetime analogies,
not process-memory allocation, a LIFO requirement, or the language's runtime GC.

Nuis-rc is the resident development-resource controller, not Yalivia, the YIR
runtime scheduler, or another dependency resolver. The historical whitepaper
already assigns it local project/toolchain indexing in
[section 15.1](../historical/nuislang-whitepaper-v0.44b.md).
This contract extends that role to managed dependency lifetimes.

The broader [software manufacturing architecture](nuis-software-manufacturing-architecture.md)
positions RC as the machine-local compilation control plane. It separates
discovery, identity, integrity and trust, and covers source, YIR, backend
artifacts and build recipes rather than treating RC as only a package directory.

Implemented now: a platform-neutral Rust policy library in
`tools/nuis-rc/src/cache_lifecycle.rs`, with no filesystem, process, or network
access. It selects heap/stack lifetime and computes deterministic GC candidates
from a complete caller-verified object/root snapshot. Tests cover reachability,
cycles, private ownership, grace periods, invalid snapshots, and quota pressure.

Not implemented yet: manifest markers, shared-store publication, durable root
and lease registration, compiler integration, GC execution, or a real daemon.
The existing `nuis-rc start` still initializes prototype state and indexes only.
There is no new cleanup command, background service, or automatic migration.
Current project-local `sync-deps` and compilation remain unchanged under the
[Galaxy resolution-lock contract](galaxy-resolution-lock-contract.md).

## Ownership Model

| Kind | Owner | Content And Lifetime |
| --- | --- | --- |
| Heap package | Nuis-rc shared store | Immutable verified content, shared by registered projects and retained while reachable. |
| Stack package | One project | Private logical instance; project commands own creation, replacement, and removal. Permitted immutable bytes may still be physically shared. RC must not sweep the private binding. |
| Build output or derived cache | Explicit project/cache policy | Immutable snapshots can use CAS with exact action/target contracts; writable outputs are not shared just because their input package is heap-owned. |

The planned declaration vocabulary is `lifetime = "heap"` or
`lifetime = "stack"`; it is **not accepted manifest syntax yet**. Omission selects
heap. A package that requires project scope, including mutable path/checkouts,
must select stack; a consumer cannot override that requirement with heap.
A consumer may request a private stack instance of otherwise shareable content.
Changing storage must never silently change the resolved version or source.

Scope belongs to a package **instance**, not merely its name/version. Independent
heap and stack bindings can refer to the same physical immutable blob inside an
authorized sharing domain. Stack ownership does not require byte duplication.
Ending one binding must preserve bytes still held by another binding or lease;
confidential/no-share policy and mutable working copies remain separate constraints.

The current policy key combines content SHA-256 with heap ownership or a
path-independent project ID: it is a logical key, not a deduplicated blob key.
The current planner counts bytes per logical object, not measured physical disk
use or savings. A future physical inventory must trace all bindings/leases before
unlinking a shared blob. Verified publication must bind the complete manifest
and file inventory; content integrity does not replace per-consumer source trust.

Stack packages may depend on heap packages, which remain rooted by the project.
This does not force an entire dependency closure into private copies. Heap
objects cannot reference private stack objects; private objects cannot reference
another project's stack. A project-bound edge requires a project-bound instance,
not a global object whose meaning changes with its consumer.

## Managed Roots And Leases

The intended default is one controller/store **per OS user/security boundary**,
covering that user's registered projects on the machine. It does not require
administrator privilege, scan every repository, or trust another user's writes.
Machine-wide or multi-user pooling needs an independent authenticated adapter.

Projects enter the registry through explicit tracking or a Nuis project command
handshake, not by finding a README somewhere on disk. Durable roots bind the
project ID, verified resolution digest, and exact transitive object closure.
Projects moving on disk update local locators without changing package identity.
Machine paths, leases, access times, quotas, and daemon generation do not enter
portable resolution locks or reproducible artifact identity.

Roots include registered project locks, active build/read leases, explicit pins,
and retained toolchain versions. Stack instances and their dependency closures
are protected even when not eligible for heap accounting. A build must acquire
its lease before exposing/reading shared paths and release it after all readers
finish. Acquire, root update, and GC admission must share a serialization barrier.
Removing a project root cannot collect dependencies still used by another root.

A disappeared directory, unmounted volume, unreadable lock, missed heartbeat,
expired timer, or reused PID is not proof of unreachability. Preserve known roots
and fail closed on an incomplete snapshot. Crash recovery must reconcile the
durable journal and prove reader termination, or retain its lease. Grace time
does not substitute for that proof. Explicit untracking/pin removal can release
roots; recovery must not infer the same permission from an I/O failure.

## GC And Disk Budgets

Use tracing/mark-and-sweep over exact dependency identities, not reference
counts alone. Unreachable cycles can be reclaimed; all root-reachable objects
remain protected. The current planner is iterative and preserves deterministic
candidate order independently of registry enumeration order.

`unreachable_since_seconds` means the first continuously unreachable observation
in the reconciled registry, not last access or file modification time. Unknown
history retains content. Reattachment resets the history before a later detach
can start another grace period. A grace-retained object also protects all its
dependencies. Future timestamps/clock rollback reject the snapshot.

The planner returns the full eligible heap set, byte totals, snapshot generation,
and `over_budget_bytes` after hypothetical collection. It never performs I/O.
If protected content alone exceeds the budget, report pressure and limit new
cache admission; do not pretend that every size target is achievable by GC.
Zero budget does not authorize deleting a live dependency. GC does not remove
source edits, final deliverables, the only unrecoverable package copy, or another
tool's cache; non-refetchable content needs a durable project root or explicit pin.

A future daemon can adjust scan intervals, admission, and bounded GC batches
using byte/inode budgets and disk headroom. A grace period avoids churn; stale
derived artifacts can receive separate retention policies. Hot/warm/cold heat
and measured reconstruction cost rank safe choices, not deletion authority.
Reclaimability is a separate safety axis. The manufacturing architecture also
distinguishes residency eviction, which preserves an exact recovery recipe,
from unreachable-object GC. Neither physical deduplication nor residency eviction
is implemented by this planner. Project-private usage remains reportable even
though heap GC cannot end its logical lifetime.

Before applying a plan, storage must atomically revalidate its generation and
root/lease closure under the barrier, mark the complete collectible cohort as
retiring, and prevent new leases from observing partially deleted graphs.
Use a same-store quarantine/publication journal and restart recovery rather than
unlinking arbitrary paths from a stale plan. A cancelled/incomplete batch must
not leave a supposedly usable object with a missing dependency. Planning alone
is not deletion authority or evidence that these concurrency properties work.

## Isolation And Cross-Platform Boundaries

The shared store publishes byte-verified immutable objects through bounded
staging and atomic admission. Never let a project's writes mutate another
project's dependency by sharing a writable directory or writable hardlink.
Reject traversal, symlink/reparse escape, and provider-controlled GC paths.
Registry authority, trust state, content, and local locator records have separate
roles. Same-user malware or administrator compromise is not solved by hashes.

Nuisc consumes verified content and the existing resolver contract; it must not
implement GC or depend on an always-running daemon for language semantics.
An offline build needs either the same lease/locking guarantees or a verified
private snapshot. Daemon absence is not permission to read an unprotected heap
while another controller might collect it. Built applications do not require RC.

The service lifecycle, IPC, directory discovery, filesystem locks and atomic
publication belong to replaceable OS adapters. No launchd/systemd/Windows service
details, GPU backend names, or absolute machine paths enter the policy library.
Unify `NUIS_HOME` and platform home discovery before enabling a shared store;
the existing RC prototype and Galaxy helpers currently differ here.

## Staged Acceptance

1. **Current:** pure lifetime policy and read-only GC candidate planning, with
   explicit errors for incomplete graphs, invalid scope edges, and byte overflow.
2. Register project identity, logical bindings, physical content inventory,
   residency, exact build recipes, roots and pins transactionally; expose
   logical/physical inspect and dry-run reports before allowing deletion.
3. Add manifest markers and verified shared-store publication; integrate resolver
   leases without changing portable lock identity. Migrate only after verification,
   preserving old project snapshots until the new path is proven usable.
4. Exercise concurrent build/sync/GC, project moves, missing volumes, process
   crashes, corrupt journals, poisoned paths, and interrupted publication.
5. Add authenticated local service adapters and bounded automatic collection;
   demonstrate two projects sharing one heap copy and independent stack cleanup
   on macOS, Linux, and Windows before claiming cross-platform service closure.

See the `package-system/nuis-rc/heap-stack-cache-lifecycle` coordinate in the
[development tensor](nuis-development-tensor.md). This work does not replace
the ns-nova application-led mainline or claim finished shared caching.
