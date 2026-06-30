mod artifact;
mod bridge_registry;
mod build_manifest;
mod clock_protocol;
mod domain_unit;
mod envelope;
mod error;
mod host_bridge_plan;
mod payload_blob;
pub mod protocol;
mod toml;

pub use artifact::{
    decode_nuis_compiled_artifact_binary, decode_nuis_compiled_artifact_section_table_binary,
    encode_nuis_compiled_artifact_binary, encode_nuis_compiled_artifact_section_table,
    encode_nuis_compiled_artifact_section_table_binary, materialize_embedded_artifact_support,
    parse_nuis_compiled_artifact, parse_nuis_lowering_index_from_source,
    validate_compiled_artifact_section_table, write_nuis_compiled_artifact, NuisCompiledArtifact,
    NuisCompiledArtifactSection, NuisCompiledArtifactSectionTable, NuisLifecycleContract,
    NuisLoweringIndex, NuisLoweringIndexUnit,
};
pub use bridge_registry::{
    parse_bridge_registry, parse_bridge_registry_from_source, BridgeRegistry, BridgeRegistryEntry,
};
pub use build_manifest::{
    parse_build_manifest, parse_build_manifest_from_source, ArtifactHashEntry, BuildManifest,
};
pub use clock_protocol::{
    parse_clock_protocol, parse_clock_protocol_from_source, ClockDomain, ClockEdge, ClockProtocol,
};
pub use domain_unit::{parse_domain_build_unit_blocks, BuildManifestDomainBuildUnit};
pub use envelope::{
    decode_nuis_executable_envelope_binary, encode_nuis_executable_envelope_binary,
    parse_nuis_executable_envelope, parse_nuis_executable_envelope_from_source,
    render_nuis_executable_envelope, write_nuis_executable_envelope, NuisExecutableEnvelope,
};
pub use error::ArtifactError;
pub use host_bridge_plan::{
    parse_host_bridge_plan_index, parse_host_bridge_plan_index_from_source, HostBridgePlanEntry,
    HostBridgePlanIndex,
};
pub use payload_blob::{
    decode_domain_payload_blob, encode_domain_payload_blob, DomainBuildUnitPayloadBlob,
    DomainBuildUnitPayloadBlobSection,
};
