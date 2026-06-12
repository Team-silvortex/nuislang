# `nuis` Compile Workflow For `0.17.0`

This file is the current compiler-facing workflow anchor for the `0.17.0`
line.

The `0.16.0` workflow file established the operational CLI route.

This `0.17.0` file narrows in on something more specific:

* what the frontend actually does today
* in what order it does it
* which crossovers now feel like one compiler story
* which project-aware helper routes are now part of that story
* where the current line is still intentionally narrower

Use it when the question is not only “which command do I run?”, but
“what is the current truthful compile spine?”

For the lowering-facing complement to this file, use:

* [nuis-0.17.0-lowering-capability-map.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-lowering-capability-map.md)

## Core Rule

For the current `0.17.0` line, the mainline story should be read as:

```text
surface syntax
  -> lambda lifting
  -> visible alias / helper assembly
  -> higher-order expansion
  -> match-scrutinee normalization
  -> signature / generic-template split
  -> generic-constraint validation
  -> const assembly
  -> generic specialization + higher-order rewrite closure
  -> frontend lowering to NIR
  -> declared-NIR validation
  -> later YIR / verifier / build steps
```

Short rule:

`frontend truth is no longer only parsing + typing; it is now the ordered composition of lambda, higher-order, generic, control-flow, and async-aware rewrite stages`

## Project-Aware Lowering Rule

For the current `0.17.0` line, multi-file project truth should now be read
with one extra rule:

`a project module is not always meaningful in isolation; helper-visible lowering must preserve project context`

The practical consequence is:

* project analysis should prefer project-aware lowering over isolated
  `lower_ast_to_nir(...)` when the route can depend on local helper modules
* visible `cpu` helpers are now part of the truthful compile story for:
  payload-shape inference, project-link validation, and support/profile
  contract checks
* “entry module only” reasoning is now too narrow for some real `0.17.0`
  project routes

Current anchor surfaces for that rule:

* [project.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project.rs)
* [shader_nova_contracts.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/shader_nova_contracts.rs)
* [multidomain_async.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/multidomain_async.rs)

## Canonical Frontend Order

Today the real front door is still
[lower_project_ast_to_nir](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/mod.rs#L157).

The checked-in order is:

1. `parse_nuis_ast(...)`
2. `expand_module_lambdas(...)`
3. local-helper discovery and visible alias assembly
4. `expand_higher_order_functions(...)`
5. `expand_effectful_match_scrutinees(...)`
6. annotation / export / host-bridge validation
7. visible struct assembly
8. initial signature build:
   generic templates are separated from already-concrete functions here
9. impl lookup build
10. generic-constraint validation
11. const assembly
12. `build_lowered_functions_and_impls(...)`
13. extern lowering
14. `validate_declared_nir_types(...)`

That order matters.

It means the current frontend no longer treats these as unrelated features:

* lambda lifting happens before higher-order specialization
* higher-order expansion happens before the main generic/lowering pipeline
* generic specialization can still trigger more higher-order rewriting later
* match rewriting is part of the same specialization story, not a side topic

## What `build_lowered_functions_and_impls(...)` Now Means

The most important current integration point is
[build_lowered_functions_and_impls](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/specialization_pipeline.rs#L19).

The current responsibility split inside that pipeline is:

* compute function-return expectations for the module and visible helpers
* rewrite generic calls in already-concrete functions
* accumulate newly specialized functions and signatures
* postprocess those specialized functions through higher-order rewrite again
* rerun generic rewrite on higher-order-rewritten specializations
* infer missing return types where the current stack can do so honestly
* lower concrete/specialized functions into `NIR`
* lower impl methods into the same executable function set

Short rule:

`specialization is no longer a one-pass rename; it is a staged closure process`

## Project Compile Spine

Today the believable project-facing compile spine should be read as:

```text
parse project modules
  -> validate module/unit/link/abi declarations
  -> lower entry with visible local helpers
  -> lower helper-sensitive project analyses with project context
  -> validate project links/contracts against NIR
  -> lower to YIR
  -> apply support-module profiles
  -> materialize project bridge/type contracts
  -> validate project links/contracts against YIR
```

Short rule:

`project compilation is no longer only “frontend once, then YIR”; helper-aware project validation is now part of the mainline compile truth`

The most important current project-aware crossover is that helper modules can
now influence:

* route payload inference
* project-link NIR validation
* shader packet contract discovery
* kernel/data bridge contract materialization
* network profile usage validation

That is the new honest read for checked-in project work.

## Current Generic Rewrite Truth

The current generic rewrite spine is now carried by:

* [generic_rewrite/mod.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/generic_rewrite/mod.rs)
* [generic_rewrite/blocks.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/generic_rewrite/blocks.rs)
* [generic_rewrite/exprs.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/generic_rewrite/exprs.rs)
* [generic_rewrite/hoists.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/generic_rewrite/hoists.rs)

For the current `0.17.0` line, that subsystem should be read like this:

* expression rewrite can infer generic substitutions from:
  direct args, expected return type, alias-aware struct/payload routes
* specialization can recurse:
  a specialized function body may itself trigger more specializations
* specialization is higher-order-aware:
  a newly specialized function body can first be rewritten through HO
  expansion, then continue through generic rewrite again
* match-arm pattern bindings now contribute type information to generic
  rewriting inside arm bodies
* direct result-wrapper hoists participate in the same generic/HO context

This is the important new closure claim:

`generic recursive async bodies with payload-alias higher-order lambdas are now part of the checked-in frontend story`

The newest anchor for that claim is
[tests_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_higher_order.rs#L530).

## What Already Feels “Internalized”

For `0.17.0`, these routes are no longer best described as isolated tricks:

* explicit generic specialization in ordinary helper calls
* expected-type-driven specialization from returns and nested call positions
* alias-aware payload and struct constructor inference
* generic method-bound validation through `if` and `match` control flow
* lambda-lifted higher-order specialization
* async generic specialization through `await`
* recursive async specialization
* higher-order specialization inside recursive async generic bodies
* project-backed compile proofs for async/generic/higher-order combinations

Short rule:

`if a route crosses generic + alias + lambda + async + match, it should increasingly be assumed to belong to the same mainline until a specific missing case proves otherwise`

## Cross-Domain Helper Closure

For `0.17.0`, the project-aware story is now strong enough to describe three
specific helper-mediated cross-domain closures as checked-in truths:

* `cpu helper -> shader/data`:
  helper-mediated packet creation, uplink/downlink use, route payload
  inference, and project-link validation
* `cpu helper -> kernel/data`:
  helper-mediated kernel profile reads, fabric roundtrip payload inference,
  and project-link validation
* `cpu helper -> network`:
  helper-mediated network profile reads surviving project-link NIR validation

Current checked-in anchors:

* [shader_nova_contracts.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/shader_nova_contracts.rs)
* [multidomain_async.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/multidomain_async.rs)

For `shader`, the current practical reading order is now much clearer than it
was earlier in the line:

* packet entry:
  `shader_packet_profile_demo`
* packet bridge:
  `shader_packet_bridge_demo`
* sync result spine:
  `shader_result_profile_demo`
* dual draw/render spine:
  `shader_draw_render_profile_demo`
* async policy:
  `shader_async_policy_profile_demo`
* async fallback:
  `shader_async_fallback_profile_demo`
* async schedule:
  `shader_async_schedule_profile_demo`
* async fanin:
  `shader_async_fanin_profile_demo`
* async windowed batch:
  `shader_async_windowed_batch_profile_demo`

Short rule:

`0.17.x shader should now be taught as one continuous packet -> bridge -> result -> async scheduling story, not as isolated packet and async tricks`

This does not mean every future bridge pattern is complete.

It does mean the current line should no longer describe helper-mediated project
validation as an accidental or fragile side route.

## What Is Still Narrower

The current line is much stronger than before, but it is not “everything”.

Important honest limits still include:

* general iterative/backedge loop lowering outside the counted/carry/flow/post-flow families
* async boundary ownership remains intentionally strict
  `ref`, `Task<...>`, and `*Result<...>` families still have narrow allowed crossings
* host/runtime truth is still more limited than compile-time truth
  especially around syscall/network/runtime uncertainty
* some alias constructor routes are still intentionally conservative when the
  alias target is not transparent enough for local inference

Short rule:

`0.17.0` is integration-first, not “all restrictions removed”

## Recommended Debug Reading Order

When a route fails, the shortest truthful reading drill is now:

1. check whether the failure happens before or after
   [lower_project_ast_to_nir](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/mod.rs#L157)
   completes
2. if it is frontend-shaped, ask which stage owns it:
   lambda, higher-order, generic rewrite, match normalization, async context,
   or lowering
3. inspect the closest checked-in regression family:
   `tests_generics`, `tests_higher_order`, `tests_generic_constraints`,
   `tests_control_flow`, `tests_async_runtime`, or project compile harnesses
4. use CLI dumps in this order:
   `dump-ast -> dump-nir -> dump-yir`

Use this rough split:

* `dump-ast`:
  parser / surface-shape / annotation confusion
* `dump-nir`:
  lambda / generic / higher-order / match / async frontend confusion
* `dump-yir`:
  lowering / scheduler / verifier / later bridge confusion

## Current Mainline Regression Spine

For the current line, the smallest believable regression stack is:

```text
frontend generics + higher-order probes
  -> frontend async crossover probes
  -> lowering async/runtime probes
  -> checked-in project compile harnesses
```

Current anchor families:

* frontend generics:
  [tests_generics.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generics.rs)
* frontend higher-order:
  [tests_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_higher_order.rs)
* frontend generic constraints:
  [tests_generic_constraints.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_constraints.rs)
* frontend control flow:
  [tests_control_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_control_flow.rs)
* lowering async/runtime:
  [tests_async_runtime.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_async_runtime.rs)
* project compile anchors:
  [task_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/task_compile.rs),
  [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs),
  [state_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/state_compile.rs)
* project integration probes:
  [shader_nova_contracts.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/shader_nova_contracts.rs),
  [multidomain_async.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/multidomain_async.rs)

Inside `shader_nova_contracts`, the current line now keeps these checked-in
project truths honest:

* packet/profile seed and slot discovery
* packet -> data bridge -> frame present closure
* pass/draw/render `ShaderResult` observer closure
* draw/render dual-result coexistence
* async policy aggregation
* async fallback with timed task route
* async schedule with explicit spawn/join_result staging
* async fanin across pass/frame observer tasks
* async windowed batch summary capture

Short rule:

`a claim is much closer to internalized when it survives frontend probes, lowering probes, and project compile harnesses together`

## CLI Workflow Still Inherits `0.16.0`

The operational command route remains the same default path introduced in
[nuis-0.16.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-compile-workflow.md):

```text
project-doctor -> check -> test -> build -> verify-build-manifest -> release-check
```

The difference in `0.17.0` is not the command list.

The difference is that those commands now stand on a more coherent frontend
story, especially across:

* generics
* higher-order calls
* control-flow-local rewrites
* async recursion
* project-backed compile closure
* helper-aware project validation closure

## Practical Reading Rule

When someone asks “what is the current `nuis` compiler workflow?”, the honest
short answer for `0.17.0` is:

`one CLI route, one frontend pipeline, one specialization closure story, and several still-honest runtime/lowering boundaries`
