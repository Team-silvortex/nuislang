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
  [module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/module.toml)
* the first auto-injectable helper surface is
  [lib/ml_contracts.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/lib/ml_contracts.ns)
* `WitSage` now also exposes a kernel-facing auto-injectable surface through
  [lib/kernel_surface.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/lib/kernel_surface.ns),
  giving projects a stdlib-owned `WitSageKernelSurface` profile instead of
  requiring every example to carry a local `KernelUnit`
* the first canonical source assets are
  [core/dataset_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/dataset_recipe.ns),
  [core/feature_stats_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/feature_stats_recipe.ns),
  [core/normalization_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/normalization_recipe.ns),
  [core/train_test_split_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/train_test_split_recipe.ns),
  [core/linear_score_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/linear_score_recipe.ns),
  [core/kmeans_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/kmeans_recipe.ns),
  [core/knn_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/knn_recipe.ns),
  [core/kernel_plan_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/kernel_plan_recipe.ns),
  [core/confusion_matrix_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/confusion_matrix_recipe.ns),
  and
  [core/evaluation_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/evaluation_recipe.ns)

Current first responsibility:

* establish canonical dataset and feature-shape summaries
* establish compact feature-statistics contracts
* establish preprocessing contracts for normalization and train/test split planning
* establish simple classical model-plan contracts
* establish first classification evaluation contracts
* establish a kernel-facing plan shape without coupling WitSage to one backend
* establish compact classifier/kernel pipeline helper scores for usable
  source-level examples
* give examples a stable `WitSageContracts` module for `galaxy = ["witsage=workspace"]`
* give kernel-backed examples a stable `WitSageKernelSurface` module for
  `galaxy = ["witsage=workspace"]`

Current official surface registry:

* `contract.witsage.dataset.v1`
* `contract.witsage.feature-stats.v1`
* `contract.witsage.classical-model.v1`
* `contract.witsage.kernel-plan.v1`
* `contract.witsage.preprocessing.v1`
* `contract.witsage.evaluation.v1`
