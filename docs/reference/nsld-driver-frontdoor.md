# Nsld Driver Frontdoor

`nsld check-next-action` and `nsld drive` are the current automation surface for
the Nsld artifact chain.

They are intentionally small. The driver is not a shell runner and does not
interpret arbitrary command strings. It asks the artifact-chain report for the
next action, then dispatches only whitelisted Nsld actions in-process.

## Commands

```sh
cargo run -p nsld -- check-next-action <artifact-output-dir>
cargo run -p nsld -- check-next-action <artifact-output-dir> --json
cargo run -p nsld -- drive <artifact-output-dir>
cargo run -p nsld -- drive <artifact-output-dir> --json
cargo run -p nsld -- drive <artifact-output-dir> --apply
cargo run -p nsld -- drive <artifact-output-dir> --apply --json
cargo run -p nsld -- drive <artifact-output-dir> --apply --until-clean
cargo run -p nsld -- drive <artifact-output-dir> --apply --until-clean --json
```

## Next Action

`nsld check-next-action` is read-only. It reports whether a next action exists,
which source layer selected it, the stable command id, the template command, the
resolved command for the current input, and the reason.

Current source layers:

* `required` means the first missing required artifact stage selected the action
* `advisory` means required artifacts are present but a consistency/readiness
  report recommends a follow-up action
* `optional` means the remaining artifact tail can still be materialized
* `final-output-boundary` means the artifact chain itself has no more safe
  apply step, but the final executable output boundary is still blocked and
  should be inspected with `nsld final-executable-output <input>`
* `final-output-materialization` means the final executable output is ready and
  the remaining safe action only writes launcher evidence, such as
  `emit-final-executable-launcher-manifest` or
  `emit-final-executable-launcher-dry-run`

The `final-output-boundary` source is read-only by default. It makes the current
linker boundary visible to automation without turning `nsld drive --apply` into
an implicit host-finalizer runner. Host-assisted final executable emission can
cross this boundary only when both explicit gates are set:
`NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke` and
`NUIS_NSLD_ALLOW_HOST_FINALIZER=1`.

The `final-output-materialization` source is intentionally narrower: it may
write Nsld-owned launcher manifest or launcher dry-run protocol files, but it
does not execute the produced binary or jump into payload code.

## Drive Modes

`nsld drive` without `--apply` is a dry run and never writes artifacts.

`nsld drive --apply` executes one whitelisted next action.

`nsld drive --apply --until-clean` repeats whitelisted next actions until one of
these stops happens:

* `clean` means no next action remains
* `not-applied` means a next action exists but the driver refused to apply it
* `host-finalizer-policy-required` means the artifact chain reached a
  host-assisted final output that is present but not Nsld-owned, so an explicit
  host-finalizer policy gate must be unlocked before native host finalization
* `final-output-missing` means the final executable output is still absent after
  the materialization pipeline reached the output boundary
* `final-output-invalid` means the final executable output is present but failed
  a final boundary validation such as image header, size, or hash checks
* `blocked-boundary` means the artifact chain reached a read-only final-output
  boundary that must be inspected, and no narrower stop reason was available
* `repeated-next-action` means the same action would be applied again
* `max-steps` means the internal loop cap stopped the drive

`repeated-next-action` should be treated as a loop guard, not as the preferred
blocked-artifact status. Host-assisted final executable routes should now
materialize the current final pipeline once, then stop as
`host-finalizer-policy-required` when the remaining default-read-only final
executable output boundary is blocked by `final-executable-output:not-nsld-owned`.
With both host-finalizer gates enabled, `drive` may invoke the final executable
emitter for that one boundary step and report `applied final-executable-output`
only if Nsld actually emitted the final output. The pipeline/output reports
carry the remaining blocker details.

Manifests built with `packaging_mode = "nuis-self-contained-image"` select the
Nsld-owned self-contained final stage instead. For that route,
`drive --apply --until-clean` can stay inside the whitelisted Nsld pipeline,
materialize the selected `.nsb` output, and stop as `clean` without unlocking
the host-finalizer gate.

JSON output reports `mutates_artifacts` for all drive modes:

* dry-run always reports `false`
* `--apply` reports whether one step was actually applied
* `--apply --until-clean` reports whether at least one step was applied

JSON output also reports `mutation_policy`, which is the automation-readable
reason behind that boolean decision. Current values include
`read-only-artifact-observe`, `read-only-boundary-observe`,
`whitelisted-artifact-mutation`, `whitelisted-boundary-materialization`,
`blocked-read-only-boundary`, and `blocked-unlisted-mutation`. This keeps
normal artifact-chain mutation, final-output boundary observation, and explicit
boundary helpers separate without asking automation to infer policy from free
text messages.

`--apply` and `--apply --until-clean` JSON also expose a small safe-next
handoff:

* `safe_next_action`
  the next safe automation posture after the drive result
* `safe_next_command`
  a command to run only when the caller deliberately accepts the boundary
  described by `safe_next_reason`; this is usually the explicit host-finalizer
  crossing command when drive stops at a final-output boundary
* `safe_next_reason`
  a short explanation for why the command is safe to show but not safe for
  drive to run implicitly

This keeps `nsld drive` deterministic: it may report an explicit crossing
command, but it still does not silently cross the host finalizer boundary.

## Command Set

`nuis` workflow and artifact surfaces expose the same commands as a structured
`nsld_drive_command_set` or `artifact_nsld_drive_command_set` object. The object
uses `protocol = "nsld-drive-command-set-v1"` and includes
`recommended_first_json_command`, which should point at the non-mutating
`nsld drive ... --json` command.

Automation should read `recommended_first_json_command` before applying a
mutating step, then choose `apply_next_json_command` or
`apply_until_clean_json_command` only after inspecting the dry-run result.

`nuis release-check` prints the same protocol and command set summary after the
build manifest and artifact self-check pass. The release-check frontdoor does
not apply linker actions by itself; it reports the safe dry-run command first
and labels mutating commands explicitly so automation can make the handoff to
`nsld drive` deliberately. It follows the same explicit-handoff rule for
runtime/debugger metadata: release-check reports the recommended
`run-artifact --json` command, but it does not run `run-artifact` or materialize
nsdb handoff files by itself.

`emit-native-object` is accepted by the CLI and driver as a protocol-facing
alias for the deterministic object emitter. It keeps final-stage/native-object
recommendations readable while still using the existing `emit-object` pipeline
and output contracts.

## Until-Clean JSON

The JSON shape for `nsld drive --apply --until-clean --json` is:

```json
{
  "tool": "nsld",
  "kind": "nsld_drive_until_clean",
  "completed": true,
  "applied_steps": 1,
  "mutates_artifacts": true,
  "capped": false,
  "stop_reason": "clean",
  "stop_command_id": null,
  "stop_source": null,
  "stop_command_resolved": null,
  "stop_action_reason": null,
  "safe_next_action": "clean",
  "safe_next_command": null,
  "safe_next_reason": "drive reached a clean artifact chain",
  "last_command_id": "emit-native-object",
  "messages": ["applied emit-native-object", "no-next-action"]
}
```

The stop context fields snapshot the next-action selector that caused the stop:

* `stop_source`
* `stop_command_resolved`
* `stop_action_reason`

These fields are part of the automation and debugging surface. They should stay
independent from any single final executable backend such as Mach-O, ELF,
PE/COFF, or a future Nuis-native container.
