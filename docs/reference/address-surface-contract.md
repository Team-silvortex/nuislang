# Address Surface Contract

This file records the current front-end surface syntax for the `ref` address
family as it exists in the repository today.

It is intentionally narrow.

If a future version expands the address surface, update this file by extending
the allowed sugar list rather than silently changing examples and tests.

## Current Address Families

There are two current address families:

* structural addresses: `ref Node`
* buffer addresses: `ref Buffer`

The semantic ownership rule still lives below this surface layer in:

* [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)

## Source-Level Rule

For ordinary `.ns` source, prefer the surface forms in this file.

Treat the builtin names below as lowering targets and semantic anchors, not as
the preferred front-door spelling for examples and new source modules.

Current repository truth:

* checked-in `.ns` examples and stdlib modules now use the surface forms
* builtin spellings remain the implementation-facing truth below the surface
* YIR / CPU docs and IR-facing notes may still name the builtin forms directly

## Canonical Builtins

Current address operations ultimately lower through these builtin forms:

* `null()`
* `borrow(ptr)`
* `borrow_end(alias)`
* `move(ptr)`
* `alloc_node(value, next)`
* `alloc_buffer(len, fill)`
* `is_null(ptr)`
* `load_value(ptr)`
* `load_next(ptr)`
* `buffer_len(buffer)`
* `load_at(buffer, index)`
* `store_value(ptr, value)`
* `store_next(ptr, next)`
* `store_at(buffer, index, value)`
* `free(ptr)`

## Structural Sugar

Current structural read sugar:

* `!head` -> `is_null(head)`
* `*head` -> `load_value(head)`
* `head.value` -> `load_value(head)`
* `head.next` -> `load_next(head)`

Current structural write sugar:

* `head.value = v;` -> `store_value(head, v);`
* `head.next = next;` -> `store_next(head, next);`

Current structural limits:

* `*expr` currently only accepts `ref Node`
* `ref Node` field sugar currently supports only `value` and `next`
* unknown structural fields are rejected explicitly
* ordinary source should prefer `.value` / `.next` over explicit
  `load_value(...)` / `load_next(...)`

## Buffer Sugar

Current buffer read sugar:

* `buffer.len` -> `buffer_len(buffer)`
* `buffer[index]` -> `load_at(buffer, index)`

Current buffer write sugar:

* `buffer[index] = value;` -> `store_at(buffer, index, value);`

Current buffer limits:

* `ref Buffer` field sugar currently supports only `len`
* `buffer.len` is read-only
* index sugar is currently read/write only; it does not imply slicing, views,
  or pointer arithmetic
* ordinary source should prefer `buffer.len` and `buffer[index]` over
  `buffer_len(...)` / `load_at(...)` / `store_at(...)`

## Ownership Reminder

The surface syntax does not distinguish owner pointers from borrowed aliases.

Both still appear as `ref T`.

Current truth is:

* read sugar may flow through owned or borrowed readable addresses
* write sugar still depends on owner authority
* borrow/owner rejection is enforced by verifier/lowering rules, not by a
  separate surface type

## Current Scope

Treat the current sugar surface as:

* a readability layer on top of existing address builtins
* not a full generalized pointer language

Not part of the current contract:

* pointer arithmetic
* arbitrary dereference targets
* arbitrary pointer field projection
* generalized assignment targets
* borrowed-write relaxation
