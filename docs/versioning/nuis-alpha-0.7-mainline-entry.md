# `nuis` `alpha-0.7.*` Mainline Entry

This file is the predecessor short entry point for the `alpha-0.7.*` line.

It does not replace the `alpha-0.6.*` Nsld entry or the `alpha-0.4.*`
hardening baseline. Those remain predecessor context. This file records what
changed enough in `alpha-0.7.*` that first-read docs should now route current
work through this line.

Short rule:

`alpha-0.7.*` is where the project starts treating std-backed tooling examples
as the default smoke surface while keeping linker, lowering, and heterogeneous
binary work on the same mainline.

## Current Line Shape

Read the current line as:

* `alpha-0.4.*` hardening baseline still applies
* `alpha-0.6.*` introduced the named Nsld linker frontdoor
* `alpha-0.7.*` made std contract consumption the default shape for tooling
  examples
* current docs should say `alpha-0.16.*` for present-tense work and link
  `alpha-0.7.*` as predecessor std/tooling smoke context

## Current Front Doors

Start here:

1. [../current-mainline-map.md](../../docs/current-mainline-map.md)
2. [../reference/std-mainline-layering-contract.md](../../docs/reference/std-mainline-layering-contract.md)
3. [../reference/toolchain-galaxy-core-boundary.md](../../docs/reference/toolchain-galaxy-core-boundary.md)
4. [../reference/nsld-linker-frontdoor.md](../../docs/reference/nsld-linker-frontdoor.md)
5. [nuis-alpha-0.6-mainline-entry.md](nuis-alpha-0.6-mainline-entry.md)
6. [nuis-alpha-0.4-system-inventory.md](nuis-alpha-0.4-system-inventory.md)
7. [nuis-alpha-0.4-mainline-hardening-plan.md](nuis-alpha-0.4-mainline-hardening-plan.md)

## What Is New Enough To Mention First

### Std-Backed Tooling Smoke

The tooling example tree now treats `std=workspace` as the normal integration
path instead of an optional follow-up.

Current shape:

* CLI runtime and workflow examples consume `StdCliContracts`
* filesystem examples consume `StdFsContracts`
* stdout/stderr and terminal examples consume `StdIoContracts`
* text, JSON, and report examples consume `StdTextContracts`
* time, clock, and benchmark examples consume `StdTimeContracts`
* heterogeneous proxy benchmark examples consume `StdHeteroContracts`

Current rule:

* tooling demos should prefer std contract helpers over raw probe totals
* process exits should use contract-level `ok/error` helpers when the demo is
  asserting success or failure
* raw compact totals are still useful inside reports, but they should not be
  the only story for runnable examples

### Tooling As Lowering Pressure

The current std/tooling lane is not just documentation polish.

It is now a pressure test for:

* `nuis build` and project manifest routing
* host FFI facades used by CLI, IO, filesystem, text, time, and process probes
* artifact/run-artifact behavior
* standard-library contract visibility through galaxy resolution
* the future linker and heterogeneous binary story

This is why `alpha-0.7.*` should keep connecting examples to std contracts
instead of growing isolated one-off demos.

### Current Nsld Relationship

`Nsld` remains the named linker frontdoor introduced in `alpha-0.6.0`.

For `alpha-0.7.*`, do not rewrite that history. Instead, treat Nsld as the
linker-side counterpart to the std/tooling smoke work:

* std/tooling proves useful build inputs exist
* Nsld proves those inputs can be inspected as linkable closure metadata
* later linker work can replace frontdoor inspection with owned binary linking

The toolchain boundary is now also clearer: `nsld` and `nsdb` should evolve as
CLI adapters over reusable core/galaxy-style capabilities. That means new
linker and debugger features should first stabilize structured metadata and
core contracts, then expose human-facing commands on top.

Reference:

* [../reference/toolchain-galaxy-core-boundary.md](../../docs/reference/toolchain-galaxy-core-boundary.md)
* [../reference/nsld-linker-frontdoor.md](../../docs/reference/nsld-linker-frontdoor.md)
* [../reference/nsdb-yir-debugger-frontdoor.md](../../docs/reference/nsdb-yir-debugger-frontdoor.md)

## What Is Still Baseline Context

The `alpha-0.4.*` docs still describe the hardening philosophy:

* consolidate before opening large new fronts
* make source-to-YIR-to-AOT paths boring and repeatable
* keep compiler/nustar coupling under audit
* make std, PixelMagic, and WitSage act as integration pressure tests

The `alpha-0.6.*` entry still describes when the linker frontdoor became a
named toolchain member.

Use those files for baseline and history, but do not call them the present line
in new top-level docs.

## What Should Not Be Claimed Yet

`alpha-0.7.*` should not claim:

* final self-hosting
* final std API stability
* final self-owned linker implementation
* final unified heterogeneous executable binary
* final GPU/NPU backend maturity
* beta-level public stability

Safe wording:

* say `contract-backed`, `smoke`, `tooling route`, `frontdoor`, or `baseline`
  when that is the current truth
* say `runs` only for examples exercised by the checked-in build/run tests
* say `current line` for `alpha-0.7.*`
* say `predecessor` for `alpha-0.6.*` and earlier; say `baseline` for the
  still-relevant `alpha-0.4.*` hardening docs
