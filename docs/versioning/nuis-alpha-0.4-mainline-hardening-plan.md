# `nuis` `alpha-0.4.*` Mainline Hardening Plan

This file is the short current-state anchor for the `alpha-0.4.*` line.

It is not a broad feature wishlist.

It is the working rule for the stretch before `alpha-0.7.0`:

`make the existing system harder to break, easier to prove, and more useful before opening large new fronts`

## Current Line

`alpha-0.4.*` should be read as:

* consolidation after the first alpha feature-expansion pass
* hardening of the existing compile and AOT path
* tighter `nustar` registration and host FFI boundaries
* practical `std`, PixelMagic, and WitSage usability work
* regression-backed integration across frontend, NIR, YIR, lowering, pack, and
  runtime-facing examples

Short rule:

`alpha-0.4.*` is where existing pieces should start feeling like one toolchain instead of many promising islands`

## Mainline Priority Before `alpha-0.7.0`

The main task is to improve what already exists.

Preferred work:

* make source-to-YIR-to-AOT paths boring and repeatable
* make `nuis workflow`, `project-doctor`, `artifact-doctor`, and packaging
  surfaces agree on the same truth
* move FFI, ABI, and `nustar` capability checks toward explicit registered
  contracts
* keep CPU, shader, kernel, network, data, and newer architecture-specific
  `nustar` surfaces pluggable rather than hard-wired into the compiler
* thicken `std` around text, IO, filesystem, task/thread, network, error, and
  FFI-safe handle boundaries
* make PixelMagic and WitSage useful as official galaxy proving grounds for
  shader/kernel/CPU cooperation
* keep docs and examples honest enough that old demos do not masquerade as
  current capability

Deprioritized work:

* large new syntax families without lowering/runtime anchors
* beta-style stability promises
* final linker/launcher/container architecture claims
* ns-nova maturity work beyond contracts needed by current AOT and shader
  lanes
* dynamic plugin loading semantics that would bypass static registration
  contracts

## Hardening Lanes

### Compile Pipeline

The pipeline should keep moving toward one obvious path:

```text
source / project
  -> frontend
  -> NIR
  -> YIR
  -> verify
  -> lower
  -> pack
  -> artifact-doctor
  -> run
```

Success means:

* the same example can be explained from source, workflow output, YIR, and
  packed artifact manifest
* packer metadata catches ABI/signature drift before runtime
* lowering failures are reported as contract gaps, not mysterious backend
  failures

### Registered Boundaries

The strongest current direction is:

`compiler knows the contract shape; registered packages own domain capability`

This applies to:

* host FFI signatures and hash-registered symbol contracts
* ABI profile capability routing
* `nustar` backend selection
* shader and kernel backend packets
* future architecture-specific CPU packages

### Standard And Official Galaxy Work

`std`, PixelMagic, and WitSage should be used as integration pressure tests.

The desired order is:

1. make `std` IO, filesystem, text, error, task/thread, network, and FFI
   surfaces coherent enough for CLI tools
2. make PixelMagic exercise CPU plus shader paths through real image-oriented
   examples
3. make WitSage exercise CPU plus kernel paths through small classical ML
   examples
4. only then raise ns-nova work beyond thin contracts and placeholder examples

## Regression Bias

Every hardening patch should prefer one of these proof shapes:

* a frontend compile test for source semantics
* a project compile test for package/workflow integration
* a YIR or packer test for contract metadata
* an AOT/runtime demo when the claim is "this can run"
* a doc update when the boundary changed

Short rule:

`if a feature cannot be shown, checked, or named precisely, it is not hardened yet`

## What Should Not Be Claimed Yet

`alpha-0.4.*` should not claim:

* self-hosting
* final binary format stability
* final ABI/linker ownership
* final GLM/ownership treatment of unsafe raw pointers
* final GPU vendor backend maturity
* final ns-nova engine maturity
* beta-level public stability

## First Reading Route

For current work, start here:

1. [../current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
2. [../reference/nuis-frontdoor-surface-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-frontdoor-surface-reference.md)
3. [../reference/nuis-native-artifact-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-native-artifact-workflow.md)
4. [../reference/ffi-pointer-safety-boundary.md](/Users/Shared/chroot/dev/nuislang/docs/reference/ffi-pointer-safety-boundary.md)
5. [../reference/nustar-capability-split-boundary.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nustar-capability-split-boundary.md)
6. [../reference/std-mainline-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-mainline-layering-contract.md)
7. [../reference/pixelmagic-mainline-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/pixelmagic-mainline-contract.md)

Then use older alpha and `0.20.*` docs as predecessor context, not as the
present-tense route.

