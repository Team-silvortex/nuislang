# Nuis Development Tensor

This file defines the first lightweight development-progress model for the
alpha line.

It answers one narrow question:

`how do we describe current system progress without flattening everything into
one vague roadmap list?`

## Model

The development tensor is a 3-axis progress model:

* `architecture`
  the broad system layer or design lane
* `module`
  the concrete repository/tool/package area carrying the work
* `function`
  the user-visible or toolchain-visible capability being matured

Each tensor cell carries:

* `status`
  short maturity label such as `stable`, `active`, `usable`, `early`,
  `blocked`, or `new`
* `progress`
  current alpha-era progress score from `0` to `100`
* `bootstrap_critical`
  whether Nuis should treat this cell as important before self-hosting
* `closure_role`
  the role this cell plays in the compiler/toolchain/runtime closure
* `evidence`
  the current proof anchor, usually tests, frontdoor fields, docs, or examples
* `next_step`
  the most useful next action for that cell

Short rule:

`architecture tells where the work lives; module tells who owns it; function
tells what capability is being matured`

## CLI

Use:

```bash
cargo run -p nuis -- dev-tensor
cargo run -p nuis -- dev-tensor --json
```

The JSON surface is intentionally simple:

* `kind = "nuis_dev_tensor"`
* `model = "architecture-module-function-progress-tensor"`
* `axis_0 = "architecture"`
* `axis_1 = "module"`
* `axis_2 = "function"`
* `bootstrap_critical_count`
* `bootstrap_critical_average_progress`
* `weakest_bootstrap_architecture`
* `weakest_bootstrap_module`
* `weakest_bootstrap_function`
* `drift_status`
* `drift_check_count`
* `drift_check_passed_count`
* `drift_check_failed_count`
* `drift_first_failed_check`
* `drift_checks = [...]`
* `cells = [...]`

Each cell includes both named coordinates and a `coordinates` array so scripts
can read it either as records or as tensor coordinates.

`nuis status` also prints the short tensor summary. That makes the model part
of the toolchain self-orientation surface, not just a separate report command.

## Drift Checks

The tensor now includes a first lightweight drift-check layer.

These checks do not replace the real test suite. They only verify that selected
progress evidence anchors still exist in the repository, such as:

* frontdoor JSON fields
* workflow/artifact runtime regression assertions
* reference-document field anchors

The current status values are:

* `clean`
  every configured evidence anchor is still visible
* `drift`
  at least one configured evidence anchor is missing

Short rule:

`drift checks make the tensor less imaginary: if a progress cell claims a
frontdoor or document exists, the tensor can at least notice when that anchor
disappears`

## Current Role

The first implementation is static and intentionally conservative. It is not a
replacement for tests, release checklists, or Nsld/Nuis frontdoor reports.

It is a development-system index over those surfaces, with a small drift-check
layer over the most bootstrap-critical anchors.

The first useful jobs are:

* keep CLI closure, Nsld, std, language-core, Nustar, and native-binary work in
  one comparable view
* make weak cells explicit instead of hiding them in broad status prose
* separate `host runnable`, `Nsld-owned ready`, and `self-owned binary assembly`
  as different functions instead of one overloaded "binary works" claim
* let `nuis` name the weakest bootstrap-critical coordinate without requiring
  a human to reread the whole roadmap
* give alpha milestones a structured progress vocabulary before beta
  self-hosting pressure grows

## Current Honesty Boundary

The tensor is a progress model, not a contract freeze.

In alpha it may change cell names aggressively when the architecture changes.
The stable part is the coordinate idea:

`architecture x module x function -> status/progress/evidence/next_step`

Future work should move cells from static entries toward generated readings
from:

* checked tests
* frontdoor JSON fields
* Nsld reports
* docs/reference anchors
* package manifests
* roadmap milestones

The first drift checks are intentionally narrow. Future checks should become
milestone-owned instead of merely field-owned, so they can verify examples,
packages, and command workflows as well as names in source files.
