# Nsbdr

`nsbdr` is the Nuis OS bundle/distribution front-door.

It consumes final binary outputs produced by `nsld` and owns cross-platform
packaging plans such as macOS `.app`/`.dmg`, Windows MSIX, Linux AppImage, and
future Nuis OS native packages. It does not own linker graph construction,
section/container assembly, or core binary emission.

Current alpha commands:

```sh
cargo run -p nsbdr -- status
cargo run -p nsbdr -- plan <nsld-final-output> <package-output-dir>
cargo run -p nsbdr -- plan <nsld-final-output> <package-output-dir> --target=nuisos.nspkg
cargo run -p nsbdr -- plan <nsld-final-output> <package-output-dir> --json
```

The current `plan` command is non-mutating. It verifies the final output file
presence, reports size/hash, derives planned bundle/package paths for each
registered package target, detects host packaging tools, and keeps writer
blockers until each platform bundler is deliberately implemented.
