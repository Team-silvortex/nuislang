# Beta 0.12 Application Session Snapshot

This minor-line anchor records Git checkpoint `505c820c` (`beta-0.12.2`).
The `.0` filename identifies the minor series, not a claim that every capability
was present at its first patch. Git history is authoritative; this document does
not create a release, change Cargo versions or freeze a public ABI.

## Direction

Continue the [beta-0.11 application-led agreement](nuis-beta-0.11-application-led-mainline.md):
one growing ns-nova image application exposes general compiler/runtime gaps,
drives measured optimization, and provides pressure for incremental Nuis-owned
compiler-module migration. Engine development is not itself compiler self-hosting.

The active goal remains `standard-library/ns-nova/interactive-image-workflow`.
Its prerequisite `standard-library/ns-nova/persistent-application-session`
remains `active/86`; this is triage for that slice, not overall engine completion.
The five bounded self-hosting preparation gates remain closed, while general
compiler replacement, native CPU frame dispatch and self-contained application
linking retain their own evidence obligations.

## Established Boundary

- Registered Nuis callbacks retain scalar aggregate state across host events.
  The compiled AppKit route uses one provider connection instead of replaying
  whole-module initialization for each frame.
- Nuis-generated image data reaches inline WGSL/Metal processing. Focused tests
  verify pixels, live/replay identity, worker/cache reuse and compiled export.
- Explicit Nuis cleanup, provider Finish and application success are separate.
  Typed first faults and late finalization failures survive cleanup.
- An independently registered CPU parent opens before its child, then receives
  one terminal outcome and runs its own cleanup under scoped executor fuel.
  Child resources, registries and provider authority do not transfer.
- Pump/window/C ABI cancellation returns an independent ticket, including for
  pending work. A ticket can outlive its window and reports host-scope retirement
  once, without implicit close, fabricated outcome or parent delivery.
- Provider sessions use the static `ScopeAdmission` boundary, not concrete
  cancellation/window state. The ticket ABI has no provider/window dependency.

Host cancellation is cooperative, not preemption or rollback. An admitted callback
may finish. Finish winning its gate leaves the original close reply intact;
accepted cancellation prevents subsequent Finish. A host receipt says nothing
about provider/device resource retirement or permission to reuse GPU resources.

The current compiled image executable embeds the YIR lifecycle runtime and is
launched with registered providers and sidecars. It is not yet a fully native CPU
frame path, an independently distributable single-file application, or a general
multi-provider resource-lifecycle proof.

## Practical CLI Baseline

The observable CLI regression builds and runs seven real project routes for
argv, stdin/stdout/stderr, file/text work, reports and small PGM transforms.
This is a useful base for bounded small tools, not blanket production readiness.
`cli_wc_demo` reads one 4096-byte chunk and uses ASCII word separators. Report
programs mentioning PixelMagic or WitSage are not by themselves GPU/NPU evidence.

## Still Open

- The packaged AppKit host has no explicit cancellation trigger/ticket handling.
  Ordinary quit still requests Nuis close and provider Finish.
- Parent cancellation, provider drain acknowledgement, resource-capability state,
  resource reuse, recovery and general multi-child routing are not closed.
- The window profile remains bounded to 256 dispatches and 64 MiB replay.
  Long-duration stability, process-RSS behavior and scheduling performance need
  dedicated measurements.
- General buffer-writing `while` and some aggregate early-return lowering shapes
  still require work; the image example uses a supported recursive/guard form.
- Richer image resources, native persistent CPU execution, non-Apple window
  adapters and self-contained provider injection remain separate milestones.
- Integrated rendering/control/ML/audio is the engine horizon, not current API
  coverage or an excuse to couple independent Galaxies/Nustars.

The next task is explicit packaged-host cancellation through the existing thin
ticket interface, then provider-owned drain before resource reuse. Shared
contracts do not mean shared concrete provider implementations or mutable state.

## Evidence Routes

Use the [validation checklist](nuis-beta-0.12.0-release-checklist.md) for commands.
The checkpoint's focused validation covers host sessions, compiled Nuis state,
window/ticket ABI ownership, real Metal image/window/failure paths, and tensor
consistency. Cancellation fixtures gate IPC replies and require real EOF; they
are not physical GPU cancellation tests. Independent Metal regressions prove the
existing render route still works, not that device drain is implemented.

For current details after this checkpoint, read:

1. [Mainline map](../current-mainline-map.md)
2. [Application lifecycle manifest](../reference/nuis-ns-nova-application-lifecycle-v1.toml)
3. [Window contract](../reference/nuis-yir-window-session-v3.md)
4. [Cancellation and host retirement](../reference/nuis-yir-application-cancellation-v1.md)
5. [Parent outcome pump](../reference/nuis-yir-application-outcome-pump-v1.md)
6. [Self-hosting readiness](../reference/nuis-self-hosting-readiness.md)

Rule of thumb: prove one real application slice, record its exact boundary,
and preserve independent ownership before widening it.
