# Nuis Packaging / Lifecycle Responsibility Map

This file is a short implementation-facing responsibility map that ties
together the current future-edge sketches for:

* launcher shell
* `nuis` runtime container
* `nuis` linker
* `nustar` package grain and ABI targets
* AOT lifecycle phases

The goal is to answer a practical question:

`which layer is supposed to own which kind of decision?`

## Shortest Rule

`host launches, linker assembles, container carries, lifecycle runs`

## Responsibility Table

| Layer | Owns | Should Not Own |
| --- | --- | --- |
| Host launcher shell | process entry, host ABI satisfaction, first jump into `nuis` bootstrap | deep packet/task/network semantics, heterogeneous composition policy |
| `nuis` linker | package resolution, `abi_target` selection, capability assembly, container freezing | host kernel startup rules, long-lived runtime scheduling details |
| `nuis` runtime container | packaged capability segments, domain metadata, packet/bridge indexes, internal program payload structure | direct host process boot semantics |
| `nuis` bootstrap | validating and activating the container, binding bridges, choosing initial lifecycle state | permanent ownership of every steady-state runtime policy |
| `nuis` lifecycle loop | task/session progression, host I/O/network/shader/kernel coordination, shutdown preparation | host-native executable packaging |
| `nustar` registration | stable capability contract, completeness checks, standards legality, registered target declarations | being the only legal frontend spelling |

## Phase-To-Layer Map

### 1. Build / Link Time

Primary owner:

* `nuis` linker

Responsibilities:

* resolve project requirements into capability families
* choose compatible `nustar` packages
* validate registration completeness
* validate standards legality
* select or normalize `abi_targets`
* group heterogeneous payload segments
* write the internal `nuis` container layout
* request a thin host launcher shell

Should not leak upward into source frontend unnecessarily.

### 2. Host Launch Time

Primary owner:

* host launcher shell

Responsibilities:

* satisfy OS-native process entry requirements
* enter the program
* locate or embed the `nuis` container payload
* call the `nuis` bootstrap entry

This layer should stay as thin as possible.

### 3. Bootstrap Time

Primary owner:

* `nuis` bootstrap

Responsibilities:

* validate the linked container payload
* discover packaged capability segments
* initialize host bridge surfaces
* bind scheduler defaults
* create the initial lifecycle state
* choose the first unit/session/task entry

This is where the program stops being “host-launched” and starts being
“`nuis`-organized”.

### 4. Steady-State Execution

Primary owner:

* `nuis` lifecycle loop

Responsibilities:

* progress task state
* progress result observation
* progress packet movement boundaries
* progress host I/O and network bridges
* coordinate shader/kernel submission
* maintain active / waiting / draining lifecycle states

This is the layer that most directly benefits from keeping frontend mostly
family-unified while pushing ABI variation deeper.

### 5. Shutdown

Primary owner:

* `nuis` lifecycle loop + shutdown stage

Responsibilities:

* flush final summaries
* tear down bridges in a valid order
* release owned resources
* map internal outcome to host exit code / host-visible termination state

## Frontend Rule

Primary owner:

* frontend conventions

Responsibilities:

* present a family-unified user-facing programming model
* express intrinsic intent with compiler/std-owned annotations where useful
* avoid early source forks that exist only because host packaging differs

Should not be treated as the strongest semantic truth.

That stronger truth lives in:

* registered package contracts
* resolved target contracts
* linker/container/lifecycle boundaries

## Nustar Rule

`nustar` should be read through three distinct responsibilities:

### Package Grain

Owns:

* capability-family grouping
* frontend family continuity

Should not imply:

* one package equals one OS x ISA artifact

### ABI Target Grain

Owns:

* `arch`
* `os`
* `object`
* `calling`
* `clang`
* optional backend specialization

Should not imply:

* source frontend must split at the same grain

### Artifact Grain

Owns:

* one concrete built implementation artifact

Should not imply:

* one logical capability family must fragment into many tiny logical packages

## Conditional Compilation Rule

Conditional compilation belongs mostly at the capability and bridge boundary.

Good fit:

* true capability differences
* backend-specific hooks
* bridge-specific startup or shutdown logic

Poor fit:

* source-level fragmentation that merely mirrors packaging detail

So when in doubt:

* keep family frontend unified
* let `abi_targets`, linker selection, and lifecycle wiring carry more of the
  platform specificity

## Annotation Rule

Annotations should help the compiler and linker describe:

* host boundary facts
* packet/serialization facts
* lifecycle hints
* bridge requirements
* packaging intent

They should not become the only semantic truth.

The responsibility split is:

* annotations express official frontend convention
* `nustar` registration expresses stable capability contract
* linker/container/lifecycle realize that contract

## Future JIT Reserved Cut

If a future managed service or JIT-adjacent runtime is ever introduced, it
should be inserted at lifecycle-owned cut points rather than changing the
launcher shell or exploding package grain.

That means future dynamic services belong conceptually near:

* bootstrap-time service registration
* lifecycle-safe managed execution delegation
* shutdown-time cleanup

and not near:

* raw host entry
* ad hoc target-specific source forks

## Practical Decision Checklist

When a new responsibility appears, ask:

1. Is this just host startup?
   If yes, it belongs in the launcher shell.
2. Is this capability assembly or target selection?
   If yes, it belongs in the `nuis` linker / `nustar` contract layer.
3. Is this internal payload organization?
   If yes, it belongs in the `nuis` container.
4. Is this runtime progression or teardown?
   If yes, it belongs in the lifecycle loop.
5. Is this only a user-facing way to express intent?
   If yes, it belongs in frontend conventions or intrinsic annotations, not in
   the deepest semantic truth.

## One-Line Summary

`keep frontend families broad, keep target contracts precise, keep lifecycle ownership inside nuis`
