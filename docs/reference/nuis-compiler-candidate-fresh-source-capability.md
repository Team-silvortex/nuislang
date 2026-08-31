# Nuis Compiler Candidate Fresh-Source Capability

`nuis-compiler-candidate-fresh-source-capability-v1` is the first bounded proof
that the stage1 candidate consumes source bytes which were not supplied through
a preexisting stage0 handoff.

The capability intentionally covers one canonical UTF-8/LF source snapshot.
It proves the ownership boundary before widening the parser surface, and it
does not claim general Nuis parsing, native object materialization, component
replacement, or final selection.

## Frontdoor

```text
nuis bootstrap-candidate-fresh-source \
  <candidate-build-root> <candidate-successor> <source.ns> \
  <fresh-source-result-output> <fresh-source-capability-output> \
  <nsld-input-output> <materialization-capability-output>
```

The command requires all four outputs to be absent and distinct. It stages a private copy of the
production-v11 adapter, clears the process environment, closes stdin, and
executes `fresh-source-v1` followed by `nsld-input-v1` without a shell or
runtime compiler provider. Fresh-source capability v1 remains unchanged; the
second execution produces a separately versioned materialization capability.

## Canonical Snapshot

V1 accepts exactly this 56-byte, five-line source:

```nuis
mod cpu Main {
  fn main() -> i64 {
    return 7;
  }
}
```

The CLI first admits canonical UTF-8/LF text without knowing this exact value.
The candidate then consumes every byte through its Nuis-owned scalar state
machine. An independent artifact-layer implementation rebuilds the expected
result from the source bytes and rejects any other snapshot.

This division matters: changing `7` to `8` reaches the candidate boundary and
fails before either create-new evidence file is persisted.

## Stage Evidence

The candidate emits one canonical 18-line
`nuis-bootstrap-candidate-fresh-source-result-v1` record. It binds five ordered
stage identities:

| Stage | Records | Identity |
| --- | ---: | ---: |
| source | 5 | 12832741133 |
| tokens | 16 | 8634151688 |
| AST | 5 | 16043672006 |
| NIR | 6 | 12661455449 |
| YIR | 6 | 9279238763 |

Their bundle fold is `357450558`. The result is canonical, path-free, reread
after persistence, and independently replayed before the capability is
accepted.

These values are front-end stage identities, not serialized general-purpose
token, AST, NIR, or YIR files. V1 is a constrained compiler kernel that proves
fresh-source ownership for one snapshot; widening grammar and representation
coverage requires a successor protocol.

## Bound Lineage

The capability binds:

* the verified stage1 candidate component and compiler-image identities;
* the deep-verified production-v11 proof and exact adapter bytes;
* the canonical successor-v1 source, file hash, and proof identity;
* the exact source bytes and SHA-256 identity;
* all stage counts, stage identities, result bytes, and result bundle fold;
* process exit `0`, empty stderr, cleared environment, and closed stdin.

The successor source is an immutable predecessor identity anchor. This
capability does not inherit or exercise successor signing authority; a later
signed transition must replay the predecessor trust chain before using this
evidence for selection.

## Authority Boundary

Every valid capability contains:

```toml
stage0_handoff_required = false
provider_dependency_required = false
candidate_owned_source_processing = true
direct_stage1_compile = true
fresh_source_compile = true
native_materialization = false
replacement_authorized = false
selection_authorized = false
```

The verdict is
`candidate-owned-canonical-fresh-source-front-end-verified-no-native-or-selection-authority`.
No field authorizes a compiler replacement or changes the selected component.

## Successor Materialization Slice

The bounded next boundary is now closed by
[Nuis Compiler Candidate to Nsld Materialization](nuis-compiler-candidate-nsld-materialization.md).
It carries this snapshot's YIR identity into an independently verified,
candidate-owned equivalent Nsld input without invoking stage0. Real native
object bytes and wider source coverage remain later versioned slices.

The machine-readable contract is
[nuis-compiler-candidate-fresh-source-capability-v1.toml](nuis-compiler-candidate-fresh-source-capability-v1.toml).
