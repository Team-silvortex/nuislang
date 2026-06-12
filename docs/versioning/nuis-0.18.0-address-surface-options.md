# Nuis 0.18.0 Address Surface Options

This file compares the realistic surface-shape choices for the current pointer
/ address family.

It is intentionally grounded in the code that already exists today.

## Current Truth

Today, nuis already has a working address core.

At the source and `NIR` surface, that core is:

* `ref Node`
* `ref Buffer`
* `borrow(...)`
* `borrow_end(...)`
* `alloc_node / load_next / store_next / load_value / store_value`
* `alloc_buffer / load_at / store_at`

At the lower runtime side, the CPU/YIR layers already treat these as pointer
values, not as ordinary nominal data:

* [crates/yir-core/src/lib.rs](/Users/Shared/chroot/dev/nuislang/crates/yir-core/src/lib.rs)
* [crates/yir-verify/src/lib.rs](/Users/Shared/chroot/dev/nuislang/crates/yir-verify/src/lib.rs)
* [crates/yir-domain-cpu/src/lib.rs](/Users/Shared/chroot/dev/nuislang/crates/yir-domain-cpu/src/lib.rs)

Short rule:

`the implementation already has pointers; the open question is surface language shape, not whether pointer semantics exist`

## Option A: Keep `ref T` As The Only Surface In 0.18.0

Shape:

* owned pointer values stay `ref T`
* borrow aliases also stay `ref T`
* owner-vs-borrow distinction remains verifier/state driven

Pros:

* matches current parser, frontend, verifier, lowering, and examples
* no broad syntax churn during `0.18.0`
* keeps existing memory / GLM / async restrictions stable
* easiest path to continue growing real examples

Cons:

* surface type alone does not tell you whether a `ref T` value is owner or
  borrow alias
* the word `ref` carries both “address-like value” and “borrowed access” vibes
* future users may expect something closer to Rust `&T` semantics than the
  current model actually provides

Best fit:

* `0.18.0`

## Option B: Rename Everything To `Ptr<T>`

Shape:

* `ref Node` becomes `Ptr<Node>`
* `ref Buffer` becomes `Ptr<Buffer>`
* `borrow(...)` still returns `Ptr<T>` unless the deeper model also changes

Pros:

* immediately signals “this is a pointer/address value”
* clearer for low-level/runtime readers than `ref T`
* lines up more obviously with YIR/runtime pointer terminology

Cons:

* mostly a rename unless owner/borrow semantics also change
* large syntax churn across parser, renderer, examples, docs, tests, and error
  messages
* risks implying a more general raw-pointer system than nuis actually exposes
* does not by itself solve the owner-vs-borrow alias ambiguity

Best fit:

* only after `0.18.0`, or together with a larger surface redesign

## Option C: Add `Address<T>` As A More Explicit Surface

Shape:

* `Address<Node>` / `Address<Buffer>`
* `ref T` could become deprecated sugar later, or remain shorthand

Pros:

* “address” is semantically broad enough for both node and buffer routes
* less “raw systems language” flavored than `Ptr<T>`
* can coexist with `ref` during a migration period

Cons:

* still mostly a rename unless semantic layers also split
* verbose compared with current source style
* introduces a second surface for the same implementation truth if `ref`
  remains supported

Best fit:

* later stabilization phase if readability wins clearly outweigh migration cost

## Option D: Split Owned And Borrowed Surface Types

Example directions:

* owner = `Ptr<T>`, borrow = `Ref<T>`
* owner = `Address<T>`, borrow = `Borrowed<T>`
* owner = `ref T`, borrow = `view T` or similar

Pros:

* surface type would finally explain the most important semantic distinction
* error messages could become much clearer
* async/task boundary rules would be easier to explain

Cons:

* this is not a rename; it is a semantic model change
* parser, type inference, verifier state, borrow-end behavior, rendering, and
  examples would all need coordinated redesign
* likely too much instability for the current `0.18.0` mainline

Best fit:

* a future minor after the current address baseline is more mature

## Recommendation

For `0.18.0`, the best choice is:

1. keep `ref T` as the only checked-in source surface
2. explicitly document that `ref` is the current address family
3. keep owner-vs-borrow distinction verifier-driven for now
4. continue strengthening compile/lowering/runtime gates around that model
5. postpone any `Ptr<T>` / `Address<T>` surface rename until there is a clearer
   semantic win

Short rule:

`0.18.0 should stabilize the pointer model before renaming the pointer syntax`

For the deeper future split between owner and borrow alias semantics, also see:

* [nuis-0.18.0-owned-borrowed-address-draft.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-owned-borrowed-address-draft.md)

## Decision Trigger For A Future Rename

Revisit `Ptr<T>` / `Address<T>` only when at least one of these becomes true:

* `ref` meaning is blocking new users from understanding ownership rules
* owner pointers and borrow aliases need separate surface types
* more memory families appear and `ref` becomes too ambiguous
* cross-domain/runtime pointer transfer semantics become important enough that
  surface naming now matters materially

## Current Workable Story

The most honest story today is:

* nuis already has a real pointer/address core
* the current source spelling is `ref`
* borrow semantics are enforced by verifier state, not by a second surface type
* `0.18.0` should make that model solid and test-backed
* any later syntax rename should happen after the model is more complete, not
  before
