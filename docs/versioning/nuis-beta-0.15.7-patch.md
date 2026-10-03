# Beta 0.15.7 Patch

Date: 2026-10-03. Source version: `beta-0.15.7`, following `a083a422`
(`beta-0.15.6`) and minor baseline `05951bef` (`beta-0.15.0`). Git history
identifies the exact revision. Cargo package versions, protocol identifiers,
public signatures and capability scores are not bumped for this release.
The [minor snapshot](nuis-beta-0.15.0-snapshot.md) and
[preceding patch](nuis-beta-0.15.6-patch.md) retain their historical evidence.

## Included Changes

Parent-entry and post-loop snapshots share a bounded summary of zero trips,
every intermediate write and all later trips. Stable leaves retain their origins;
varying leaves receive independent binding/boundary identities. Boundary failure
publishes neither a partial environment nor a partially advanced snapshot clock.
The 65536-work/64-depth budget, exact nominal checks, original source admission,
whole-function revalidation and inner-only/original-body fallback remain intact.

Private capture normalization projects partly observed total record copies and
exposes unwritten input fields through single-definition scalar aliases. Local
versions, separate opaque evaluations, codecs and checked constructors do not
grant input-alias authority. Required calls and selected checks stay evaluated.
The guarded 64-field fixture uses 2/3 private arguments, or 3/3 with an unused
selected call retained, while its public input remains 64 words. Fewer private
arguments do not establish a measured speedup or general capture closure.

Typed literal origins compare exact i64/i32/bool/finite-float storage identities
across joins and loop boundaries. Scalar kinds and floating signed zero remain
distinct; bounded canonical integer casts preserve wrapping/sign extension.
Float text is charged before parsing. Arithmetic, calls, codecs, invalid or
nonfinite float text grant no literal authority. This proof does not remove
evaluated RHS work or relax native bounds.

A separate changed-tag/count fixture now fits inner/outer 60/64 carried words at
record width 64, rather than 61/65, because one literal field is invariant.
Replacing that field with an opaque call still rejects at 65 outer words. Mixed
constant fields reduce the outer carry further to 60; independent inner return
storage still needs 60. Native probes preserve every public output word, scalar
bits, overlap/canaries, observed exits and zero aggregate allocation/drop in the
admitted pure-value profile. The public/source/callback/FFI ABI is unchanged.

Cache copies and host-binary materialization now use
[fresh-file publication](../reference/nuis-artifact-file-publication-v1.md):
exclusively stage a private sibling, synchronize and rename rather than overwrite
a previously executed file identity. Existing leaf symlinks/directories are
rejected, partial writes retain the old destination, and materialization enables
execution only after artifact admission. This is not a whole-bundle transaction
or protection against arbitrary parent-directory mutation.

A minimal alternating-image cache regression reproduced signal 9 on the local
macOS aarch64 host despite a valid on-disk signature; a hard-link regression
independently exposed mutation of the previous file identity. Fresh publication
passes six alternating launches and four same-path source-free materializations
without launch retries, signing workarounds or callback-profile changes. The
earlier isolated workflow's exact phase/PID was not retained, and a specific
kernel rejection reason is not established.

The [loop snapshot contract](../reference/nuis-native-scalar-loop-snapshots-v1.md)
records the scoped proofs and chronological validation checkpoints. LLVM carry
validation also retains exact pair shape and the 64-word limit, with a new
zero/odd/exact-bound/over-limit regression. Small LLVM lint fixes do not increase
the required Rust API version or suppress warnings.

## Validation Evidence

The latest typed-literal worktree checkpoint on local macOS aarch64 passed:

* 850 selected compiler lowering unit tests, including seven literal-origin tests
  and one differential test exercising 72 independent return-storage cases
* 39 selected native bridge tests across the broad/focused runs
* Six compiler-cache tests and three source-free workflow tests
* Five reference image/window session tests
* 26 development-tensor tests and one host-path policy test
* 167 LLVM library tests, with one pre-existing ignored test

Overlapping reruns are excluded. The final three literal native probes and two
literal source-free workflows passed again after the equivalent LLVM cleanup.
Those workflows cover repeated events, cache hits, pre-open tamper rejection,
source/manifest/build deletion, byte-identical LLVM/state after two same-path
materializations and a selected arithmetic failure before state publication.
Earlier fresh-file publication acceptance separately includes 140 artifact
library tests, including ten publication tests; it is not added to the latest
checkpoint's selected-test count.

LLVM's all-target strict Clippy check passes. The broader `nuisc`/`nuis` all-target
check is still blocked by 11 observed compiler lints, chiefly long lowering
signatures plus style/borrow findings. No suppression was added; this is not a
claim that later targets are lint-clean.

The rebuilt CLI reports 1700 clean drift checks and clean coverage, hierarchy
and selected-task lineage at `active/86`. Formatting, diff whitespace, UTF-8 and
changed-file line caps pass. The latest implementation checkpoint verified 2993
local documentation links before this release record was added.

Release preparation reran 28 tensor-related tests, 20 maintenance-script tests
and the host-path policy case. The live CLI retained 1700 clean drift checks and
clean coverage/hierarchy/lineage. Formatting, diff whitespace, 3000 local links,
4628 UTF-8 text files and line caps for all 61 changed/new files passed. These
checks are separate from the implementation checkpoint; native execution suites
were not repeated solely for the version/documentation update.

These are selected local CPU/reference checks, not a full-workspace test run,
fresh Linux/GPU/Windows acceptance, formal-safety certification or a measured
performance improvement. The
[validation checklist](nuis-beta-0.15.0-release-checklist.md) lists wider suites;
a listed command does not mean it ran for this patch. Remote CI acceptance must
be checked for the pushed revision separately.

## Remaining Work

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session` at `active/86`.
Next is further changing backedges, local-version/opaque snapshots and
call-backed wide private captures under bounded proof, retaining checked work,
observed exits and fallback. Incompressible state cannot bypass the native limit.

Resource-bearing state, deferred tasks, native provider dispatch, general
interactive cancellation and self-contained application packaging remain
separate work. This patch neither freezes ABI nor completes compiler self-hosting
or general native application closure. Independent Nustars and Galaxies retain
shared contracts without new implementation dependencies.
