# `.ns` Examples

This folder contains the current front-end source examples for:

* `mod <domain> <unit>` parsing
* `AST -> NIR -> YIR` lowering
* source-language ownership and async/task staging
* source-level host facade mirrors before project expansion

Canonical short map:

* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  Use that file first when you want the shortest current route.
* [docs/versioning/nuis-alpha-0.4-system-inventory.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.4-system-inventory.md)
  Use that file when the question is whether a source route is current
  frontdoor, companion-only, or only predecessor/probe material.

Current source-style rule:

* checked-in `.ns` examples now prefer `ptr.value`, `ptr.next`, `buffer.len`,
  and `buffer[index]`
* if you need the lowering/builtin explanation behind that surface, use
  [docs/reference/address-surface-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/address-surface-contract.md)

Alpha hardening rule:

* this tree is still `active` in the
  [examples freshness audit](/Users/Shared/chroot/dev/nuislang/docs/examples-freshness-audit.md)
* its best `alpha-0.4.*` role is narrow semantic anchoring, not competing with
  multi-file project onboarding
* the current goal is to keep one short basic-language ladder, one short
  ownership/task ladder, and one short host-facade ladder obvious before the
  longer single-file tail is reclassified further

Subdirectories:

* [core](/Users/Shared/chroot/dev/nuislang/examples/ns/core/README.md)
* [types](/Users/Shared/chroot/dev/nuislang/examples/ns/types/README.md)
* [data](/Users/Shared/chroot/dev/nuislang/examples/ns/data/README.md)
* [ffi](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/README.md)
* [memory](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/README.md)
* [demos](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/README.md)

## Current Frontdoor Ladders

If you only want the shortest current `.ns` route, start with these ladders.

Basic language ladder:

* [hello_world.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_world.ns)
* [hello_if.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_if.ns)
* [hello_ref_struct.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/types/hello_ref_struct.ns)

This is the shortest source-facing route for:

* smallest `mod cpu Main` entry
* ordinary conditional shape
* smallest ownership-sensitive aggregate example

Ownership and task ladder:

* [hello_borrow_end.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_borrow_end.ns)
* [hello_task_glm_value_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_value_path.ns)
* [hello_task_result_control_flow.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_result_control_flow.ns)

This is the shortest source-facing route for:

* explicit local borrow closure
* narrow task completed-value observation
* single-file `await` / `?` / control-flow composition

Host facade ladder:

* [hello_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_ffi.ns)
* [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns)
* [hello_path_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_runtime_facades.ns)

This is the shortest source-facing route for:

* plain host symbol facade reading
* input/runtime facade mirroring
* path/runtime facade mirroring

## Companion Detail

## Short Source Map

* basic language
  - [hello_expr.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_expr.ns)
  - [hello_let_expr.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_let_expr.ns)
  - [hello_call.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_call.ns)
  - [hello_method.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_method.ns)
* types and ownership
  - [hello_task_glm_status_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_status_path.ns)
  - [hello_task_glm_lifecycle_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle_path.ns)
  - [hello_task_glm_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_compare.ns)
* data path
  - [hello_data.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_data.ns)
  - [hello_data_window.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_data_window.ns)
  - [hello_instantiate.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_instantiate.ns)
* host facades
  - [hello_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_ffi.ns)
  - [hello_c_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_c_ffi.ns)
  - [hello_cli_host_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cli_host_facades.ns)
  - [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns)
  - [hello_task_cli_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns)
  - [hello_path_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_runtime_facades.ns)
* single-file demo path
  - [window_controls_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/window_controls_demo.ns)
  - [shader_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/shader_profile_demo.ns)
  - [kernel_profile_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/kernel_profile_demo.ns)
  - read these as compact mirrors, not as the canonical project-validation
    route for domain-heavy lanes

## Reading Rule

* use the frontdoor ladders first
* use the companion detail map after you know which source lane you care about
* use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  for the shortest repo-level route
* use local subdirectory READMEs for area detail
* use [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
  once a route grows into multi-file project form
* use [examples/ns/ffi/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/README.md)
  when you want the host facade long tail
* use [examples/ns/memory/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/README.md)
  when you want task/ownership detail

## Notes

* `mod` is a top-level builtin declaration, not a nested construct
* `cpu` is currently the only domain that can declare `async fn`
* current explicit task-style async surface is intentionally still small:
  `spawn`, `join`, `cancel`, `timeout`, `join_result`, and `task_*`
