# Window Session V1 Migration

The current profile is [window session v2](nuis-yir-window-session-v2.md).
Version 1's `close(state)` signature has been replaced by
`close(state, reason: i64)` so Nuis can distinguish requested and failed cleanup.

Rebuild the registered Nuis window helpers and their compiled host bundle together.
The current launcher rejects v1 bundles; it does not silently supply a missing
argument. Shared application-session v1 and provider IPC v3 are unchanged.
