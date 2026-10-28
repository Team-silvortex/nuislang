# Repo File Line Policy

The repository now follows a simple file-size rule:

* default maximum: `800` lines per Rust/Nuis source or config text file
* test-source maximum: `1000` lines
* Markdown maximum: `2000` lines per `.md` reference/history document
* checked categories:
  * `.rs`
  * `.ns`
  * `.toml`
  * `.md`

This rule is enforced by:

* [tools/nuisc/tests/file_line_limit.rs](../tools/nuisc/tests/file_line_limit.rs)

The repository README, documentation index and current mainline map are short
routers, capped separately at 300 lines by the source-only
[hygiene tests](../scripts/tests/test_repository_hygiene.py). Detailed proof
narratives belong in focused references, reached through the
[native session proof index](reference/nuis-native-session-proof-index.md).

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

There are still legacy code/config/source files above the `800` line default.
Markdown files are allowed to be longer because reference material often needs a
larger narrative envelope, but they should still stay below `2000` lines.

Those files are tracked in an explicit exception budget table inside
[tools/nuisc/tests/file_line_limit.rs](../tools/nuisc/tests/file_line_limit.rs).

Important rule:

* an exception is a temporary ceiling, not permission to keep growing
* if a non-Markdown file is already oversized, it may stay at or below its
  recorded budget
* if a file drops in size, its budget should be lowered with it
* new files should not be added to the exception list casually

The ratchet test requires each listed file to remain above its default limit
and match its exact recorded ceiling. Remove an exception once it is no longer
needed. The 2026-10-18 audit retired 42 obsolete entries, leaving 14 actual
oversized files; this shrinks debt permissions, not the covered test surface.
The subsequent AOT packer modularization removes its 3,888-line entrypoint
exception as well, leaving 13 oversized files. All new packer modules use the
default limits rather than receiving replacement exceptions.

The shader/ns-nova contract-suite split removes the 2,115-line test exception,
leaving 12 historical exceptions. Its original test entrypoint is now a 163-line
registration router, with private fixture/suite modules no larger than 486 lines.
The original fully qualified test names remain available; no replacement
exception or `include!`-based size workaround is added. See the
[maintenance record](repo-cleanup-candidates.md#2026-10-28-shadernova-contract-suite-split).

## Practical Working Rule

When a file approaches the limit:

* prefer extracting a coherent helper module
* prefer splitting by responsibility, not by arbitrary line number
* if a temporary exception is absolutely necessary, record it intentionally and
  treat it as debt

The rule is intentionally strict because the repository is entering a stage
where compiler organization matters as much as feature count.
