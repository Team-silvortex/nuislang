# Window Session V1 Migration

The current profile is [window session v3](nuis-yir-window-session-v3.md).
Version 1's `close(state)` signature has been replaced by
`close(state, reason: i64, failure: i64)` so Nuis can distinguish close intent and
observed failure category.

Rebuild the registered Nuis window helpers and their compiled host bundle together.
The current launcher rejects v1/v2 bundles; it does not silently supply missing
arguments. Shared application-session v1 and provider IPC v3 are unchanged.
