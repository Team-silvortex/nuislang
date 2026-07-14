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
  protocol-owned maturity label. In `dev-tensor-status-v1`, valid values are
  `stable`, `usable`, `active`, and `early`
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
* `status_protocol_version`
* `status_protocol = [...]`
* `hierarchy_root_status`
* `hierarchy_root_progress`
* `hierarchy_root_weakest_child_path`
* `bootstrap_critical_count`
* `bootstrap_critical_average_progress`
* `weakest_bootstrap_architecture`
* `weakest_bootstrap_module`
* `weakest_bootstrap_function`
* `coverage_status`
* `coverage_expected_count`
* `coverage_covered_count`
* `coverage_missing_count`
* `coverage_orphaned_count`
* `coverage_stale_count`
* `coverage_first_gap`
* `coverage_missing_coordinates`
* `coverage_orphaned_coordinates`
* `coverage_stale_coordinates`
* `drift_status`
* `drift_check_count`
* `drift_check_passed_count`
* `drift_check_failed_count`
* `drift_first_failed_check`
* `drift_checks = [...]`
* `hierarchy = {...}`
* `cells = [...]`

Each cell includes both named coordinates and a `coordinates` array so scripts
can read it either as records or as tensor coordinates.

## Status Protocol

The tensor status field is now protocolized rather than free-form text. The
current protocol is `dev-tensor-status-v1`:

* `stable`
  rank `4`, phase `validated`, terminal for the current milestone slice
* `usable`
  rank `3`, phase `usable`, strong enough to consume but still evolving
* `active`
  rank `2`, phase `in-progress`, actively maturing and allowed to move fast
* `early`
  rank `1`, phase `exploratory`, not mature enough to anchor bootstrap-critical
  closure by itself

Coverage treats an unknown status as stale metadata. This keeps the tensor from
quietly drifting into ad-hoc labels.

## Recursive Hierarchy

The flat `architecture/module/function` cells are also projected into a
recursive hierarchy:

`root -> architecture -> module -> function`

Each hierarchy node carries:

* `level`
* `path`
* `status`
* `status_rank`
* `progress`
* `cell_count`
* `bootstrap_critical_count`
* `weakest_child_path`
* `children`

Branch status is derived from the weakest child status, and branch progress is
the weighted average of descendant function cells. This means the tensor can be
read both as a table and as a recursively inspectable project tree. The
recursive form is intended to support future bootstrap planning where a weak
architecture lane can be expanded into its weakest module and then into the
exact function cell that needs work.

`nuis status` also prints the short tensor summary. That makes the model part
of the toolchain self-orientation surface, not just a separate report command.

## Coverage Manifest

The tensor now has a small built-in coverage manifest. The manifest lists the
coordinates that the alpha line expects to see in the tensor:

`expected architecture/module/function coordinates`

The coverage layer compares that expected coordinate set with the actual
`DEV_TENSOR_CELLS` entries and reports:

* `coverage_status`
  `clean` when required expected coordinates are covered and no stale/orphaned
  cells are present; otherwise `gap`
* `coverage_missing_coordinates`
  expected coordinates that do not currently have a tensor cell
* `coverage_orphaned_coordinates`
  tensor cells that exist but are not declared by the coverage manifest
* `coverage_stale_coordinates`
  cells with invalid metadata, such as empty evidence or out-of-range progress
* `coverage_first_gap`
  the first missing, orphaned, or stale coordinate for quick CLI triage

Short rule:

`drift checks ask whether evidence anchors still exist; coverage asks whether
the tensor itself still spans the expected project map`

This is not yet automatic repository discovery. It is the first guardrail that
prevents the tensor from becoming only a hand-written status list. Future
versions can derive expected coordinates from galaxy manifests, Nustar
registries, std module manifests, and milestone files.

## Drift Checks

The tensor now includes a first lightweight drift-check layer.

These checks do not replace the real test suite. They only verify that selected
progress evidence anchors still exist in the repository, such as:

* frontdoor JSON fields
* workflow/artifact runtime regression assertions
* reference-document field anchors
* standard-library smoke-test and example-lane anchors

The current status values are:

* `clean`
  every configured evidence anchor is still visible
* `drift`
  at least one configured evidence anchor is missing

Short rule:

`drift checks make the tensor less imaginary: if a progress cell claims a
frontdoor or document exists, the tensor can at least notice when that anchor
disappears`

The first std-oriented checks deliberately anchor the bootstrap-critical
`host-io-filesystem-text` cell to:

* `tools/nuis/tests/std_filesystem_smoke.rs`
* `examples/projects/tooling/README.md`
* `stdlib/std/README.md`

That keeps the standard-library progress cell tied to the project-form
filesystem, IO, text, terminal, and tooling smoke chain instead of only a broad
roadmap phrase.

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
