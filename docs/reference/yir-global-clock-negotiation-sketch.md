# YIR Global Clock Negotiation Sketch

This document is a forward-looking sketch for another future `YIR` concern:

* if `nuis` keeps default async/hetero semantics visible
* and if multiple hardware or `nustar` domains maintain their own local clocks
* then `YIR` will eventually need a reliable global clock and time-negotiation
  contract

This is not a current implementation claim.

## Why This Matters

The repository already has three partial timing layers:

* compiler-known host clock reads
* task timeout / observation behavior
* explicit clock-domain bridge naming

Those are already useful.
But they are not yet the same thing as a final cross-domain timing protocol.

If the repository later wants:

* reliable task timeout reasoning
* reliable cross-domain ordering
* safe hot sync contraction
* real multi-hardware orchestration

then timing cannot remain just a host convenience.

## Core Direction

The likely direction is:

* **one global reference clock contract**
* **many local hardware/domain clocks**
* **explicit bridge/negotiation rules between them**

This does **not** mean “one universal physical clock.”

It means:

* one global comparison/reference layer
* plus explicit local-domain timing metadata
* plus explicit conversion assumptions

## Likely Layers

### 1. Local Clock

Each `nustar` or hardware-facing domain may eventually need to state:

* its local clock identity
* epoch kind
* scale / resolution
* drift / tolerance assumptions
* preferred bridge direction

The repository already has the beginning of this shape in `nustar` metadata and
clock-domain docs.

### 2. Global Reference

The global clock is best thought of as:

* a comparison/reference contract
* not necessarily the only clock that “really exists”

It gives the compiler/runtime a way to say:

* how two local timestamps should be compared
* how timeout and completion reasoning should remain coherent
* how cross-domain traces can share one temporal frame of reference

### 3. Bridge / Negotiation Contract

This layer is the key missing piece.

It would eventually need to carry things like:

* declared source domain
* declared target/reference domain
* bridge kind
* epoch conversion rule
* scale conversion rule
* drift / tolerance note
* resolution loss note
* whether the bridge is exact, staged, or approximate

Today the repository already has the beginning of this style in names like:

* `global_to_monotonic_tick_bridge`

But that is still a small front-door/compiler naming surface, not yet a full
multi-domain protocol.

## Relationship To Nustars

This sketch especially matters because the repository already wants each
`nustar` to own its own local runtime semantics.

That means future clock work should probably avoid:

* pretending all domains share the same native clock

and instead prefer:

* per-domain local clock ownership
* plus explicit conversion/negotiation

This applies naturally to:

* CPU
* data/fabric
* shader/frame
* kernel/dispatch
* future hardware-facing runtimes

## Why This Helps Reliability

Without an explicit negotiation layer, several future features become
dangerous:

* timeout reasoning
* cross-domain task observation
* global traces
* local async-to-sync contraction
* scheduler/lane timing assumptions

In all of those cases, the compiler/runtime must know more than:

* “there is some local tick”

It must know:

* whose tick
* how it relates to the global reference
* what precision or trust is lost during conversion

## Relationship To Hot Sync Contraction

This sketch is tightly connected to:

* [yir-hot-sync-contraction-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-hot-sync-contraction-sketch.md)

The key connection is:

* local async contraction is only trustworthy if timing assumptions remain
  explicit and comparable

So if a region is going to be contracted, future analysis will likely need to
prove not only ownership and observer conditions, but also:

* that the clock bridge assumptions for the region are stable enough
* that no required cross-domain timing distinction is being erased

## Current Repository Anchors

The current repository already has some early anchors for this direction:

* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
* `clock_domain`
* `clock_policy="bridge"`
* `resolved_clock_domain`
* `resolved_clock_bridge`
* `resolved_clock_surface`
* `nustar` clock metadata fields

These are not the final protocol.
But they are good seeds.

## Current Working Direction

The likely healthy order is:

1. keep clock-domain naming explicit
2. keep bridge naming explicit
3. keep per-`nustar` local clock ownership explicit
4. add negotiation metadata before adding aggressive timing-sensitive
   optimization
5. only later decide how much of this becomes first-class `GLM`/`YIR`
   structure

That is slower than pretending timing is already solved, but much safer.

## Related References

* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
* [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
* [yir-hot-sync-contraction-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-hot-sync-contraction-sketch.md)
