# `WitSage`

`WitSage` is an official `Galaxy` for classical machine-learning workloads in
the `nuis` standard-library family.

Its first job is not deep learning. It gives the kernel domain a practical,
source-level ML vocabulary for data shape, feature statistics, simple model
plans, and kernel-backed dispatch contracts.

Target character:

* classical machine-learning package
* preprocessing, statistics, and feature-shaping contracts
* small model-plan recipes for linear scoring, clustering, and nearest-neighbor work
* evaluation summaries for classification and regression-style workflows
* pipeline summaries that connect preprocessing, model plans, evaluation, and
  kernel dispatch contracts
* kernel-facing execution summaries that can sit on top of `nuis` heterogeneous lowering

Relationship:

* `core`
  smallest semantic base
* `std`
  host/runtime helper layer
* `witsage`
  classical ML Galaxy built on `core + std`
* `kernel`
  heterogeneous compute domain that can execute tensor-style WitSage plans

Current source-asset status:

* `WitSage` is a checked-in stdlib package skeleton through
  [module.toml](module.toml)
* the first auto-injectable helper surface is
  [lib/ml_contracts.ns](lib/ml_contracts.ns)
* `WitSage` now also exposes a kernel-facing auto-injectable surface through
  [lib/kernel_surface.ns](lib/kernel_surface.ns),
  giving projects a stdlib-owned `WitSageKernelSurface` profile instead of
  requiring every example to carry a local `KernelUnit`
* the first canonical source assets are
  [core/dataset_recipe.ns](core/dataset_recipe.ns),
  [core/feature_stats_recipe.ns](core/feature_stats_recipe.ns),
  [core/normalization_recipe.ns](core/normalization_recipe.ns),
  [core/train_test_split_recipe.ns](core/train_test_split_recipe.ns),
  [core/linear_score_recipe.ns](core/linear_score_recipe.ns),
  [core/kmeans_recipe.ns](core/kmeans_recipe.ns),
  [core/knn_recipe.ns](core/knn_recipe.ns),
  [core/kernel_plan_recipe.ns](core/kernel_plan_recipe.ns),
  [core/confusion_matrix_recipe.ns](core/confusion_matrix_recipe.ns),
  [core/evaluation_recipe.ns](core/evaluation_recipe.ns),
  and
  [core/pipeline_recipe.ns](core/pipeline_recipe.ns)

Current first responsibility:

* establish canonical dataset and feature-shape summaries
* establish compact feature-statistics contracts
* establish preprocessing contracts for normalization and train/test split planning
* establish simple classical model-plan contracts
* establish first classification evaluation contracts
* establish a small end-to-end classical ML pipeline summary that ties dataset,
  preprocessing, model, evaluation, and kernel plan contracts together
* establish a kernel-facing plan shape without coupling WitSage to one backend
* establish compact classifier/kernel pipeline helper scores for usable
  source-level examples
* give examples a stable `WitSageContracts` module for `galaxy = ["witsage=workspace"]`
* give kernel-backed examples a stable `WitSageKernelSurface` module for
  `galaxy = ["witsage=workspace"]`

Current native execution baseline:

* a four-element affine CoreML model verifies deterministic CPU-preferred
  prediction and output evidence
* a `16x64x64` feature-grid projection verifies deterministic
  Neural-Engine-preferred prediction on the M2 smoke host
* both models cross the same provider-neutral buffer, kernel, model-asset, and
  compute-plan contracts; Nsdb does not recognize WitSage operation names
* one ordered provider request collection executes feature-grid, affine, and a
  dependency-bound chained affine, emitting independently identified outputs
  plus order-sensitive graph and collection hashes
* each model binds a versioned `f32` output comparison descriptor with shape,
  expected asset hash, absolute/relative tolerance, and non-finite policy
* Nsdb independently compares all 65,536 feature-grid values and both
  four-element affine outputs before accepting the collection; the chained
  request consumes the first affine's real CoreML output and verifies
  `[7, 11, 15, 19]`
* missing, duplicate, cyclic, forward, or buffer-mismatched dependency edges
  block the collection before provider execution
* `nuis-provider-input-binding-v1` gives every input an ordered name, source,
  type, shape, byte length, and hash; the current requests publish explicit
  artifact/dependency bindings and fan-in is protocol-validatable
* the CoreML adapter executes ordered named features; a real Add model fans
  affine and chained-affine outputs into `[10, 16, 22, 28]` through the same
  independent output comparison boundary

Current official surface registry:

* `contract.witsage.dataset.v1`
* `contract.witsage.feature-stats.v1`
* `contract.witsage.classical-model.v1`
* `contract.witsage.kernel-plan.v1`
* `contract.witsage.preprocessing.v1`
* `contract.witsage.evaluation.v1`
* `contract.witsage.pipeline.v1`
