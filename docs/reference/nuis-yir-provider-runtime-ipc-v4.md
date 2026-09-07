# Provider Runtime IPC v4

`nuis-yir-provider-runtime-ipc-v4` replaces v3's text-only rejection with a
producer-typed, request-bound failure envelope. This is a shared YIR contract,
not an ns-nova, Metal, Kernel or OS-specific error vocabulary.

## Wire And Admission

The existing four-byte little-endian header length, UTF-8 newline-separated
fields, bounded binary uploads and result payloads remain. A rejection header is:

```text
nuis-yir-provider-runtime-ipc-v4
rejected
<phase>
<sequence>
<code>
<diagnostic>
```

The previously admitted target and private connection bind the source identity.
The sequence must exactly match the pending dispatch or Finish count. Integers
are canonical decimal, without signs or leading zeros. Dispatch rejection uses
`0..255`; receive/Finish rejection can also carry `256`, allowing a fully used
session to finish without granting a 257th dispatch.

| Phase | Meaning | Permitted Codes |
| --- | --- | --- |
| `receive` | Request could not be read or admitted at the current frontier | Request, Exchange |
| `dispatch` | Failure handling the admitted dispatch | Request, Budget, Execution, Result, Exchange |
| `finish` | Failure validating/completing the received Finish | Result, Finalization, Exchange |

A client accepts `receive` at either pending operation because a failed request
read may not identify its message kind. Otherwise the phase must match the
pending operation. Unknown phases/codes, illegal phase/code pairs, stale/future
sequences, missing/extra fields and out-of-range counts fail closed. No rejection
advances the sequence, grants completion, resets budgets or authorizes a retry.

## Producer Categories

| Wire Code | Name | Producer Boundary | Application Failure v2 |
| --- | --- | --- | --- |
| `1` | Request | Target, ordering or message admission rejected | `ProviderRejected` (`4`) |
| `2` | Budget | Registered output reservation rejected before execution | `ProviderBudget` (`9`) |
| `3` | Execution | The admitted execution operation returned an error | `ProviderExecution` (`10`) |
| `4` | Result | Result identity/extent or completion evidence failed validation | `ProviderContract` (`6`) |
| `5` | Finalization | Provider close or evidence publication failed | `ProviderFinalization` (`11`) |
| `6` | Exchange | Request/reply I/O, serialization or framing failed | `ProviderExchange` (`5`) |

Wire codes and application failure codes are separate namespaces. The client
validates the envelope before projecting a category. A well-formed but unrelated
rejection becomes a local `ProviderContract` failure; malformed wire remains an
exchange/framing error. Diagnostic wording never chooses the category.

Execution is deliberately a boundary category. The current execution adapter
still returns String diagnostics that may describe validation, worker or device
errors; v4 does not claim to distinguish device loss, cancellation or driver
failure. Likewise Exchange does not split transport from framing, and
Finalization does not make publication transactional.

The server retains the full local diagnostic and appends a later provider-close
error without replacing the first code/phase/sequence. Only the wire diagnostic
is sanitized: control characters are removed, at most 60 Unicode scalars remain,
and an empty result becomes a nonempty generic rejection. This fits the existing
256-byte field limit without splitting UTF-8. The text carries no authority.

## Lifecycle Boundary

The server still reserves replay space before device work without refund.
`Closed(count)` requires ordered Finish, successful provider close and successful
evidence persistence. A publication/close failure sends no success acknowledgement.
Rejection delivery is best effort after a transport failure; inability to deliver
it does not turn the server result into success.

The scoped application retains the first admitted failure through Nuis cleanup.
Window v3's `(state, reason, failure)` close signature is unchanged; its failure
vocabulary is now `nuis-yir-application-failure-v2`. Codes `0..8` retain their
meaning and `9..11` add the remote boundary categories. Unsupported Nuis codes
remain `Unknown`, never success.

A Finish rejection may arrive after the Nuis close callback. The terminal host
reply reports it, but cannot retroactively change the earlier Nuis state or run
close again. Explicit late-failure observation by Nuis and owned-resource
cancellation remain separate work. Failure before target Hello is also outside
this admitted-session envelope and can still surface as admission/disconnect.

## Migration And Evidence

Rebuild the supervising tools, embedded runtime/host artifact and Nuis library
helpers together. Mixed v3/v4 peers are rejected by the header contract; there
is no implicit negotiation, text-only fallback or reconnect. Window callback
signatures do not change. Existing replay needs its normal source/target checks;
changing Nuis helpers changes source identity, so do not re-label old evidence.

Core tests cover boundary roundtrips, malformed/legacy messages, exact sequence
admission and bounded UTF-8 diagnostics. Server tests check reservation-before-
execution, retained accounting, result validation and first-fault finalization.
Client/window tests propagate typed failures into Nuis cleanup and reject
wrong-phase/wrong-sequence replies without advancing or repeating close. The
compiled Metal window regression injects a request-reader failure and checks
`ProviderExchange`, failed exit, one cleanup and unchanged saved replay; explicit
replay exhaustion remains a distinct `ReplayExhausted` failure.

This does not establish hardware-specific error taxonomy, recovery, unlimited
sessions, resource retirement, fully native CPU lowering or self-contained Nsld
provider injection.
