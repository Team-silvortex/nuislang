# Nuis Development Tensor

This file defines the repository's lightweight development-progress model.
It answers one narrow question: `how do we describe current system progress
without flattening everything into one vague roadmap list?`

## Model

The development tensor is a 3-axis progress model:

* `architecture`
  the broad system layer or design lane
* `module`
  the concrete repository/tool/package area carrying the work
* `function`
  the user-visible or toolchain-visible capability being matured

Each tensor cell carries:

* `status`
  protocol-owned maturity label. In `dev-tensor-status-v1`, valid values are
  `stable`, `usable`, `active`, and `early`
* `progress`
  current progress score from `0` to `100`
* `bootstrap_critical`
  whether Nuis should treat this cell as important before self-hosting
* `closure_role`
  the role this cell plays in the compiler/toolchain/runtime closure
* `evidence`
  the current proof anchor, usually tests, frontdoor fields, docs, or examples
* `next_step`
  the most useful next action for that cell
* `blocker`
  the current concrete blocker that makes this cell weaker than done
* `next_action`
  the action-oriented task-card step; this can mirror `next_step` while tools
  migrate from narrative guidance to machine-consumable planning
* `validation_command`
  the narrow command that should prove the next action worked
* `expected_artifact`
  the concrete artifact or surfaced contract expected after the next action

Short rule: `architecture tells where the work lives; module tells who owns it;
function tells what capability is being matured`.

## Source Structure

The repository applies an 800-line default to Rust and Nuis sources, 1000
lines to test sources, and 2000 lines to Markdown. Runtime drift checks,
trust-anchor regressions, workflow tests, packet annotation tests, ABI and
registry tests, and Nuisc integration tests are split into ordered focused
modules. Their former line-budget exceptions are removed rather than raised.

## CLI

Use:

```bash
cargo run -p nuis -- dev-tensor
cargo run -p nuis -- dev-tensor --json
```

The JSON surface is intentionally simple:

* `kind = "nuis_dev_tensor"`
* `model = "architecture-module-function-progress-tensor"`
* `axis_0 = "architecture"`
* `axis_1 = "module"`
* `axis_2 = "function"`
* `status_protocol_version`
* `status_protocol = [...]`
* `hierarchy_protocol_version`
* `hierarchy_validation_status`
* `hierarchy_validation_node_count`
* `hierarchy_validation_max_depth`
* `hierarchy_validation_error_count`
* `hierarchy_validation_first_error`
* `hierarchy_validation_errors`
* `hierarchy_root_status`
* `hierarchy_root_progress`
* `hierarchy_root_weakest_child_path`
* `bootstrap_critical_count`
* `bootstrap_critical_average_progress`
* `weakest_bootstrap_architecture`
* `weakest_bootstrap_module`
* `weakest_bootstrap_function`
* `weakest_bootstrap_status`
* `weakest_bootstrap_progress`
* `weakest_bootstrap_closure_role`
* `weakest_bootstrap_evidence`
* `weakest_bootstrap_next_step`
* `weakest_bootstrap_blocker`
* `weakest_bootstrap_next_action`
* `weakest_bootstrap_validation_command`
* `weakest_bootstrap_expected_artifact`
* `weakest_bootstrap_task_card_protocol`
* `weakest_bootstrap_task_card_source`
* `weakest_bootstrap_task_card_status`
* `weakest_bootstrap_task_card_ready`
* `weakest_bootstrap_task_card_coordinate`
* `weakest_bootstrap_task_card_priority_reason`
* `weakest_bootstrap_task_card_action`
* `weakest_bootstrap_task_card_command`
* `weakest_bootstrap_task_card_expected_artifact`
* `weakest_bootstrap_task_card_handoff_mode`
* `weakest_bootstrap_task_card_handoff_coordinate`
* `weakest_bootstrap_task_card_handoff_reason`
* `weakest_bootstrap_task_card_handoff_action`
* `weakest_bootstrap_task_card_handoff_command`
* `weakest_bootstrap_task_card_handoff_expected_artifact`
* `weakest_bootstrap_task_card_lineage_protocol`
* `weakest_bootstrap_task_card_lineage_status`
* `weakest_bootstrap_task_card_lineage_error_count`
* `weakest_bootstrap_task_card_lineage_first_error`
* `weakest_bootstrap_task_card_lineage_errors`
* `weakest_bootstrap_task_card_task_ancestry`
* `weakest_bootstrap_task_card_handoff_ancestry`
* `weakest_bootstrap_task_card_common_ancestor_path`
* `weakest_bootstrap_task_card_transition_depth`
* `coverage_status`
* `coverage_expected_source`
* `coverage_expected_fallback_used`
* `coverage_expected_source_error`
* `coverage_expected_count`
* `coverage_covered_count`
* `coverage_missing_count`
* `coverage_orphaned_count`
* `coverage_stale_count`
* `coverage_first_gap`
* `coverage_missing_coordinates`
* `coverage_orphaned_coordinates`
* `coverage_stale_coordinates`
* `manifest_coverage_status`
* `manifest_coverage_source`
* `manifest_backed_coordinates`
* `manifest_missing_modules`
* `manifest_untracked_modules`
* `milestone_coverage_status`
* `milestone_coverage_source`
* `milestone_schema`
* `milestone_coordinates`
* `milestone_missing_coordinates`
* `milestone_untracked_coordinates`
* `milestone_constant_drift_count`
* `milestone_constant_drift_coordinates`
* `drift_status`
* `drift_check_count`
* `drift_check_passed_count`
* `drift_check_failed_count`
* `drift_first_failed_check`
* `drift_checks = [...]`
* `hierarchy = {...}`
* `cells = [...]`

Each cell includes both named coordinates and a `coordinates` array so scripts
can read it either as records or as tensor coordinates.

## Status Protocol

The tensor status field is now protocolized rather than free-form text. The
current protocol is `dev-tensor-status-v1`:

* `stable`
  rank `4`, phase `validated`, terminal for the current milestone slice
* `usable`
  rank `3`, phase `usable`, strong enough to consume but still evolving
* `active`
  rank `2`, phase `in-progress`, actively maturing and allowed to move fast
* `early`
  rank `1`, phase `exploratory`, not mature enough to anchor bootstrap-critical
  closure by itself

Coverage treats an unknown status as stale metadata. This keeps the tensor from
quietly drifting into ad-hoc labels.

## Recursive Hierarchy

The flat `architecture/module/function` cells are also projected into a
recursive hierarchy:

`root -> architecture -> module -> function`

The recursive representation is governed by
`nuis-dev-tensor-hierarchy-v1`. Its validator walks the full tree and checks
legal level transitions, parent-derived paths, unique nodes, progress bounds,
branch status/progress/count aggregates, weakest-child selection, and the
two-way mapping between function leaves and registered tensor cells. A clean
tree reports `hierarchy_validation_status = "clean"`; malformed trees report
the first error and the complete deterministic error list.

Each hierarchy node carries:

* `level`
* `path`
* `status`
* `status_rank`
* `progress`
* `cell_count`
* `bootstrap_critical_count`
* `weakest_child_path`
* `children`

Branch status is derived from the weakest child status, and branch progress is
the weighted average of descendant function cells. This means the tensor can be
read both as a table and as a recursively inspectable project tree. The
recursive form is intended to support future bootstrap planning where a weak
architecture lane can be expanded into its weakest module and then into the
exact function cell that needs work.

The summary mirrors the weakest bootstrap-critical function cell as a small
closure bundle: status, progress, closure role, evidence, and next step. The
task-card selector uses that lane while any bootstrap-critical cell remains
below `stable/100`. Once every bootstrap-critical cell is closed, it
automatically falls through to the weakest incomplete cell across the full
tensor. The selected cell is projected into a small task-card surface:
protocol, source, status, ready flag, coordinate, priority reason, action,
validation command, and expected artifact. That gives scripts and future
self-hosted tooling one stable bundle to consume without reassembling many
`weakest_bootstrap_*` fields by hand.

At alpha closeout the transition reason was explicit:
`all bootstrap-critical cells are stable at 100/100`. Beta may then register
new foundation coordinates instead of pretending that an alpha-complete tensor
means the runtime is finished. The runtime loader and lifecycle dispatch slices
have now closed, so the `beta-0.1-foundation-hardening` milestone reopens
bootstrap selection with finer coordinates rather than downgrading those
historical closures.

The `beta-0.1` calibration baseline is:

* `standard-library/std/concurrency-task-thread-lock`: `stable/100`, required;
  recursive selected-prefix lowering and the native cancel/unlock project close
  both dynamic branches; `Mutex<i64>` now has opaque scheduler handles,
  generation-bound guards, worker ownership, acquire/release epochs, strict YIR
  metadata, deterministic contention, replay rejection, and native LLVM ABI
  evidence; `SharedMutex<i64>` now declares a literal permit cardinality in
  `1..=64` at share time, with the one-argument form defaulting to `2`; lane
  admission derives from that single YIR fact across the interpreter and native
  runtime, and a three-permit project closes as a native binary with exit `33`
  without exposing the handle; `mutex_shared_close` now consumes shared
  authority, rejects active leases, release-publishes closure, revokes pending
  same-generation permits, invalidates the runtime slot, and returns the revoked
  count through interpreted and native paths; `mutex_lease_replace` now keeps
  linear lease authority while returning the old scalar, publishes a release
  epoch, and makes the replacement visible to a permit issued before mutation
  in interpreter, C-runtime, LLVM, and native task-project evidence; matching
  branch-local share/permit/lease prefixes now select inputs before emitting one
  capability chain, reject cardinality/lane drift, and execute both native paths
  with exits `25` and `43`; the scalar-v1 follow-up now preserves
  `MutexPermit<i32>` across task packing/helper reconstruction, carries signed
  i32 bits through kind-checked native slots, replaces `17` with `23`, observes
  the replacement, and exits `63` without deferred mutex lowering; a
  scheduler-private C11 atomic admission gate now protects all mutex, guard,
  and permit table operations, while a 32-pthread harness proves simultaneous
  unique slot allocation, 32 live leases, concurrent release/close, exact
  counters, and zero residue; runtime-dynamic cardinality, bool/float native
  payloads, per-mutex parking/fairness, and a mature parallel Nuis executor
  remain open
* `host-compatibility/cffi/registered-pointer-string-object-boundary`:
  `stable/100`, required; five real borrowed UTF-8 calls, one owned
  `ref Buffer` return, one owned read-only `ref String` return, and one opaque
  owned `ref FfiObject` return carry exact signature plus memory-capability hashes
  through compile, project metadata, and Nsld validation; the owned path now
  has self-verifying YIR metadata, runtime-header length recovery, exact
  destructor dispatch, and native execution; one GLM-typed conditional transfer
  accepts only direct owners with identical ABI/destructor/hash identity, drops
  the unselected owner, and merges pointer plus runtime length; one synchronous
  helper may now call one direct owner-producing helper before returning that
  owner to its entry caller through a `{ptr,i64}` runtime ABI; YIR retains
  static ABI/destructor/hash identity across both boundaries and enforces one
  caller release; owned UTF-8 has a separate Res-producing operation, runtime
  validity/length checks, bounded byte reads, one direct exact destructor, and
  a native zero-live-owner proof; FfiObject independently binds
  `size=static:16`, `read=i64_slots`, and its exact destructor, exposes only
  bounded size/slot reads, rejects generic Buffer fallback, and has its own
  native zero-live-object proof; writes and every helper/branch/task/async/loop
  escape for String and FfiObject remain closed, as do raw pointers and
  arbitrary external object layouts; project metadata now queries
  `official.cffi` rather than inferring permissions from AST types, includes
  the registered destructor as a static authority dependency, and carries the
  exact object descriptor through build-manifest verification and the same
  link-plan entry used by Nsld; hash-consistent size/read drift and missing
  destructor authority fail closed, while a full Nsld drive leaves the native
  project artifact runnable with exit `0`
* `package-system/galaxy/source-import-and-lock-resolution`: `usable/99`,
  required; root and generated build
  locks now share one portable SHA-256-bound
  compiler resolution protocol covering direct/transitive edges, package
  identity, manifest/source/library content, import policy, and actual module
  selection; sync transactionally materializes the verified closure as a
  `sha256/<resolution>` compiler provider with a canonical index, cache
  manifest, and lock copy; locked compiles consume only that provider and
  re-render the closure against the root lock; project release admission now
  requires both lock and synchronized cache before writing outputs;
  `nuis-galaxy-resolution-provider-v1` now statically registers workspace,
  locked-cache, and offline-layout providers, exposes content-bound request and
  result hashes, and preserves exact or opaque-workspace compatibility;
  `nuis-galaxy-candidate-set-v1` binds provider identity, generation, raw index,
  canonical candidates, and unique Ed25519 signers before caret, tilde, or
  explicit bounded-range solving; the capped deterministic solver selects the
  highest compatible closure and backtracks across later transitive conflicts;
  unsigned ranges, malformed/unbounded ranges, response tampering, ambiguity,
  and escaping candidates fail closed; two separate offline mirrors still
  produce byte-identical locks and caches, and locked compilation remains valid
  after provider deletion; persistent signer trust/rollback state, revocation,
  remote discovery/transport, and cache collection remain open
* `linker-toolchain/nsld/os-native-executable-finalization`: `stable/100`,
  required; a provider-neutral,
  hash-bound static registry now consumes an
  `NHOB`-bound pair of actual
  `program-llvm` and `runtime-shim` Mach-O objects, validates their LinkPlan
  identity, hashes, roles, sections, symbol/string tables, and ARM64 relocation
  references; `nuis-nsld-macho-placement-binding-v3` now deterministically
  merges compatible sections, assigns aligned contribution offsets, binds
  section-backed cross-object definitions, coalesces common declarations by
  maximum size/alignment into a reserved provider-owned
  `__DATA,__nuis_common` zero-fill section, lets strong definitions override
  tentative ones, preserves unresolved C/system symbols as a compatibility
  boundary, preserves `N_ABS` values as absolute coordinates, resolves
  multi-hop `N_INDR` definitions through one cycle-safe alias graph, and rejects
  duplicate strong definitions, incompatible flags, reserved-section claims,
  missing alias targets, and alias cycles before mutation;
  `nuis-nsld-macho-arm64-relocation-application-v1` now maps every verified
  relocation to deterministic source plus image-offset-or-absolute targets,
  carries alias-chain evidence, registers all eight kinds
  emitted by the real fixtures, preserves paired ADDEND/SUBTRACTOR metadata,
  separates direct and platform-structure work, fails closed on unknown kinds,
  and projects one placement-bound plan through JSON, text, and persisted invoke
  plans; `nuis-nsld-macho-arm64-materialization-preview-v1` now copies verified
  section payloads into deterministic merged buffers, audits section/image
  hashes, and generates non-mutating checked byte previews for
  `UNSIGNED`/`SUBTRACTOR`, `BRANCH26`, and paired
  `PAGE21`/`PAGEOFF12`/`ADDEND`;
  `nuis-nsld-macho-arm64-patch-application-v1` independently reconstructs the
  source image, accepts only hash- and audit-verified preview bytes, rejects
  duplicate/overlapping spans and source drift, commits each direct patch once,
  and emits deterministic post-write image, patch-audit, and ledger hashes
  through JSON, text, and persisted invoke plans;
  `nuis-nsld-macho-arm64-platform-structure-plan-v1` uses a provider rule
  registry to deduplicate semantic targets, assign checked 12-byte stub and
  8-byte GOT slots, bind every deferred relocation to one target offset, and
  hash the registry, applied-image ledger, layout, targets, and bindings; the
  real `_puts` fixture receives stub offset 16 and aligned GOT offset 32;
  `nuis-nsld-macho-arm64-platform-patch-application-v1` now extends the applied
  image to the planned span, emits checked ARM64 stubs and internal/external GOT
  entries, rewrites every deferred relocation once, keeps external values as
  explicit unresolved bind records, and publishes identical image/write/patch/
  bind ledger evidence through JSON, text, and persisted invoke plans;
  `nuis-nsld-macho-arm64-shell-layout-plan-v1` maps that hash-bound working image
  into deterministic 16 KiB-page `__PAGEZERO`, content, and `__LINKEDIT`
  segments, merged plus provider-owned common/stub/GOT sections, a role-aware entry,
  symbol/indirect records, rebase/bind requirements, linkedit offsets, and
  ordered load commands, always includes the libSystem executable baseline,
  and derives a deterministic `LC_UUID` from the shell-plan identity;
  `nuis-nsld-macho-arm64-shell-image-serialization-v2` now consumes the exact
  plan and platform ledger, emits private Mach-O header, command, content, dyld,
  symbol, indirect-symbol, and UTF-8 string-table bytes, and re-encodes direct/
  platform relocations, stubs, and internal GOT pointers against final VM
  addresses with deterministic rewrite and image ledgers;
  `nuis-nsld-macho-arm64-ad-hoc-signature-v1` appends a SHA-256 SuperBlob and
  CodeDirectory, while `nuis-nsld-macho-arm64-signed-image-validation-v1`
  independently reparses all command boundaries, UUID, signature fields,
  padding, and signed slots. `nuis-nsld-macho-arm64-os-loader-probe-v1` is
  plan-only by default and explicitly admits only signed zero-unresolved/zero-
  bind inputs into a bounded, exact-byte, owner-only temporary execution with
  complete cleanup evidence. A fully internal ARM64 fixture is accepted by the
  real macOS kernel and dyld, exits zero, and cleans up; an external CLI fixture
  remains blocked before materialization. Internal-rebase, external-bind,
  tamper, probe, and JSON/text/TOML CLI evidence pass. Successful apply persists
  a canonical, strict, owner-only, SHA-256-bound
  `nuis-nsld-macho-arm64-publication-admission-v1` receipt at one stable relative
  filename. Independent replay rebuilds the current product and checks registry,
  target, private-image, signature, probe, cleanup, and zero-bind identities. A
  real compiled-artifact fixture passes, while receipt tamper and regenerated-
  artifact drift fail closed. The selected finalizer now exposes one hash-bound
  optional private-image publication capability through the provider-neutral
  registry. Planning is non-mutating; invalid apply preserves compatibility
  bytes; valid explicit apply writes, syncs, rereads, atomically installs, and
  verifies the exact owner-executable image. The real installed private Mach-O
  exits 0 through macOS with empty output. The common fixture executes
  `ADRP`/`ADD`/`STR` against VM-only provider storage before that same admission
  and publication chain. `nuis-nsld-final-output-selection-registry-v1` now
  keeps `compatibility-default` as the non-mutating default while an explicit
  `admitted-private-image` request delegates through the registered finalizer.
  Plan-only calls preserve bytes; apply requires valid receipt replay and binds
  receipt, publication, candidate, and installed SHA-256 identities under
  `nuis-nsld-final-output-selection-evidence-v1`. The real ordinary final-output
  command selects and executes that private image. The first Linux route,
  `nsld.finalizer.elf.amd64.artifact-image-v1`, now selects through the same
  registry for `x86_64-linux-elf + native-cpu-llvm`. It binds one ELF64
  `program-llvm` and one `runtime-shim` `ET_REL` object to their LinkPlan ids,
  roles, formats, sizes, and FNV hashes; parses bounded section/name/string and
  symbol tables plus a registered explicit-addend `R_X86_64` subset; rejects
  malformed symbol partitions, duplicate strong definitions, unsupported
  relocations, and out-of-range patch sites; and derives internal versus external
  symbol closure across the pair.
  `nuis-nsld-elf-amd64-placement-binding-v1` groups `SHF_ALLOC` contributions
  into deterministic text, read-only data, data, zero-fill, and common classes;
  page-separates their permission boundaries; orders program before runtime;
  assigns checked aligned file/image offsets and x86_64 virtual addresses;
  coalesces common declarations by maximum size and alignment with
  strong-definition precedence; preserves absolute values; maps unmatched weak
  references to zero; binds section/common/absolute definitions and
  cross-object references; and retains unresolved system symbols as a
  compatibility boundary. Reversed object order
  preserves the canonical plan hash, while TLS, compressed,
  writable-executable, malformed zero-fill, excessive-alignment, and overflow
  cases fail before mutation.
  `nuis-nsld-elf-amd64-relocation-application-v1` maps every registered
  `R_X86_64_NONE/64/PC32/PLT32/32/32S` record to one placed source and bound
  target, computes checked `S+A` or `S+A-P`, and emits deterministic
  little-endian previews. Placement drift, shape mismatch, and signed/unsigned
  overflow fail before mutation; unresolved system symbols stay explicit
  platform-structure work. Reversed object order preserves the relocation plan
  hash. `nuis-nsld-elf-amd64-materialization-preview-v1` reparses the hash- and
  size-bound objects, copies verified file-backed placements into a
  deterministic merged memory image, audits zero-fill ranges separately, and
  binds source-object, file-image, memory-image, placement, and relocation
  hashes. Direct relocations become non-overlapping write-once patch spans with
  source and encoded byte evidence without mutating the source image; unresolved
  compatibility targets remain deferred.
  `nuis-nsld-elf-amd64-patch-application-v1` independently rebuilds the source
  image and preview, rejects identity/order/width/hash drift, applies each
  direct span once in an isolated buffer, and binds source/applied file and
  memory hashes plus every write audit into a deterministic ledger. Deferred
  targets remain untouched.
  `nuis-nsld-elf-amd64-platform-structure-plan-v1` validates that exact ledger,
  groups external PLT32 targets through a static provider rule registry,
  deduplicates dynamic symbols, and assigns shared nonlazy PLT/GOT slots plus
  dynamic string and `R_X86_64_JUMP_SLOT` records across page-separated
  RX/RW/RO regions. Every deferred source receives one checked i32 patch
  preview without changing the applied image; unsupported shapes, overlap,
  registry ambiguity, and ledger drift fail closed.
  `nuis-nsld-elf-amd64-platform-patch-application-v1` rebuilds the exact plan
  and direct-applied image, reserves inherited patches, emits checked nonlazy
  PLT/GOT, `Elf64_Sym`, dynamic-string, and `Elf64_Rela` records in an isolated
  extended image, and applies each deferred source once. Its deterministic
  ledger hashes every structure/source write plus final file/memory images;
  repeated calls share platform records while retaining distinct patch audits.
  `nuis-nsld-elf-amd64-shell-layout-plan-v1` now rebuilds the deterministic
  placement/relocation envelope, validates the exact platform image and ledger,
  maps base and platform RX/RW/RO regions into non-overlapping `PT_LOAD`
  records, assigns `PT_PHDR` and optional `PT_DYNAMIC`, section-name/header and
  dynamic-tag coordinates, and selects a registered source entry inside a
  file-backed executable segment. Every coordinate audit binds the platform
  application ledger; static/external plans remain object-order deterministic,
  while ledger drift and missing entry definitions fail closed.
  `nuis-nsld-elf-amd64-shell-image-serialization-v1` rebuilds that exact
  envelope, copies the platform-applied file bytes into a private planned image,
  and emits checked ELF64, program, dynamic, section-name, and section-header
  bytes at their registered coordinates. Non-overlapping zero-reserved writes
  carry source/encoded/post-write hashes; every file-backed source span remains
  byte-identical, zero-fill remains `SHT_NOBITS`, and the final image plus all
  audits are bound by one deterministic ledger.
  `nuis-nsld-elf-amd64-shell-image-validation-v1` reparses the bytes without the
  encoder, checking header/table/dynamic/name coordinates, source spans, write
  audits, unexplained prefix/tail bytes, and the final ledger.
  `nuis-nsld-elf-amd64-os-loader-probe-v1` revalidates that report and accepts
  static closure or hash-bound dynamic provenance through the Mach-O-shared
  bounded runtime. Real x86_64 Linux runs now pass static `_start`, versioned
  `sched_yield@libc`, and combined `getrandom@libc`/`cos@libm`/`sched_yield@libc`.
  The dynamic plan assigns global version indexes, emits and reparses two
  `DT_NEEDED`, two Verneed, three Vernaux, GNU versym, and bind-now PLT/GOT; the
  GNU loader executes exact Nsld-owned bytes with exit `0`. Generic admission
  replays registry, target, capability, image, validation, and dependency
  evidence before publication. Ordinary `final-executable-output` preserves
  implicit compatibility, keeps explicit plan-only non-mutating, and installs
  only admitted bytes on apply. Explicit requests atomically persist path-free
  owner-private selection JSON; valid `cos` signature drift keeps ELF bytes but
  blocks stale admission before mutation. `official.cffi` now owns two generated GNU resolver providers and four version rows; Nuisc validates and preserves them, Nsld consumes a static build-time table, and runtime registry identity remains `0xc6631e590d61aca8`. Architecture and PE/COFF parity remain open
* `heterogeneous-runtime/data/provider-neutral-data-fabric`: `early/32`,
  optional; provider-neutral movement exists, but no physical DPU/IPU backend is
  claimed

These scores describe the new beta slices only. Existing `stable/100` cells
remain evidence that their narrower protocol milestone closed; they are not a
claim that the containing architecture is finished forever.

The task-card protocol is `nuis-dev-tensor-task-card-v1`. A ready task card
means the tensor found an actionable bootstrap or global incomplete
coordinate, coordinate coverage is clean, the recursive hierarchy is clean,
and the task/handoff lineage validation is clean. If every registered cell is
`stable/100`, the card reports `complete` instead of inventing more work.

Task-card selection uses the deterministic ordering
`status_rank -> progress -> coordinate`. Before bootstrap closure its source is
`weakest-bootstrap-status-progress-path`; after closure it becomes
`weakest-global-incomplete-status-progress-path`. Lower status maturity is
weaker, progress breaks status ties, and the full coordinate makes selection
stable when input registration order changes. When no incomplete coordinate
remains, the source becomes `all-cells-complete` and lineage is clean but empty.

The task-card also exposes a handoff bundle. When the weakest coordinate is the
tensor itself, `weakest_bootstrap_task_card_handoff_mode` becomes
`self-maintenance-handoff` and the handoff coordinate names the next weakest
bootstrap-critical non-tensor cell to continue after refreshing the model.
Otherwise the handoff mode is `direct` and mirrors the current task-card
coordinate. The same status/progress/path ordering chooses the non-tensor
handoff, so a completed stable frontdoor does not hide a merely usable runtime
lane just because it appears earlier in the source snapshot.

Task and handoff coordinates are independently bound back to the recursive
hierarchy by `nuis-dev-tensor-task-card-lineage-v1`. The validator recursively
resolves each coordinate from the root and requires it to terminate at a
function leaf. It publishes the complete root-to-leaf task ancestry and
handoff ancestry, their deepest common ancestor, and the edge count needed to
move from one leaf to the other. A `direct` handoff must retain the same
coordinate and ancestry with transition depth zero. A
`self-maintenance-handoff` must advance to a different reachable leaf. Missing
leaves, invalid modes, inconsistent ancestry, or a non-clean hierarchy make
the lineage `invalid` and keep the task card blocked.

`nuis status` also prints the short tensor summary plus hierarchy protocol and
validation state. That makes the model part of the toolchain self-orientation
surface, not just a separate report command, and prevents task handoff from
trusting an invalid recursive projection.

## Coverage Manifest

The tensor now has a milestone-owned expected-coordinate source. The primary
source is:

`docs/reference/nuis-development-tensor.milestones.toml`

That manifest lists the coordinates that the current mainline expects to see in the
tensor:

`expected architecture/module/function coordinates`

The coverage layer derives expected coordinates from that manifest, falls back
to the Rust `DEV_TENSOR_EXPECTED_COORDINATES` emergency snapshot only if the
manifest cannot be read, compares the expected coordinate set with the actual
`DEV_TENSOR_CELLS` entries, and reports:

* `coverage_status`
  `clean` when required expected coordinates are covered and no stale/orphaned
  cells are present; otherwise `gap`
* `coverage_expected_source`
  the active source for expected coordinates, normally
  `docs/reference/nuis-development-tensor.milestones.toml`
* `coverage_expected_fallback_used`
  true only when the Rust fallback snapshot was used because the manifest could
  not be loaded
* `coverage_expected_source_error`
  the manifest load error when fallback was needed, otherwise `<none>`
* `coverage_missing_coordinates`
  expected coordinates that do not currently have a tensor cell
* `coverage_orphaned_coordinates`
  tensor cells that exist but are not declared by the coverage manifest
* `coverage_stale_coordinates`
  cells with invalid metadata, such as empty evidence or out-of-range progress
* `coverage_first_gap`
  the first missing, orphaned, or stale coordinate for quick CLI triage

Short rule:

`drift checks ask whether evidence anchors still exist; coverage asks whether
the tensor itself still spans the expected project map`

This is not yet automatic repository discovery. It is the first guardrail that
prevents the tensor from becoming only a hand-written status list. Future
versions can derive additional coordinates from galaxy manifests, Nustar
registries, and std module manifests, while the milestone file remains the
human-owned phase planning map. Existing `alpha-*` milestone IDs retain the
line that first established each coordinate; they are provenance, not a claim
that alpha is still current.

## Manifest-Backed Coordinate Coverage

The tensor now has a first manifest-backed coordinate view. It reads the stdlib
galaxy layout from `stdlib/index.toml`, compares those module names with the
current `standard-library/*/*` tensor cells, and reports:

* `manifest_coverage_status`
* `manifest_coverage_source`
* `manifest_backed_coordinates`
* `manifest_missing_modules`
* `manifest_untracked_modules`

This is intentionally advisory during early beta. A manifest module such as `core` or
`ns-nova` can be reported as untracked without failing coverage, because not
every official galaxy is ready to become a tensor cell at the same time.

The useful invariant is narrower:

`if a standard-library tensor cell claims progress for std, PixelMagic, or
WitSage, the dev tensor can now verify that the matching official stdlib module
manifest still exists`

## Milestone-Owned Coordinate Coverage

The tensor now also has a milestone-owned expected-coordinate manifest:

`docs/reference/nuis-development-tensor.milestones.toml`

This file groups expected tensor coordinates by their establishing milestone, marks whether
the milestone is bootstrap-required or optional, and gives the tensor a
project-owned source of truth outside the Rust constant table.

The current Rust `DEV_TENSOR_EXPECTED_COORDINATES` table still exists as a
checked snapshot and emergency fallback. The important change is that the
tensor now derives the primary expected-coordinate set from the milestone
manifest and compares all three sides:

* milestone manifest coordinates
* current `DEV_TENSOR_CELLS`
* Rust expected-coordinate snapshot

The milestone coverage reports:

* `milestone_coverage_status`
  `clean` when the milestone manifest covers all cells, all manifest
  coordinates have cells, and the Rust snapshot has no drift
* `milestone_coordinates`
  derived records in `milestone:requiredness:architecture/module/function`
  form
* `milestone_missing_coordinates`
  milestone coordinates that do not have tensor cells
* `milestone_untracked_coordinates`
  tensor cells that are not owned by any milestone manifest entry
* `milestone_constant_drift_count`
  parity failures between the manifest-derived coordinates and the Rust
  expected-coordinate snapshot
* `milestone_derived_cache_protocol`
  the protocol name for the generated coordinate snapshot metadata
* `milestone_derived_cache_status`
  `cacheable` when the manifest-derived coordinate set has a reproducible
  cache key; this does not imply that a cache file was written
* `milestone_derived_cache_key`
  a stable hash over normalized `milestone:requiredness:coordinate` records
* `milestone_derived_cache_coordinate_count`
  the number of coordinates covered by that generated snapshot key

Short rule:

`milestone coverage makes the tensor less hand-written: milestones own the map,
Rust constants must prove they still mirror it`

The milestone-derived cache metadata is intentionally zero-write for now. It
gives future tooling a deterministic key for generated coordinate snapshots
without creating hidden disk usage. The Rust `DEV_TENSOR_EXPECTED_COORDINATES`
table remains an emergency fallback mirror, not the preferred editing surface.

## Drift Checks

The tensor now includes a first lightweight drift-check layer.

These checks do not replace the real test suite. They only verify that selected
progress evidence anchors still exist in the repository, such as:

* frontdoor JSON fields
* workflow/artifact runtime regression assertions
* reference-document field anchors
* standard-library smoke-test and example-lane anchors
* registered Nustar domain contract anchors, including dispatch readiness and
  bridge materialization fields

The current status values are:

* `clean`
  every configured evidence anchor is still visible
* `drift`
  at least one configured evidence anchor is missing

Short rule:

`drift checks make the tensor less imaginary: if a progress cell claims a
frontdoor or document exists, the tensor can at least notice when that anchor
disappears`

The first std-oriented checks deliberately anchor the bootstrap-critical
`host-io-filesystem-text` cell to:

* `tools/nuis/tests/std_filesystem_smoke.rs`
* `tools/nuis/tests/official_galaxy_hetero_smoke.rs`
* `examples/projects/tooling/README.md`
* `stdlib/std/README.md`

That keeps the standard-library progress cell tied to the project-form
filesystem, IO, text, terminal, and tooling smoke chain instead of only a broad
roadmap phrase. The current std evidence also includes the observable CLI smoke
`std_tooling_observable_cli_smoke_checks_reports_and_stdin`, which checks
`run-artifact --json` prelaunch readiness, stdout/stderr report output from the
host IO report lane, direct stdin consumption by the built binary, and
`host_stdin_read` / `host_stdout_write` / `host_stderr_write` lowering anchors.
The std side of that lane now ends at `std-preprocessed-pgm` host preprocessing
evidence. The static `nuis-device-sample-input-registration-v1` table then
dispatches image construction to the `nuis.pixelmagic` registration, so the
generic frontdoor never owns `gray8` shape, payload, kernel, hash, or persistence
details. PixelMagic keeps that evidence visible through provider-sample
materialization and `execute-provider-samples` comparison metadata, including
the input evidence hash used by later shader output comparison. These values
lower into package-independent `nuis-provider-buffer-descriptor-v1` and
`nuis-provider-kernel-descriptor-v1` requests; Nsdb converts legacy evidence
into the same model but native adapters consume only the registered request.
The official heterogeneous smoke verifies persistence of the four source
pixels plus exact invert and chained-threshold baselines. On supported Apple
hosts it submits a registered two-request collection through the generic Metal
gray8 adapter. The second request consumes the first request's completed output
through the provider dependency/input-binding contracts. A GLM ownership token
and request-0-completed to request-1-dispatch-ready clock edge bind that
transfer, and the execution payload records a consumed/released transport
receipt. The resulting chain maps `[0,4,9,8]` to `[15,11,6,7]` and then
`[15,15,0,0]` with zero comparison mismatches. Unsupported hosts keep the
deterministic provider fallback.

The WitSage side now uses the same registered provider request model for a
contiguous four-element `f32` tensor and `witsage.vector.affine` kernel. On
macOS, Nuis persists a deterministic `.mlmodel` asset and binds its path,
length, hash, input feature, and output feature through
`nuis-provider-model-asset-descriptor-v1`. Nsdb validates that descriptor,
compiles and loads the model through CoreML, requests `CPUAndNeuralEngine`
compute units, executes `predictionFromFeatures`, and verifies the affine
result `[3, 5, 7, 9]` through stable output bytes and hash evidence. This is a
real `MLModel` prediction closure. It does not prove that CoreML scheduled the
operation on ANE. The adapter now loads `MLComputePlan` and emits
`nuis-coreml-compute-plan-evidence-v1`, including layer count plus preferred
and supported compute-device sets. On the M2 smoke host the affine model has
four CoreML plan layers, supports CPU, GPU, and Neural Engine, but prefers CPU.
That result is preserved rather than upgraded into a false ANE-execution
claim: CoreML's public plan API describes anticipated device usage, and this
small graph is not an effective Neural Engine workload.

The second registered model is a deterministic `16x64x64` feature-grid
projection. Nuis persists its 256 KiB all-ones input and hash-bound model asset;
Nsdb consumes the same generic buffer/kernel/model descriptors, without
matching a WitSage operation name. The prediction returns 65,536 `f32` ones,
while its one-layer compute plan supports CPU, GPU, and Neural Engine and
prefers Neural Engine on the M2 smoke host. The affine and feature-grid tests
therefore provide honest CPU-preferred and ANE-preferred baselines.

The operations now coexist in one `nuis-provider-request-collection-v1`
record. Collection order is explicit (`feature-grid`, `affine`, then
`chained-affine`), every
request retains independent buffer/kernel/model validation, and Nsdb executes
all entries fail-closed. `nuis-provider-output-collection-v1` mirrors indexed
request identities, byte counts, hashes, execution/compute-plan evidence, and
an order-sensitive collection hash. Each model request also binds a
`nuis-provider-output-comparison-descriptor-v1` to its output buffer, `f32`
shape, hash-bound expected asset, absolute/relative tolerance, and non-finite
policy. Nsdb reads the expected asset independently and compares every returned
element before emitting `comparison-passed`; shape/byte-count mismatches,
tampered expected assets, invalid policies, NaN/Inf under `reject`, and values
outside tolerance all fail closed. The official M2 lane compares 65,536 dense
elements plus both four-element affine outputs with zero mismatches.

The cross-provider Metal request now sources its comparison policy from
WitSage's checked-in
`provider-comparison-profiles/cross-provider-f32.nwcp`. The
`nuis-witsage-output-comparison-profile-v1` parser requires package identity,
strictly positive finite absolute and relative tolerances, and
`non_finite_policy=reject`. The request still carries the exact expected asset
length and FNV hash, so tolerance applies only after the expected baseline has
been authenticated. The deterministic M2 result remains an exact zero-error
match, while the protocol can now safely admit bounded backend variance.

`nuis-provider-request-dependency-v1` now binds a producer request/output
buffer to a consumer input buffer. Nsdb validates request identity, unique
edges, buffer names, acyclicity, and producer-before-consumer topology before
execution. The third request consumes the preceding CoreML affine output and
predicts `[7, 11, 15, 19]`, proving the edge carries real device data rather
than metadata alone.

`nuis-provider-input-binding-v1` adds ordered named inputs with artifact or
dependency source, element type, shape, byte length, and content hash. Legacy
single-input descriptors normalize into the same model, while protocol tests
represent fan-in and reject duplicate names or edge/binding mismatch. The
CoreML adapter now consumes ordered named feature/path/shape inputs. A real
two-input Add request fans affine and chained-affine outputs into
`[10, 16, 22, 28]` and passes independent comparison.

`nuis-provider-request-adapter-binding-v1` moves provider-family selection onto
each request while leaving adapter choice to the registry. The official graph
executes four CoreML requests, then transports Add output into a Metal `f32`
bias kernel and compares `[11, 17, 23, 29]`.

A sixth request adds a second classical model family without changing Nsdb or
the CoreML runner. WitSage emits an `InnerProduct` model whose two outputs are
nearest-centroid-equivalent scores `2c*x - ||c||^2`. For input
`[1, 2, 3, 4]` and centroids `[0, 0, 0, 0]` / `[4, 4, 4, 4]`, real CoreML
returns `[0, 16]`. Its `1x1x4` input and `1x1x2` output use the existing
multi-input-shape runner protocol, while
`witsage.model-predict.f32` independently supplies a hash-identified non-zero
tolerance profile with `reject` non-finite policy. The request follows the
Metal node, so the collection also proves CoreML worker reuse across the
non-contiguous `CoreML -> Metal -> CoreML` order.

`nuis-provider-edge-transport-v1` now binds that cross-adapter edge to an exact
GLM ownership token, `host-visible-owned-file` staging mode, producer-completed
clock, and consumer-dispatch-ready clock. Cross-provider dependencies without
that descriptor fail closed, while the provider output payload mirrors the
transport count and ordered evidence. The next boundary is a runtime receipt
that proves materialization, consumption, and release against one payload hash.
`nuis-provider-edge-transport-receipt-v1` now records those three transitions,
the 16-byte carrier size, an ownership-derived carrier identity, and the same
hash at every stage. The carrier is re-read after provider execution and the
receipt reaches `released` only after deletion succeeds. The remaining boundary
was implementation selection. `nuis-provider-edge-staging-registry-v1` now
selects `host.visible.owned-file.v1` deterministically for explicit mode or
`auto` fallback and owns materialize, consume, release, and cleanup operations
outside the graph executor. Receipts include registry source, selected adapter,
and capability status. `nuis-provider-carrier-input-v1` now supplies `path` and
`opaque-bytes` variants. `auto` selects `memory.owned-bytes.v1`, and the Metal
f32 runner consumes its handle-bound bytes directly through the native `hex:`
input boundary. CoreML named inputs now consume the same opaque bytes, so all
four dependency edges use independent `memory.owned-bytes.v1` carriers: the
chained edge, both Add fan-in edges, and the final CoreML-to-Metal edge. Four
receipts preserve their distinct ownership tokens, clocks, handles, and hashes.
`nuis-provider-carrier-channel-v1` removes that argv boundary. Rust emits a
binary stdin packet with fixed magic, frame count, ordered frame index, byte
length, FNV-64, and raw payload. CoreML validates and consumes multiple named
frames, while Metal consumes the same single-frame protocol. Receipts expose
`framed-stdin` as the channel mode.
`nuis-provider-carrier-channel-registry-v1` now moves that choice behind a
provider-neutral registry. On Unix, `auto` selects `inherited.fd.v1`; the parent
keeps `FD_CLOEXEC`, the forked child alone clears it before `exec`, and the
descriptor argument binds frame index, packet length, and packet FNV-64. The
temporary carrier is unlinked immediately after creation. CoreML and Metal
validate both the outer descriptor evidence and the inner per-frame evidence.
Other hosts select `framed.stdin.v1`, which remains an explicit portable
fallback. Receipts preserve registry source, adapter identity, capability
status, and selected mode. The Unix child now maps that anonymous carrier
read-only instead of reading it into a new allocation, and frame payloads are
no-copy `NSData` views over the verified mapping. CoreML carrier inputs use
contiguous `MLMultiArray` data-pointer views rather than element-wise copies.
`NUISPFD1` now gives inherited carriers a separate page-aligned layout with
ordered frame records, original and mapped lengths, aligned offsets, and
per-frame hashes. Metal successfully wraps the mapped page span with
`newBufferWithBytesNoCopy`, while CoreML keeps its direct `MLMultiArray` view.
Framed stdin retains the compact `NUISPCV1` layout and path inputs keep their
compatibility behavior. `nuis-provider-output-carrier-registry-v1` now closes
the asymmetric output boundary. Unix selects `inherited.fd.output.v1`: the
parent preallocates an unlinked writable descriptor, only the child inherits
it, CoreML and Metal `pwrite` raw result bytes, and stdout carries only channel
and FNV metadata. Rust maps the sealed packet read-only and verifies the exact
declared result before comparison. `hex.stdout.output.v1` remains the portable fallback. Native output
records expose registry source, adapter, and mode, and the adapter participates
in the ordered collection hash. Native outputs are sealed as valid single-frame
`NUISPFD1` objects. Every dependency edge clones that sealed carrier for its
consumer, inherits the same storage, and records `provider.output.transfer.v1`,
`inherited-frame`, and `transferred-output` evidence. Releasing a consumer
closes only its cloned descriptor; producer ownership remains valid until graph
teardown. CoreML configures each transferable input channel independently, and
its native runner maps and validates every fd-backed packet separately. This
covers the chained edge, both Add fan-in edges, and the final CoreML-to-Metal
edge without rebundling dependency bytes. `ProviderOutputPayload` keeps that
verified mmap view alive for comparisons, summaries, hashes, and completed
output metadata, while portable hexadecimal output uses an owned fallback.
Dependency execution and observation therefore share the inherited storage.
Writable single-frame carriers write only their fixed `NUISPFD1` metadata and
use `set_len` to establish the aligned payload span, avoiding output-sized
zero-filled construction buffers. `nuis-provider-output-residency-v1` now binds
each adapter to a residency kind, transfer scope, observation mode, and explicit
device-retention status. Those fields are preserved in every native output and
its ordered collection hash. Current adapters truthfully expose
`host-visible-file` or `host-owned-bytes`; inherited-fd remains the portable
comparison and cross-provider path. Device residency is still blocked by the
one-request child-process runner model: Metal and CoreML objects disappear when
the process exits. The next contract must therefore define a registered
provider session lease and GLM-owned output-handle lifetime before any backend
claims device-local retention. `nuis-provider-session-registry-v1` now selects
the explicit `logical.request-process.v1` fallback. One deterministic lease is
shared per runner adapter, request begin/complete hooks advance a strict
sequence, and `nuis-provider-output-handle-v1` binds each result to a GLM token.
Graph teardown drops completed payloads, closes every lease, and records handle
release. Official execution proves CoreML sequence `0..3` and an independent
Metal sequence `0`. This is ownership continuity, not device continuity: each
request still launches a new child. A provider-neutral persistent worker
transport is required before session adapters may advertise device retention.
`nuis-provider-worker-transport-registry-v1` now registers
`framed.stdio.worker.v1`. Its request and close frames bind lease, provider,
sequence, request, and worker PID. A real child-process regression proves two
ordered same-provider requests and close execute under one PID. This adapter is
intentionally protocol-ready rather than graph-selected: stdio cannot pass fds
created after worker startup. The next Unix adapter must bind `SCM_RIGHTS`
descriptor counts to the same frame identity and close received descriptors on
every mismatch before persistent CoreML or Metal execution can safely consume
direct carriers. `unix.scm-rights.worker.v1` now provides that low-level
adapter. Unix datagram frames bind lease, sequence, request, and declared fd
count to `SOL_SOCKET/SCM_RIGHTS`; received descriptors immediately become
`CLOEXEC` `OwnedFd` values. Tests read real file content through a transferred
descriptor and prove identity mismatch closes the received fd with `EBADF`.
The socketpair test and persistent-child PID test remain separate. The next
step is inheriting one endpoint into a child worker, completing a PID-bound
handshake, and transferring two distinct post-spawn carrier descriptors to that
same process. `UnixWorkerProcessTransport` now closes that gap. Only the worker
endpoint loses `FD_CLOEXEC` in `pre_exec`; the child handshake PID must equal
`Child::id()`. A compiled worker receives two different post-spawn file
descriptors at sequence `0/1`, reads first-byte evidence `17/29`, returns both
receipts from the same PID, and exits through an explicit close request.
Handshake failure kills and waits the child. `nuis-provider-worker-request-envelope-v1`
now closes the request-meaning boundary. `NUISPWU2` separates a UTF-8 control
header from opaque bytes, binds the bytes by length and FNV-1a hash, and requires
one ordered semantic role for every transferred descriptor. The C worker parses
binary payloads containing NUL, newline, and non-UTF-8 bytes, independently
checks the hash and role count. `NUISPWUR7` completes the reverse direction
without echoing the request body: it binds request length/hash identity, input
and output role manifests, bounded adapter-protocol length/hash, output carrier
mode, and worker-owned descriptor length/hash. It carries the positive status
returned by the Nuis invoker; decoding fails closed for zero or negative
status, ancillary-count drift, carrier-layout drift, or payload/packet hash
drift. `NUISPWUE1` preserves the failing worker stage and status instead of
collapsing every failure into the final process exit code. The worker
control plane now begins in Nuis itself: `StdProviderWorkerContracts` owns
ordered lifecycle state, and
`provider_worker_runtime_recipe.ns` checks, AOT-builds, and executes its native
accept/reject/commit loop with deterministic output `14`. C and Objective-C
runners remain bootstrap ABI probes or one-shot fallbacks, not owners of worker
policy. The next boundary is exposing system request ingress, descriptor
handles, and Kernel Nustar dispatch as registered Nuis intrinsics so a Nuis
entrypoint can own the persistent worker loop end to end.

`StdProviderWorkerDispatchContracts` now closes the Nuis-side dispatch half of
that boundary. It models opaque request and descriptor-table handles plus an
open-ended registered binding containing provider key, dispatch token,
capability hash, and enabled state. Nuis validates handle shape and binding
identity, rejects a mismatched capability hash, then advances the existing
ordered lifecycle. `provider_worker_dispatch_recipe.ns` loads CPU and data
Nustars, emits a visible `data.handle_table`, AOT-builds, and executes with
output `14`; the official std recipe smoke runs both worker recipes. The
remaining gap is a registered request-ingress YIR intrinsic whose platform
adapter imports real transport handles without owning validation or dispatch.

That ingress intrinsic is now registered. The Nuis builtin
`provider_request_ingress(request, descriptor_table, count, provider,
capability)` lowers to effectful `data.provider_request_ingress`; its five
inputs remain explicit YIR dependencies, Data Nustar interpretation returns the
opaque request handle, GLM records five value reads, and CPU LLVM lowering
performs only scalar passthrough. The dispatch recipe now obtains every request
handle through this node and still executes with output `14` and no ingress
deferred notes.

The same intrinsic now also accepts the capsule form
`provider_request_ingress(request, descriptor_table, count, provider,
capability, capsule_token, input_roles, output_roles)`. The compatibility form
remains valid, while the capsule form emits eight explicit YIR dependencies and
eight GLM reads. The persistent Nuis worker reads these three additional
scalars from the hash-verified request, rejects invalid token or role counts,
and only then returns the ingress status used by the dispatch permit. The host
shim performs bounded scalar extraction only; it does not choose providers,
operations, or policy.

That scalar boundary is now concrete. Parameterized `@export` functions may
expose a non-async, non-generic `i64` ABI; exported helpers are materialized
even when `main` does not call them, and LLVM scalar calls no longer impose an
artificial three-argument ceiling. The dispatch recipe exports
`nuis_provider_worker_request_v1(request, descriptor_table, count, provider,
capability)`, performs handle checks in Nuis, enters the registered Data Nustar
intrinsic, remains visible in the native symbol table, and still executes with
output `14`. `nuis-provider-worker-ingress-adapter-v1` is deliberately
policy-free: after `NUISPWU2` verification and runtime handle registration it
only maps the request handle, descriptor-table handle/count, provider key, and
capability hash into the compatibility signature. Its capsule mapping contract
additionally carries the token and input/output role counts as eight neutral
scalars; the AOT worker reads the extra three through registered host symbols
to avoid imposing an eight-argument platform ABI. The remaining gap is no
longer the scalar contract. `provider_worker_image.ns` now owns an async
`open -> while receive -> worker_request -> reply -> close` lifecycle. Its AOT
shim contributes only one-frame `recvmsg`/`sendmsg`, envelope verification, and
descriptor ownership primitives when the worker host-symbol surface is used.
The Nsdb transport regression compiles and links this Nuis source in-process,
then proves one worker PID receives two post-spawn SCM_RIGHTS descriptors with
first-byte evidence `17/29`, acknowledges both opaque payload identities, and exits through the
Nuis-owned close branch. The standalone C worker probe has been removed. The
next gap is registration-driven worker image selection and build reuse rather
than selecting this official source directly inside the regression.

Worker image selection and build reuse are now registration-driven.
`nuis-provider-worker-image-registry-v1` accepts any valid `domain:backend`
family, derives stable positive provider/capability scalars, and points to one
provider-neutral official worker source plus a versioned cache identity. The
resolver salts the normal `nuisc` content-addressed key with that identity,
restores a cached AOT image when available, and injects only the registered
launch scalars into the worker command. The persistent transport regression
resolves twice into independent output directories, requires the second result
to be a cache hit with the same key, and runs only that restored binary. The
remaining integration gap is moving resolver ownership into the normal
provider execution/session path so one registered worker instance is leased
and reused automatically rather than being opened directly by a regression.

That integration is now present. Normal provider sample execution leases one
registration-resolved Nuis worker per adapter/session, binds worker sequence to
the logical session sequence, transfers every available prepared input through
SCM_RIGHTS, records worker identity, resolver/cache status, descriptor count,
and payload hash in each indexed native output, and closes all workers at graph
teardown. Startup began as `NUISPWUH0 -> NUISPWUH1` and is now upgraded to the
capacity-bearing `NUISPWUH3` reply, with bounded socket I/O. Content cache
entries remain shared, while each live adapter gets a
separate transient restored executable directory; this avoids rewriting a
running Mach-O image when a second backend starts, and all transient copies are
removed after close. The seven-request official WitSage graph now proves CoreML
worker order `0..4`, Metal worker order `0..1`, two-descriptor fan-in,
cross-provider descriptor transfer, successful native comparisons, and one
worker-owned output descriptor per request. The registered PixelMagic gray8,
WitSage f32 bias, and generic f32 argmax Metal adapters cross that boundary:
Nsdb materializes and hash-binds their adapter images, while persistent Nuis
workers launch them with `path-fd` or `carrier-fd` input descriptors and return
the real adapter protocol output. At that integration stage, CoreML model
execution still used a parent-side compatibility path.

That earlier parent-side compatibility stage was fail-closed behind
`nuis-provider-worker-dispatch-permit-v1`.
`nuis-provider-worker-operation-registry-v1` accepts any frame-safe
provider/adapter/operation identity and derives a stable operation token without
enumerating Metal, CoreML, or future backends. That contract, identity, token,
kernel, and input count are carried inside the outer hash-bound worker payload.
Only an exact worker receipt grants the permit; every indexed native output
records the token, permit contract, and `granted` state before the concrete
runner branch can execute. This establishes an operation-level authorization
gate but does not yet move the runner process itself: the next boundary is a
registered execution capsule and output-carrier reply owned by the Nuis worker.
`nuis-provider-execution-capsule-v1` now closes the descriptor half: it binds
provider, adapter, operation token, and ordered input/output carrier roles into
a stable capsule id/token, and final output evidence records its honest
`worker-authorized-parent-adapter-v1` authorization mode.
`nuis-provider-execution-capsule-invoker-v1` now derives an open invoker
identity from that capsule. The persistent Nuis worker explicitly calls the
invoker after eight-scalar Data Nustar ingress succeeds. Its thin Unix adapter
creates an anonymous result descriptor and returns it through `NUISPWUR7` with
an output role, carrier mode, byte length, and FNV hash. Nsdb receives the
descriptor through SCM_RIGHTS and independently verifies all fields
before recording `worker-invoked` and `verified`; it does not construct the
descriptor or decide its contents. Capsule invocation and generic output
allocation are therefore closed, and the compatibility stage is no longer the
normal official execution route. `nuis-provider-worker-process-adapter-v4`
adds open ordered `literal`, `verified-path`, `descriptor-path`, and
`descriptor-carrier` argument templates without adding a Metal or CoreML
operation switch to the worker. PixelMagic gray8, WitSage Metal f32 bias, the
256 KiB feature-grid CoreML request, and chained single-input CoreML now execute
below persistent Nuis workers. Each adapter writes into a worker-created
`NUISPFD1`; stdout carries only bounded protocol metadata, and Nsdb verifies
the frame layout, stored payload hash, computed payload hash, and whole-packet
hash before restoring mmap-backed transferable ownership. The CoreML model path
is independently FNV-verified before `execv`. Ordered feature/carrier/shape
triples now also move two-input CoreML fan-in beneath the same worker lease, so
all seven official nodes execute through the worker process adapter. The next practical
boundary was eliminating repeated clang compilation of identical adapter
images.

`nuis-provider-process-adapter-cache-v1` now derives an open cache identity from
adapter source, runner contract, ordered framework manifest, operating system,
and architecture. The cache is graph-scoped: images are immutable while any
request can execute them, then their source and executable files are removed at
graph close instead of accumulating on disk. Official evidence requires the
five CoreML requests to report `compiled,hit,hit,hit,hit`; the bias and argmax
Metal contracts each report an independent `compiled` identity. Cache identity
and status remain local Nsdb evidence rather than being copied into the worker
request. This preserved the bounded worker frame after a real fan-in regression
showed that extra line-oriented metadata could cross the macOS Unix datagram
limit.

`nuis-provider-worker-adapter-control-v1` now replaces repeated
`adapter_argument_N` lines with one compact record containing a fixed launch
header and an ordered open argument sequence. The whole request hash binds this
record, individual paths and arguments must fit the worker ABI buffers, and a
portable 1800-byte dispatch budget rejects growth before `sendmsg`. The real
two-input fan-in route passes through the compact record, while a unit
regression proves oversized metadata fails closed. The next boundary is a
hash-bound control-carrier class for manifests that exceed the inline budget;
it must remain distinct from semantic `input.N` capsule roles.

`nuis-provider-worker-adapter-control-v2` and
`nuis-provider-worker-process-adapter-v5` replace the scalar output length
with an ordered output count, role manifest, and byte-length manifest. Nsdb
rejects empty, duplicate, malformed, zero-length, or count-mismatched
registrations before dispatch. The v27 Nuis worker creates one page-aligned
`NUISPFD1` carrier per slot and exposes the ordered
`NUIS_PROVIDER_OUTPUT_FDS` manifest as
`role=fd:descriptor:payload-offset:semantic-length:hash-offset`.
`NUIS_PROVIDER_OUTPUT_FD` remains the slot-zero compatibility view for existing
Metal/CoreML adapters. Multi-output adapters return an ordered decimal
`output_hashes` manifest. The worker verifies each protocol hash against both
the carrier's stored hash and the actual payload, then hashes and returns every
whole carrier. Nsdb consumes the same slot-indexed hashes and preserves every
result as a transferable carrier. A real portable child-process regression
writes distinct `u64[3]` primary and audit outputs through two inherited
descriptors and proves both payloads survive independently.

`nuis-provider-worker-adapter-control-carrier-v1` now closes that boundary.
Manifests above the 384-byte inline control budget are written to an immediately
unlinked file, while the request carries only its contract, byte length, and
FNV hash. Its SCM_RIGHTS role is `control.adapter`, must be unique and last, and
is excluded from Nuis ingress descriptor count, capsule input roles, input byte
sum, and adapter argument descriptor indices. The v27 worker independently
reads the exact declared length, rejects trailing bytes or hash drift, and then
parses the same open control record. Official CoreML execution reports
`worker_adapter_control_mode = carrier` while the Add node still reports two
semantic descriptors.

`nuis-provider-worker-descriptor-capability-v1` closes the hidden fixed-capacity
boundary. Worker image registration declares 31 semantic input slots and one
transport-control slot, the launch environment carries the same values, and
the PID-bound `NUISPWUH3` handshake rejects disagreement before any request is
sent. Nsdb validates every role set before `sendmsg`, while the C ABI adapter
enforces the negotiated quotas after ancillary receipt. A provider-neutral
worker regression transfers three semantic inputs plus one trailing
`control.adapter`; only the semantic bytes contribute to the input sum and Nuis
capsule count. Native output evidence exposes the negotiated contract and both
limits.

`nuis-provider-worker-output-descriptor-capability-v1` now separately registers
an eight-output budget without conflating output fan-out with input/control
capacity. `NUISPWUH3` proves both capability contracts before request traffic.
`NUISPWUR7` replaces the single output length/hash/mode fields with ordered
manifests aligned to output roles and received descriptors; Nsdb independently
reads and hashes every ordinary output. A provider-neutral regression now
combines three semantic inputs, one control descriptor, and two output roles,
then verifies two distinct 24-byte carriers with different hashes. Existing
Metal/CoreML adapters remain valid single-output consumers of slot zero.

`nuis-provider-output-binding-v1` first lifted that ordered fan-out into
`ProviderRequest`: each output has a distinct role and buffer identity while
the first binding remains the compatibility `kernel.output_buffer`.
`ProviderWorkerLease` consumes and verifies every returned descriptor instead
of dropping descriptors above slot zero, and reports the retained payload or
transferable-carrier state for each additional role. Provider sessions allocate
one `nuis-provider-output-handle-v1` handle and GLM ownership token per role,
with the compatibility handle pointing at the primary binding. The graph
boundary is now explicit through
`nuis-provider-graph-output-ownership-v1`. Completed outputs are indexed by
producer request plus output buffer, retain the registered role for audit
evidence, and reject duplicate publication. Dependency preparation selects the
exact producer buffer instead of falling back to a request-wide primary
result. A two-consumer regression selects different payloads from one producer,
and graph close reports the released output count and roles. Worker additional
outputs are also converted through their registered bindings before
publication.

`nuis-provider-output-binding-v2` additionally carries `layout` and
`row_stride_bytes` beside `element_type`, `shape`, `byte_length`, and the
optional `comparison_id`. V1 remains readable and infers buffer-compatible or
contiguous semantics, while normalized request payloads always write v2.
Dependency validation resolves the selected producer output before checking
consumer type, layout, shape, stride, and length. Completion summaries retain
the same ordered layout and stride manifests instead of claiming v2 while
dropping those fields.
Lease consumption compares the unpacked semantic payload length rather than a
wrapped `NUISPFD1` carrier length. The provider-neutral Nuis worker now proves
the complete shortest route with two distinct `u64[3]` outputs: both
descriptors are verified, published into graph ownership, consumed through
separate dependency bindings, and released together at graph close. The open
runner registry now also exposes `data.host.provider-worker-native`, so normal
`execute-provider-samples` execution reaches that same dual-output path without
a Metal/CoreML identity check. Final output evidence publishes ordered roles,
buffers, element types, shapes, byte lengths, and comparison identities beside
descriptor and graph release evidence.
`nuis-provider-output-comparison-collection-v1` now binds unique comparison IDs
to those output bindings. The native worker frontdoor independently compares
both `u64[3]` payloads against hash-bound assets and publishes ordered IDs,
buffers, statuses, element counts, and mismatch counts through
`nuis-provider-output-comparison-collection-result-v1`; the compatibility
comparison fields still mirror the first result. Ordered process-adapter
control and multi-FD carrier materialization are now closed at lease level.
The normal `execute-provider-samples` route now selects the Native execution
registration for the same request. Its content-addressed cache compiles or
reuses a portable thin C ABI adapter, process-adapter v5 supplies two inherited
output descriptors, and the adapter returns distinct primary/audit hashes.
Both `NUISPFD1` results remain transferable, both fixed expected assets compare
with zero mismatch, and graph close releases both roles. The ABI shim owns no
scheduling or backend selection; those remain in Nuis and the registration
contract.

`nuis-provider-execution-adapter-registry-v1` now removes concrete backend
execution branches from that frontdoor. Native, Metal, and CoreML each register
their own worker-descriptor requirement, optional process-adapter preparation,
request validation, execution, and result interpretation callbacks. The common
registry owns only callback composition and contract validation.
`provider_sample_execute` selects one registration and invokes it without
naming a backend, operation, model API, or runner protocol. A source-level
regression rejects reintroduction of concrete runner names into the frontdoor.
This is static registration for AOT determinism, not a dynamic plugin segment.

`nuis-provider-runner-profile-registry-v1` applies the same boundary to runner
selection. Data, Metal, and CoreML each own their provider family, probe
callback, available adapter, and fallback adapter. The central selector only
performs a contract-checked lookup, invokes the registered probe, and chooses
available or fallback state. Tests require unique families and reject
reintroduction of a provider-specific `match` into the selector.

`nuis-provider-bundle-registry-v1` now cross-binds each provider's unique
bundle ID, runner profile, and Unix execution adapter as one static
contribution. Runner-family and execution-adapter selection consume the same
AOT membership list, so adding or removing a provider cannot silently update
only one side.

The membership list is no longer handwritten.
`nuis-provider-bundle-manifest-entry-v1` records are owned by the registered
Data, Shader, and Kernel Nustar package manifests and survive `.nustar`
manifest round trips. Nuisc validates their syntax and registry-wide unique
bundle IDs, provider families, adapter kinds, and static implementation
bindings. The Nsdb build stage consumes that verified registry, sorts entries
by bundle ID, and emits `nuis-provider-bundle-manifest-v1` as an AOT-only Rust
table with a canonical FNV hash. Runtime selection independently verifies the
entry count, order, hash, package provenance, runner identity, execution kind,
and concrete static bundle before exposing it. No repository path or dynamic
plugin segment enters the resulting executable.

Nsdb now carries that registry contract, manifest contract, canonical hash,
entry count, package ID, and bundle ID through provider output payloads,
materialized sample manifests, text reports, and JSON reports. Nsld consumes
the same provider-neutral fields and mirrors them into final heterogeneous
output metadata. A sample cannot become `ready` when the bundle evidence is
missing or malformed; it becomes `provider-bundle-evidence-invalid` instead.
Nuis now independently parses and validates the same provider-neutral fields
rather than trusting Nsld's conclusion. Device-sample final output,
artifact-doctor mirrors, object-package audit aliases, and frontdoor closure
JSON all expose the registry identity, manifest hash/count, opaque package and
bundle IDs, and evidence status without naming a concrete provider family.
Multi-provider graphs now also publish
`nuis-selected-provider-bundle-set-v1`. Nsdb walks graph records in execution
order, keeps each opaque bundle ID at its first occurrence, and hashes the
ordered index/package/bundle/family tuples. Nsld and Nuis independently rebuild
that sequence from every manifest record; a missing or mismatched contract,
count, or FNV hash fails closed. The first-bundle fields remain compatibility
mirrors, while package and closure audit surfaces carry the complete selected
set identity. Nsld now places that identity in its open `metadata_binding`
table as `identity.selected-provider-bundle-set`. The binding table hash
participates in `metadata_table_hash`, each complete binding record
participates in `container_hash`, and disagreement prevents both container
metadata and payload emission. This remains provider-neutral: Nsld sees an
opaque contract/count/hash record, not a fixed Metal/CoreML combination.
Pending or blocked device execution can still carry a verified selected-set
identity through drive diagnostics, so selection integrity is not confused
with execution completion. The final NSB image embeds the canonical container
bytes. Nsld's container loader now extracts `metadata_binding` records from
that real image payload, independently recomputes the table hash, validates
required records and the selected-set shape, and blocks handoff after direct
image mutation. Final-output JSON/text expose the loader-observed binding
count, table hash, validation status, and selected-set contract/count/hash.
`nuis-host-runner` now repeats the verification directly from mapped final
image bytes and publishes count, parsed count, table hash, validation status,
and selected-set identity. Nuis launch evidence requires this independently
observed proof before reporting ready; a regression also proves that updating
the outer NSB hash cannot hide an inner binding mutation. Run-artifact now
persists a canonical `nuis-final-image-binding-proof-v1` declaration. Nuis and
Nsdb independently recompute its proof hash, Nsdb preserves it across handoff
record merges, and replay blocks after selected-set mutation. The direct Nsld
final-output writer now sends the same provider-neutral loader evidence through
Nsdb's claim API; Nsdb computes the proof, rejects conflicting final-image
identity, and Nsld final-output summaries mirror `verified` or
`verified-empty`. Proof-less handoffs remain readable as `legacy-unbound`, but
the public replay summary, concrete Nsdb replay plan/transcript, and Nuis's
independent final-output closure block them with
`rebuild-final-output-binding-proof`. Provider completion before final-image
evidence remains a valid mergeable lifecycle intermediate, not replay-ready
evidence. `nsdb-yir-replay-identity-v1` now carries the verified proof hash
through debugger transcripts, persisted cursors, resume validation, and every
cursor-lineage event. Nuis independently compares the handoff, cursor, and
lineage identities through its cursor and lineage mirrors before exposing each
surface as ready. The official PixelMagic graph now proves the complete
transition: its provider-complete intermediate is blocked as `legacy-unbound`,
the project-declared self-contained rebuild feeds the provider-neutral
`nsld seal` frontdoor, and the resulting Metal/CoreML NSB upgrades the same
handoff before three-frame cursor and lineage replay. Seal preflight rejects
host-finalized packaging and incomplete provider manifests before any bounded
stage runs; success performs exactly prepare, final pipeline, and publication.
The sealed NSB now owns a provider-neutral
`nuis-final-image-provider-dispatch-v1` table. Each ordered entry binds its
package, bundle, provider family, runner contract, adapter contract, and
adapter ID. The required `runtime.provider-dispatch-table` metadata binding
places its count/hash under both the metadata root and complete container
hash. Nsld's final-image loader independently extracts the complete capsule
from actual NSB bytes, recomputes dispatch and selected-set hashes, and rejects
adapter drift even when the sidecar remains unchanged. An explicit
`# nuis-nsld-container-end-v1` marker replaces the former field-position
capsule boundary. Host runner now independently recomputes the table and
selected-set hashes, verifies both required bindings, exposes all entries, and
blocks lifecycle handoff on drift. Nsdb owns a separate final-image parser:
launcher-less or not-ready-launcher work is explicitly
`pre-seal-acquisition`, while a ready launcher makes missing or damaged NSB
state fatal. Before launching a worker,
Nsdb matches package, bundle, family, actual runner/adapter identity, and the
registered runtime adapter against the final image. The official CoreML/Metal
route executes again after seal and proves all selected bundles were
authorized by the NSB. Mutable sidecars are now request-detail carriers rather
than dispatch authority. Provider completions bind their matched final-image
entry and dispatch/selected-set hashes under
`nuis-provider-completion-dispatch-authority-v1`; those fields participate in
the signed completion-set digest. Nsdb and Nuis independently aggregate the
same identity into replay transcripts and
`nsdb-yir-replay-cursor-record-v2`, and both reject cursor drift. Cursor
lineage v2 repeats the provider dispatch identity in its header and every
retained entry. Repair journal v6 binds the same identity into each event hash,
the rotated-prefix ancestry, and the canonical repair-window hash. Nsdb and
Nuis independently reject history from another dispatch table even when the
broader final-image proof is unchanged. Acquisition and self-contained rebuild
remain explicit Nuis orchestration rather than linker policy.

The lineage mirror is also the sole input to
`nuis-validated-provider-dispatch-identity-capability-v1`. This capability does
not reopen the handoff or NSB: it copies the identity already accepted as
`debugger_cursor_lineage_provider_dispatch_identity_hash`. A ready lineage with
an identity is `verified`; a ready CPU-only lineage without a dispatch table is
`verified-empty`; unavailable or invalid lineage is `blocked`. Final-output and
closure JSON/text project the same capability under
`object_package_provider_dispatch_identity_*` and
`debugger_api_provider_dispatch_identity_*`. These are core interfaces for
future package-summary and debugger Galaxy APIs, not separate authority
implementations.
The contract keeps Nsld container table hashes in `0x<16-hex>` form and uses
`fnv1a64:<16-hex>` for selected-set and proof hashes; independent consumers
validate those field-specific encodings rather than conflating them.

Return-producing `if` lowering now preserves control dependence for nested
extern-call comparisons. The open `compare_call_result` mode of
`cpu.guard_host_call_return` executes the host call only inside the selected
LLVM block, compares its result there, and returns the matched or unmatched
scalar without eager evaluation. The persistent provider worker uses this
shape for its close reply, so its two-request native regression also proves
that an unselected reply call cannot release the active request descriptors.

The language-core checks anchor the bootstrap-critical
`language-core/nuisc/type-control-flow-generics` cell to:

* `tools/nuis/tests/language_bootstrap_smoke.rs`
* `examples/projects/task/task_result_enum_demo`
* `examples/projects/state/generic_method_bound_guarded_nested_match_demo`
* `examples/projects/state/glm_buffer_roundtrip_state_demo`
* `examples/projects/state/std_style_language_bootstrap_demo`
* `examples/projects/state/std_style_language_import_bootstrap_demo`

That smoke is intentionally higher-level than an isolated parser or frontend
unit test. It builds the project through the `nuis` CLI, checks the
`run-artifact --json` prelaunch contract, verifies NIR/YIR/LLVM anchors for
generic `Result<T, E>`, higher-order specialization, enum variant lowering,
task-result control flow, and host-FFI signature whitelist evidence, then runs
the produced binary and asserts its deterministic Result/task/error exit code.
It also builds and directly executes the generic trait-bound guarded nested
match project and the GLM buffer roundtrip project. Those checks anchor
monomorphized trait method calls (`impl.Addable.for.i64.add`), alias-expanded
generic functions (`bump__i64`), buffer length/load/store/free lowering, and
YIR lifetime/effect edges around `cpu.store_at` / `cpu.free`. The same smoke
now also builds and executes the chained try/await Result HOF project, which
keeps `?` continuations alive across `normalize`, `decorate`, and `pipeline`
helper boundaries, feeds dynamic `host_argv_count()` input into helper-side
Result `?` continuations, asserts the produced native binary exit code, and
checks the LLVM output contains no deferred lowering. Sequence-level early
return folding now lifts `?` continuations to whole-Result selects instead of
selecting between an Err struct and an Ok payload. The std-style language
bootstrap workload now combines that dynamic Result path with Buffer
load/store/free lowering, higher-order lambda specialization, trait-bound
method calls, pointer borrow/free control flow, and async helper boundaries in
one native executable, with a deterministic exit code and no LLVM deferred
lowering. The import-boundary version now splits public helper enum/struct/type,
generic `Result`/HOF helpers, and Buffer/pointer helpers into
`StdStyleLanguageSupport`, consumes them through `use cpu
StdStyleLanguageSupport`, verifies the project module/import reports, keeps
entry-local trait-bound generic execution alive, and still produces the
deterministic native exit code. Helper modules now participate in lambda/HOF
expansion, helper public generic functions are visible as imported templates,
and helper-private `__hof_` / `__lambda_` synthetic functions are retained for
internal lowering. Helper-module impl method emission now also keeps
support-side trait-bound calls such as `bump<T: Addable>` executable through
the imported helper workload. The same workload now leaves imported
`result_map(...)` calls and Result helper constructors unannotated, proving that
cross-helper expected-type inference can carry generic arguments through the
std-shaped HOF boundary. A second package-shaped workload now splits the same
surface across `StdPkgCore`, `StdPkgOps`, and `Main`, including helper-to-helper
imports, imported aliases, Result/HOF inference, trait-bound methods,
Buffer/pointer control flow, and a deterministic native exit code. That path is
backed by partial expected-type propagation, so a helper HOF argument can retain
known generic slots such as `Result<T, Error>` while payload constructors infer
the remaining `T`. That surface has now started moving into the real std
galaxy as auto-injected `lib/language_core.ns` and `lib/language_ops.ns`; the
`std_language_galaxy_bootstrap_demo` consumes them via `std=workspace`, verifies
std galaxy module/import reports, and runs the same helper-to-helper
Result/HOF/trait/memory path as a native binary. The
`std_language_cli_report_demo` now extends that surface into a CLI-shaped std
consumer by combining language contracts with `StdTextContracts` and
`StdIoContracts`, writing a real stdout report, and validating the text/IO
gates through native execution. `std_language_report_file_demo` then pushes the
same language surface through `StdReportContracts`, writes an argv-selected
report file plus stdout, and validates the reusable report-file gates. The next
step, `std_language_workflow_demo`, feeds `StdLanguageOps.build_report` into a
two-step host command workflow through `StdCliContracts`, proving the same
Result/HOF/trait/memory surface can participate in command gates rather than
only report output. `std_language_build_pipeline_demo` extends that route into
a four-stage prepare/check/compile/package gate through
`StdCliContracts.build_pipeline_total` with no LLVM deferred-lowering notes.
`std_language_task_cli_demo` then carries the same surface into a task-backed
CLI path through `StdTaskContracts` and real stdout output. Integer scalar task
payloads now cross the native scheduler ABI as pending handles. Arbitrary-arity
`bool`/`i32`/`i64` async bodies are emitted as deferred helper thunks, then
normalized through LLVM-generated `i64(ptr context)` wrappers and one runtime
spawn ABI. Task polling invokes the wrapper on the next lifecycle tick, commits
completion, and reads through the runtime handle without LLVM deferred-lowering
notes. Timeout limits
now bind to the same scheduler slot: a zero limit produces a native `TimedOut`
terminal state and a positive limit preserves completed thunk execution.
Cancellation now transitions a pending slot to the native `Cancelled` terminal
state before join. Runtime slot storage is now one normalized thunk packet with
a common invoker and opaque context. All terminal paths and shutdown release
owned contexts. The larger `cli_build_pipeline_demo` also retains its
auto-injected language gate through native LLVM execution. The remaining
task/native closure gap is aggregate payload ownership and a mature worker
executor.

The source frontend now recognizes `ready_after(task, ticks)`, carries it
through every NIR visitor to `cpu.ready_after`, stores overflow-safe ready ticks
in native task slots, and applies completion-at-equal-positive-tick ordering
consistently with the built-in CPU interpreter. Native smoke coverage locks
both completion-before-deadline and timeout-before-readiness behavior. The
same smoke matrix also covers mixed `bool`/`i32` arguments, signed `i32`
returns, and `bool` returns through the normalized eight-byte slot ABI. The
same packed ABI now carries `f32` and `f64` by bit pattern rather than numeric
conversion, with native exact-value smoke coverage. Non-empty recursive source
structs with scalar leaves now encode their complete type tree while
materializing declaration-ordered leaves as tagged scalar/blob slots in one
native `NuisSchedulerOwnedPayloadV1` allocation. Type identity covers the
recursive shape, and one-shot take reconstructs nested field SSA before
drop-hook cleanup. Native mixed `bool`/integer/float/`String` nested field
coverage returns through await, direct join, and TaskResult paths. Text leaves
copy UTF-8 bytes into GLM-tokened task-owned blobs, re-intern on take, and are
released by the common self-describing aggregate drop hook. The shared native
text registration boundary now validates UTF-8 with Rust-compatible strictness;
compiled coverage accepts multibyte Chinese text and rejects overlong,
surrogate, truncated, and out-of-range encodings without leaking blobs.
The aggregate helper now remains a YIR `call_owned_struct` lane and executes
from the lifecycle poll through an owned invoker, rather than being evaluated
at submission. A null owned-invoker result now enters the explicit `Failed`
terminal state, is observable through `task_failed(...)`, and is covered by a
compiled C runtime harness. Native timeout and cancellation probes also prove
that deferred aggregate helpers do not execute before context cleanup. The
explicit Buffer conversion now materializes through LLVM as a GLM-tokened blob,
transfers through recursive task aggregates, and is detached with `take_blob`
before aggregate cleanup. Source Nuis now exposes typed `bytes_len` and
`drop_bytes` operations; GLM rejects reuse after drop, and a native recursive
task smoke returns the expected 24-byte length. The compiler now synthesizes
reverse-declaration-order cleanup for straight-line fallthrough and explicit
returns while preserving return-value evaluation and recognizing explicit drops
plus aggregate ownership transfer. Path-sensitive `if` cleanup now handles
branch-local scope exits, equal ownership-state merges, one-sided early returns,
and two-way terminal returns. Conditional YIR drop-return operations lower to
real LLVM basic blocks, so only the selected path releases the blob.
Ownership-neutral `while` loops may now carry outer bytes unchanged across
backedges and reach normal post-loop cleanup; conditions and nested loop-body
expressions are checked for hidden owned-byte creation or transfer. The
NIR cleanup pass now also releases per-iteration locals before linear-body
fallthrough, direct `break`, and direct `continue`, and GLM verifies the
generated edge cleanup with the outer Buffer lifetime. The backend now covers
both the first resource-aware direct-break loop and iterative counted loops.
`cpu.loop_owned_bytes_copy_drop_break` handles the selected break path, while
extensible `cpu.loop_while_i64_effect` metadata registers
`cpu.owned_bytes_copy_drop` without coupling the generic induction/backedge
skeleton to `Bytes`. Native coverage re-evaluates a changing condition across
two copy/drop iterations; tail `continue` lowers to the same deterministic
copy, update, cleanup, and backedge sequence. Direct guarded `break` now lowers
through `cpu.loop_while_i64_effect_flow`; the selected exit and natural backedge
both cross cleanup, and native aggregate return observes final induction value 2
through exit 26. Effect-flow metadata now also carries linear scalar state:
guarded `continue` skips `add_current`, while the normal update edge applies the
carry before both edges perform registered cleanup. GLM now treats a same-name
`let` after move/drop as a fresh identity. Native payload observation combines
break iteration 2 and carry score 7. Ordered multiple carries now accept
`add_carryN` dependencies on earlier same-edge results and reject forward
references; native `weighted += score` observes 10, producing exit 43 with the
24-byte blob. Uniform-action compound `and`/`or` guards now reuse the recursive
flow condition vocabulary through a length-delimited effect-flow payload; LLVM
evaluates the full tree after the induction update and still releases the blob
exactly once on either selected action or normal update. Carry records are now
arity-driven rather than fixed pairs. The affine multiplicative recurrence
`scaled *= current + 1` composes updated induction state with its invariant
payload only on the two normal update edges, producing factors 4 and 5. Native
multi-state resolution now also shares the common term vocabulary: grouped
`weighted += current + carry0` consumes the earlier same-edge score and reaches
17. Scaled recurrence records now reuse the canonical scaled-source resolver:
`scaled *= (current + 1) * 2` emits `mul_scaled_current_plus_invariant` and
reaches 80. Its invariant-factor ABI stores the additive offset after scaling,
so LLVM resolves it as `terms * factor + scaled_offset`; a native regression
locks this ordering against double scaling. The exit-130 baseline therefore covers compound
continue, multi-state addition, affine multiplication, and scaled multiplication
together. State-driven scaling is also encoded through
`mul_scaled_by_carry0_current_plus_invariant`; its LLVM regression proves that a
later carry reads the earlier carry's new value on the same edge. Remaining gaps
no longer include factor groups: linear effect-flow carries now reuse the
async-post-flow factor-group payload grammar, and
`grouped += (current + carry0) * ((current + -3) * (carry0 + -2))`
reaches 55 in the native aggregate. Exit 185 covers that path together with the
24-byte owned payload. Mixed-action resource controls now use terminal-local
`flow_break`/`flow_continue` tokens and ordered LLVM leaf blocks; recursive
cleanup rewriting releases the iteration blob once on either action or the
normal update path. Nested ownership scopes now recurse safely in NIR cleanup:
inner continue/break edges drop only inner iteration owners and preserve the
outer owner until its own edge. Registered `cpu.scoped_call` actions now
materialize scalar helpers as static function lanes, pass the current iteration
through `$current`, and lower an outer loop whose helper owns an inner Bytes loop
through LLVM without a fixed nested-loop opcode. Scoped helpers now also borrow
an outer `ref Buffer` through one logical YIR parameter expanded to LLVM
`(ptr, len)`; a Lifetime edge spans the loop, and task invokers reject the
borrowed ABI kind. An explicit `copy_bytes(buffer)` scoped argument now becomes
the `copy_owned:<buffer>` descriptor, carries Dep and Lifetime edges, performs a
scheduler-owned deep copy on each iteration, and enters the helper through
`cpu.param_owned_bytes`. Compiler cleanup drops that helper-owned payload
exactly once. Passing an existing `Bytes` value directly is rejected rather
than becoming an implicit clone. Outside scoped loops, `move(Bytes)` lowers
through the general `cpu.move_owned_bytes` operation; interpreted and LLVM
paths preserve the existing blob identity without copying. A scoped
`move(bytes)` becomes `move_owned:<bytes>` only when constant loop facts prove
exactly one execution. Zero-trip, repeating, non-constant, and unnamed-owner
moves are rejected. Direct and recursive helpers now transfer return ownership
through `cpu.return_owned_bytes` / `cpu.call_owned_bytes` and the LLVM `ptr`
ABI; the caller becomes the unique owner without another copy. The remaining
scoped-loop gap no longer includes outer rebinding: `scoped_call_owned_return`
keeps the blob in an LLVM `ptr` backedge slot and `cpu.loop_owned_result`
projects the final owner into the outer binding. GLM treats that projection name
as an output and the moved descriptor as a resource-own access. Dynamic `if`
branches can now converge the same explicit `move(Bytes)` owner through
`cpu.select_owned_bytes`; GLM records resource ownership on the branch inputs
and LLVM emits a native pointer select without copying. Conditional unary
`Bytes -> Bytes` helper returns now use `cpu.branch_call_owned_bytes`: the
helpers are statically outlined, LLVM emits mutually exclusive call blocks,
and a `phi ptr` carries the selected owner forward. A counted segmented YIR ABI
also carries branch-specific pure `bool/i32/i64/f32/f64` arguments without
duplicating the owner or eagerly executing opaque effects. Distinct owners lower
through `cpu.select_owned_bytes_drop_unselected`: GLM owns both candidates,
LLVM drops only the unselected branch value, and a `phi ptr` carries the
survivor. Exact-one scoped-loop moves can now be proved from cycle-safe local
constant chains, integer arithmetic, comparisons, and casts instead of only
literal YIR nodes; unresolved, zero-trip, repeated, and overflowing cases still
fail closed. Nested move-return `if` trees now carry survivor proofs through
`cpu.select_owned_bytes_tree`: a deduplicated owner table
and prefix decision tree let GLM consume aliases once, while LLVM performs
leaf-local cleanup and a multi-entry pointer merge. Leaves now also encode
registered static `(Bytes, scalar...) -> Bytes` helpers with pure scalar
arguments; their scalar dependencies remain explicit and LLVM invokes only the
selected leaf after dropping other owners. Three-arm scalar matches now reuse
the same prefix tree directly, and enum payload matches may
discard pure arm-leading bindings only when the remainder never references
them. Tagged `value`, `variant_field`, and recursively nested `struct_field`
scalar descriptors now provide the selected-leaf projection action required by
payload-using helpers. GLM depends only on the root projection base, while CPU
interpretation and LLVM resolve the complete field path only in the selected
leaf. Wrong variants or missing nested fields in unselected leaves therefore
remain unevaluated. A closed `cast` descriptor now composes all eight existing
NIR scalar conversions with those paths; unknown casts are protocol errors and
LLVM emits conversion instructions only inside the selected leaf. Pointer leaf
policies now admit non-optional `ref Buffer` arguments through direct values and
recursive structure/enum field projections. LLVM represents these borrows as
provenance-carrying `ptr + len` values, so aggregate assembly, variant selection,
and leaf-local projection retain the complete Buffer ABI without relying on a
projected SSA name. They remain read-only GLM dependencies owned and cleaned up
by the caller. Nullable Buffer fields may cross the same leaf ABI only through
`require_non_null(...)` under a matching branch-local null proof. The frontend
encodes a recursive `non_null` descriptor only when the exact source expression
is dominated by the non-null branch; the CPU interpreter rechecks it and LLVM
emits a leaf-local `llvm.assume`. Unproven uses fail closed. Read-only traversal
pointers are now a separate selected-leaf capability: a non-optional `ref Node`
must cross every call boundary through explicit `borrow(...)`, the tree records
`traversal_borrow <descriptor>`, GLM retains a `Read` on the root, and LLVM uses
a single-pointer ABI. The selected leaf rejects a null traversal pointer, while
unselected leaves do not inspect it; ownership and final cleanup remain with the
caller. Traversal pointers cannot be returned or placed in task payloads, and
owned pointer transfer now has a deliberately narrow exact-one contract for
selected helper trees. Every reachable leaf must contain exactly one
`move(<named Node>)` for the same transfer set, encoded as `owned_transfer`;
GLM marks each root as `Own` and requires its lifetime edge. The receiving
helper must contain exactly one `free(...)` on every exit path; verification is
path-sensitive across `if` and early return, while loops remain fail-closed.
Matching conditional effects can be merged before LLVM emission. Differing
effect-only branches now lower through the composition-independent
`cpu.branch_effect` protocol: each leaf carries an ordered list of
`module/instruction/result/arity/(access, operand)` actions. Nustars expose
their supported leaf signatures through the declarative
`BranchEffectActionCapability` registry contract. CPU currently registers
`load_value` as `i64(resource_read)` and `free` as `unit(resource_own)`; GLM
derives `Read` versus `Own` from operand metadata without an instruction-name
white list. NIR semantics exposes registration keys and operands without
lowering metadata, while nuisc obtains result/access plans from the active
static all-Nustar `ModRegistry`; an injected empty registry test proves the
source path fails closed before encoding. The interpreter rejects forged
contracts and evaluates only the selected list, and LLVM emits explicit
then/else/merge blocks plus a
continuation effect edge. Matching terminal `i64` actions can now declare an
`i64` branch-level merge result: CPU returns the selected heap value, LLVM emits
`phi i64`, and an `if` expression retains the merged binding. A native result
smoke executes both leaves in one binary and returns their `41 + 73` sum. The
native selected-transfer smoke runs
both leaves of one binary, observes distinct helper output, exercises a
branch-local load, and confirms a single Node allocation with no deferred tree
lowering.
Asymmetric paths, duplicate moves, null selected transfers, non-consuming
helpers, projected transfers, non-`i64` merge-visible branch-action results,
and task or return transport remain closed. Branch composition execution is
now hosted by YIR core rather than `CpuMod`: registry validation covers every
leaf, selected execution delegates to the owning `RegisteredMod`, and
`execute_module_with_registry` proves an injected `probe` Nustar can return the
selected `i64` value under a CPU composition parent. LLVM action emission uses
`BranchEffectLlvmEmitterRegistry`; `emit_module_with_registries` proves an
injected probe emitter can generate both values and the common `phi` without
changing the composition loop. Registered YIR actions with no matching LLVM
emitter fail closed. The ordinary source and project AOT paths now load the
manifests named by `loaded_nustar`, resolve static providers by
`yir_lowering_entry`, and pass the assembled YIR/emitter registries into LLVM.
CPU and AArch64 CPU install their emitters through this path. Provider
descriptors now live in the LLVM backend's static Nustar catalog, so `nuisc`
contains no CPU entry names or emitter functions. The paired YIR semantic
registry is now assembled from the same manifests through a verifier-owned
provider catalog. Unloaded domains remain absent, unknown providers fail during
assembly, and a catalog-coverage test locks every indexed official manifest.
Branch composition also has its first ownership-carrying result:
`owned_ptr` requires both paths to consume the same two live, distinct,
unborrowed owners through `cpu.take_ptr_drop_other`. Interpretation frees only
the discarded object, GLM returns `Res`, heap verification moves both source
names, and LLVM emits path-local frees plus `phi ptr`. Typed source lowering is
now exposed as `select_owned_ptr(condition, move(left), move(right))`. Both
candidates must be same-typed, named, distinct owners; NIR verification rejects
aliasing and any later reuse, while cleanup synthesis removes both consumed
inputs. The YIR merge now carries explicit `address_kind=node|buffer` and
`nullable=true|false` metadata. Heap verification rejects kind/object mismatch;
source may widen two live owners into an optional result but still rejects
nullable candidates. Selected helper leaves now encode every moved address as
`owned_transfer address_kind=<node|buffer> nullable=false <value>`. The YIR
parser includes kind and nullability in cross-branch equality, CPU validates
the selected heap object, and LLVM checks Node versus Buffer helper ABI shape
while retaining the Buffer length. `owned_pointer_select_demo` executes Node,
nullable-result, and Buffer selections, transfers one Node plus one Buffer
through exact-one-consuming helper leaves, reaches both `load_value` and
`load_at`, and proves final survivor cleanup in a native binary with exit `94`.
Projected, nullable-transfer, returned, and task-carried address results remain
closed.
The runtime now defines `NuisSchedulerOwnedBlobV1` as the first GLM-tokened
dynamic leaf primitive. It deep-copies borrowed bytes and has scheduler-native
move/drop hooks; a compiled harness covers take and cancellation. Recursive
String lowering now consumes it through self-describing aggregate slots, while
borrowed Buffer remains deliberately unavailable as a task input. The new
source-level `copy_bytes(ref Buffer) -> Bytes` conversion now reaches
`cpu.copy_buffer_owned`; interpreted YIR deep-copies the elements and remains
independent after source mutation. LLVM now emits the byte copy, recursive task
packing, and ownership-taking unpack path. Source-level observation and explicit
destruction now reach the same runtime. Straight-line exits and path-sensitive
`if` exits synthesize cleanup through that runtime, including real conditional
LLVM drop-return blocks. Ownership-neutral loops preserve outer owners across
backedges and reach post-loop cleanup. Linear per-iteration ownership cleanup is
synthesized and GLM-verified; direct-break and iterative counted copy/drop forms
now reach native LLVM, including changing-condition fallthrough and tail
`continue`. Conditional resource flow is covered, and nested resource loops
compose through static scoped helpers. Borrowed Buffer capture preserves
pointer, length, and lifetime metadata; owned resource transfer across that
boundary remains open.
Aggregate construction now has a transactional `finish` boundary: unset,
duplicate, or invalid slots poison the build and release already attached
blobs. Deferred helpers surface null as `Failed`, while immediate awaits reject
partial aggregates deterministically.
Direct floating literals inside
spawned calls still need stronger callee-parameter expected-type propagation;
explicitly typed bindings currently preserve the intended `f32` boundary.

The Nustar checks anchor the bootstrap-critical
`heterogeneous-runtime/nustar/registered-domain-contracts` cell to:

* `tools/nuisc/src/registry_contract.rs`
* `tools/nuisc/src/registry_domain_json.rs`
* `tools/nuis/src/surface_render/link_plan.rs`
* `tools/nuis/src/workflow/link_plan_domain.rs`

That keeps shader/kernel/network execution readiness in the registry contract
surface itself. Nuis workflow and link-plan readiness now consume the registry
dispatch readiness status, missing signals, bridge materialization, and
execution-readiness materialization for each heterogeneous domain. Nsld final
output blocker ordering is still the next integration point; the current
frontdoor deliberately exposes enough normalized facts for that step without
hardcoding shader/kernel/network-specific logic.

The native-binary checks anchor the bootstrap-critical
`native-binary-system/nsb-nsld/self-owned-binary-assembly` cell to the shared
Nsld final-output replay vocabulary. Nsld still owns the concrete object and
package summaries, while Nsdb owns the YIR replay transcript contract, but Nuis
now captures their shared replay facts once through
`nuis-final-output-replay-vocabulary-v1`. The typed vocabulary contains the
source replay contract, readiness/status, checkpoint and replayable-checkpoint
counts, replay command, next action/command, and first blocker. Object-package
and debugger-transcript frontdoors project that complete vocabulary rather than
reconstructing it independently:
`nsld_final_executable_output_object_package_*`,
`nsld_final_executable_output_debugger_transcript_*`, and
`closure_summary_*_debugger_transcript_*`. This keeps run-artifact, workflow,
project-status, and release/build-report surfaces aligned without coupling the
frontdoor to Mach-O, ELF, PE, or any one future object format. Ready and blocked
run-artifact regressions assert that both projections retain the same source
contract, counts, commands, and blocker. Nsdb now layers
`nsdb-yir-replay-control-v1` over that transcript: `--frame` consumes one exact
index or frame id, while `--break-at` consumes the ordered prefix through an
exact frame and reports `breakpoint-hit`. Missing or ambiguous targets fail
closed. Typed `execution_phase` and `entry_symbol` predicates now stop at the
first ordered AND-match through `nsdb-yir-breakpoint-predicate-v1`. Every
successful stop emits `nsdb-yir-replay-resume-cursor-v1` with the stopped frame
and deterministic next frame, or an explicit terminal status.
`nsdb-yir-replay-resume-input-v1` now consumes a stopped/next frame pair only
when both resolve as immediate neighbors, then replays the suffix from the next
frame; stale, mismatched, incomplete, and terminal cursors consume nothing. The
PixelMagic smoke now proves a real multi-checkpoint stop-resume-stop command
chain against heterogeneous trace records: Nsdb falls back from absent
payload-handoff events to ordered metadata/device-dispatch trace frames, persists
`nsdb-yir-replay-cursor-record-v2`, resumes exactly at the advertised successor,
and stops again through `--resume-cursor`. Cursor loading validates the record,
transcript/source contracts, and manifest before applying the exact successor.
Nuis adapts that file through `nuis-debugger-cursor-handoff-v1`, mirroring its
expected path, readiness, and status through final-output and closure summaries
without importing Nsdb types. Missing cursors remain optional/unavailable while
malformed, stale-contract, and wrong-manifest records are invalid. The next gap
was a cursor-specific resume command handoff; ready mirrors now publish that
command through final-output and closure summaries, while unavailable/invalid
mirrors publish none. Nuis now owns a first-class `debug-resume` route that
validates the abstract handoff before dispatching Nsdb with structured argv;
unavailable/invalid cursors fail before dispatch. Exact and typed breakpoint
controls plus optional cursor persistence now flow through that route. The
PixelMagic proof now uses real data, kernel, and shader records to save at the
first checkpoint, replace the cursor at the second, and resume to the third.
That work also removed the compiler's global first-two/next-two data-pipe
assumption: registered data units are stitched and validated through their own
handle-table and window ancestry. Cursor replacement now uses a synced,
same-directory temporary file, validates it through the normal loader, then
atomically renames it; invalid replacements preserve the previous cursor.
An optional sibling lineage sidecar retains the latest eight replacements as a
monotonic FNV-1a hash chain over public cursor identities. A damaged sidecar is
preserved without invalidating the authoritative cursor. Nuis does not import
Nsdb types: its artifact adapter mirrors lineage protocol, path,
readiness, status, bounded depth, latest hash, and the validated provider
dispatch identity through final-output text/JSON and closure JSON. These
surfaces copy the mirror result rather than recomputing authority. The latest
hash must match the authoritative cursor bytes.
Invalid lineage now carries a stable blocker, repair action, and executable
Nsdb command. Repair validates the authoritative cursor, archives the damaged
sidecar under a content hash, atomically rebuilds one current entry, and is
idempotent once healthy. Nuis owns the structured `debug-lineage-repair`
frontdoor while Nsdb remains the repair implementation owner; native execution
remains outside this metadata-level debugger control.

## Self-Hosting Phase Roadmap

The roadmap governance coordinate is `developer-system/dev-tensor/self-hosting-phase-roadmap`.

`stable/100` records schedule agreement, not self-hosting implementation. Its
three boundaries are:

* foundation readiness through `beta-0.9.*`
* formal `stage0 -> stage1` migration from `beta-0.10.*`
* stage2-equivalent completion window from `gamma-0.5.*` through `gamma-0.10.*`

Executable readiness is tracked independently by
`nuis-self-hosting-readiness-v1` and `nuis bootstrap-status --json`.

The five required coordinates are:

* `language-core/nuisc/bootstrap-language-subset`
* `standard-library/std/compiler-data-model`
* `language-core/nuisc/stage-neutral-ir-boundary`
* `compiler-toolchain/bootstrap/stage0-stage1-driver`
* `developer-system/bootstrap/differential-reproducibility-gate`

Subset v2 is `stable/100` through `nuisc bootstrap-check`; no gate inherits the
roadmap score, and readiness waits for all five to reach `stable/100`.

The bounded Nuis token decoder moves the stage-neutral boundary to `usable/84`;
its attested candidate and two cache-bypassed clean builds move both driver and
reproducibility gate to `usable/87`. Data model v2 remains `usable/80` through
four pages and native score `59`; the compiler data model is now weakest.

## Runtime Lifecycle Loader Bootstrap

The first beta runtime coordinate is:

`native-binary-system/nuis-runtime/lifecycle-loader-bootstrap`

Its protocol is `nuis-runtime-lifecycle-bootstrap-plan-v1`. The
platform-neutral planner consumes verified image admission, container handoff,
loader entry identity, relocation agreement, lifecycle hooks, provider
dispatch status, clock/GLM runtime-service bindings, complete mapped-section
and applied-relocation sets, and scheduler identity. It emits a strictly
ordered variable-length chain and emits no stages when any required identity drifts. `nuis-host-runner` now acts
as an adapter into this shared runtime contract rather than owning a parallel
post-validation launch sequence. Nsld derives `runtime.clock-root` from its
validated clock protocol and `runtime.glm-root` from the compiled artifact plus
heterogeneous `glm.*` sidecar contracts. Both are immutable metadata bindings,
and Nsdb now preserves the verified binding-table hash even when no provider
selection is present.

The coordinate is `stable/100`. The
`nuis-runtime-lifecycle-bootstrap-plan-identity-v1` deterministically covers
entry, section and relocation tables, every normalized mapping/application,
runtime services, lifecycle, provider dispatch, and scheduler state. Nsld
derives it from real patch application plus byte audit, persists it in the
container-loader handoff, and Nsdb independently validates it before replay.
`nuis-runtime-lifecycle-bootstrap-execution-v1` now binds one owned image
mapping plus non-cloneable section, relocation, and runtime-service
capabilities to that exact identity. Its private context is created only when
the resource sets and mapped ranges match, is consumed exactly once, activates
clock/GLM services after relocation consumption, and produces a fail-closed
`nuis-runtime-compiled-entry-transfer-v1` result. `nuis-host-runner` now gates
lifecycle entry on that transfer result rather than plan readiness alone.
`nuis-executable-memory-adapter-v1` adds the first native-host boundary. Its
Unix adapter validates section identity/hash, the fixed lifecycle-context ABI,
entry bounds, canonical target architecture, host-architecture equality, and
instruction alignment before allocating. It writes through RW pages only,
flushes the instruction cache where required, then seals the mapping RX. The
runtime test thunk returns the ABI version read through the immutable context
pointer. The call remains an explicit unsafe boundary because hashes and
architecture identity cannot prove arbitrary machine-code ABI behavior.

Nsld now emits a deterministic `nuis-native-entry-code` asset for AArch64 and
x86_64. Eight reserved bytes precede the instructions as the lifecycle
relocation slot; the loader symbol starts after that slot and hashes only the
callable bytes. The entry is appended only to the unified container, so native
object writers remain independent. The container persists
`nuis-runtime-lifecycle-entry-context-i64-v1` plus canonical `aarch64` or
`x86_64` machine identity. Nsld includes that identity in the container hash and
structured verifier, while runtime plan identity, execution context, compiled
entry transfer, and executable-memory request preserve it. Both runtime
planning and `nuis-host-runner` reject ABI, symbol-range, symbol-hash, or
machine-identity drift. Host-runner
now locates the real container payload after the capsule marker and zero
alignment padding, uses its actual hash and size as the runtime mapping
identity, restores only the declared relocation slot before checking the
enclosing section hash, and independently verifies the exact loader-symbol
code slice. Those bytes reach `NativeHostExecutableMemoryAdapter::prepare` and
become a sealed RX mapping. A tamper test
changes the inner code while recomputing the outer NSB hash and proves that no
executable mapping is prepared. Invocation authority is now type-enforced:
`ExecutableEntryPreparation` has no direct call operation. The opaque,
non-cloneable `nuis-native-entry-invocation-permit-v1` must be issued from the
same ready transfer and consumed to create an `AuthorizedNativeEntry`; permit
identity drift invokes nothing. Ordinary host-runner validation intentionally
does not issue a permit. The explicit `--invoke-native-entry` path constructs a
fixed 64-byte `nuis-runtime-lifecycle-entry-context-v1`, verifies its transfer
claim, consumes one permit, and records invocation/return evidence. The actual
Nsld AArch64 thunk reads context version `1` before returning `0`; x86_64 SysV
and Win64 use their registered first-argument registers. The loader-bootstrap
coordinate is therefore stable. The next runtime coordinate is capability-safe
lifecycle-context dispatch: opaque clock/GLM/scheduler handles are present, but
the native entry cannot yet invoke a versioned runtime service table through
them.

## Linux CUDA Provider Bring-Up

The next heterogeneous coordinate is:

`heterogeneous-runtime/linux-cuda/cuda-provider-bringup`

Its first protocol is `nuis-linux-cuda-host-probe-v1`. The repository-owned
probe separates PTX compiler readiness from driver/device launch readiness.
The first x86_64 Linux host emits PTX 8.0 for an `sm_89` vector-add kernel. A
maintenance reboot repaired the discovered driver mismatch, and the independent
`nuis-cuda-runtime-smoke-v1` fixture now proves real allocation, transfer,
launch, synchronization, readback, and `[11,22,33,44]` comparison.

The coordinate is `stable/100`. Kernel Nustar now
registers `kernel.cuda.ptx8_0.v1`, YIR emits a CUDA backend variant, and AOT
produces deterministic PTX 8.0 with both an internal source hash and the
existing payload/artifact hash envelope. The exact sidecar PTX assembles and
executes through the CUDA Driver API on the real device with
`[11,22,33,44]` output.

Kernel Nustar also owns `cuda.nvidia-gpu.bundle.v1`. The generated provider
manifest now contains four sorted bundles and cross-binds the CUDA family,
runner profile, execution adapter, and Rust static registration without adding
CUDA branches to either generic selector. The execution registration now
materializes a content-addressed 64-bit Linux Driver ABI adapter only on a
probe-ready CUDA host.

The provider-neutral `nuis-provider-code-asset-descriptor-v1` now carries
format, target, visible entry, package-relative path, byte length, digest
contract, and content hash through both single and collection requests.
Partial descriptors, absolute or traversing paths, and malformed FNV bindings
fail closed. `nuis-kernel-code-asset-registry-v1` is now the single authority
for the CUDA PTX bytes, target, entry, package-relative file name, and digest
contract. The asset also owns its minimum compute capability. The bytes are
now produced by `nuis-kernel-ptx-emitter-registry-v1`, which consumes a
`nuis-kernel-yir-codegen-table-v1`. Normal AOT reparses and verifies its
emitted project YIR, requires its CUDA target contract, and records source
version/hash, total and function-owned Kernel node counts, target, extracted
source functions, entries, typed parameters/results, and
`nuis-kernel-yir-codegen-function-v1` bodies represented by YIR
`Node`/`Operation` values. The table is emitted as
`nuis.domain.kernel.codegen-table.toml`, participates in artifact hashing, and
is passed explicitly to the PTX emitter. Registered `kernel.add_f32` and
`kernel.mul_f32` nodes lower into parameter ABI, thread indexing, bounds
guards, global loads/stores, and PTX arithmetic; unsupported instructions fail
closed. A real x86_64 project binds five Kernel source nodes, and both optional
`ptxas -arch=sm_89` differential assembly and the real CUDA Driver route accept
the table-produced bytes.

`nuis-yir-function-table-v1` now provides that missing backend-neutral
boundary. Functions carry entry/helper/provider roles, typed parameter nodes,
typed value/borrowed/owned results, and ordered body membership. Syntax
round-trips all records, verification rejects unknown or multiply owned nodes,
and lowering preserves `main`, direct-call helpers, structured returns, and
profile-ref rewrites. Empty returns infer canonical `Unit`, and a `Unit` entry
receives a deterministic `i64 0` executable ABI boundary. The real CUDA
project reports one `main` entry, five total Kernel nodes, four function-owned
Kernel nodes, and an i64 value result.

The coordinate remains active because the extracted project body uses tensor,
add-scalar-axis, reduce, and element operations while the first PTX ABI
accepts vector add/mul provider entries. The two emitted workload bodies are
still selected from the provider-neutral Kernel registration. The next step is
to select verified reachable source nodes and adapt the first supported
project arithmetic shape into an exact provider codegen entry, rejecting
unsupported shapes explicitly.
The `official.kernel` input registration verifies those emitted bytes and
produces an Nsdb-validated two-request graph. Vector-add consumes two ordered
f32 artifact inputs; scale consumes its transferable output through a
GLM/time-bound dependency edge. The Linux runner executes both entries beneath
the normal persistent Nuis worker,
dynamically resolves `libcuda.so.1` without CUDA headers or SDK linkage, loads
the hash-bound PTX, launches vector-add on a real RTX 4050, writes the result
into the worker-owned `NUISPFD1` descriptor, passes exact output comparison,
and closes the graph-owned result. The graph close now validates equal session
and persistent-worker sequence clocks, a positive Nuis dispatch receipt, the
request's complete output-role set, and its GLM ownership tokens before it
emits hash-bound `nuis-provider-completion-evidence-v1` and
`nuis-provider-glm-release-evidence-v1` records. Nsdb now rereads the actual
post-execution payload, verifies its evidence and collection hashes, recomputes
every completion and release token, and persists those structured fields in
the completion handoff. Nsld final-output replay carries them beneath the
existing final-image binding proof and immutable dispatch authority. The
official post-seal heterogeneous smoke seals the image first, executes only
after dispatch authorization, verifies the newly written payload hash, and
then observes the same completion/GLM evidence through Nsld.

`nuis-cuda-device-selection-registry-v1` now keeps selection policy inside the
CUDA provider registration. Both requests carry the
`capability-ranked-lowest-ordinal` policy plus the asset-owned `sm_80` floor,
not a concrete ordinal. The adapter enumerates a
`nuis-cuda-device-inventory-v1` inventory, chooses the highest capable device
with the lowest ordinal as a deterministic tie-breaker, and emits
`nuis-cuda-device-selection-v1`; Nsdb independently verifies the inventory
count and request-bound policy/capability values. Synthetic multi-device tests
prove ordering while the real host continues to record
`cuda:nvidia-gpu:ordinal-0:sm_89` in both native outputs. The compiler
frontdoor, request parser, Nsld object writer, and replay logic gained no CUDA
branch.

Runtime completion is deliberately not represented as immutable compile-time
image data. It is append-bound to the sealed image lineage after execution.
Nsld now writes a real ELF64-amd64 relocatable object that Linux `file` and
`readelf` accept, including 16 sections, `__nuis_entry`, and six
`R_X86_64_64` relocations. The generic seal path produces a self-contained
CUDA NSB, verifies its immutable dispatch table, authorizes post-seal execution
on a real RTX 4050, receives exact hashes `0xdc51cb8047a381e1` and
`0x40372e9bd3b02048`, proves one direct-transfer receipt and two completion
records, refreshes repeated materialization evidence, and replays verified
completion and GLM release tokens through Nsdb and Nsld. Remaining work is
breadth: inventory-backed multi-device policy, additional kernel shapes, CUDA
graphs, and native device-code evolution. NVIDIA `nvcc`/`ptxas` remain optional
differential-validation tools and must not become build, packaging, deployment,
or runtime requirements.

See
[linux-cuda-provider-bringup.md](linux-cuda-provider-bringup.md)
for the host-neutral probe command and bring-up order.

## Current Role

The first implementation is static and intentionally conservative. It is not a
replacement for tests, release checklists, or Nsld/Nuis frontdoor reports.

It is a development-system index over those surfaces, with a small drift-check
layer over the most bootstrap-critical anchors.

The first useful jobs are:

* keep CLI closure, Nsld, std, language-core, Nustar, and native-binary work in one comparable view
* make weak cells explicit instead of hiding them in broad status prose
* separate `host runnable`, `Nsld-owned ready`, and `self-owned binary assembly` as different functions instead of one overloaded "binary works" claim
* let `nuis` name the weakest bootstrap-critical coordinate without requiring a human to reread the whole roadmap
* preserve alpha milestone provenance while early-beta foundation and later self-hosting pressure grow
* recalibrate completed broad slices into narrower beta coordinates instead of erasing historical closure evidence or reporting a false project-wide 100%
* preserve the foundation-through-`beta-0.9.*`, migration-from-`beta-0.10.*`,
  and `gamma-0.5.*` through `gamma-0.10.*` completion window without confusing roadmap
  agreement with implementation completion

## Current Honesty Boundary

The tensor is a progress model, not a contract freeze.

In early beta it may still change cell names through explicit protocol
updates when the architecture changes. The stable part is the coordinate idea:

`architecture x module x function -> status/progress/evidence/next_step/task-card`

The task-card layer is intentionally small: protocol/source/status/ready,
handoff metadata, `blocker`, `next_action`, `validation_command`, and
`expected_artifact`. It lets the weakest bootstrap coordinate become a concrete
work item without turning the tensor into a full issue tracker. Its recursive
lineage adds reachability evidence rather than scheduling policy: downstream
tools can prove where a handoff came from without coupling the tensor to a
specific Nustar or finite backend combination.

Future work should move cells from static entries toward generated readings
from:
* checked tests
* frontdoor JSON fields
* Nsld reports
* docs/reference anchors
* package manifests
* roadmap milestones

The first drift checks are intentionally narrow. Future checks should become
milestone-owned instead of merely field-owned, so they can verify examples,
packages, and command workflows as well as names in source files.
