# Nuis Compiler Component Differential Gate

`nuis-compiler-component-differential-v1` is the first fail-closed comparison
boundary between an attested stage0 compiler component and a separately
identified `stage1-candidate` component.

Its machine-readable contracts are:

* [nuis-compiler-diagnostic-report-v1.toml](nuis-compiler-diagnostic-report-v1.toml)
* [nuis-compiler-candidate-production-v6.toml](nuis-compiler-candidate-production-v6.toml)
* [nuis-compiler-component-differential-v1.toml](nuis-compiler-component-differential-v1.toml)

This is an `early` preparation capability. It compares evidence; it does not
create a stage1 compiler and it never authorizes replacing stage0.

## Evidence Inputs

Each side supplies a verified `nuis.compiler-component-build.toml`, its
five-stage handoff, all handoff payloads, and a sibling
`nuis.compiler-diagnostics.toml`.

The path-based frontdoor also requires the stage0 candidate-execution proof and
the candidate's `nuis.compiler-candidate-production.toml`. It verifies those
cross-bindings and the exact adapter before a differential report can be
written. The in-memory comparison builder remains reusable for protocol unit
tests, but cannot bypass this repository frontdoor requirement.

The diagnostic report binds:

* producer identity
* exact component `record_sha256`
* frozen bootstrap-subset protocol
* accepted/rejected state and semantic-pipeline state
* normalized module, code, logical path, and message records
* normalized diagnostic-set identity and exact report identity

Module identities and messages remain canonical UTF-8. Diagnostic paths are
portable logical paths: absolute locations, backslashes, and parent traversal
are rejected so two hosts cannot differ only because of workspace placement.

Successful `nuis bootstrap-build` output is necessarily `accepted`,
`semantic_pipeline = "checked"`, and diagnostic-free. The separate protocol
also supports rejected diagnostic sets so later negative stage0/stage1 probes
can use the same normalization rules.

## Frontdoor

Compare two independently produced records with:

```bash
nuis bootstrap-diff \
  stage0/nuis.compiler-component-build.toml \
  stage1/nuis.compiler-component-build.toml \
  audit/nuis.compiler-component-diff.toml
```

The first record must declare `stage0`; the second must declare
`stage1-candidate`; their producer IDs must differ. Missing or tampered
component payloads, handoffs, diagnostics, roles, or identities fail before a
differential report is emitted.

The report path must also differ from both input records, preventing the audit
write from destroying either side of the evidence pair.

Valid but non-equivalent evidence writes a canonical audit report and exits
with failure. This preserves the exact mismatch without allowing a build or
replacement workflow to accidentally treat drift as success.

## Comparison Set

The report contains a fixed ordered comparison set:

1. component ID
2. component domain
3. component unit
4. bootstrap-subset protocol
5. normalized source
6. normalized token stream
7. normalized AST projection
8. normalized NIR projection
9. canonical YIR
10. complete stage bundle
11. normalized diagnostics
12. dependency closure
13. native output

Header values are compared through domain-separated SHA-256 identities. Stage,
diagnostic, dependency, and native records reuse their already verified
lowercase SHA-256 identities.

The exact build manifest and outer compiled container are intentionally not
cross-producer equality requirements because cache bookkeeping may change them.
They remain protected by each component's exact audit record.

## Verdicts

`blocked-drift` means at least one required comparison differs.

`equivalent-awaiting-authorization` means all thirteen comparisons agree. It
still emits:

```text
replacement_authorized = false
replacement_authority_contract = "nuis-compiler-replacement-authorization-separate-v1"
```

This separation is deliberate. A future reversible replacement record must be
an independent protocol with rollback and repeated-build evidence. A
differential report cannot be edited into an authorization because its report
identity and canonical parser reject any such mutation.

## Current Boundary

The comparison engine, diagnostic sidecar, CLI, canonical readers, identity
recomputation, and drift tests are implemented. The checked-in token, AST, and
NIR page materializer is now the first Nuis-written leaf producer: it consumes
all five serialized payloads through the exact scalar ABI, emits a bound
candidate bundle fold, receives a distinct `stage1-candidate` component record,
and reaches repository-native `13/13` equivalence.

`nuis bootstrap-reproducibility` now repeats this complete path in two empty,
compile-cache-bypassed roots and binds both reports plus stable component
identities into `nuis-compiler-component-reproducibility-v1`. See
[Nuis Compiler Component Reproducibility](nuis-compiler-component-reproducibility.md).

This is not replacement authority or full compiler self-hosting. The next
closures are a non-identity transformation, independent machine/attester
evidence, and a separate reversible authorization protocol.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_diff -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_reproducibility -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_candidate_production -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_diagnostic_report -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_data_model_bootstrap -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuisc --lib parse_bootstrap_diff_command -j 1
```
