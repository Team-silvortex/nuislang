# Registered Window Sessions

`nuis-yir-window-session-v1` is a bounded host adapter over the
[persistent application session](nuis-yir-application-session-v1.md). It is not a
new Nuis application language, a native CPU ABI or a general GUI toolkit.

## Binding And Input

The host explicitly selects an embedded `application-session` ID. It never guesses
an ID or application function name. Before provider admission or Nuis execution,
the worker checks the shared YIR session signature and this host profile:

| Helper | Scalar Arguments After State | Result |
| --- | --- | --- |
| Open | `width: i64`, `height: i64` (no incoming state) | Owned scalar aggregate |
| Event | `kind: i64`, `code: i64` | Same owned scalar aggregate |
| Close | None | Same owned scalar aggregate |

Event kinds are `0,0` for redraw and `1,code` for one Unicode scalar. Surrogates,
out-of-range codepoints and unknown kinds are rejected. Dimensions are restricted
to 1..=16384. The Nuis helpers own frame timing, rendering decisions and state;
the host does not inspect Ns-Nova fields. The example registers `window_open`,
`window_event`, and `window_close`: initial redraw renders phase zero, space toggles
the checkerboard, and other key scalars preserve state without drawing.

Each callback may present at most one frame. Close must not present any frame.
Frames require actual RGBA8 pixels; glyph-only surfaces are rejected rather than
rasterized as substitute images. Extent/length checks cap a frame at 16 MiB of
RGBA8. These checks execute in the
worker **before provider finish**, so an invalid close presentation cannot persist
a successful provider lifecycle. The host converts accepted frames to PPM at scale
one, then to its window image. There is no alternate/reference renderer on failure.

## Compiled Host

The packer includes the runtime for statically registered sessions even without a
`cpu.tick_i64` node. Its bundle advertises this optional window contract. The
registered launch mode bypasses both `nuis_yir_entry()` and whole-module timer
execution, starts one event pump, and polls without blocking the GUI thread.
There is one initial redraw, not a 30 Hz graph replay. Raw single-scalar AppKit
key events use the same Nuis ingress; command shortcuts are left to AppKit.
IME composition, multi-scalar text, pointer/resize input and general widgets are
not provided by this first profile.

The C ABI owns an opaque, single-host-thread handle. Source text and ID are copied
before asynchronous startup. The provider environment must select exactly one
IPC or replay path. Buffer outputs must be empty and use the existing runtime
free function. Freeing the handle abandons its scope, never implicitly calling
Nuis close. Applications still run in the Rust bootstrap YIR executor; AppKit is
OS compatibility code, not an alternative implementation of the image program.

Window close/quit defers termination until the existing close/finish gate replies.
The poll timer also runs in the modal run-loop mode used by
[AppKit termination](https://developer.apple.com/documentation/appkit/nsapplication/terminatereply/terminatelater?language=objc).
Script completion schedules the quit request outside the poll timer callback,
avoiding a wait that depends on the same timer firing recursively. Cleanup
preserves failure exit status in `applicationWillTerminate`; it must not rely on
code after `NSApplication.run` being reached. Failed callbacks remain failed even
when cleanup runs, and teardown does not retry a completed/failed close.

## Running And Replaying Input

From the repository root on an Apple Silicon host with the registered Metal
adapter, after building the image example:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -q -p nuis -- build examples/projects/domains/ns_nova_image_showcase build/ns-nova-image
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -q -p nuis -- run-artifact --window-session window build/ns-nova-image
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -q -p nuis -- run-artifact --window-session window --window-events 32,128578 build/ns-nova-image
```

`--window-events` is a comma-separated sequence of up to 64 Unicode scalars. An
empty sequence still performs initial redraw and explicit close. It posts
**logical input events** to the actual AppKit queue, not fake physical keyboard
events; physical key maps cannot faithfully synthesize arbitrary Unicode text.
The sequence owns key delivery during replay, advances only after each reply, and
closes afterwards. The `run-artifact` replay frontdoor has a 180-second total child
deadline; expiry kills/reaps that child and closes its provider connection.
Interactive launch has no added total deadline. Both retain the provider's
existing 120-second I/O/idle timeout, 256-dispatch limit and 64 MiB replay budget.
These limits are not reset or worked around by reconnecting.

`--window-session` is mutually exclusive with `--json` and `--export-frame`.
The old no-option preview remains a compatibility route; it is **not migrated**
and must not be mistaken for this explicit registered mode. The one-shot compiled
frame export remains a separate complete-lifecycle regression. Provider sidecars
and the supervising launcher are still needed; this is not self-contained Nsld
injection, fully native CPU execution or a stable unlimited interactive engine.

## Evidence And Remaining Work

The compiled-window regression opens AppKit, posts logical space and non-BMP
input, verifies one open/two presentations/one close and successful exit, and
checks the actual persisted Metal pixels through identity-bound replay. It checks
one worker, sequences `0,1`, cache `compiled,hit`, physical fences, and an ignored
key with no GPU dispatch. Missing host profile/invalid script cases fail, and the
same test exercises the production `run-artifact` frontdoor. Protocol tests cover
signature drift, invalid Unicode, multiple presentations, close presentation
before finish, missing/malformed RGBA8, dimension overflow and C ABI output
ownership. This does not prove physical keyboard
layouts or IME behavior across systems.

The current Nuis event wrapper uses a leading guard returning existing state.
Bare `if { return aggregate_call(...); }` with no else is still unsupported by the
minimal lowering path; the equivalent guard form is not a compiler fix.
Sustained idle/budget handling, cancellation/resource retirement, richer input,
textures and non-Metal/native-CPU parity remain separate work.
