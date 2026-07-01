# `nuis` ABI Compile Vocabulary For `0.20.*`

This file is the naming anchor for the `0.19.* -> 0.20.*` transition.

It exists for one practical reason:

the repository now has enough real ABI / contract / project metadata structure
that ambiguous words are more expensive than missing features.

Use this file when the question is not only:

`what ABI-related nodes exist today?`

but:

* which names are now preferred
* which names refer to verifier-constrained facts
* which names refer to user-facing metadata views
* which names should remain distinct even when they look similar

Use it together with:

* [nuis-0.19.0-compile-workflow.md](nuis-0.19.0-compile-workflow.md)
* [nuis-0.19.0-snapshot.md](nuis-0.19.0-snapshot.md)
* [std-shader-kernel-project-contract.md](../../docs/reference/std-shader-kernel-project-contract.md)
* [yir-reference.md](../../docs/reference/yir-reference.md)

## Core Rule

For the current line, ABI-related wording should now be read as four layers:

```text
selection
  -> which ABI profile was chosen

summary
  -> a text snapshot of chosen ABI facts

target_config
  -> the executable / domain-facing target configuration node

graph
  -> the top-level summary that says which ABI layers are present for this compile route
```

Short rule:

`selection picks, summary records, target_config executes, graph indexes`

## Canonical Terms

### `ABI resolution`

Preferred meaning:

* the project-level result of `resolve_project_abi(...)`
* a sorted list of `(domain, abi)` requirements plus `explicit` vs `auto`

Do not use it to mean:

* lowered `cpu.target_config`
* one specific backend runtime token

Short rule:

`resolution is the compile-time decision set, not the lowered target node`

### `ABI selection`

Preferred meaning:

* the fact that one domain selected one ABI profile
* the bridge between project ABI resolution and domain target materialization

Current checked-in examples:

* `project_profile_shader_<Unit>_abi_selection_contract_type`
* `project_profile_kernel_<Unit>_abi_selection_contract_type`
* `project_profile_network_<Unit>_abi_selection_contract_type`

These are verifier-constrained facts.

They must encode:

* `mode`
* `abi`
* `arch`
* `runtime`
* `lane_width`

Short rule:

`selection contract means the chosen ABI has already been projected into target-facing fields`

### `ABI summary`

Preferred meaning:

* a project-level text record of ABI facts for one domain
* especially useful for domains that do not currently materialize a project
  `target_config` node

Current checked-in examples:

* `project_abi_cpu_selection_summary_type`
* `project_abi_data_selection_summary_type`

These pair with:

* `project_abi_<domain>_selection_entry`

Current canonical fields:

* `mode`
* `abi`
* `arch`
* `os`
* `object`
* `calling`
* `backend`

Short rule:

`summary records the chosen ABI facts even when there is no domain target node to execute`

### `target_config`

Preferred meaning:

* the domain-facing runtime/lowering node that downstream execution or lowering can consume

Current checked-in examples:

* `cpu.target_config`
* `kernel.target_config`
* `shader.target_config`
* `network.target_config`

Do not use `target_config` to mean:

* the ABI profile string itself
* a summary-only text node

Short rule:

`target_config is an operational node, not only a metadata line`

### `target contract`

Preferred meaning:

* a verifier-constrained text node that must agree with a sibling
  `*.target_config` node

Current checked-in examples:

* `project_profile_kernel_<Unit>_target_contract_type`
* `project_profile_shader_<Unit>_target_contract_type`
* `project_profile_network_<Unit>_target_contract_type`
* `lowering_cpu_target_contract_type`

Short rule:

`target contract is the text-side truth guard for target_config`

### `ABI graph`

Preferred meaning:

* the top-level compile-workflow summary of ABI coverage
* the shortest answer to:
  `which ABI layers are currently present in this compile route?`

Current checked-in examples:

* YIR nodes:
  * `project_abi_graph_summary_type`
  * `project_abi_graph_summary_entry`
* metadata line:
  * `graph\tmode=...`
* CLI line:
  * `project_abi_graph: graph\t...`

Current canonical fields:

* `mode`
* `domains`
* `cpu_summary`
* `data_summary`
* `kernel_target`
* `shader_target`
* `network_target`

Short rule:

`ABI graph is the index-of-indexes, not one more domain detail`

### `index`

Preferred meaning:

* an exported human-readable metadata artifact

Current canonical file:

* `nuis.project.abi.txt`

Current reading order inside that file:

1. `# mode=...`
2. `graph ...`
3. `domain ...`

Short rule:

`index means exported text view, not verifier fact node`

## Current Naming Rules

For the `0.20.*` transition, prefer these suffix meanings consistently:

* `_entry`
  * one text payload node that acts as a durable fact record
* `_summary_type`
  * one verifier-facing summary contract that must agree with its entry
* `_abi_selection_contract_type`
  * one verifier-facing domain selection contract that must agree with a
    domain `target_config`
* `_target_contract_type`
  * one verifier-facing target contract that must agree with a domain
    `target_config`
* `_target_config_auto`
  * one auto-materialized domain `target_config`

Short rule:

`entry stores, summary mirrors, selection projects, target guards, auto materializes`

## Current Output Rules

For user-facing compile output, prefer this order:

1. project summary
2. project plan summary
3. project ABI graph
4. per-domain ABI entries
5. output artifact paths

That is now the preferred outer reading order because it goes:

```text
what project
  -> what compile route
  -> what ABI graph
  -> which ABI details
  -> which written artifacts
```

## Current Scope Boundary

Do not collapse these pairs:

* `ABI summary` vs `ABI selection`
  * summary may exist without domain `target_config`
  * selection currently assumes projection into target-facing fields
* `target contract` vs `target_config`
  * one is verifier-facing text
  * one is executable/lowered truth
* `graph` vs `index`
  * graph is one logical summary object
  * index is one exported file/view that may contain the graph

## Migration Rule

For `0.19.* -> 0.20.*`, prefer:

* adding aliases only when a rename would be too disruptive
* updating docs and output wording before mass-renaming internals
* preserving current checked-in node names unless there is a strong semantic
  reason to break them

Short rule:

`stabilize vocabulary first, then rename only where the new words clearly buy readability`

## Rule Of Thumb

If `0.19.*` is where the compile workflow started becoming legible,
`0.20.*` should be where that legibility gets a stable vocabulary.
