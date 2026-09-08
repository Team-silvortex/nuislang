# CI Cold-Start Contract

The [Build workflow](../../.github/workflows/build.yml) must work on a clean
Linux runner without prior Cargo registry contents or generated build outputs.
Local cached tests are not evidence that this path works.

## Order And Failure Handling

1. Check out source with a pinned Node-24 checkout action and read-only repository
   permission; build steps do not retain checkout credentials.
2. Run the [CI guard regressions](../../scripts/tests/test_ci_checks.py), UTF-8
   validation (including Python), and local Markdown-link validation.
3. Install the stable Rust toolchain and run `cargo fetch --locked`.
4. Run the exact `checked_in_docs_do_not_embed_host_absolute_paths` integration
   test, not every test binary in the compiler package.
5. Run `cargo build --workspace --locked`.

Build/test debug information and incremental compilation are disabled for this
disposable CI profile; two Cargo jobs bound parallel resource use. This does not
change local developer profiles or disable runtime checks. The workflow has a
30-minute timeout and no allowed-failure steps.

The path guard uses `--locked`, not an unconditional `--offline`. CI explicitly
fetches first; an intentional offline environment is valid only after its
dependencies have been populated. The document-link guard must propagate failed
extraction/enumeration and count real checks. A link to a generated build
directory is not a valid checked-in-source dependency.

## Reproduction

Run from a clean checkout, with Rust and the ordinary host build prerequisites:

```sh
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=2
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
python3 -m unittest discover -s scripts/tests -v
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
cargo fetch --locked
bash scripts/check-host-absolute-paths.sh
cargo build --workspace --locked
```

For cold-start evidence, use a separate empty Cargo home and target directory,
not deletion of another checkout's caches. Prefer remote Linux for that full
workspace exercise. The guard tests also reconstruct a source-only tree from
Git's file inventory to detect accidental dependencies on ignored local outputs.

## Recorded Repair

The [beta-0.12.3 failing run](https://github.com/Team-silvortex/nuislang/actions/runs/34178353345)
stopped at offline resolution of `ed25519-dalek`, before the workspace build.
The previous beta-0.12.2 run showed the same fault. Inspection also found an
overescaped Markdown-link pattern that silently checked nothing and one link
to the untracked `target/` directory.

The repair was exercised in an isolated Linux x86_64 source tree with an empty
Cargo home: locked fetch, the focused path test and the complete workspace build
passed. Subsequent build execution with Cargo offline mode enabled confirms the
fetched dependency set is sufficient. Platform-specific `nsdb` dead-code warnings remain;
this is a build gate, not a claim of warning-free Linux or GPU execution coverage.
Remote-host reproduction does not change existing GitHub run conclusions: a new
run of the pushed revision is required before calling Actions green.
