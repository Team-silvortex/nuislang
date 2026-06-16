# `nuis` `0.20.0` `std` Refactor Frontdoor

This file is the short frontdoor for the current `std` refactor effort.

It is not a promise that the filesystem has already been fully reshaped.
It is the current repository truth about:

* which `std` lanes are already real enough to treat as mainline
* which layers should be normalized first
* which moves are still documentation/router work rather than source relocation

Read this together with:

* [../../stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
* [../reference/std-mainline-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-mainline-layering-contract.md)
* [../reference/std-net-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-net-layering-contract.md)
* [nuis-0.20.0-branch-runtime-lowering-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-branch-runtime-lowering-matrix.md)
* [nuis-0.20.0-compile-gap-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-compile-gap-checklist.md)

## Short Rule

For `0.20.*`, `std` should stop reading like one giant flat module bucket.

The immediate goal is not:

`move every file now`

The immediate goal is:

`make the current mainline lanes obvious, make ownership clearer, and only then move source assets when the route is already readable`

## Current Refactor Target

The current `std` surface is broad enough that it should be read as five
mainline clusters:

1. task/runtime
2. host I/O and text
3. filesystem/path/location
4. command/workflow/tooling
5. net/session

Short rule:

`new std work should first decide which cluster it belongs to before adding another cross-cutting umbrella`

## What Is Already Good Enough To Treat As Mainline

### Task/runtime lane

Already coherent enough:

* `task_runtime_recipe`
* `task_status_recipe`
* `task_value_recipe`
* `task_compare_recipe`
* `task_lifecycle_recipe`
* task batch/result/windowed/task-cli companions
* local lane router:
  [../../stdlib/std/task/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/task/README.md)

Why this is mainline:

* the layering contract is already documented
* async/task lowering now has real regression depth
* task-facing wrappers are already part of the checked-in compile spine

### Host I/O and text lane

Already coherent enough:

* `io_runtime_recipe`
* `stdin_runtime_recipe`
* `tty_runtime_recipe`
* `input_runtime_recipe`
* `terminal_io_recipe`
* `host_text_runtime_recipe`
* `text_format_runtime_recipe`
* `json_runtime_recipe`
* `text_json_recipe`
* local lane router:
  [../../stdlib/std/host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md)

Why this is mainline:

* it already reads like a narrow pure layer plus composition layer
* source/project mirrors already exist

### Filesystem/path/location lane

Already coherent enough:

* `path_runtime_recipe`
* path inspection/mutation companions
* `fs_metadata_runtime_recipe`
* `directory_runtime_recipe`
* `stat_runtime_recipe`
* `directory_stat_recipe`
* `cwd` / `temp` / `home` / `location`
* local lane router:
  [../../stdlib/std/filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)

Why this is mainline:

* the surfaces are broad, but the reading order is already teachable
* the lane mostly suffers from flat listing, not from missing core vocabulary

### Command/workflow/tooling lane

Already coherent enough:

* `command_runtime_recipe`
* `subprocess_runtime_recipe`
* `workflow_runtime_recipe`
* CLI session/report/build/workflow companions
* local lane router:
  [../../stdlib/std/tooling/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/tooling/README.md)

Why this is mainline:

* there is already a consistent workflow frontdoor story
* the lane already has reusable execution context/report/stage vocabulary

### Net/session lane

Already coherent enough:

* endpoint / transport / syscall / socket / control / protocol / http
* result spine / task spine / session

Why this is mainline:

* the lane already has its own router and contract
* the mainline problem is readability and ownership, not absence of material

## What The First Refactor Pass Should Actually Do

### Pass 1. Frontdoor normalization

Do now:

* make `stdlib/std/README.md` point readers into the five clusters first
* make `docs/current-mainline-map.md` and versioning docs point to the current
  `std` refactor frontdoor
* keep network using the dedicated router instead of re-listing every file in
  multiple places

Do not do yet:

* mass file moves
* renaming every module family
* inventing a package-system answer before the compile chain is ready

### Pass 2. Lane-level source normalization

Do next:

* tighten one lane at a time
* reduce repeated flat inventories when a lane already has a local router
* prefer `pure runtime recipe -> composition recipe` as the canonical local
  order

Good candidates:

* command/workflow/tooling
* host I/O and text
* task/runtime

### Pass 3. Filesystem layout reshaping

Only do after the lane already reads cleanly:

* move families into clearer subdirectories
* adjust module metadata/router files
* update example/doc mirrors together

Short rule:

`if a lane still needs a long prose explanation to understand its order, it is too early to do the higher-risk filesystem move`

## Current Non-Goals

This refactor frontdoor does not yet claim:

* that `std` is already a finished auto-import library
* that all runtime facades are stable public API
* that network naming is frozen
* that every recipe belongs in its final directory today

That restraint matters for `0.20.*`.

The line should optimize for:

* clearer ownership
* clearer reading order
* easier regression placement
* fewer accidental duplicate mini-frontdoors

## Practical Reading Order For The Current Refactor

If you only want the shortest route:

1. read [../../stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
2. read [../reference/std-mainline-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-mainline-layering-contract.md)
3. read [../reference/std-net-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-net-layering-contract.md)
4. then pick exactly one lane to normalize further

Good current lane order:

1. command/workflow/tooling
2. host I/O and text
3. task/runtime
4. filesystem/path/location
5. net/session

The reason for that order is simple:

* command/workflow/tooling already wants a cleaner frontdoor
* host I/O/text has relatively low semantic risk
* task/runtime is important but tied to active compiler/runtime truth
* filesystem/path/location is broad but less ambiguous
* net/session is the richest lane and should move after the simpler clusters
  have been normalized first

## Immediate Repository Rule

For the current line, treat this as the first stable refactor rule:

`when adding or editing std source, improve the lane frontdoor first; only then widen the lane or move files`
