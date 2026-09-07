# Application Outcome Pump

`nuis-yir-application-outcome-pump-v1` carries one
[owned terminal delivery](nuis-yir-application-outcome-v1.md) to a separately
registered parent, then explicitly closes that parent. It is a bounded host
adapter, not a new Nuis syntax or a general multi-child supervisor.

## Registration And Ownership

The existing application-session registration supplies three helpers:

| Callback | Arguments | Return |
| --- | --- | --- |
| Open | None | Owned scalar aggregate |
| Event | Flattened state, then status/cleanup/failure as value-owned i64 | Same aggregate |
| Close | Flattened state only | Same aggregate |

The caller selects an ID, not function names inferred by the compiler or host.
`ApplicationOutcomePump::spawn` receives an explicit registry. Its worker owns
one verified context and opens the parent once. After the Open reply is consumed,
`finish_with_outcome` consumes one delivery and explicitly authorizes one event
followed by one close. Cleanup is attempted even when that event fails, with the
last accepted state. Effects are not rolled back; delivery and close are not
retried. A failed parent event or close cannot produce a successful pump result.

Only scalar data crosses from the child. Parent state, effects and budget remain
independent; no child registry, transport, resource handle or physical completion
witness is transferred. A successful parent may record a failed child, but cannot
change the child's immutable result or certify its resource retirement.

Poll and Drop do not execute Nuis code or join the worker. Dropping an idle parent
abandons it without cleanup. Already admitted work is not cancelled by dropping
its handle. Reply state is descriptive scalar data, not completion authority.

## Packaged Window Profile

Rebuild the artifact to obtain `window_parent_contract` in `bundle.txt`, then:

```sh
nuis run-artifact --window-session window --window-parent-session parent --window-events 32,128578 build/ns-nova-image
```

The parent ID must be distinct from the child ID. Both are embedded static
registrations. Missing or mismatched parent signatures fail before child admission.
Without the explicit option, the previous window path is unchanged.

The AppKit bridge opens the parent asynchronously and waits for its Open reply
before opening the child. It polls both workers without invoking Nuis callbacks
on the GUI thread. After a terminal child reply, it issues the one owned delivery
and waits for parent event/cleanup before completing deferred termination. Parent
failure forces a failed process exit; parent success never clears child failure.
Failure before child-handle creation abandons the parent, not implicit cleanup.

The compatibility adapter explicitly admits CPU orchestration only. Other
registered modules remain available for verification of embedded child code,
but their execution and branch effects are rejected in the parent's registry.
There is no reference GPU/network fallback and no inherited provider connection.
This CPU profile is local to the packaged adapter, not a restriction on the
generic pump's explicitly supplied registry. It is not a security sandbox.

The packaged parent uses **root-scoped initialization**. The executor follows
all three callback bodies, static function operands, described dependencies and
incoming graph edges, including ordering edges. It initializes only global nodes
in that closure once. Unrelated Shader/Data globals do not execute just because
the child shares the embedded module. A global side effect required by the parent
must be an explicit dependency. Root-discovery is conservative about function
symbols in operands; it is not a minimal dead-code optimizer. Invoking a function
outside the admitted closure is rejected. Ordinary `FunctionSession::new` and
generic pump startup retain whole-module global initialization semantics.

Initialization, open, event and close each receive their own 100,000-step budget
in this adapter. A step is one scoped function entry or graph node, shared through
nested calls and loops. Verification, registered executor internals, allocation,
blocking FFI and device execution are not preempted by this fuel. The worker keeps
GUI polling nonblocking, but this is not a hard wall-time or cancellation promise.

## Evidence And Limits

Runtime tests cover one-attempt delivery, retained state, separate cleanup fuel,
init/open/close exhaustion, invalid ingress, no cleanup on Drop, root-dependent
global effects, helper dependencies and rejected unadmitted execution. Compiled
Nuis tests inspect the showcase parent's actual scalar state and empty provider
witnesses. The real Metal window regression checks parent-before-child ordering,
two exact GPU frames, one device worker, one delivery and one parent close. The
same binary's injected provider-exchange and replay-exhaustion cases stay failed
after successful parent processing, without replacing earlier replay evidence.

This is still embedded-YIR execution. Native parent dispatch, independent parent
device scopes, multiple-child routing, explicit cancellation/resource retirement,
durable delivery, long-duration soak and non-AppKit adapters remain open. It does
not establish a scheduling-performance advantage over another compiler/runtime.
