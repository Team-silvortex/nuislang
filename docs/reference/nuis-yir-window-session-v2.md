# Window Session V2 Migration

The current profile is [window session v3](nuis-yir-window-session-v3.md).
Version 2's `close(state, reason: i64)` signature is replaced by
`close(state, reason: i64, failure: i64)`. Close intent and producer-observed
failure category are distinct; neither grants completion or recovery authority.

Rebuild the Nuis close helper, its application-state artifacts and host bundle
together. The current launcher rejects v1/v2 bundles rather than supplying missing
arguments. Application-session v1 and provider IPC v3 remain unchanged.
