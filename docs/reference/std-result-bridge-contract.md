# `std` Result Bridge Contract

This note records the current shared shape for the checked-in `std` result
bridge modules.

Current bridge modules:

* [result_enum_runtime.ns](../../stdlib/std/result_enum_runtime.ns)
* [net_result_enum_recipe.ns](../../stdlib/std/net_result_enum_recipe.ns)
* [task_result_enum_recipe.ns](../../stdlib/std/task_result_enum_recipe.ns)
* [shader_result_enum_recipe.ns](../../stdlib/std/shader_result_enum_recipe.ns)

Shared contract:

* success paths normalize into `Result<i64, ErrorEnvelope>`
* failure paths normalize into `ErrorEnvelope`
* `ErrorEnvelope` always contains:
  - `error: ErrorCode`
  - `diagnostic: DiagnosticCode`

Stable shared surface:

* error classification vocabulary:
  - `kind_invalid_input`
  - `kind_result_missing`
  - `kind_diagnostic_message`
* envelope construction vocabulary:
  - `build_error_code`
  - `build_diagnostic_code`
  - `build_error_envelope`
* shared result helpers:
  - `result_is_err`
  - `result_unwrap_or`
  - `result_score`
* bridge entry vocabulary:
  - `lift_error_kind`
  - `lift_host_result`
  - `lift_network_result`
  - `lift_task_result`
  - `lift_pass_result`
  - `lift_frame_result`

Lowering/runtime contract:

* `match` over `Result.Ok` / `Result.Err` lowers through explicit
  `variant_is` and `variant_field` nodes
* selecting between sibling enum variants must preserve a tagged union view in
  both LLVM lowering and the YIR interpreter/AOT packer
* payload extraction from the non-active variant is legal only as part of the
  guarded-select shape emitted by branch lowering; the union value preserves
  those payloads so strict AOT execution does not reject valid branch-selected
  Result code
* plain wrong-variant field access on a concrete non-union struct remains an
  error

Bridge-local helper layer:

* these helpers are expected to vary per bridge:
  - `result_map`
  - `result_map_err`
  - `raise_value`
  - `soften_error`
  - `combine_results`
  - `combine_network_results`
  - `map_ok_metric`
* if a bridge needs a specialized helper because current generic/HOF behavior
  is narrower than the desired abstraction, prefer the specialized helper over
  forcing one uniform implementation too early

Current backed coverage:

* run-backed native CPU:
  [result_enum_runtime_demo](../../examples/projects/tooling/result_enum_runtime_demo)
* run-backed task bridge:
  [task_result_enum_demo](../../examples/projects/task/task_result_enum_demo)
* run-backed network bridge:
  [net_result_enum_recipe_demo](../../examples/projects/domains/net_result_enum_recipe_demo)
* artifact-doctor/build-report backed shader/data AOT bundle:
  [shader_result_enum_demo](../../examples/projects/domains/shader_result_enum_demo)
  as a ready-to-run `window-aot-bundle` with CPU/data/shader domain units,
  Metal shader lowering, and a heterogeneous bundle-pack link plan

Current remaining gap:

* branch-local shader effects inside `Result.Ok` arms are intentionally not
  claimed yet; shader effects should be hoisted into a straight-line chain
  before Result scoring until conditional effect lowering grows that shape

Current stable error kind mapping:

* `1202`
  meaning:
  `kind_result_missing`
  intended use:
  missing/failed/non-ready result family fallback
* `1303`
  meaning:
  `kind_diagnostic_message`
  intended use:
  diagnostic-oriented or secondary failure fallback

Current practical rule:

* keep checked-in bridge modules self-contained for now
* duplication across bridge modules is currently intentional
* do not force a shared import layer until source-module reuse and generic
  helper reuse are stable enough to reduce code instead of adding fragility
* when aligning naming, prefer matching the stable shared surface first before
  deduplicating bridge-local helpers

Recommended reading order:

1. [stdlib/core/result_patterns.ns](../../stdlib/core/result_patterns.ns)
2. [stdlib/std/result_enum_runtime.ns](../../stdlib/std/result_enum_runtime.ns)
3. [stdlib/std/task_result_enum_recipe.ns](../../stdlib/std/task_result_enum_recipe.ns)
4. [stdlib/std/net_result_enum_recipe.ns](../../stdlib/std/net_result_enum_recipe.ns)
5. [stdlib/std/shader_result_enum_recipe.ns](../../stdlib/std/shader_result_enum_recipe.ns)
