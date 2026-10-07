# Native Literal Print Policy V1

This beta-0.16 worktree follow-up admits a small, explicit native callback effect
profile. Registered open/event/close callbacks can carry exact scalar state while
executing authorized literal i64 prints. The default pure bridge is unchanged.
It advances the [typed selected-return proof](nuis-native-typed-effect-returns-v1.md)
from ordinary source-function AOT to registered native callback transport, not to
a default native application host or a self-contained heterogeneous image.

## Explicit Grants

`LiteralPrintPolicy::new(nodes)` accepts at most 64 distinct nonempty YIR node
names. `emit_registered_with_literal_prints(module, id, policy, loop_limit,
entry_limit)` rechecks the entire registered module and selected acyclic CPU
closure before returning any LLVM unit. An empty policy grants no effects.
Unknown, unselected, duplicate, non-print and surplus grants reject. Names are
per-emission selectors, not persistent identities, signatures or trust tokens.

Every selected print needs its own grant, including prints inside reachable
helpers. Authorizing a parent never implicitly authorizes its callees. This
profile accepts only `cpu.print` and `cpu.guard_print`, whose printed operand is
a direct `cpu.const` or `cpu.const_i64` containing a valid i64 literal in the same
function. Computed values, typed i32/f32/f64/bool prints, strings, resources,
print-return compound operations and other effects are excluded. Guarded prints
require an exact bool value, not a numeric truthiness conversion.

The existing pure opcode admission list is not extended. The separate policy
supplies only the checked effect authority; strict scalar lowering still checks
all reached value-producing nodes. Print operations are effects, not fictitious
SSA values. They require actual printable inputs, not deferred lowering.

## Order And Work Bounds

Within a function, granted sites must be dependency-ordered with other effects,
calls, loops, returns, guards and checked division/remainder. The check uses both
semantic dependencies and stored edges, not vector order. Effect authority is
propagated through the selected call graph only for order checking: calls into
effectful helpers must themselves be ordered. It does not confer new grants.

Order proof uses bounded upstream/downstream traversals, not an all-pairs node
matrix. Existing closure, node, loop and helper-entry bounds still apply. Every
exported invocation has fresh independent work counters. With `S` admitted static
print sites and helper-entry limit `E`, at most `S * E` source prints can execute;
the bridge reports this conservative bound in u128 without multiplication wrap.
This is not a separate print fuel counter, elapsed-time guarantee or I/O quota.

ABI shape and canonical input checks still precede any source-body entry,
printing or output writes. Overlapping input/output buffers are allowed because
all input words are read first. Returned state is published only after successful
completion. A fatal trap never publishes a partial state, but earlier successful
prints are irreversible; the profile promises no rollback, catch or retry.

## Registered Native Evidence

The shared Nuis source covers exact i32/f32/f64 selected returns, let/const
bindings, full and partial tails, exit-only continuing tails and prefixes with
or without internal exits. The default pure bridge rejects every effectful
variant. Only source-parent print nodes receive explicit grants.

Thirty linked native artifacts run 1296 input/order/mode cases through actual
registered open/event/close exports, for 3888 successful transitions. Each event
and close receives the preceding native returned state, not an oracle-generated
replacement. The driver checks all nine state words, status, six admitted
source-function entry counters, exact print order and surrounding canaries.
The raw-word oracle covers sign-extended i32 extremes and floating signed zeros,
subnormals, finite values and signed quiet/signaling NaN payloads.

Registered YIR execution independently checks the same transitions, values,
selected calls and prints in ordinary and reversed node/edge/function/body order.
Native counters instrument admitted compiled source entries after the existing
fuel check; they never replace the source calculation. A test-only delegating
print wrapper flushes observations without changing production I/O.

The same native artifacts exercise 1010 rejected callback invocations: 450 invalid
shapes and 560 noncanonical inputs, including unused prior-result slots. These
return status 1 or 2 with no source entries, no prints and unchanged outputs.
Twenty-four separate trap artifacts
cover both selected divisions, zero root-entry fuel and selected helper-entry
exhaustion across all three kinds and both storage orders. Before the original
fatal trap, a test observer checks all nine output canaries remain untouched.

## Reproduction And Acceptance

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- native_literal_print_policy --test-threads=1 --quiet
```

Literal-print policy acceptance: The 2026-10-07 selected acceptance passes 266
distinct tests across completed cohorts:

* 171 LLVM lowering tests, including ignored tests
* 50 native bridge/ordinary-AOT tests
* Eight compiler selected-return and pure/effect-boundary tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

All six newly added tests pass. Focused reruns overlap the 50-test native cohort
and add no distinct units. The final focused run additionally checks noncanonical
unused prior-result slots; that is test-only coverage, not a production change or
an additional broad native rerun. Earlier receipts are not added to this selection.

Static drift passes 2112 definitions/14410 literal patterns. All 88 changed/new
file caps, formatting/whitespace, 4766 recognized UTF-8 files and 4729 local
Markdown links pass. The rebuilt CLI confirms 2112/2112 drift checks and clean
coverage, hierarchy and lineage. All six complete preceding typed-return fields
pass SHA-256 history identity checks. Post-receipt tensor reruns overlap this selection.

The previous typed-return receipt remains historical and is not added to this
selection. The mainline remains `active/99`; the six complete prior tensor fields
are preserved under named typed-return history markers.

At this historical API checkpoint the policy was not automatic application
build/cache/launch selection. The subsequent
[explicit build policy](nuis-native-literal-print-build-policy-v1.md) carries checked
grants and limits through those boundaries without widening the pure default. No
fresh Linux/GPU/Windows run, full-workspace acceptance, measured speedup, formal
memory-safety certificate or default native application closure is claimed.
Historical socket and strict-Clippy findings remain pending.
