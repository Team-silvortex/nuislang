# Nuis Compiler Candidate to Nsld Materialization

`nuis-compiler-candidate-nsld-materialization-capability-v1` closes the first
bounded YIR materialization slice of the active stage0-to-stage1 migration. It
proves that the signed Nuis candidate can turn the canonical fresh-source YIR
identity into an exact input consumed by Nsld without invoking stage0 or a
runtime compiler provider.

It does not claim native object bytes, general source coverage, compiler
replacement, or final selection.

## Frontdoors

```text
nuis bootstrap-candidate-fresh-source \
  <candidate-build-root> <candidate-successor> <source.ns> \
  <fresh-source-result-output> <fresh-source-capability-output> \
  <nsld-input-output> <materialization-capability-output>

nsld candidate-input <nsld-input-output> [--json]
```

The Nuis command stages one private byte-verified copy of the production-v11
adapter. It first runs `fresh-source-v1`, then runs `nsld-input-v1` against the
same source with a cleared environment and closed stdin. All four output paths
must be absent and distinct. No evidence is persisted until both candidate
executions and both independent artifact models agree.

## Candidate Ownership

Subset v8 remains frozen at twenty-one exact exports. This slice adds no export
and no language capability. The existing bundle-fold export reserves ordinals
`40..53`; the Nuis candidate interprets those ordinals as fourteen
materialization values after validating the complete source and YIR states.

The adapter prints fixed field names and transports those values. It does not
choose the entry operation, return value, target backend, time order, or GLM
shape. The artifact layer independently rebuilds the expected input from the
source bytes and rejects any semantic or authority drift.

## Canonical Input

`nuis-compiler-candidate-nsld-input-v1` binds:

* source identity `12832741133` and YIR identity `9279238763`;
* open target selector `registered-native-cpu`;
* one `Main.main` function and one `nuis-yir-return-i64-v1` operation;
* return value `7`, time ordinal `0`, and zero dependencies or relocations;
* zero GLM-owned resources under a snapshot-specific GLM contract;
* entry-symbol identity `1040689614` and materialization fold `1403051547`.

The target selector deliberately names a registry class rather than Mach-O,
ELF, COFF, Apple, AMD, or Nvidia. Concrete object format and ABI selection stay
inside registered Nsld object writers.

## Nsld Consumption

`nsld candidate-input` reparses the complete canonical input through the shared
artifact contract and reports `select-registered-object-writer` as the next
action. A malformed field, changed return value, target-specific selector, or
authority bit fails before a writer can be selected.

This is an equivalent Nsld input, not an object-writer input derived from a full
`LinkPlan`. Keeping those layers separate prevents stage1 from depending on
Nsld's internal section-layout and relocation hash implementation.

## Authority Boundary

Every valid capability keeps:

```toml
candidate_owned_yir_materialization = true
equivalent_nsld_input = true
native_object = false
stage0_handoff_required = false
provider_dependency_required = false
replacement_authorized = false
selection_authorized = false
```

The double-clean generation-three regression also changes the source literal
from `7` to `8` and requires all four output paths to remain absent.

## Next Slice

The bounded stage driver gate is now stable. A later version should hand this
input to one registered object writer, emit real native object bytes, and bind
their independent structural and content verification without weakening the
target registry or rollback chain.

Machine-readable contracts:

* [candidate Nsld input v1](nuis-compiler-candidate-nsld-input-v1.toml)
* [materialization capability v1](nuis-compiler-candidate-nsld-materialization-v1.toml)
