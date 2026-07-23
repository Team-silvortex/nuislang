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
* per-request adapter bindings then carry the Add output into a real Metal
  `f32` bias kernel, producing and comparing `[11, 17, 23, 29]`
* `nuis-provider-edge-transport-v1` binds that CoreML-to-Metal edge to a GLM
  ownership token, host-visible staging, and producer/consumer request clocks;
  cross-provider edges without valid transport evidence fail before execution
* `nuis-provider-edge-transport-receipt-v1` proves that carrier was
  materialized, consumed, and released with one stable payload hash before the
  provider output is accepted
* `nuis-provider-edge-staging-registry-v1` selects the staging implementation;
  `auto` selects `memory.owned-bytes.v1` and retains
  `host.visible.owned-file.v1` as an explicit compatibility adapter
* `nuis-provider-carrier-input-v1` lets the Metal f32 runner consume the
  CoreML Add output as opaque bytes without a provider-edge file
* CoreML named inputs consume independent inherited carrier descriptors, so
  chained affine and both Add fan-in edges avoid dependency-byte rebundling
* `nuis-provider-carrier-channel-v1` carries those named inputs as binary stdin
  frames with explicit index, length, and FNV-64 instead of hexadecimal argv
* `nuis-provider-carrier-channel-registry-v1` selects `inherited.fd.v1` on Unix
  with child-only inheritance plus packet length/hash validation, while
  `framed.stdin.v1` remains the portable fallback on other hosts
* Unix native runners consume the inherited packet through read-only mmap and
  no-copy frame views; CoreML wraps carrier-backed f32 inputs directly as
  contiguous `MLMultiArray` data pointers
* inherited frames use the page-aligned `NUISPFD1` layout, allowing Metal to
  wrap verified input spans with `newBufferWithBytesNoCopy`
* `nuis-provider-output-carrier-registry-v1` returns CoreML and Metal result
  bytes through verified writable inherited fds on Unix, with hexadecimal
  stdout retained only as a portable fallback
* Unix output observation retains one read-only mmap-backed payload view for
  comparison, hashes, summaries, and dependency metadata instead of a result copy
* writable output carriers create only fixed frame metadata and a sparse aligned
  file span, avoiding output-sized zero-filled construction buffers
* `nuis-provider-output-residency-v1` reports residency, transfer scope,
  observation mode, and device-retention capability without backend assumptions
* `nuis-provider-session-registry-v1` assigns deterministic per-adapter leases,
  ordered lifecycle hooks, and GLM-owned output handles released at graph close
* `nuis-provider-worker-transport-registry-v1` proves ordered requests can share
  one persistent child PID; its portable stdio path remains descriptor-free
* Unix additionally registers `unix.scm-rights.worker.v1`, binding lease and
  request frames to counted `SCM_RIGHTS` descriptors with close-on-error ownership
* a PID-validated persistent child receives two distinct post-spawn descriptors
  under ordered requests and exits through the same session control socket
* `nuis-provider-worker-request-envelope-v1` carries hash-checked opaque binary
  request bytes plus one ordered semantic role for every transferred descriptor
  and returns independently hash-checked opaque result bytes plus the positive
  Nuis ingress status through `NUISPWUR4`
* `std` now provides a Nuis-authored provider-worker lifecycle contract and an
  AOT-executed native loop recipe; C and Objective-C runners remain thin ABI
  probes or one-shot fallbacks rather than worker control-plane implementations
* Nuis-owned worker dispatch bindings now carry opaque request/descriptor-table
  handles and capability hashes without enumerating finite backend combinations
* provider/adapter/operation registration derives an identity-bound operation
  token; the worker reply carries the positive Nuis ingress status through
  `NUISPWUR4`, and non-positive status fails closed before execution is accepted
* final output summaries preserve the operation token, permit contract, permit
  status, worker dispatch status, PID, sequence, descriptor count, and payload
  hash for each CoreML and Metal request
* all four dependency edges reuse sealed producer output carriers directly
  through `provider.output.transfer.v1`, with independent lifecycle receipts

Current alpha-0.17 boundary:

* CoreML and Metal execution is real on the supported Apple smoke host
* lifecycle and dispatch authorization are Nuis-owned
* concrete provider invocation still occurs after worker authorization
* the next step is a provider-neutral execution capsule invoked inside the
  persistent worker, returning a verified output-carrier receipt

Current official surface registry:

* `contract.witsage.dataset.v1`
* `contract.witsage.feature-stats.v1`
* `contract.witsage.classical-model.v1`
* `contract.witsage.kernel-plan.v1`
* `contract.witsage.preprocessing.v1`
* `contract.witsage.evaluation.v1`
* `contract.witsage.pipeline.v1`
