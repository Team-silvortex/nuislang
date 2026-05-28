# Versioning Notes

This directory is the lightweight anchor set for phase snapshots and later
versioning policy documents.

## Current Snapshot

Start with:

* [nuis-0.13.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.13.0-snapshot.md)
* [nuis-0.13.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.13.0-release-checklist.md)

Those files are the shortest current “what is real enough to stand on” summary
for the `0.13.0` phase.

## Expected Broader Scope Later

This directory is also the natural home for later policy documents around:

* language/toolchain surface versioning
* `YIR` format/version compatibility policy
* `nustar` package format/version compatibility policy
* ABI and loader-contract evolution rules

## Current Reading Rule

Phase snapshots here are anchors, not replacements for implementation truth.

For exact current behavior, still prefer:

* [../reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
* [../reference/yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
* current checked-in parsing/lowering/verification code
