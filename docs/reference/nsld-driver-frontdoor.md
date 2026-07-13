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

## Drive Modes

`nsld drive` without `--apply` is a dry run and never writes artifacts.

`nsld drive --apply` executes one whitelisted next action.

`nsld drive --apply --until-clean` repeats whitelisted next actions until one of
these stops happens:

* `clean` means no next action remains
* `not-applied` means a next action exists but the driver refused to apply it
* `repeated-next-action` means the same action would be applied again
* `max-steps` means the internal loop cap stopped the drive

`repeated-next-action` should be treated as a loop guard, not as the preferred
blocked-artifact status. Host-assisted final executable routes should now
materialize the current final pipeline once, then stop as `clean` while the
pipeline/output reports carry the remaining blocker details.

JSON output reports `mutates_artifacts` for all drive modes:

* dry-run always reports `false`
* `--apply` reports whether one step was actually applied
* `--apply --until-clean` reports whether at least one step was applied

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
`nsld drive` deliberately.

## Until-Clean JSON

The JSON shape for `nsld drive --apply --until-clean --json` is:

```json
{
  "tool": "nsld",
  "kind": "nsld_drive_until_clean",
  "completed": false,
  "applied_steps": 0,
  "mutates_artifacts": false,
  "capped": false,
  "stop_reason": "not-applied",
  "stop_command_id": "emit-native-object",
  "stop_source": "required",
  "stop_command_resolved": "nsld emit-native-object manifest.toml",
  "stop_action_reason": "future native object stage is not whitelisted yet",
  "last_command_id": null,
  "messages": ["next-action-not-whitelisted:emit-native-object"]
}
```

The stop context fields snapshot the next-action selector that caused the stop:

* `stop_source`
* `stop_command_resolved`
* `stop_action_reason`

These fields are part of the automation and debugging surface. They should stay
independent from any single final executable backend such as Mach-O, ELF,
PE/COFF, or a future Nuis-native container.
