# Native Staged Return Effects v1

This bounded CPU worktree proof follows the
[interleaved atom-alias proof](nuis-native-return-print-aliases-v1.md) after
`98ca713e` (`beta-0.15.8`). It is not a new release, general effectful-helper
admission or completed native application closure.

The later [logical staged-initializer proof](nuis-native-logical-staged-initializers-v1.md)
separately extends matching initializer roots. The ordinary-root exclusions and
receipt below describe this historical checkpoint, not that later follow-up.

## Selected Initializers

Fresh exact owned `bool/i64` bindings may now contain an ordinary scalar
computation before or between top-level prints in an admitted return arm:

```nuis
if outer {
    print(88);
    let first = observe(stamp);
    print(first);
    const second: i64 = 100 / first;
    print(second);
    let copy = second;
    print(copy);
    return helper(produce(first));
}
```

The prefix ends at the last top-level print. Each computed initializer receives
one private selected value helper and one typed ready snapshot in the parent.
Only the chosen arm evaluates its original complete initializer; the other arm
publishes an inert exact `false` or `0` seed. Later initializers, prints and the
return tail read the snapshot rather than replaying the expression. Proven atom
copies retain the existing nonexpanding substitution route.
Print arguments remain exact owned `i64`; bool snapshots may feed subsequent
computations or return predicates, not a new bool-print route.

The original entry is validated in the original parent scope and saved once.
Each original binding, print and complete tail validates before substitution.
Ready snapshots have disjoint fresh private names across sibling arms, even
when source local names match or have different types. Only original pure
catalog authority is used; generated helper signatures authorize nothing.
Initializers capture exact ready scalar atoms, including earlier snapshots,
never future values, resource handles or unevaluated arguments.

All actual prints remain in the parent, in source order. Unused initializers
remain executed in their original selected position, including checked work
and calls whose complete record result contains an otherwise unused check.
This is not dead-code elimination and does not make failed work speculative.

## Exit Authority And Limits

A separately proven checked/call-backed selected initializer may supply the
work authority for a pure scalar return tail. It cannot invent a source return
or turn fallthrough into an exit. Ordinary computed print arguments still grant
no tail eligibility; a total arithmetic initializer alone does not supply this
authority either. Actual exits, partial readiness, continuation suffixes and
zero return values retain their existing independent proofs.

Both original arms and the final staged tail must succeed before names,
helpers or parent statements are installed. An invalid second arm leaves the
entire module unchanged. Preflight precedes recursive typing and cloning.
Every removed copy and every complete initializer/print expression remains
charged against original and expanded admission: 32 statements, 4096 expression
nodes, depth below 64 and the tail's separate 32 logical edges. No native word
limit, registered interface, YIR instruction, ABI, GLM or ownership exception
is added.

Initializers and print arguments remain ordinary roots, without logical-leaf
permission. Logical initializers, internal/intervening control or effects,
record/resource bindings, borrowed or generic captures, mutation and FFI regions
remain outside this proof. Inline pure record production may be used only as
part of a validated complete scalar expression; it grants no record transport.

## Reproduction

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_staged_initializers --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_staged_initializers conditional_return_print_aliases conditional_return_computed_prints conditional_return_effects --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_prefixes_retain_outer_rebind_effect_kind_scope_and_shape_vetoes --test-threads=1
```

Seven new tests include six semantic/structure/admission/budget tests and one
default-source native AOT test. The independent core oracle covers 616 cases
and 1232 ordinary/reversed YIR executions; reversal includes nodes, edges,
functions and each function's body-node array. Another 39 semantic probes cover
bool/i64 tail reuse, unused work, chained stages, current parent rebinding and
same-name differently typed sibling locals. A separate runtime hygiene probe
forces collisions with private name prefixes. Successful traces check once-only
`observe` calls and their position between indexed print events.

Structural probes check pure typed helpers, exact inert seeds, minimal original
captures, unchanged original callees and idempotence. Malformed NIR probes check
original scope/type admission, self/forward/unbound uses, shadows, generated-name
capture, effectful callees, hidden logical leaves, borrowed/optional/generic captures,
no-source-return veto and whole-module rollback. Budget tests retain original
and expanded 32/33, complete 4096/4097 and depth 63/64 boundaries; oversized
ill-typed original roots reject before recursive inference.

Eighteen AOT variants include selected/skipped initializers, dependent stages,
pure-tail reuse, unused work, partial/continuation behavior and real zero exits.
Five selected failures are checked through native trap signals; buffered stdout
at a trap is not certified. Calls and prints have not been benchmarked.

## Acceptance Receipt

Staged initializer verification: the 2026-10-05 follow-up passes
2094 distinct selected tests:

* 1988 compiler units: 1987 successful units in the wide frontend/lowering/optimizer sweep and one corrected existing prefix-boundary unit rerun
* Eleven separately selected native AOT units
* 58 native bridge tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

All 30 combined print-prefix/computed-print/alias/staged-initializer focused
tests pass. Their six native units and five separately rerun prior native units
make the eleven native units above; overlapping ordinary tests and post-receipt
reruns are not extra acceptance cases.

The first wide run passed 1987/1988 units and failed one obsolete rejection
assertion for a call-backed bool initializer followed by a print and return.
That existing test now positively executes the newly admitted shape and retains
an internal-print/control-flow veto instead. The complete corrected unit and
five prior native units pass together. Production lowering did not change after
the wide run; this receipt does not describe that initial run as all green.

Static drift passes 1971 literal definitions/13447 patterns. All 52 changed/new
file caps, formatting, whitespace, 4743 UTF-8 files and 3089 local documentation
links pass. The rebuilt CLI reports `active/98`, 1971/1971 live drift checks and
clean coverage, hierarchy and task-card lineage. Original evidence prefixes,
task prefixes and historical receipts remain intact. The prerequisite remains
active, not complete. The historical atom-alias checkpoint remains 2087 distinct
selected tests at `active/97`, not this follow-up's receipt.

This proof does not establish full-workspace acceptance, fresh Linux/GPU
execution, self-contained heterogeneous application closure, a measured speedup
or formal memory-safety certification. Three historical blocked Unix-socket
tests and eleven historical strict-Clippy findings remain pending and uncounted.
