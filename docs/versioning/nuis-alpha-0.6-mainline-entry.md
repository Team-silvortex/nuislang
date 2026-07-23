# `nuis` `alpha-0.6.*` Mainline Entry

This file is the predecessor short entry point for the `alpha-0.6.*` line.

It does not replace the `alpha-0.4.*` inventory and hardening documents. Those
remain the baseline for the current hardening philosophy. This file records what
changed enough in `alpha-0.6.*` that first-read docs at the time stopped calling
`alpha-0.4.*` the current line. Present-tense work should now start with the
`alpha-0.17.*` entry.

Short rule:

`alpha-0.6.*` is where the toolchain starts naming linker ownership explicitly while std examples begin proving contract-backed CLI behavior.`

## Line Shape

Read this predecessor line as:

* `alpha-0.4.*` hardening baseline still applies
* `alpha-0.6.*` adds the first dedicated Nsld linker frontdoor
* std filesystem examples now include build/run-backed contract consumers
* current docs should say `alpha-0.17.*` for present-tense work and link this
  file as predecessor context

## Predecessor Front Doors

Start here:

1. [../current-mainline-map.md](../../docs/current-mainline-map.md)
2. [../reference/nsld-linker-frontdoor.md](../../docs/reference/nsld-linker-frontdoor.md)
3. [nuis-alpha-0.4-system-inventory.md](nuis-alpha-0.4-system-inventory.md)
4. [nuis-alpha-0.4-mainline-hardening-plan.md](nuis-alpha-0.4-mainline-hardening-plan.md)
5. [../reference/std-mainline-layering-contract.md](../../docs/reference/std-mainline-layering-contract.md)

## What Is New Enough To Mention First

### Nsld

`Nsld` became the named linker toolchain member in this line.

Current scope:

* inspect real `nuis build` output directories and build manifests
* render link plans and closure reports
* surface clock/order/linker-facing metadata
* emit and verify `nuis.nsld.link-inputs.toml`
* keep the self-owned linker boundary explicit without pretending the final
  Nuis binary linker is complete

Reference:

* [../reference/nsld-linker-frontdoor.md](../../docs/reference/nsld-linker-frontdoor.md)

### Std Contract Smoke

The std work moved beyond source-only helpers for the filesystem lane in this
line.

Current smoke set:

* `file_read_demo`
* `file_write_demo`
* `file_copy_demo`
* `file_roundtrip_demo`
* `file_output_demo`
* `directory_create_demo`
* `directory_remove_demo`
* `filesystem_report_demo`
* `filesystem_report_file_demo`
* `filesystem_io_report_demo`
* `benchmark_report_file_demo`

Current rule:

* examples that claim run-artifact smoke value should use `std=workspace`
* filesystem examples should consume `StdFsContracts`
* successful paths should return process-style `fs_ok()` / `fs_error()` instead
  of leaking compact probe totals as process exits

## What Is Still Baseline Context

The `alpha-0.4.*` docs still describe the main hardening philosophy:

* consolidate before opening large new fronts
* make source-to-YIR-to-AOT paths boring and repeatable
* keep compiler/nustar coupling under audit
* make std, PixelMagic, and WitSage act as integration pressure tests

Use those files for broad status, but do not call them the present line in new
top-level docs.

## What Should Not Be Claimed Yet

`alpha-0.6.*` should not claim:

* final self-hosting
* final binary/container format stability
* final self-owned linker implementation
* final Nsdb/YIR debugger maturity
* final GPU/NPU backend maturity
* final std API stability
* beta-level public stability

Safe wording:

* say `frontdoor`, `contract`, `smoke`, `probe`, or `baseline` when that is the
  current truth
* say `runs` only for examples that are exercised by `nuis build` and
  `run-artifact`
* say `predecessor line` for `alpha-0.6.*`; say `baseline` or `predecessor`
  for `alpha-0.4.*`, `alpha-0.1.*`, and earlier
