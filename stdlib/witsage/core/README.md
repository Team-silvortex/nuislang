# `witsage-core`

`witsage-core` is the smallest checked-in source layer of `WitSage`.

It defines classical ML source contracts that can be compiled and inspected
before a full package/runtime layer exists.

Current intended responsibility:

* dataset description
* preprocessing plan description
* feature statistics
* linear model scoring
* k-means style clustering plans
* k-nearest-neighbor style scoring plans
* kernel-facing plan dispatch
* classification and regression-style evaluation summaries

Current source anchor:

* [dataset_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/dataset_recipe.ns)
* [feature_stats_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/feature_stats_recipe.ns)
* [normalization_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/normalization_recipe.ns)
* [train_test_split_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/train_test_split_recipe.ns)
* [linear_score_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/linear_score_recipe.ns)
* [kmeans_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/kmeans_recipe.ns)
* [knn_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/knn_recipe.ns)
* [kernel_plan_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/kernel_plan_recipe.ns)
* [confusion_matrix_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/confusion_matrix_recipe.ns)
* [evaluation_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/witsage/core/evaluation_recipe.ns)
