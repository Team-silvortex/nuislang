# Native Stateful Typed Effectful Loop Returns V1

This bounded CPU receipt advances the
[i64 loop-return checkpoint](nuis-native-stateful-loop-returns-v1.md).
The Git baseline remains beta-0.17.2, commit 00ab72d3. There is no release bump
or stable-ABI declaration.

## Word Readiness Repair

The first real typed workload exposed a missing shared admission case:
source floating negation already lowers to typed pack/unpack and i64 xor,
but the scoped loop temporary validator rejected those expressions.

[The repair](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs)
checks exact f32/f64-to-word and word-to-f32/f64 kinds and admits i64 xor.
Every operand recurses through the existing source-order readiness checker,
including its permitted self-read, unavailable sibling set and depth limit.
This is not a pointer cast, a general unsafe conversion or new write authority.

[Negative controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/word_temporaries_tests.rs)
reject wrong widths/kinds, unknown names, bool xor, excessive depth and a sibling
read hidden inside conversions before that sibling becomes available.
Pure and effectful value routes share the repair; Buffer and print capabilities
are not broadened. Pure coalescing/invariant optimizations remain separate.

## False And Zero Returns

[The baseline Nuis source](../../tools/nuisc/tests/native_application_bridge/state_loop_return_values.ns)
returns a mixed `State{result:bool;total:i64;gain:f32;marker:f64}` record.
The [source generator and independent oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_return_values_cases.rs)
also produce exact bool, f32 and f64 worker/callee signatures, without changing
the callback State layout. These are real parsed Nuis programs, not native
substitutes or special cases in production lowering.

All four kinds run original/mirrored guards and leading/trailing child steps:
sixteen native/reference source variants. A leading parent and its child update
current integer/bool/floating locals. A child return exits both loops; a later
parent return follows completed child work. Local continue and ordinary
fallthrough retain their own paths. The latter intentionally computes a
different bool or floating sign, so a false/zero return cannot masquerade as
an unused return seed.

The callback's event work reads actual prior State, not the different complete
event argument values. Parent-only return, fallthrough and another child return
then exercise those newly published values in order.
The actual callee computes an i64 proof check after print 30 even when a scalar
payload does not otherwise use that check.
Root 0 traps before entering the callee; root 2 passes a zero proof and traps
after its print. Successful returns still execute callback print 35 and its
post-return tail check before State publication.

Starting at total 4, the record child return publishes total 109 for trailing
steps and 120 for leading steps. The selected bool result can be false.
The raw f32/f64 payloads can be positive or negative zero without losing the exit.
Six paired raw vectors include signed zeros, signed subnormals, quiet NaNs and
negative signaling NaN payloads. Native probes compare words, not float equality.

## Execution And Failure Evidence

[Compiler tests](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_return_values_tests.rs)
check exact declared results, completed typed dependency admission, independent
return storage and unchanged pure rejection, plus manually calculated witnesses.
Computed prints and wrong return values/callee effects reject the whole route.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_return_values.rs)
run both ordinary and reversed YIR order. A flushed LLVM observer reads the real
callee's seven arguments, including bool extension and both raw floating widths,
without replacing its source computation. The shared word driver copies actual
callback output into the next input, and checks status, canaries and all four
State words.

The new main matrix checks 9,600 healthy accepted callbacks and 224 arithmetic
fatal events after 672 accepted prefix callbacks.
The reference engine also checks the last published State after failed events,
Faulted phase, rejection of later events and failed completion.
Fatal native proof is flushed prefix evidence, not post-abort memory inspection.

The new budget matrix checks 96 healthy accepted callbacks and 80 reservation
traps after 240 accepted prefix callbacks.
A two-trip parent plus a three-trip child still reserves five units even when
the child returns immediately. Budget 4 rejects before child print 10; budget 5
accepts the return and never reserves the now-unreached later child.
Unentered i64::MAX children and invalid return divisors remain inactive;
entered oversized parent/child bounds still reject.

[AOT controls](../../tools/nuisc/src/aot_application_effect_call_state_loop_return_values_tests.rs)
check exact four-word State transport, no owned-aggregate heap calls, per-site
literal-print grants and transitive Effect order.
No backend-specific return opcode, function-name whitelist or new ABI is added.

## Source-Free Delivery

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_values.rs)
covers plain/trailing and mirrored/leading source programs for each of the four
kinds: eight programs, each with fresh, cache-hit and two source-free images,
for thirty-two delivered images. Crossed guard/step combinations are native/
reference evidence above, not additional CLI images.

The CLI matrix checks 640 healthy accepted callbacks and 64 fatal events after
192 accepted prefix callbacks. Healthy sessions run before and after failures,
checking exact stdout, evolving four-field State, raw signed-zero/subnormal
round trips in both float widths and completion.
The existing CLI renders f32 through its exact widened f64 value; the oracle
matches that display contract and independently parses back to f32 bits.
Native raw-word tests, not decimal CLI output, certify NaN payload bits.

Source, manifest, project cache and original build output are removed before
restoration. Binary bytes, host objects, envelope, lifecycle, packaging mode and
the input container remain unchanged through cache hits and two restorations.

## Acceptance And Tensor

Focused serial acceptance passes 238 distinct tests, including ten new tests:
188 compiler tests, six native/reference tests (three new and three replayed
i64-return tests), thirteen existing native counted-return/typed-carry tests,
one CLI delivery test, three host fault tests and 27 tensor tests.
Changed Rust files pass targeted formatting checks; UTF-8, relative links and
source/test/document line caps pass.
This is not full-workspace, strict-Clippy, new Linux/GPU/Windows, performance,
formal-verification or global-memory-safety acceptance.

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six complete predecessor fields/hashes are retained; old validation commands
remain quoted inert here-document data. The rebuilt ordinary CLI matches source
fields, complete hashes and the ready selected task card, with clean drift/
coverage/manifest/milestone/hierarchy/lineage status.
There are 3,366 drift checks, with no failures or duplicate IDs.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `e6e62a1b9a90c42006652617ba38a37c44c9ea8d46948eb8eb18966fc43ec3bd` |
| `next_step` | `a9ff4698b64f00821a746c3e2e145eb72164da8d369f4e98cd761c8a9da8fea3` |
| `blocker` | `c8bd9ad5fa0a62e843ddae740fad72772caaca8c9bd5f6546cb57c84b9c77e8c` |
| `next_action` | `5bdb4acb9c921b8ad2ec004a565acdc0ea4e93a507a7d5e00172aea779525cae` |
| `validation_command` | `8c1160d4ee7cba9a339661b05de39b5a63d796a4454117a3a6cb66fd449744c4` |
| `expected_artifact` | `f77df27d1691516b83a887c9c66b155d4174d7b2cfe62c806f46a45a5a459d49` |

The [selected typed-return follow-up](nuis-native-stateful-loop-return-selected-v1.md)
now combines these exact typed returns with conditionally invoked children and
selected-only fallible bound math/reservations. Trailing-parent steps remain next.
Independent storage remains the baseline; effectful snapshot/coalescing
optimization needs its own prefix-preservation proof.
Generalized I/O/resources, asynchronous effects, trailing-parent composition,
broader entering-State writes, stable ABI and completed self-hosting remain separate.
