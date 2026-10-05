# Native Logical Staged Initializers v1

This bounded CPU worktree proof follows the
[ordinary staged initializer proof](nuis-native-staged-return-effects-v1.md)
after `98ca713e` (`beta-0.15.8`). It is not a release, general effectful-helper
admission or completed self-contained heterogeneous application closure.

## Selected Logical Roots

An admitted fresh exact owned `bool` initializer before or between top-level
parent prints may now have a direct `&&` / `||` root, including nested logical
children and pure checked/call-backed operands:

```nuis
if outer {
    let ready = gate && helper(produce(left));
    print(88);
    const copy: bool = ready;
    print(89);
    let sample = observe(tail);
    print(sample);
    let again = copy && helper(produce(right));
    print(90);
    return again;
}
```

The original parent entry is validated and evaluated once. Each selected
initializer retains its original position, including an unused result. Its
complete original logical tree is validated before creating helper names or
substituting local atoms. One exact typed ready snapshot is published per
computed binding, so later initializers, prints and return tails cannot replay
its computation. The unselected arm returns an inert exact `false` seed.
Actual prints remain ordered parent effects, never pure-helper statements.

The existing subsequent conditional-value pass processes the new bool-return
initializer helpers. It retains left-to-right evaluation, executes a guarded
RHS only when selected and evaluates each original LHS once. No new NIR/YIR
instruction, runtime provider, Nustar dependency, native transport limit, ABI,
GLM or ownership exemption is added.

## Authority And Budgets

Only an initializer root gains logical authority. Direct logical children
inherit it; comparisons, ordinary call arguments, record fields and other
ordinary leaves do not. Print arguments remain ordinary exact `i64` roots,
even when interleaved with an admitted logical initializer. Bool printing and
logical arguments hidden under an integer-producing call remain outside this
proof. Original exact scalar captures and original pure catalog authority are
unchanged; generated signatures cannot authorize an invalid original scope.

Original entry/binding/print/tail checks happen before snapshot substitution.
Both arms must validate before installing names, helpers or parent statements.
Unbound/private-name captures, forward uses, shadows, invalid scalar types and
an invalid second arm leave the entire module unchanged. Actual source exits
are still required. Total-only logical initializer work does not confer return
eligibility; ordinary computed-print work still cannot confer it either.

Original and expanded arms share 32 statements, 4096 expression nodes, depth
below 64 and 32 logical edges. A new staged reservation distinguishes initializer
roots from ordinary print leaves and charges every prefix root together with
the expanded tail. Removed aliases still count. Multiple initializers do not
each receive a fresh logical budget; tail fanout can exhaust the shared budget
even when the original tree fits. The older ordinary print-prefix reservation
is preserved without widened authority. Oversized malformed original roots
reject before recursive typing or cloning.

Internal/intervening control or effects, record/resource/FFI bindings, changing
or borrowed captures and arbitrary effectful helper bodies remain open. This
proof supersedes only the logical-initializer-root exclusion in the preceding
ordinary staged checkpoint; it does not erase its boundaries or receipt.

## Reproduction

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_logical_initializers --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_logical_initializers conditional_return_staged_initializers conditional_return_print_aliases conditional_return_computed_prints conditional_return_effects --test-threads=1
```

Seven new tests comprise six semantics/structure/admission/budget tests and one
default-source AOT test. An independent short-circuit evaluator supplies the
expected outcome, helper count and successful print sequence for 864 core cases:
four logical trees, three operand kinds, three tail modes, three arm shapes and
eight inputs. Inferred/typed let and const declarations rotate across inputs;
this is not the full declaration cross-product. Every source is checked in
ordinary and reversed YIR order, including nodes, edges, functions and per-body
node arrays. Another 24 probes cover unused results, work before the first
print, chained bool/i64 snapshots, computed entries and private-name collisions.
The combined 888 sources have 1776 ordinary/reversed semantic executions;
successful dependent-stage traces also check one `observe` invocation positioned
between indexed print events.

Structural tests retain exact bool helper results/seeds, sorted original scalar
captures, pure bodies, unchanged original callees, idempotence and atomic veto.
Budget tests include shared logical 32/33, expanded-tail logical 32/33,
original/expanded statement 32/33, complete node 4096/4097 and depth 63/64.
The independent native graph/word limits are not widened by NIR admission.

Twenty-four default-source AOT variants include computed entries, selected and
skipped failures, early exits, unused work, chained snapshots and true zero
returns. Seven selected failures require native trap signals; no partial stdout
or helper trace is certified at a trap. Successful runs require exact stdout and
normal process completion. The ordinary/reversed interpreter error probes
check the zero-divisor error, not a post-failure partial event trace.

## Acceptance Receipt

Logical staged-initializer verification: the 2026-10-05 follow-up passes
2101 distinct selected tests:

* 1994 compiler units in one fresh wide frontend/lowering/optimizer sweep
* Twelve separately selected native AOT units
* 58 native bridge tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

All 42 combined logical/staged-initializer, print-prefix, computed-print,
atom-alias and prior logical/return AOT tests pass. Their 30 ordinary units
overlap the wide sweep; only the twelve native units add distinct acceptance
cases. The initial seven-test focused run and post-receipt reruns are not extra
cases. Production lowering did not change after the focused native pass; a
test-oracle correction makes the unasserted partial failure expectation exact,
and the complete corrected ordinary suite passes in the fresh wide sweep.

Static drift passes 1981 literal definitions/13506 patterns. All 58 changed/new
file caps, formatting, whitespace, 4749 UTF-8 files and 3095 local documentation
links pass. The rebuilt CLI reports `active/99`, 1981/1981 live drift checks and
clean coverage, hierarchy and task-card lineage. Original evidence/task prefixes
and the 2094/2087/2079 prior receipts remain intact. The preceding ordinary staged
checkpoint remains 2094 selected passing tests at `active/98`; it is historical,
not a fresh receipt for this follow-up. The prerequisite remains active.

This proof does not establish full-workspace acceptance, fresh Linux/GPU
execution, self-contained heterogeneous application closure, a measured speedup
or formal memory-safety certification. Three historical blocked Unix-socket
tests and eleven historical strict-Clippy findings remain pending and uncounted.
