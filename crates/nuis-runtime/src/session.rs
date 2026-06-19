use nuis_artifact::{
    BridgeRegistry, BuildManifest, BuildManifestDomainBuildUnit, HostBridgePlanIndex,
    NuisCompiledArtifact, NuisExecutableEnvelope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedExecutable {
    pub artifact: NuisCompiledArtifact,
    pub envelope: NuisExecutableEnvelope,
    pub manifest: BuildManifest,
    pub domain_units: Vec<BuildManifestDomainBuildUnit>,
    pub bridge_registry: Option<BridgeRegistry>,
    pub host_bridge_plan_index: Option<HostBridgePlanIndex>,
}

impl LoadedExecutable {
    pub fn heterogeneous_units(&self) -> impl Iterator<Item = &BuildManifestDomainBuildUnit> {
        self.domain_units.iter().filter(|unit| unit.is_heterogeneous())
    }
}
