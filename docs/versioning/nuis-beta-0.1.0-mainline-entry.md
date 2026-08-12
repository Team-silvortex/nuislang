# `nuis` `beta-0.1.0` Mainline Entry

This file is the current short entry point for the second beta minor line.

`beta-0.1.0` keeps the early-beta foundation-hardening posture while making
host compatibility an explicit registered domain. It does not mean stable
public APIs, package compatibility, self-hosting, or production-complete
heterogeneous execution.

The recorded predecessor is:

* [nuis-beta-0.0.1-mainline-entry.md](nuis-beta-0.0.1-mainline-entry.md)

The self-hosting horizon remains:

```text
beta-0.0.* through beta-0.9.*
  -> foundation closure, bug fixing, performance, and repeatability
beta-0.10.*
  -> formal stage0-to-stage1 self-hosting migration begins
gamma-0.5.*
  -> target stage2-equivalent compiler ownership
```

Short rule:

`beta-0.1.0 turns C compatibility from an implicit CPU privilege into a
registered, whitelistable, lifecycle-visible Nuis domain`

## Canonical Reading Order

1. [../current-mainline-map.md](../current-mainline-map.md)
2. [../reference/nuis-development-tensor.md](../reference/nuis-development-tensor.md)
3. [../reference/cffi-von-neumann-domain-contract.md](../reference/cffi-von-neumann-domain-contract.md)
4. [../reference/nuis-native-artifact-workflow.md](../reference/nuis-native-artifact-workflow.md)
5. [../reference/nsld-linker-frontdoor.md](../reference/nsld-linker-frontdoor.md)
6. [../reference/nsld-binary-assembly-gap-map.md](../reference/nsld-binary-assembly-gap-map.md)
7. [../reference/nustar-multi-backend-artifact-contract.md](../reference/nustar-multi-backend-artifact-contract.md)
8. [../reference/nuis-binary-format-protocol.md](../reference/nuis-binary-format-protocol.md)
9. [../reference/nsdb-yir-debugger-frontdoor.md](../reference/nsdb-yir-debugger-frontdoor.md)
10. [../reference/std-mainline-layering-contract.md](../reference/std-mainline-layering-contract.md)
11. [nuis-beta-0.0.1-mainline-entry.md](nuis-beta-0.0.1-mainline-entry.md)

## Current Connected Spine

```text
nuis source / nuis.toml
  -> nuis frontdoor
  -> nuisc
  -> NIR
  -> YIR + GLM / clock / domain verification
  -> registered Nustar packages and backend artifacts
  -> Nsld container / closure / final-output planning
  -> NSB image or explicit host-assisted final-output boundary
  -> run-artifact / Nsdb trace, replay, and completion evidence
  -> development tensor handoff
```

The compiler knows the shape of registration contracts, not a finite list of
backend implementations. Domain-specific parsing, ABI policy, lowering,
artifacts, execution adapters, and verification evidence remain registered
capabilities.

## Beta-0.1.0 Boundary

This line establishes:

* `official.cffi` as a first-class registered Nustar
* `mod cffi` as the required Nuis source boundary for `extern` declarations
* exact signature and hash-whitelist ownership in the CFFI manifest
* a CFFI YIR registration surface and lifecycle-visible artifact unit
* an explicit bootstrap bridge from CFFI policy to registered CPU/LLVM host
  machine-code production
* continued rejection of implicit `mod cpu` extern escape hatches
* migrated checked-in std, example, project, and test FFI sources
* development-tensor drift coverage for the new current line

The current bridge intentionally loads both `official.cffi` and
`official.cpu`. CFFI owns source admission and ABI policy; CPU owns the current
host machine-code realization. This is a visible bootstrap dependency, not a
claim that CFFI and CPU are the same domain.

## Data Hardware Direction

Programmable DPU and IPU hardware is a credible physical backend family for
the Data Nustar. It should enter through provider registration rather than
redefine the domain:

```text
Data YIR / GLM / clock / movement contract
  -> registered DPU, IPU, RDMA, CPU-memory, or device-memory provider
  -> backend artifact and lifecycle adapter
```

The Data Nustar remains broader than any current DPU. Placement, movement,
ownership, residency, synchronization, and future data-fabric hardware must
share the same provider-neutral contract.

## Beta-0.1 Main Target

1. keep the CFFI source and whitelist boundary strict while widening pointer,
   string, and object support only through registered contracts
2. preserve the closed lifecycle-dispatch slices, then advance the
   tensor-selected foundation coordinate rather than hard-coding one subsystem;
   the current concurrency tranche is tracked at
   `standard-library/std/concurrency-task-thread-lock`
3. preserve GLM, clock, dependency, replay, and final-image evidence across
   heterogeneous provider graphs
4. keep Nsld and Nsdb provider-neutral while real backends mature
5. model DPU/IPU support as an open Data provider family without vendor
   coupling
6. harden package/import and std behavior under larger CLI and heterogeneous
   programs
7. update the development tensor after each implementation tranche

The `beta-0.1-foundation-hardening` recalibration keeps alpha and runtime-loader
`stable/100` cells as historical milestone slices. It adds required
fine-grained coordinates for concurrency, CFFI memory authority, Galaxy lock
resolution, and OS-native Nsld finalization, plus an optional early Data/DPU
direction coordinate. A terminal `all-cells-complete` card must not return
until those newly registered slices also close.

The first concurrency hardening tranche closes matching branch-local mutex-lock
and thread-spawn/join-result prefixes through select-before-consume lowering.
This advances `standard-library/std/concurrency-task-thread-lock` to
`active/76`: recursive selected-prefix lowering now emits one mutex
new/lock/unlock chain and one spawn/cancel/join-result chain, while the native
project executes both dynamic branches. The tensor consequently hands current
work to the lower `active/72` registered CFFI memory boundary.

The first CFFI memory-authority tranche then advances that boundary to
`active/77`. `official.cffi` now registers five real borrowed UTF-8 parameters
with two independently reviewable hashes: the exact ABI symbol-signature hash
and a `nuis-ffi-memory-v1` capability hash over kind, slot, length policy,
mutability, lifetime, and destructor authority. Compilation rejects a `String`
extern before lowering unless that exact borrowed capability exists; project
host-FFI metadata preserves it, and Nsld includes it in per-ABI and aggregate
link-plan validation.

The same protocol can validate an owned `ref Buffer` return only when it names
an exact registered `i64(ref_Buffer)` destructor and uses runtime-header length,
unique mutability, and owned lifetime. This is intentionally not execution
support yet: source pointer-return lowering still fails closed until the host
allocation can enter YIR/GLM as a length-bearing owner and reach exact-once
native destruction. Raw pointers and retained host borrows remain unopened.
With CFFI at `active/77`, deterministic tensor ordering returns the next task
card to the lower `active/76` concurrency coordinate instead of pinning work to
one subsystem by hand.

The first scheduler-mutex runtime tranche then advances concurrency to
`active/80`. `Mutex<i64>` now lowers through opaque monotonic scheduler handles
and generation-bound one-shot guard tokens. Cooperative worker IDs, acquire and
release fences, monotonically increasing release epochs, held-lock contention,
reacquisition, replay/forgery rejection, and lifecycle cleanup are covered by a
compiled runtime harness. Generated mutex YIR carries one shared ordered
contract for scheduler identity, visibility, linear guard authority, and the
`i64`-native typed-fallback policy; the CPU domain rejects drift, LLVM emits the
runtime ABI, and the existing Nuis branch demo still executes both native paths
with exits `81` and `89`.

This does not open raw shared handles: Nuis source retains consume-on-lock and
consume-on-unlock GLM authority, non-`i64` payloads remain staged, and no
OS-thread-parallel mutex claim is made. With concurrency above CFFI, the normal
bootstrap selector returns the current task card to
`host-compatibility/cffi/registered-pointer-string-object-boundary` at
`active/77`.

The first executable owned-return tranche then advances CFFI to `active/83`.
One exact-whitelisted `host_owned_buffer_make(i64) -> ref Buffer` now lowers
through a dedicated Res-producing NIR/YIR operation. Its
`nuis-ffi-owned-buffer-v1` metadata independently revalidates producer,
capability, and destructor hashes; LLVM recovers the runtime-header length,
traps on invalid pointer/length state, and retains the exact registered
destructor until `cpu.free`. The YIR gate requires one same-function free and
rejects branch, loop, return, async, and secondary-extern escape while the
owner is live. A native CLI smoke reads the payload and releases it successfully.

Raw pointers, arbitrary `ref T`, retained borrows, and branch/task-carried host
owners remain closed. With CFFI now above concurrency on the same `active`
status rank, deterministic tensor ordering hands the next task back to
`standard-library/std/concurrency-task-thread-lock` at `active/80`.

The first shared-mutex authority tranche then advances concurrency to
`active/85`. `mutex_share` consumes one ordinary `Mutex<i64>` into
`SharedMutex<i64>`; lane `0` and lane `1` may each issue one non-cloneable,
generation-bound `MutexPermit<i64>`. A freshly issued permit may cross one task
boundary as an opaque scalar, but the worker can only consume it into a linear
`MutexLease<i64>` and cannot observe the scheduler handle. Shared capability
YIR carries strict scheduler, visibility, permit/lease, fixed-lane, one-shot,
and payload-policy metadata through CPU validation and dedicated LLVM lowering.

The checked-in native project emits one share, two permit issues, and two task
invocations; both workers observe `17`, unlock exactly once, and the process
exits `34`. GLM rejects permit replay and post-unlock lease reads, while the C
runtime harness rejects duplicate lanes, out-of-range lanes, stale permits, and
replay. This remains a cooperative `i64` lane with fixed cardinality: mutable
lease updates, explicit close/revocation, generalized payloads, and OS-thread
parallel safety are still open. The tensor now hands the next foundation task
to `host-compatibility/cffi/registered-pointer-string-object-boundary` at
`active/83`.

The first conditional registered-owner tranche then advances CFFI to
`active/88`. Two direct `extern_call_owned_buffer` values may enter one
GLM-typed `take_owned_buffer_drop_other_v1` branch effect only when their ABI,
destructor symbol, and destructor signature hash are identical. CPU heap
verification treats both as owned buffers; LLVM releases the unselected owner
inside the chosen branch, merges pointer and runtime length together, and
retains the exact destructor for one final release. The checked-in example
executes both selection directions as native binaries.

This does not open a generic pointer escape: mixed heap/external values,
destructor drift, nested transfer, loops, function return, tasks, async work,
and secondary extern calls still fail closed. With CFFI now above concurrency,
the deterministic tensor selector returns the task card to
`standard-library/std/concurrency-task-thread-lock` at `active/85`.

The first explicit shared-mutex close tranche then advances concurrency to
`active/90`. `mutex_shared_close(shared) -> i64` consumes the shared authority
under GLM, carries `lifecycle=explicit-close-revoke-v1` through every shared
YIR node, and returns the number of pending permits revoked. The interpreter
and native scheduler both reject close while a lease is active, invalidate
same-generation permits, reject post-close use, and prevent reopening. Native
close performs a release fence before invalidating the mutex slot.

The checked-in task project now joins both permit workers, explicitly closes
the shared mutex, and still exits `34`; the runtime harness additionally proves
pending-permit revocation and repeated-close rejection. Mutable lease updates,
static cardinality beyond the current fixed pair, generalized payloads, and
OS-thread safety remain open. The tensor therefore hands the next task back to
`host-compatibility/cffi/registered-pointer-string-object-boundary` at
`active/88`.

The first registered-owner function-return tranche advances CFFI to
`usable/93`. A synchronous helper may now return one direct registered
`ref Buffer` producer through `nuis-ffi-owned-buffer-function-transfer-v1`.
The YIR function result is explicitly owned and retains the ABI, destructor,
and destructor-signature hash as static identity; LLVM carries only the
runtime `{ ptr, i64 }` payload. CPU heap verification consumes the producer,
reconstructs one caller-owned result, and requires one direct caller release.
Compiler regressions reject identity drift and missing cleanup, while the CLI
smoke builds, links, and runs two helper transfers with exit `0`.

This remains a narrow synchronous path. Source helper chains may normalize to
the one admitted boundary, but runtime helper-to-helper or recursive transfer,
tasks, async work, loops, retained borrows, raw pointers, and arbitrary `ref T`
remain closed. With CFFI promoted to usable, deterministic status-first tensor
selection returns to `standard-library/std/concurrency-task-thread-lock` at
`active/90`; the lower-progress Galaxy lock cell remains behind it because
`active` work ranks ahead of `usable` work.

The first shared-mutex mutation tranche advances concurrency to `active/95`.
`mutex_lease_replace(lease, replacement) -> i64` performs one GLM `Write`
through the live `MutexLease<i64>` without consuming or widening that
authority, returns the previous value, and publishes a release epoch before a
subsequent unlock publishes again. Every shared capability YIR node now carries
`mutation=lease-replace-release-epoch-v1`; CPU validation, interpreter state,
LLVM lowering, and the scheduler runtime consume the same contract.

The interpreter now keeps one central shared value rather than copying the
payload into each permit. The C runtime independently marks shared leases so a
normal mutex guard cannot enter lease mutation. The checked-in
`task_shared_mutex_replace_demo` issues both permits before mutation, replaces
`17` with `23` in the first worker, observes `23` in the second worker, contains
no deferred mutex lowering, and exits `65` as a native binary. Configurable
static permit cardinality, generalized payloads, branch-local shared-capability
lowering, and OS-thread parallel safety remain open.

The static shared-mutex cardinality tranche then advances concurrency to
`active/97`. `mutex_share(lock)` remains source-compatible and normalizes to
cardinality `2`; `mutex_share(lock, N)` accepts a literal `N` in `1..=64`.
`mutex_permit(shared, lane)` accepts a literal lane in `0..=63`, while the CPU
interpreter and native scheduler enforce the exact relation `lane < N` from the
cardinality carried by the share node's second YIR dependency. Every shared
capability node now identifies that protocol with
`permit_cardinality=share-literal-1-to-64-v1`, replacing the fixed-pair marker
without exposing the scheduler handle.

The checked-in `task_shared_mutex_cardinality_demo` declares three permits,
sends lanes `0`, `1`, and `2` through three native task invocations, emits no
deferred mutex lowering, and exits `33`. Interpreter and C-runtime regressions
also reject lane `3` for that shared mutex, while preserving generation-bound
one-shot tokens, lease replacement visibility, and explicit close/revocation.
Generalized payloads, branch-local shared-capability lowering, runtime-dynamic
cardinality, and OS-thread parallel safety remain open.

The branch-selected shared-capability tranche advances concurrency to
`active/99`. Matching `mutex_share`, `mutex_permit`, `mutex_permit_lock`,
`mutex_lease_replace`, `mutex_lease_value`, and `mutex_lease_unlock` prefixes
now select their inputs before emitting one shared-capability YIR node. Share
branches must retain one identical static cardinality literal, and permit
branches must retain one identical static lane literal; drift fails before YIR
instead of turning those contracts into runtime-dynamic values. Every emitted
node reuses the canonical shared-mutex metadata array.

`task_shared_mutex_branch_demo` selects initial values `11`/`19` and replacement
values `13`/`23`, while YIR and LLVM each contain exactly one operation for the
entire share/permit/lease chain. Its two native paths exit `25` and `43` with no
deferred mutex lowering. Shared payloads remain `i64`, and runtime-dynamic
cardinality plus OS-thread parallel safety remain open.

The first non-i64 shared-mutex tranche keeps concurrency at `active/99` while
closing the requested scalar protocol slice. Canonical mutex metadata now says
`payload_policy=scalar-i32-i64-native-staged-fallback-v1`. YIR function
parameters retain `MutexPermit<T>` identity, LLVM carries an opaque permit token
plus its scalar kind, task contexts transport only token bits, and helper entry
reconstructs the typed permit before lock acquisition.

The scheduler slot records and validates an explicit scalar kind. Signed i32
payloads are sign-extended into the native slot and truncated back to i32 on
read or replacement; existing i64 entry points remain compatibility wrappers.
`task_shared_mutex_i32_demo` sends two `MutexPermit<i32>` values through native
tasks, replaces `17` with `23`, observes `23`, emits no deferred mutex lowering,
and exits `63`. OS-thread parallel slot safety, runtime-dynamic cardinality, and
bool/float native payloads remain open.

The scheduler slot-admission tranche closes concurrency at `stable/100` for
the current beta foundation scope. A scheduler-private C11 atomic gate now
serializes every mutex, guard, and permit lookup, allocation, mutation,
observer, and reset operation. Nested permit-lock and lease-unlock routes call
already-admitted internal operations, so this protection does not introduce a
recursive gate deadlock or expose a host lock through source, YIR, or LLVM ABI.

A dedicated harness starts 32 real pthread workers at one barrier. Every worker
allocates a unique mutex, shares one permit, consumes it into a simultaneously
live lease, verifies its payload, and then crosses a second barrier before all
workers unlock and close concurrently. The harness observes 32 live handles,
32 unique identities, exact successful-unlock counts, and zero live residue.
This proves scheduler-table host-thread admission safety, not a mature parallel
Nuis executor, per-mutex parking/fairness, runtime-dynamic cardinality, or the
final memory-visibility model.

The first Galaxy reproducibility tranche advances
`package-system/galaxy/source-import-and-lock-resolution` to `usable/84`.
`nuisc` now owns `nuis-galaxy-resolution-lock-v1`; every project build emits a
portable `nuis.project.galaxy.lock` whose canonical payload binds direct and
transitive dependency edges, package identity, import policy, actual library
selection, and SHA-256 records for each Galaxy manifest, source module, and
library module. No physical workspace path enters the snapshot.

Manifest content identity is captured from the same resolver read, and each
library source must still match its resolved byte count and SHA-256 before AST
parsing. Lock emission uses that frozen identity rather than rereading mutable
workspace files after compilation.

The build manifest independently records the resolution digest, and its normal
verifier rejects payload mutation or lock/manifest digest drift. That first
tranche closed the generated-build evidence slice, not the package-manager
admission slice: at that point the root `nuis.galaxy.lock` still used the older
direct-bundle format and needed migration before `lock-deps`, `sync-deps`,
build admission, and `project-status` could share one fail-closed resolution
authority. Tensor selection therefore moved to the lower `usable/82` Nsld
OS-native finalization cell.

The first OS-native finalizer registration tranche advances
`linker-toolchain/nsld/os-native-executable-finalization` to `usable/85`.
`nuis-nsld-executable-finalizer-registry-v1` now canonicalizes target identity,
hashes the static provider set, rejects missing or ambiguous selection, and
persists provider identity through dry-run JSON and the verified invoke plan.
The Mach-O arm64 host-command shell writer owns command planning and actual
process invocation behind its provider callback; the generic emit path no
longer spawns a platform tool directly, and host execution is pinned to the
driver path verified at dry-run instead of repeating `PATH` lookup. ELF and
PE/COFF are explicit `registered-not-implemented` providers rather than hidden
fallbacks.

This does not claim a pure Nsld Mach-O writer: native relocation application
and executable-shell byte emission still depend on future work, and the ready
Mach-O provider still uses a registered host tool. With Nsld at `usable/85`,
the weakest bootstrap coordinate returned to the Galaxy lock/admission slice
at `usable/84`.

The follow-up root-lock tranche advances that Galaxy coordinate to
`usable/93`. `lock-deps`, `verify-lock`, `sync-deps`, project-status, and build
admission now consume the compiler-owned resolution protocol. Root locks no
longer contain absolute bundle paths; direct and transitive package identity,
edges, selection, and content SHA-256 records form the authority. Sync
re-verifies every frozen file into a staged tree and transactionally replaces
the old materialization, while an existing mismatched lock rejects build
before cache restore or output creation. The build-side snapshot is
byte-identical to the committed lock.

This is not yet the final package resolver. A missing root lock remains legal
for early-beta development examples, and compilation still reads the workspace
closure before comparing it with the lock rather than resolving directly from
a content-addressed synchronized cache. Required-lock release admission and
cache-owned resolution are the next bootstrap package milestones. The weakest
bootstrap coordinate therefore returns to Nsld OS-native finalization at
`usable/85`.

The OS-native materialization follow-up advances that coordinate to
`usable/90`. For `native-cpu-llvm` on Mach-O arm64, the registry now selects
`nsld.finalizer.mach-o.arm64.artifact-image-v1`. This internal provider parses
the compiled artifact, verifies its target and ABI identity, validates a thin
or universal arm64 `MH_EXECUTE` image, and atomically installs executable bytes
without host policy variables or a second clang invocation. The registered
host-command provider remains an explicitly gated compatibility fallback, and
ELF plus PE/COFF remain visible `registered-not-implemented` routes.

This is a real runnable executable materialization boundary, not yet pure Nsld
linking. Nuisc still asks the host toolchain for the prelinked image embedded in
the compiled artifact. The next tranche must hand relocatable host/runtime
objects to Nsld and move Mach-O load-command, symbol, and relocation ownership
behind the same provider boundary.

## Honesty Boundary

`beta-0.1.0` should not claim:

* stable public API or package-format compatibility
* final self-hosting
* a native DPU/IPU backend already exists in the repository
* complete ELF, PE/COFF, and Mach-O parity
* production-complete GPU/NPU portability
* replacement of every host linker, debugger, runtime, or device toolchain
* mature Ns Nova framework readiness

Safe wording:

* `early beta foundation hardening`
* `registered CFFI source and ABI boundary`
* `registered heterogeneous provider closure`
* `Nsld/NSB image closure`
* `Nsdb trace/replay evidence`
* `host-assisted final-output boundary`
* `provider-neutral Data hardware direction`
* `self-hosting migration begins at beta-0.10.*`

## Version Surface Rule

`beta-0.1.0` is the project release line. Cargo package versions may remain on
their internal workspace versions until the repository adopts one explicit
package-version synchronization policy. Documentation must not infer the
project phase from an individual crate manifest.
