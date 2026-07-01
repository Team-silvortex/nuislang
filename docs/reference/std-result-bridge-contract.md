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

Bridge-local helper layer:

* these helpers are expected to vary per bridge:
  - `result_map`
  - `result_map_err`
  - `raise_value`
  - `soften_error`
  - `combine_results`
  - `combine_network_results`
  - `map_ok_metric`
  - `draw_metric_from_pass`
* if a bridge needs a specialized helper because current generic/HOF behavior
  is narrower than the desired abstraction, prefer the specialized helper over
  forcing one uniform implementation too early

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
