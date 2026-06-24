use nuis_artifact::{
    BridgeRegistry, BuildManifest, BuildManifestDomainBuildUnit, DomainBuildUnitPayloadBlob,
    HostBridgePlanIndex, NuisCompiledArtifact, NuisExecutableEnvelope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedExecutable {
    pub artifact: NuisCompiledArtifact,
    pub envelope: NuisExecutableEnvelope,
    pub manifest: BuildManifest,
    pub domain_units: Vec<BuildManifestDomainBuildUnit>,
    pub domain_payload_blobs: Vec<DomainBuildUnitPayloadBlob>,
    pub bridge_registry: Option<BridgeRegistry>,
    pub host_bridge_plan_index: Option<HostBridgePlanIndex>,
}

impl LoadedExecutable {
    pub fn heterogeneous_units(&self) -> impl Iterator<Item = &BuildManifestDomainBuildUnit> {
        self.domain_units
            .iter()
            .filter(|unit| unit.is_heterogeneous())
    }

    pub fn payload_blob_for_domain(
        &self,
        domain_family: &str,
    ) -> Option<&DomainBuildUnitPayloadBlob> {
        self.domain_payload_blobs
            .iter()
            .find(|blob| blob.domain_family == domain_family)
    }
}
