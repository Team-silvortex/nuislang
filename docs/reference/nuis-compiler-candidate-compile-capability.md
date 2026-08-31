# Nuis Compiler Candidate Compile Capability

`nuis-compiler-candidate-compile-capability-v1` is the first execution proof
that a production-bound Nuis stage1 candidate driver can accept the canonical
`bootstrap-build` request shape and produce a verified component rebuild.

It is deliberately not a generation-three transition and not a self-hosting
claim. The candidate still delegates final compilation to the exact stage0
compiler image recorded by the stage0 component. The protocol proves that the
Nuis candidate owns request admission and orchestration while the remaining
stage0 dependency stays explicit and measurable.

## Frontdoor

```text
nuis bootstrap-candidate-compile-capability \
  <candidate-build-root> \
  <stage0-provider-image> \
  <project-dir|nuis.toml> \
  <fresh-build-output> \
  <output>
```

The candidate build root must come from `nuis bootstrap-candidate-build`. The
provider image, fresh output path, and receipt path are runtime inputs. Their
filesystem paths never enter canonical identity fields.

## Candidate Ownership

The executable driver is the exact adapter already bound by
`nuis-compiler-candidate-production-v11`. Its compile route feeds every byte of
the command, project path, output path, and provider path through the existing
Nuis stage-fold and bundle-fold exports before any provider process starts.

The host compatibility layer performs only bounded path-shape checks and the
OS process operation. It uses `fork` plus `execl` with separate arguments. It
does not construct a shell command and does not use `system` or `sh -c`.

The provider path arrives through
`NUIS_BOOTSTRAP_STAGE0_PROVIDER_V1`. Before execution, the CLI verifies its
complete bytes against the stage0 component's compiler-image length and
SHA-256, stages a private reread copy, and passes only that copy to the
candidate driver.

## Verification

The frontdoor re-reads and verifies:

* the stage0 component and candidate execution proof;
* the stage1-candidate component, handoff, payloads, and production-v11 proof;
* the production-bound adapter bytes;
* the supplied stage0 provider image bytes;
* the request and result compiled artifacts;
* the complete result component record and its disk-bound payloads.

The result must satisfy the same canonical rebuild predicate used by
`nuis-compiler-component-compile-dispatch-v1`. Component identity, dependency
closure, handoff, native output, compiler image, and reproducible identity must
match. Raw compiled-artifact hashes remain separately recorded, while their
decoded path-neutral semantic identities must agree.

## Fail-Closed Boundary

The adapter exits before provider execution when the command shape is wrong,
the provider environment is absent, a runtime path is empty or overlong, or
the project and output paths are equal. The CLI rejects production-proof,
adapter, or provider-image drift before execution. A missing Nuis admission
marker, nonzero exit, nonempty stderr, or result semantic drift prevents the
receipt from being written.

The receipt always records:

```toml
replacement_authorized = false
selection_authorized = false
verdict = "candidate-compile-capability-verified-no-selection-authority"
```

## Honest Next Step

This closes the candidate compile-driver capability boundary, not self-hosting.
The signed generation-two forward image remains unchanged, and the candidate
still consumes an explicit stage0 provider. The capability is now consumed by
`nuis-compiler-candidate-preselection-v1`, whose owner signature also keeps
selection authority false. The next protocol must prove direct stage1-owned
compilation without silently treating provider delegation as self-hosting.

The machine-readable contract is
[nuis-compiler-candidate-compile-capability-v1.toml](nuis-compiler-candidate-compile-capability-v1.toml).
