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

`repeated-next-action` is expected for some blocked/reporting routes. For
example, a host-assisted final executable route can repeatedly emit a blocked
pipeline report until a later backend or policy gate makes finalization
available.

## Until-Clean JSON

The JSON shape for `nsld drive --apply --until-clean --json` is:

```json
{
  "tool": "nsld",
  "kind": "nsld_drive_until_clean",
  "completed": false,
  "applied_steps": 0,
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
