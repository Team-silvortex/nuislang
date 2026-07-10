# `nuis` `alpha-0.8.*` Documentation Sync Inventory

This file records the alpha-0.8 documentation refresh that made the top-level
entry points match the current binary-linking and standard-library mainline.

It does not replace the current alpha entry:

* [nuis-alpha-0.8-mainline-entry.md](nuis-alpha-0.8-mainline-entry.md)

Use this file when the question is "which broad README surfaces were refreshed
for alpha-0.8, and what wording should stay safe?"

## Updated Entry Points

The broad entry points now route through the current alpha-0.8 line first:

* [../../README.md](../../README.md)
* [../README.md](../README.md)
* [../reference/README.md](../reference/README.md)
* [../../examples/README.md](../../examples/README.md)
* [../../stdlib/README.md](../../stdlib/README.md)

The intended order is:

1. top-level README for the current repo shape
2. current mainline map for detailed routing
3. alpha-0.8 mainline entry for current line wording
4. reference docs for implementation truth
5. examples and stdlib READMEs for local routes

## Safe Current Wording

Use these phrases for present-tense work:

* `alpha-0.8.*`
* `binary-linking convergence`
* `minimal runnable route`
* `host-assisted finalization`
* `nustar registration boundary`
* `YIR semantic execution boundary`
* `std / PixelMagic / WitSage proving surfaces`

Avoid claiming these as complete:

* final self-owned executable linking
* final unified heterogeneous executable binary
* final std API stability
* final GPU/NPU backend maturity
* beta-level public stability
* self-hosting

## Current Documentation Shape

The top-level README should stay compact. It should answer:

* what Nuis is
* what alpha line the repo is on
* what currently works
* what does not yet work
* which command path to run first
* where to read next

The detailed inventory should live in:

* [../current-mainline-map.md](../current-mainline-map.md)
* [../reference/README.md](../reference/README.md)
* [README.md](README.md)
* local READMEs under `examples/` and `stdlib/`

## Follow-Up Watch List

Future doc sync passes should keep an eye on:

* keeping `nsld` wording aligned with the actual executable writer path
* keeping `nsbdr` clearly separate from linker responsibilities
* keeping `nsdb` framed as YIR-semantic debugging, not LLDB replacement
* keeping `stdlib/ns-nova` later-stage until AOT/std/PixelMagic/WitSage are
  less soft
* keeping `examples/projects` as the primary runnable/compile-contract route
* updating this file when alpha-0.9 or alpha-0.10 changes the binary closure
  truth
