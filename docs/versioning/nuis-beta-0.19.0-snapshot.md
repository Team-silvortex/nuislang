# Nuis beta-0.19 Git Snapshot

Audit date: 2026-11-27.

| Identity | Git commit | Commit date |
| --- | --- | --- |
| Minor entry, beta-0.19.0 | `2b392a16` | 2026-11-13 |
| Intermediate patch, beta-0.19.1 | `bdeca227` | 2026-11-20 |
| Latest committed patch at this audit, beta-0.19.2 | `335e49bb` | 2026-11-21 |

Relative to the minor entry, the committed patches extend guarded native
resource callbacks to scalar projections of local records and checked
returns. See the [receipt index](../reference/nuis-native-session-proof-index.md)
for the individual proofs; these do not establish full native frame dispatch.

At this audit, ordered guarded buffer writes and the application-led tensor
recalibration are working-tree follow-ups, not part of the patch above. The
[mainline map](../current-mainline-map.md) tracks the native/reference frame
comparison. Use the [checklist](nuis-beta-0.19.0-release-checklist.md) to
reproduce checks; a snapshot is not a release or a CI result.
