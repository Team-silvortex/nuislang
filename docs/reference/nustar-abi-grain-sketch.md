# Nustar ABI Grain Sketch

This file sketches a short design direction for the grain of `nustar`
registration, ABI targeting, and final built artifacts.

The goal is to avoid mixing together three different questions:

* how large one logical capability package should be
* how fine-grained target compatibility should be expressed
* how concrete one final built implementation artifact should be

## Shortest Rule

`packages stay capability-coarse; abi_targets stay platform-fine; artifacts stay concrete`

## Why This Needs A Separate Rule

The repository wants several things at the same time:

* frontend shape that stays unified within the same capability family
* replaceable `nustar` implementations
* exact host ABI validation
* cross-compilation
* heterogeneous static packaging

Those goals are compatible, but only if package grain and ABI grain are not
forced to be identical.

## Three Grains

### 1. Package Grain

`nustar` packages should stay relatively coarse and follow capability families.

Current examples:

* [cpu.toml](../../nustar-packages/cpu.toml)
* [network.toml](../../nustar-packages/network.toml)
* [shader.toml](../../nustar-packages/shader.toml)
* [kernel.toml](../../nustar-packages/kernel.toml)

This keeps source-facing frontend shape stable:

* CPU frontend remains “CPU”
* network frontend remains “network”
* shader frontend remains “shader”

instead of forking early by host platform.

### 2. ABI-Target Grain

Inside one package, compatibility should be expressed through fine-grained
registered `abi_targets`.

Current target fields already include:

* `arch=...`
* `os=...`
* `object=...`
* `calling=...`
* `clang=...`
* optional `backend=...`

This is the correct place to express distinctions such as:

* `x86_64` vs `arm64`
* Linux vs Darwin vs Windows
* `sysv64` vs `win64` vs `aapcs64`
* backend-family differences like fallback CPU vs Metal/Vulkan/DirectX style
  lowering families

The package stays coarse; the target description stays fine.

### 3. Artifact Grain

The final built implementation artifact should stay concrete.

That means:

* one logical package may register many compatible `abi_targets`
* one final produced implementation artifact should still correspond to one
  resolved target contract

This matches the repository's current exact-match direction better than a
single universal built artifact pretending to be equally native everywhere.

## Current Repository Shape

The current repository already leans in this direction.

Examples:

* [cpu.toml](../../nustar-packages/cpu.toml) keeps
  one logical `official.cpu` package while registering:
  * `cpu.arm64.apple_aapcs64`
  * `cpu.arm64.linux.aapcs64`
  * `cpu.x86_64.sysv64`
  * `cpu.x86_64.win64`
* [network.toml](../../nustar-packages/network.toml)
  keeps one logical `official.network` package while registering:
  * host-adaptive socket target
  * Darwin arm64 socket target
  * Linux x86_64 socket target
  * Windows x86_64 socket target

This is already a strong sign that:

* package grain should remain coarse
* ABI grain should remain explicit

## Host-Adaptive Targets

Some manifests also use host-adaptive target descriptions such as:

* `arch=host`
* `os=host`
* `object=host`
* `calling=host`

These are useful convenience forms for:

* local development
* fast host-matching builds
* fallback profiles

But they should still resolve to a concrete target contract before final
artifact production.

So the rule should be:

* host-adaptive registration is acceptable
* host-adaptive final artifacts are not the deepest truth
* build resolution still normalizes them into concrete machine/OS/object/calling
  facts

## Frontend Unification Rule

The main reason to keep package grain coarse is not just convenience.

It preserves the real value of unified heterogeneous programming:

* frontend shape follows capability families
* ABI variation stays deeper in registration, lowering, and packaging

For example:

* CPU source frontend should not split early because the target is Darwin vs
  Linux
* network source frontend should not split early because the socket backend
  lands on different host ABIs

Surface source differences should appear only when capability differences are
real, not simply because packaging facts differ.

## Replaceability Rule

Replaceability should be read at the package implementation level, not as
“every artifact must be universal”.

That means:

* a package implementation may be replaced
* the replacement must still pass registration completeness checks
* the replacement must still satisfy standards legality
* the replacement must still expose valid `loader-contract` and target facts

So the stable truth is:

* registered package capability contract
* resolved ABI target contract

not:

* one particular implementation blob
* one particular frontend spelling

## Cross-Compilation Rule

Cross-compilation should be understood as:

* selecting a registered target contract that differs from the host
* validating it through `abi_targets`
* producing a concrete artifact for that target

not as:

* widening the frontend into target-specific language forks

This keeps source shape stable while still allowing platform-specific results.

## Conditional Compilation Rule

Conditional compilation still matters, but it should express:

* genuine capability differences
* backend differences
* host bridge differences

It should not be the primary tool for carving package grain by platform when a
single capability family already exists.

## Practical Recommendation

The practical recommendation for the repository is:

* keep `official.cpu` / `official.network` / `official.shader` / `official.kernel`
  style package grain
* keep `abi_targets` explicit and fine-grained
* keep final artifacts concrete
* let build/link resolution decide which target is chosen
* keep frontend mostly unified within each family

## Shortest Decision Matrix

If the question is:

* “should each OS x ISA pair become its own logical `nustar` package?”
  * generally no
* “should one logical package be allowed to register many ABI targets?”
  * yes
* “should one final built artifact still land on one concrete ABI target?”
  * yes

So the shortest rule is:

`one family package, many registered targets, one concrete built artifact`
