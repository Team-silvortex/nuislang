# `nuis` Compile Workflow For `0.16.0`

This is the current recommended compile workflow to stabilize and document for
the `0.16.0` line.

The goal is simple:

* one front door: `nuis`
* one default validation path: `doctor -> check -> test -> build`
* one release gate: `release-check`
* one debug path when things go wrong: `dump-ast -> dump-nir -> dump-yir`

Use this file as the shortest operational route for day-to-day project work.

## Canonical Stages

```text
project or source input
  -> nuis project-doctor
  -> nuis check
  -> nuis test
  -> nuis build
  -> nuis verify-build-manifest
  -> nuis release-check
```

In practice:

* `project-doctor` tells you whether the project shape is healthy enough to
  work on.
* `check` validates parsing, frontend, project wiring, `NIR`, `YIR`, ABI
  selection, and verifier rules without producing the final build directory.
* `test` runs language-level `test(...)` functions and project-declared test
  inputs.
* `build` emits the AOT output bundle.
* `verify-build-manifest` checks the generated build manifest directly.
* `release-check` is the canonical final gate because it runs `check`, `build`,
  and manifest verification together.

## Default Project Workflow

For a normal multi-file project:

```bash
cargo run -p nuis -- project-doctor <project-dir|nuis.toml>
cargo run -p nuis -- check <project-dir|nuis.toml>
cargo run -p nuis -- test <project-dir|nuis.toml>
cargo run -p nuis -- build <project-dir|nuis.toml> <output-dir>
```

Then verify the emitted manifest if you want the build artifact checked as a
standalone package description:

```bash
cargo run -p nuis -- verify-build-manifest <output-dir>/nuis.build.manifest.toml
```

For a final pre-release pass, prefer:

```bash
cargo run -p nuis -- release-check <project-dir|nuis.toml> <output-dir>
```

## Default Single-Source Workflow

For a single `.ns` file, the default route is shorter:

```bash
cargo run -p nuis -- check <input.ns>
cargo run -p nuis -- test <input.ns>
cargo run -p nuis -- build <input.ns> <output-dir>
```

Use this when you are iterating on language behavior or compiler contracts
without needing a full `nuis.toml` project.

## Debug Workflow

When `check` fails, use the compiler IR dumps in this order:

```bash
cargo run -p nuis -- dump-ast <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- dump-nir <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- dump-yir <input.ns|project-dir|nuis.toml>
```

Use:

* `dump-ast` for parser, annotations, surface syntax, and binding shape issues
* `dump-nir` for frontend typing, generic rewrite, pattern lowering, and
  project validation issues
* `dump-yir` for lowering, scheduling, result-family, and verifier-adjacent
  issues

If the problem looks scheduling-specific, add:

```bash
cargo run -p nuis -- scheduler-view <input.ns|project-dir|nuis.toml>
```

## Project Triage Workflow

When a project feels unhealthy before you even get to compile errors, use:

```bash
cargo run -p nuis -- project-status <project-dir|nuis.toml>
cargo run -p nuis -- project-doctor <project-dir|nuis.toml>
```

Use:

* `project-status` to inspect the current visible project plan and package
  shape
* `project-doctor` to get fix-oriented project guidance

If you want to freeze today’s recommended ABI choices into the project:

```bash
cargo run -p nuis -- project-lock-abi <project-dir|nuis.toml>
```

## Test Workflow

The default test command is:

```bash
cargo run -p nuis -- test <project-dir|nuis.toml>
```

Useful variants:

```bash
cargo run -p nuis -- test --list <project-dir|nuis.toml>
cargo run -p nuis -- test --ignored <project-dir|nuis.toml>
cargo run -p nuis -- test --include-ignored <project-dir|nuis.toml>
cargo run -p nuis -- test --exact <project-dir|nuis.toml> <test-name>
```

Use these when:

* you want to inspect available language tests without running them
* you want to run ignored probes intentionally
* you want to pin one exact test during compiler work

## Build Output Workflow

The default build command is:

```bash
cargo run -p nuis -- build <input.ns|project-dir|nuis.toml> <output-dir>
```

Use explicit CPU/target overrides only when you are intentionally freezing the
output ABI:

```bash
cargo run -p nuis -- build --cpu-abi <abi> <project-dir|nuis.toml> <output-dir>
cargo run -p nuis -- build --target <triple> <project-dir|nuis.toml> <output-dir>
```

The `0.16.0` rule should be:

* default to auto ABI selection while iterating
* only lock ABI/target when release packaging or reproducibility matters

## Cache Workflow

Cache commands are part of the compile workflow, not a side channel:

```bash
cargo run -p nuis -- cache-status <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- clean-cache <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- cache-prune <input.ns|project-dir|nuis.toml>
```

Use them when:

* build reuse looks suspicious
* you want to confirm cache hits/misses while iterating
* you want to keep the compile surface reproducible during release prep

## Recommended `0.16.0` Reading Rule

If someone asks “how do I compile a `nuis` project today?”, the shortest answer
should be:

1. run `nuis project-doctor`
2. run `nuis check`
3. run `nuis test`
4. run `nuis build`
5. run `nuis release-check` before calling it release-ready

If that answer starts growing caveats, we should tighten the toolchain surface
instead of teaching a more complicated ritual.
