# Nuis Compiler Component Attestation

`nuis-compiler-component-attestation-v1` adds a cryptographic witness above the
path-free two-clean-build reproducibility aggregate. It does not replace or
mutate `nuis-compiler-component-reproducibility-v1`, and it never authorizes a
stage0 replacement.

The machine-readable contracts are
[attestation v1](nuis-compiler-component-attestation-v1.toml) and
[attester trust registry v1](nuis-compiler-component-attester-trust-registry-v1.toml).

## Signing Frontdoor

An attester first runs `nuis bootstrap-reproducibility` on its own clean
environment. It then receives a fresh 64-character lowercase SHA-256 challenge
from the verifier and signs the already verified roots:

```bash
NUIS_COMPILER_ATTESTER_SIGNING_KEY_HEX=<private-key> \
  nuis bootstrap-attest-reproducibility \
  build/repro/nuis.compiler-component-reproducibility.toml \
  build/repro/clean-build-0 build/repro/clean-build-1 \
  <challenge-sha256> <attester-id> <environment-id> \
  build/repro/nuis.compiler-component-attestation.toml
```

The private key is read only from the environment. It is never accepted as a
command-line argument and is not emitted in logs or artifacts. The output is
created without replacement, so an existing claim cannot be silently
overwritten.

Before signing, the frontdoor rereads both clean roots, their cache-bypass
manifests, candidate production v11 proofs, binaries, and differential reports.
The signature binds the exact canonical aggregate bytes, both production proof
hashes, stable compiler and native identities, challenge, attester identity,
environment identity, and `replacement_authorized = false`.

## Verification Frontdoor

The verifier provisions a canonical registry and its SHA-256 through a trusted
channel, then runs:

```bash
nuis bootstrap-verify-reproducibility-attestation \
  build/repro/nuis.compiler-component-reproducibility.toml \
  build/repro/nuis.compiler-component-attestation.toml \
  trust/attesters.toml <pinned-registry-sha256> <challenge-sha256>
```

Verification reparses both canonical artifacts, checks the caller-owned
registry pin, resolves the exact active key for the attester and environment,
requires the independent-machine trust scope, and performs strict Ed25519
verification. Wrong challenges, unpinned registry bytes, revoked keys, changed
lineage, and changed signatures fail closed.

## Honest Boundary

Cryptography proves that a registered key signed the exact lineage. It cannot
measure which physical machine held that key. Operational independence
therefore comes from key provisioning and the pinned registry policy: an
attester key registered for `linux-amd64-cleanroom` must only be available in
that environment.

Unit tests still use a fixed same-machine key to exercise signing and mutation
cases. In addition, the repository now retains a real
[Linux amd64 generation-one evidence set](../evidence/compiler-attestation/linux-amd64-cleanroom/generation-1/nuis.compiler-component-remote-evidence.toml):
two cache-bypassed clean builds reached `13/13`, a random Ed25519 seed was
generated and retained only on the attester, and the returned claim verifies
against registry pin
`90b8f7f4c9d336c72caa7dc4dc9a91c41ec263a7bfffa282ee8211088b164f01`.
The regression uses no private key and proves that the exact claim verifies
while a wrong challenge or pin fails closed. This is operational evidence of a
separately operated machine, not cryptographic proof of physical independence.
The next evidence step is a second attester or an explicit higher-generation
key-rotation ceremony.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_attestation -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis parses_bootstrap_attestation_commands -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_remote_attestation_evidence -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate two_uncached_clean_candidates_bind_one_reproducibility_aggregate -j 1 -- --test-threads=1
```
