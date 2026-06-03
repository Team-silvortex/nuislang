# Repo File Line Policy

The repository now follows a simple file-size rule:

* default maximum: `600` lines per text file
* checked categories:
  * `.rs`
  * `.ns`
  * `.toml`
  * `.md`

This rule is enforced by:

* [tools/nuisc/tests/file_line_limit.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/file_line_limit.rs)

## Why This Exists

The compiler and stdlib have accumulated a number of very large files. That
made feature work possible, but it also made refactors slower, review harder,
and ownership fuzzier.

The goal of this rule is not to pretend the repository is already clean.
The goal is:

* stop creating new oversized files
* make existing oversized files explicit
* force large files to move downward over time instead of drifting upward

## Historical Exceptions

There are still legacy files above the `600` line default.

Those files are tracked in an explicit exception budget table inside
[tools/nuisc/tests/file_line_limit.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/file_line_limit.rs).

Important rule:

* an exception is a temporary ceiling, not permission to keep growing
* if a file is already oversized, it may stay at or below its recorded budget
* if a file drops in size, its budget should be lowered with it
* new files should not be added to the exception list casually

## Practical Working Rule

When a file approaches the limit:

* prefer extracting a coherent helper module
* prefer splitting by responsibility, not by arbitrary line number
* if a temporary exception is absolutely necessary, record it intentionally and
  treat it as debt

The rule is intentionally strict because the repository is entering a stage
where compiler organization matters as much as feature count.
