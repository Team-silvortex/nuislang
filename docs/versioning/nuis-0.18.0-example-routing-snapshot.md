# `nuis` 0.18.0 Example Routing Snapshot

This file is the short history anchor for the example-tree cleanup that landed
during the `0.18.*` line.

It is not a language-feature snapshot.

It is the point where the repository’s checked-in examples started reading more
like one teachable route system and less like one flat archive.

## What This Snapshot Means

For `0.18.*`, the example tree should now be read in four roles:

* frontdoor
  the shortest current entrypoints we actively recommend
* companion
  narrow feature, contract, or regression anchors
* probe
  validation, experiment, or future-facing routes that still matter, but are
  not default onboarding material
* legacy
  intentionally historical bridge material

Short rule:

`present in the tree no longer means equally recommended`

## Highest-Signal Current Surface

The most important example-tree truths now are:

* top-level routers are shorter and more honest:
  - [examples/README.md](/Users/Shared/chroot/dev/nuislang/examples/README.md)
  - [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
  now describe frontdoor/companion/probe/legacy roles explicitly
* subtree READMEs now agree with the current mainline map:
  - task
  - tooling
  - filesystem
  - domains
  all now describe a narrow frontdoor plus grouped companion families
* the repository now has one explicit cleanup board:
  - [docs/examples-freshness-audit.md](/Users/Shared/chroot/dev/nuislang/docs/examples-freshness-audit.md)
  tracks subtree status, current frontdoor anchors, archived routes, and
  future cleanup targets
* tooling now has a real legacy split:
  - older low-level shell, line-input, automation, and report probes now live
    under [examples/legacy/tooling](/Users/Shared/chroot/dev/nuislang/examples/legacy/tooling)
  - current tooling frontdoor starts from `cli_runtime_demo`,
    `command_runtime_demo`, and `workflow_runtime_demo`
* filesystem routing is more complete:
  - current companion routes now clearly include path mutation/output and
    directory removal/output cases instead of leaving them implicit
* domain routing is more honest:
  - network runtime validation probes and experiment routes are now called out
    as probes instead of being left to read like ordinary frontdoor examples
* task routing is more explicit:
  - current task companion families are now listed more completely
  - `task_join_nonconsuming_probe_demo` is now clearly marked as future/probe
    material instead of an equal-entry route

## What Is Still Intentionally Narrow

This snapshot should still avoid overclaiming:

* probe routes are still inside the active tree when they support living
  runtime-validation or design documents
* the task probe route still remains in-place because GLM and hot-sync docs
  point to it directly
* many `path_*` and network ladder examples are still intentionally dense; they
  are better grouped than deleted
* this snapshot improves readability and routing honesty, not feature
  completeness by itself

## Best Current Reading Order

For example-tree work on the `0.18.*` line, the shortest route is:

1. [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
2. [docs/examples-freshness-audit.md](/Users/Shared/chroot/dev/nuislang/docs/examples-freshness-audit.md)
3. [examples/README.md](/Users/Shared/chroot/dev/nuislang/examples/README.md)
4. [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
5. one local subtree README:
   - [examples/projects/task/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task/README.md)
   - [examples/projects/tooling/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/README.md)
   - [examples/projects/filesystem/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/README.md)
   - [examples/projects/domains/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/README.md)

## Recommended Practical Checks

For quick documentation sanity:

```bash
rg -n "frontdoor|probe|legacy|examples-freshness-audit" examples docs
```

For quick route validation:

```bash
cargo run -p nuis -- status
cargo run -p nuis -- workflow examples/projects/window_controls_demo
cargo run -p nuis -- workflow examples/projects/tooling/cli_runtime_demo
cargo run -p nuis -- workflow examples/projects/task/task_runtime_demo
```

## Rule Of Thumb

If the early `0.18.*` line was about making more compile truths line up, this
example-routing snapshot is about making those truths easier to find:
the repository should increasingly teach one clear entry path, then one clear
companion path, then explicit probe or legacy paths instead of making readers
guess which examples are current.
