# Nustar Capability Split Boundary

This file records the current practical boundary for splitting capabilities out
of the compiler core and making them explicit through `nustar`.

It is written for the current `0.20.* -> alpha-0.0.1` handoff question:

`what should already belong to stable nustar capability contracts, what is still staged or hard-coded, and how should the next split pass be judged?`

## Short Rule

The current line should be read this way:

* frontend spelling can stay compiler-managed and replaceable
* stable capability truth should increasingly live in registered `nustar`
  contracts
* the compiler core should keep moving from “knows every family directly” to
  “binds through explicit package capability surfaces”

That does **not** mean every domain must already be fully externalized today.

It means new capability work should increasingly justify why it belongs in:

* compiler core
* `nustar` manifest contract
* std-facing annotation/registration surface
* project/runtime validation layer

## What `nustar` Already Owns Today

The current checked-in registry/manifests already make `nustar` responsible
for stable package-facing facts such as:

* package identity and domain family
* frontend family label
* `AST` / `NIR surface` / `YIR lowering` / `part verify` entry names
* loader entry / loader ABI
* ABI profiles and ABI targets
* ABI capability declarations
* support surfaces and profile slots
* clock-domain contract metadata

Current practical entrypoints:

* [yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
* [annotation-intrinsic-stdlib-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/annotation-intrinsic-stdlib-sketch.md)
* [nustar-abi-grain-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nustar-abi-grain-sketch.md)
* [nustar-packages](/Users/Shared/chroot/dev/nuislang/nustar-packages)

Short rule:

* package registration is already the stronger stable truth
* compiler-owned surface syntax is increasingly the replaceable convenience
  layer on top

## Current Contract Shape

The current code line now has an explicit registry-owned domain contract shape:

* `nustar-domain-contract-v1`

Current practical meaning:

* this is the stable schema label for the aggregated per-domain registration
  contract surfaced by the toolchain
* it is coarser than frontend syntax and finer than raw manifest text
* it is the right place to attach future per-domain registered facts before
  inventing more ad hoc CLI-only fields

Current contract groups inside that shape:

* package identity: package id, domain family, frontend
* loader contract: loader ABI, loader entry
* ABI contract: machine ABI policy, ABI profiles
* host bridge contract: host FFI surfaces, host FFI ABIs, host FFI bridge
* runtime capability contract: support surfaces, profile slots, default lanes,
  clock summary
* scheduler contract: result/scheduler/observer summaries
* domain extension groups: today `std_net` is one example

Current code-side group labels are now explicit:

* `package_identity`
* `loader_contract`
* `abi_contract`
* `host_bridge_contract`
* `runtime_capability_contract`
* `scheduler_contract`
* `std_net_extension`

Short rule:

* additive fields can extend `nustar-domain-contract-v1` while preserving old
  meaning
* semantic re-interpretation or structural breakage should mint
  `nustar-domain-contract-v2`
* future domains should plug into this contract shape first, not invent
  one-off frontdoor reporting structures

## What “Capability Split” Should Mean Now

At this stage, capability split should **not** mean:

* invent a plugin API for everything
* move arbitrary compiler internals behind dynamic indirection
* pretend the bootstrap shims are already gone

It should mean:

* make capability families explicit enough that the compiler can name, inspect,
  validate, and route them through registration instead of only through
  hard-coded assumptions
* reduce places where a capability exists “only because one compiler file knows
  about it”
* keep domain/package boundaries coarse and readable

The right current question is:

`can this capability be described as a stable registered contract, even if the implementation is still temporarily bootstrapped in-tree?`

## Current Capability Grain

The current repository already points toward coarse package grain, not
hyper-fragmented micro-packages.

Current family examples:

* `cpu`
* `data`
* `shader`
* `kernel`
* `network`

Short rule:

* split by domain/capability family
* not by every individual operation
* not by every per-OS/per-ISA surface spelling

That matches the existing `nustar` grain direction:

* replaceable package implementation
* stable registered capability contract
* explicit ABI and loader compatibility

## What The Compiler Core Should Still Own

Before `alpha-0.0.1`, compiler core should still own:

* core parsing and syntax validation
* core `NIR` ownership / verifier truth
* core `YIR` structure and registry binding machinery
* bootstrap compatibility shims where a registered entry is not yet fully
  late-bound
* project/build/test front-door orchestration

Short rule:

* core language truth stays in the compiler
* replaceable domain capability truth should increasingly be named through
  `nustar`

## What Should Increasingly Move Behind Registered Capability Contracts

The next split pass should favor moving these facts into explicit package
contracts whenever possible:

### 1. Domain surface identity

Examples:

* which frontend family a domain claims
* which `AST` surface names are official
* which `NIR` surface names are official
* which lowering/verify entries are the package contract

### 2. Runtime/support capability identity

Examples:

* support surfaces
* profile slots
* scheduler/result/summary capability stacks
* clock-domain/default bridge facts

### 3. ABI truth

Examples:

* registered ABI targets
* required `op:` / `surface:` capability declarations
* machine ABI compatibility wording

### 4. Loader/package shape

Examples:

* canonical loader ABI
* canonical entry symbol
* implementation format section names
* package validation requirements

## Current Practical Boundary

Read the current line in three buckets.

### Bucket A: already good `nustar` contract material

These are already meaningful and should keep expanding:

* manifest-declared domain identity
* ABI profiles / targets / capabilities
* loader-contract surface
* support/profile-slot declaration
* scheduler/clock contract declaration

### Bucket B: acceptable bootstrap shims

These are okay for now, but should increasingly feel temporary:

* bootstrap lowering dispatch that still maps known entry names in compiler
  code
* in-tree compatibility shims for domain lowering providers
* compiler-known packaging defaults that still exist only to keep the line
  runnable

### Bucket C: capability drift to reduce

These are the real targets for the next split pass:

* capability facts that live only in scattered compiler files
* capability families whose examples/docs/runtime validation do not clearly
  point back to one registered contract
* std-facing feature families whose stable truth is still prose-heavy but not
  surfaced as registry/package facts

## Practical Split Checklist

When deciding whether a capability has been “properly split”, ask:

1. Can the capability be named by one registered domain/package contract?
2. Can the toolchain surface that contract through `registry` /
   `loader-contract` / project validation output?
3. Can examples/docs point to that same contract instead of describing a
   compiler secret?
4. If bootstrap code still exists, is it clearly only a compatibility layer
   rather than the only real source of truth?

If the answer is mostly “yes”, the split is probably good enough for the
current line.

## What The Next Pass Should Prioritize

Before `alpha-0.0.1`, the highest-value `nustar` split work is likely:

* closing obvious gaps between std-facing capability docs and registered
  package facts
* reducing compiler-only knowledge around domain support/profile/summary
  surfaces
* making capability output from `registry`, `loader-contract`,
  `project-status`, and `project-doctor` feel like one coherent contract story

The goal is not maximal abstraction.

The goal is:

* fewer hidden capability assumptions
* fewer ad hoc compiler-only boundary facts
* one clearer package contract story that `alpha` can stand on

## Why This Matters For `alpha-0.0.1`

For `alpha`, `nustar` does not need to be “fully pluginized”.

It does need to be believable as the place where replaceable domain capability
truth is increasingly carried.

That means `alpha` should be able to honestly say:

* capability families are being separated by explicit package contract
* loader/ABI/support surfaces are increasingly registry-first
* compiler core is still the semantic backbone, but less of the package truth
  is hidden inside it

That is the right current maturity target:

* not finished
* but clearly moving from architecture idea to enforceable mainline boundary
