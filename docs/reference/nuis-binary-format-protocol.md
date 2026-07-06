# Nuis Binary Format Protocol

This file is the current implementation-facing protocol note for the native
`nuis` artifact family.

The goal is not to freeze the final linker forever. The goal is to make the
current binary/container vocabulary explicit enough that a future self-owned
`nuis` linker can depend on stable names instead of reverse-engineering build
directories.

## Short Rule

`host formats launch; the Nuis binary protocol describes the program structure`

Today the host-native executable is still finalized through the host toolchain
for `native-cpu-llvm`, but the deeper program shape is already represented by
the `nuis-artifact` protocol family.

## Current Protocol Fact Source

Implementation anchor:

* [protocol.rs](../../crates/nuis-artifact/src/protocol.rs)

Current protocol id:

* `nuis-binary-format-protocol-v1`

Current schema ids:

* `nuis-build-manifest-v1`
* `nuis-compiled-artifact-v1`
* `nuis-executable-envelope-v1`
* `nuis-lifecycle-contract-v1`

Current binary containers:

* `NART` version `1`: compiled artifact container
* `NART` version `2`: compiled artifact section-table container
* `NENV` version `1`: executable envelope binary wrapper
* `NDPB` version `3`: domain payload blob

Short compatibility rule:

`schema ids describe semantic records; magic/version pairs describe binary container layouts`

That split is intentional. A future layout bump can be represented as a binary
version change without pretending the whole language contract changed.

## Layer Model

### 1. Build Manifest

The build manifest is the inspectable build index.

It currently records:

* source input and output directory
* packaging mode
* envelope path and schema
* compiled artifact path and schema
* lifecycle contract fields
* CPU target ABI facts
* bridge registry and host bridge plan indexes
* artifact hashes
* per-domain build units

The future linker should treat this as the first structured index, not as a
loose report string.

### 2. Executable Envelope

The envelope is the program-structure header.

It currently records:

* executable kind
* package count
* domain families
* contract families
* function kind
* graph kind
* default time mode

The future linker should use the envelope to answer:

* which capability families are packaged
* which contracts are expected
* which global execution/time model the container claims

### 3. Compiled Artifact

The compiled artifact is the current single-file carrier for the runnable
native bundle.

`NART` version `1` currently embeds:

* encoded executable envelope
* packaging mode
* selected CPU target ABI facts
* binary name and binary blob
* lifecycle contract
* build manifest source

For now, `native-cpu-llvm` artifacts still carry a host-linked executable blob.
Later, the same top-level protocol can carry a thinner launcher plus richer
internal sections.

`NART` version `2` is the current section-table draft. Its standard sections
are:

* `metadata_toml`
* `envelope_binary`
* `lifecycle_toml`
* `build_manifest_toml`
* `lowering_index_toml`
* `host_binary`

`lowering_index_toml` is generated from the embedded build manifest's
`[[domain_build_unit]]` records. It snapshots the package id, domain family,
backend family, selected lowering target, optional IR sidecar path, contract
family, and packaging role for each registered domain unit. This keeps the
future linker connected to lowering decisions without hard-coding
shader/kernel/network internals into the generic artifact reader.

Current write rule:

`the default writer still emits NART v1; NART v2 exists as a linker-facing draft route that the generic decoder can already read`

Current section-table validation:

* required standard sections must be present
* section names must be unique
* empty section names are rejected
* lookup helpers expose section names, raw bytes, and UTF-8 text where
  appropriate

Current tool visibility:

* `nuisc inspect-artifact --json` reports the artifact container kind and binary
  version
* `NART` v2 inputs additionally expose section count and section names
* `NART` v2 inputs additionally parse `lowering_index_toml` into lowering unit
  count, domain family summary, selected lowering target summary, and structured
  `lowering_units`
* `NART` v1 inputs report an empty section list because the v1 layout is a fixed
  field stream rather than a section table
* `nuisc` regression coverage now proves the inspect frontdoor can load a
  section-table `NART` v2 artifact
* generated link-plan JSON includes artifact container kind/version, section
  metadata, lowering summaries, and structured lowering units when the artifact
  file is available
* generated link plans carry `artifact_lowering_alignment` as a structured
  linker field, and JSON renders it by comparing artifact lowering units against
  manifest domain units by package/domain/backend/target/sidecar/contract/
  packaging role
* `nuis artifact-doctor --json` passes through the same artifact container and
  lowering visibility so the top-level workflow can diagnose artifact/linker
  drift without reopening lower-level tools manually

### 4. Domain Payload Blob

The domain payload blob is the per-heterogeneous-domain binary sidecar.

It currently records:

* domain family
* package id
* backend/vendor/device metadata
* multi-backend artifact metadata:
  `target_device`, `ir_format`, `dispatch_abi`, `backend_priority`,
  `verification`
* selected lowering target
* contract family
* packaging role
* payload kind and format
* named sections

Compatibility note:

* v3 adds the multi-backend artifact metadata fields above
* the current decoder still accepts v2 blobs and maps the v3-only fields to
  absent values

Current standard section names:

* `contract_toml`
* `lowering_plan`
* `backend_stub`
* `bridge_plan`
* `shader_ir_sidecar`
* `kernel_ir_sidecar`
* `network_ir_sidecar`

Short linker rule:

`domain payload blobs are selected by registration and ABI facts; the core compiler should not special-case shader/kernel/network internals`

## Linker Direction

The self-owned `nuis` linker should eventually consume this protocol in this
order:

1. read the build manifest
2. verify schema and artifact hashes
3. decode the compiled artifact envelope
4. resolve domain build units against registered `nustar` capabilities
5. load domain payload blobs by declared packaging role
6. freeze the internal container layout
7. ask the host toolchain only for the final launcher shell when needed

This keeps the current host linker useful while moving the semantic center of
gravity into `nuis`.

## Current Honest Boundary

Already real:

* compiled artifact, envelope, manifest, and payload blob decode paths exist
* protocol ids and binary magic/version facts are now centralized
* `NART` v2 section-table encoding/decoding exists as a compatibility-preserving
  draft path
* `nuis` frontdoors can inspect, verify, materialize, doctor, and launch current
  artifacts
* representative native control-flow binaries now compile and launch through
  the current artifact route

Not yet real:

* a self-owned final `nuis` linker
* default artifact emission through the `NART` v2 section-table layout
* embedded multi-domain payload packing inside a single final launcher shell
* cryptographic integrity or signature verification
* OS-loader integration for reading the Nuis container directly

Design pressure for the next step:

`NART v1 is enough to prove the route; NART v2 is where the linker should start learning to assemble sections without knowing every domain's internals`
