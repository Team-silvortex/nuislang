# Nsbdr Bundler Frontdoor

`nsbdr` is the Nuis OS bundle/distribution toolchain member.

It sits after `nsld`: `nsld` produces the core binary artifact, while `nsbdr`
adapts that artifact to host operating-system distribution shapes such as
macOS `.app` bundles, `.dmg` images, Windows MSIX packages, Linux AppImage
bundles, and future Nuis OS native packages.

`nsbdr` must not become a second linker. Its inputs are already-linked Nuis
final outputs plus packaging metadata. Its outputs are platform distribution
plans and, later, platform bundles/images.

Current commands:

```sh
cargo run -p nsbdr -- status
cargo run -p nsbdr -- plan <nsld-final-output> <package-output-dir>
cargo run -p nsbdr -- plan <nsld-final-output> <package-output-dir> --target=macos.dmg
cargo run -p nsbdr -- plan <nsld-final-output> <package-output-dir> --target=nuisos.nspkg
cargo run -p nsbdr -- plan <nsld-final-output> <package-output-dir> --json
```

The alpha `plan` command is intentionally non-mutating. It records:

* final output path, presence, size, and hash
* selected package target id and target OS
* planned staging directory
* primary bundle/package paths for the selected target
* candidate package targets for macOS, Windows, Linux, and Nuis OS
* host packaging tool availability per candidate
* explicit blockers for unimplemented bundle/package writers

This keeps OS distribution outside `nsld` while still making packaging a
first-class protocol boundary.

Current registered package targets:

* `macos.dmg`: macOS `.app` plus `.dmg`
* `windows.msix`: Windows app layout plus MSIX
* `linux.appimage`: Linux AppDir plus AppImage
* `nuisos.nspkg`: future Nuis OS native package
