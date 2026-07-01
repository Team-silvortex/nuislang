# `nuis` Compile Workflow For `0.18.0`

This file is the current compiler-facing workflow anchor for the `0.18.0`
line.

The `0.17.0` workflow file explained the integration-first frontend/project
story.

This `0.18.0` file narrows the story to one sharper question:

`what is the honest compile mainline now that control flow is being promoted from partial support to project-backed workflow truth?`

Use it when the question is not only “what compiles?”, but:

* which ordered compiler spine we should now teach as current truth
* how `if` / `match` / `while` now fit into that spine
* which project-backed anchors actually defend the story
* how `state`, `task`, `shader`, and `network` now connect back to one
  compile workflow

Use it together with:

* [nuis-0.18.0-mainline-goals.md](nuis-0.18.0-mainline-goals.md)
* [nuis-0.18.0-control-flow-completion-plan.md](nuis-0.18.0-control-flow-completion-plan.md)
* [nuis-0.17.0-lowering-capability-map.md](nuis-0.17.0-lowering-capability-map.md)
* [nuis-0.18.0-host-boundary-address-abi.md](nuis-0.18.0-host-boundary-address-abi.md)

## Core Rule

For the current `0.18.0` line, the mainline should now be read as:

```text
surface syntax
  -> lambda lifting
  -> visible helper / alias assembly
  -> higher-order expansion
  -> effectful match-scrutinee normalization
  -> signature / generic-template split
  -> generic-constraint validation
  -> const assembly
  -> generic specialization + higher-order closure
  -> frontend lowering to NIR
  -> control-flow shape survival into project-aware lowering
  -> YIR loop-family / async / verifier / contract steps
```

Short rule:

`0.18.0` compile truth is no longer only “frontend integration works”; it is “frontend integration plus project-backed control-flow truth survives into lowering families we can name directly”

## Current CLI Frontdoor

At the tool level, the current `0.18.0` compile workflow should now be taught
through one grouped frontdoor route instead of several unrelated commands.

Today the honest outermost reading order is:

```text
nuis status / nuis help
  -> nuis workflow
  -> nuis project-doctor / nuis project-status / nuis scheduler-view
  -> nuis check
  -> nuis test
  -> nuis build
  -> nuis release-check
```

The practical reason is simple:

* `status` and `help` now present the default compile frontdoor
* `workflow` classifies the input into single-file vs project-facing routes
* `project-doctor`, `project-status`, and `scheduler-view` now all expose the
  same grouped frontdoor summary shape before their deeper detail payloads
* `check/test/build/release-check` remain the action spine rather than being
  replaced by a new hidden orchestration layer

Short rule:

`the current CLI story is now one frontdoor family with deeper detail commands, not several disconnected command descriptions`

## What Changed In `0.18.0`

The practical change in this line is not that control flow suddenly appeared.

The practical change is that project-backed control-flow anchors now cover a
much larger part of the ordinary source-level loop family.

That means the compile workflow should now be explained through four linked
tracks instead of one generic frontend story:

* state/control-flow projects
* task/async-control-flow projects
* shader/helper-mediated project routes
* network/http/session project routes

Short rule:

`if a route matters to the current mainline, we increasingly want a real project anchor for it`

## Canonical Frontend Order

The real frontend entry remains:

* [lower_project_ast_to_nir](../../tools/nuisc/src/frontend/mod.rs#L157)

The checked-in order still matters:

1. `parse_nuis_ast(...)`
2. `expand_module_lambdas(...)`
3. visible helper discovery and alias assembly
4. `expand_higher_order_functions(...)`
5. `expand_effectful_match_scrutinees(...)`
6. annotation / export / bridge validation
7. visible struct assembly
8. signature build with generic-template separation
9. impl lookup build
10. generic-constraint validation
11. const assembly
12. `build_lowered_functions_and_impls(...)`
13. extern lowering
14. `validate_declared_nir_types(...)`

Short rule:

`control flow is now part of the same ordered frontend story as lambda, higher-order, generic, and async rewrite closure`

One extra `0.18.0` boundary rule now matters here:

* internal `ref` / address semantics are real inside the frontend and verifier
* ordinary `extern` ABI surfaces are still value-only
* so host-boundary pointer shapes are intentionally rejected during declared
  NIR type validation instead of being allowed to drift into misleading lowering

## Project Compile Spine

Today the believable project-facing compile spine should be read as:

```text
parse project modules
  -> validate module/unit/link/abi declarations
  -> lower entry with visible local helpers
  -> keep helper-aware project context during control-flow-sensitive lowering
  -> validate project links/contracts against NIR
  -> lower to YIR
  -> materialize loop-family / async / contract truth
  -> validate project links/contracts against YIR
```

Short rule:

`0.18.0` project truth is now partly defined by whether control-flow-heavy projects keep their shape through NIR and into named YIR loop families`

## Current Control-Flow Families We Can Name Honestly

The current line can now point to these loop families as real workflow truth:

* `loop_while_i64`
* `loop_while_i64_chain`
* `loop_while_i64_cond_chain`
* `loop_while_i64_flow_chain`
* `loop_while_i64_flow_cond_chain`
* `loop_while_i64_post_flow_chain`
* `loop_while_i64_post_flow_cond_chain`

And on the async/lowering side, the repo still carries:

* `loop_while_i64_async_chain`
* `loop_while_i64_async_flow_chain`
* `loop_while_i64_async_flow_cond_chain`
* `loop_while_i64_async_post_flow_chain`
* `loop_while_i64_async_post_flow_cond_chain`

Short rule:

`0.18.0` should be read as the line where these families become easier to map back to ordinary source examples`

## Current Project-Backed State Spine

The strongest current state/control-flow route is now broad enough to teach as
one progression:

* basic loops:
  `counted_while_demo`,
  `inequality_while_demo`
* carry loops:
  `accumulating_while_demo`,
  `chained_while_demo`
* guarded / bounded post-flow:
  `bounded_while_demo`,
  `equality_while_demo`
* branch-driven cond flow:
  `branching_while_demo`,
  `match_branching_while_demo`,
  `bool_match_branching_while_demo`,
  `lambda_match_branching_while_demo`
* flow / continue crossover:
  `flow_continuing_while_demo`,
  `equality_branching_while_demo`,
  `lambda_match_flow_continuing_while_demo`,
  `lambda_match_or_flow_continuing_while_demo`
* break / post-flow / carry crossover:
  `post_flow_breaking_while_demo`,
  `post_flow_continuing_while_demo`,
  `post_flow_branching_while_demo`,
  `post_flow_branching_continuing_while_demo`,
  `carried_breaking_while_demo`,
  `double_branching_while_demo`

Primary compile gate:

* [state_compile.rs](../../tools/nuisc/tests/state_compile.rs)

Short rule:

`state is now the clearest current project-backed proof that source-level while/control-flow shapes map into named lowering families`

## Current Project-Backed Task Spine

The strongest current task/control-flow route is now:

* result/lifecycle base:
  `task_lifecycle_branch_demo`,
  `task_result_family_branch_demo`,
  `task_result_policy_branch_demo`,
  `task_fallback_branch_demo`
* batch and windowed batch:
  `task_batch_branch_demo`,
  `task_result_batch_branch_demo`,
  `task_windowed_batch_branch_demo`,
  `task_result_windowed_batch_branch_demo`
* compile-adjacent async recursion and specialization anchors:
  recursive, mutual-recursive, generic-recursive, and memory/session task demos

Primary compile gate:

* [task_compile.rs](../../tools/nuisc/tests/task_compile.rs)

Short rule:

`task is now the clearest current project-backed proof that async result selection, timeout/fallback, and batch/windowed summaries belong to the same compile story`

## Current Project-Backed Shader Spine

Shader is still best read through project-aware helper closure.

Current practical reading order:

* packet entry:
  `shader_packet_profile_demo`
* packet bridge:
  `shader_packet_bridge_demo`
* sync result:
  `shader_result_profile_demo`
* draw/render split:
  `shader_draw_render_profile_demo`
* async policy/fallback/schedule/fanin/windowed:
  `shader_async_policy_profile_demo`,
  `shader_async_fallback_profile_demo`,
  `shader_async_schedule_profile_demo`,
  `shader_async_fanin_profile_demo`,
  `shader_async_windowed_batch_profile_demo`

Primary integration gate:

* [shader_nova_contracts.rs](../../tools/nuisc/src/project/tests/shader_nova_contracts.rs)

Short rule:

`shader now reads as packet -> bridge -> result -> async summary, not isolated packet tricks`

## Current Project-Backed Network Spine

Network is still the widest current project ladder, but the part that now
matters most to the honest compile story is narrower:

* HTTP/session/request anchors:
  `net_http_request_recipe_demo`,
  `net_http_client_lane_recipe_demo`,
  `net_http_service_lane_recipe_demo`
* HTTP-ish/session packet anchors:
  `net_httpish_header_session_recipe_demo`,
  `net_httpish_client_session_packet_recipe_demo`,
  `net_httpish_service_session_packet_recipe_demo`
* current loop/session bridge anchor:
  `net_http_session_loop_bridge_recipe_demo`

Primary compile gate:

* [network_compile.rs](../../tools/nuisc/tests/network_compile.rs)

Short rule:

`network is currently the broadest proof that helper-heavy session/result/http routes can still survive project compilation honestly`

## Practical Reading Order

If you want the shortest current `0.18.0` compile story, read in this order:

1. [nuis-0.18.0-mainline-goals.md](nuis-0.18.0-mainline-goals.md)
2. [nuis-0.18.0-control-flow-completion-plan.md](nuis-0.18.0-control-flow-completion-plan.md)
3. [state_compile.rs](../../tools/nuisc/tests/state_compile.rs)
4. [task_compile.rs](../../tools/nuisc/tests/task_compile.rs)
5. [shader_nova_contracts.rs](../../tools/nuisc/src/project/tests/shader_nova_contracts.rs)
6. [network_compile.rs](../../tools/nuisc/tests/network_compile.rs)

If you want the shortest source/project route, use:

1. [examples/projects/state/README.md](../../examples/projects/state/README.md)
2. [examples/projects/task/README.md](../../examples/projects/task/README.md)
3. [examples/projects/domains/README.md](../../examples/projects/domains/README.md)

## Current Honest Gate

For `0.18.0`, the smallest believable compile gate should now be read as:

```text
frontend control-flow probes
  -> lowering loop-family probes
  -> state project anchors
  -> task project anchors
  -> shader helper/project anchors
  -> network/session/http anchors
```

Short rule:

`if a claimed mainline route only survives unit probes but not the project gates, it is not yet honest enough for the current line`
