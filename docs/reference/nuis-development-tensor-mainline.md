# Application-Led Mainline Selection

`nuis-dev-tensor-mainline-v1` adds a dependency selection overlay to the existing
architecture/module/function tensor. The canonical plan is
[nuis-development-tensor.mainline.toml](nuis-development-tensor.mainline.toml).
It does not change language semantics, Nustar registration, compiler replacement
authority, or the recursive tensor hierarchy.

## Selection Contract

The plan declares an ID, ordered `goals`, ordered `interrupts`, and `nodes` with
explicit `depends_on` edges. Goal order is deliberate work ordering; node and
dependency registration order do not decide priority.

1. Validate the complete plan against registered tensor coordinates. Reject
   unknown fields, missing assignments, duplicate fields/nodes/array entries,
   unknown coordinates or dependencies, cycles, and unreachable nodes.
2. A node is closed only when its own cell is `stable/100` and every dependency
   is closed. Reopening a prerequisite reopens its dependent closure even if
   the dependent's historical evidence remains stable.
3. Select the first incomplete interrupt, otherwise the first incomplete goal.
   Interrupts are explicitly reviewed correctness regressions such as corruption,
   leaked resources, invalid synchronization, or broken execution. Their tensor
   cells must carry the reproduction and fix acceptance; a low score alone is
   not an interrupt.
4. Walk that root's dependency closure. Only incomplete nodes whose prerequisites
   are closed are actionable. Choose the weakest actionable node by
   `status_rank -> progress -> coordinate`.
5. Publish the goal, selected coordinate, pending goals and a deterministic
   shortest dependency path. Task/handoff ancestry still resolves through the
   existing hierarchy; dependency edges do not become hierarchy parent edges.

JSON and text expose `mainline_protocol`, `mainline_plan_source`,
`mainline_status`, `mainline_id`, `mainline_target`, `mainline_selected`,
`mainline_reason`, `mainline_dependency_path`, and `mainline_pending_goals`.
The existing `weakest_bootstrap_task_card_*` fields retain their transport names
but now carry the selected mainline task. Separate `weakest_bootstrap_*`
statistics remain preparation-gate diagnostics, not current task selection.
No hardware or application name is hardcoded in the algorithm.

Task-card source values are:

* `mainline-goal-dependency-frontier`: selected prerequisite or goal work;
* `mainline-blocking-regression`: an explicit interrupt or its prerequisite;
* `mainline-plan-invalid`: blocked, with no global fallback or actionable card;
* `mainline-plan-complete`: all declared goals and interrupts are closed, not
  every project capability and not the self-hosting programme.

A ready card still requires clean coverage, hierarchy and lineage validation.
A missing or invalid plan never silently falls back to another project area.
An invalid hierarchy or coordinate inventory also prevents a complete card.
The manifest uses a strict TOML subset: quoted ASCII protocol identifiers, string
arrays (inline or multiline), full-line comments, and `[[nodes]]` tables. Escapes,
inline comments and unknown syntax are rejected. It is bounded to 64 KiB and
256 nodes, and is read repository-relatively without writing caches.

## Recalibrated Coordinates

The previous `application-rendering-framework` score accumulated many bounded
proofs under one broad engine label. Its `active/99` is replaced by an
`active/20` scope marker. This is a conservative triage indicator, not a measured
percentage of engine completion or a regression in the verified three-frame
artifact. Existing Metal, replay, completion and GLM evidence stays intact.

| Coordinate Suffix | Current Triage | Evidence Needed To Close |
| --- | --- | --- |
| `ns-nova/persistent-application-session` | active/55 | Static YIR lifecycle registration and scoped live Metal/replay verified; connect the owned GUI event pump without whole-module replay. |
| `application-session/lifecycle-failure-resource-safety` | early/0 | Cancellation, backpressure, failure and in-flight close account for owned resources. |
| `shader/shader-resource-bindings` | early/15 | Image and parameter bindings execute through registered, reflected resource contracts. |
| `ns-nova/interactive-image-workflow` | early/0 | Load, zoom, parameter change, redraw and export operate in one Nuis-owned application. |
| `ns-nova/sustained-runtime-performance` | early/0 | Repeatable cold/warm and sustained measurements with semantic and resource checks. |
| `bootstrap/compiler-component-ownership-transfer` | early/10 | A named Nuis compiler component is selected in an application build with verified rollback. |
| `nuisc/native-cpu-frame-dispatch` | early/10 | Native stateful frame/event execution matches the retained embedded-YIR reference. |

These are new scope-specific coordinates, not retroactive changes to the five
self-hosting preparation gates. `required = false` and `bootstrap_critical = false`
mean they are outside that historical prerequisite set; they are nevertheless
mandatory dependencies/goals of the declared mainline plan.

The current goal is `standard-library/ns-nova/interactive-image-workflow`.
Its first actionable prerequisite is
`standard-library/ns-nova/persistent-application-session`. Shader resources and
failure testing follow that prerequisite, rather than their lower scores
prematurely displacing session work. A reopened shared std/runtime prerequisite
can correctly take priority within the same dependency chain.

The [persistent session boundary](nuis-yir-application-session-v1.md) now calls
explicit compiled Nuis helpers in one execution context. It preserves scalar
aggregate state and provider clock frontiers between separate host calls, drains
per-call frames, and bounds close to one attempt. A second integration path now
uses one real Metal worker through one admitted IPC connection for two independent
events, validates every image pixel, and reproduces both events through replay.
Static `application_sessions` declarations now retain host-call helper roots,
survive canonical YIR/binary embedding and select the scoped session by ID. The
shared YIR contract rejects signature drift before provider connection; changing
the registration also changes source identity and invalidates old replay.
The GUI timer and default compiled-host entry are not migrated yet; this
coordinate remains open, and `active/55` is not an engine-completion percentage.

## Validation And Scope

Each cell separates existing `evidence`, the next action, baseline validation
commands, and the new `expected_artifact`. Passing an existing three-frame test
does not establish persistent state, sustained performance, or compiler ownership.
New acceptance tests must exist and execute before a cell can be closed.

Use `nuis dev-tensor --json` to inspect the plan and task card, and
`cargo test -p nuis --bin nuis dev_tensor -j 1 -- --test-threads=1` to check
selection, malformed plans, dependency regressions, hierarchy, coverage and drift.
Pattern drift checks bind declarations; they are not substitutes for runtime
or device execution tests.

Development policy and module-migration boundaries are described in
[the application-led roadmap](../versioning/nuis-beta-0.11-application-led-mainline.md).
