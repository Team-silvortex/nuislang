use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::RuntimeError;

pub trait DomainAdapter: Send + Sync {
    fn adapter_id(&self) -> &'static str;
    fn supports(&self, unit: &BuildManifestDomainBuildUnit) -> bool;
}

#[derive(Default)]
pub struct AdapterRegistry {
    adapters: Vec<Box<dyn DomainAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, adapter: Box<dyn DomainAdapter>) {
        self.adapters.push(adapter);
    }

    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }

    pub fn resolve<'a>(
        &'a self,
        unit: &BuildManifestDomainBuildUnit,
    ) -> Result<&'a dyn DomainAdapter, RuntimeError> {
        self.adapters
            .iter()
            .find(|adapter| adapter.supports(unit))
            .map(|adapter| adapter.as_ref())
            .ok_or_else(|| {
                RuntimeError::new(format!(
                    "no adapter registered for domain `{}` backend `{:?}` lowering `{:?}`",
                    unit.domain_family, unit.backend_family, unit.selected_lowering_target
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use nuis_artifact::BuildManifestDomainBuildUnit;

    use super::{AdapterRegistry, DomainAdapter};

    struct TestAdapter;

    impl DomainAdapter for TestAdapter {
        fn adapter_id(&self) -> &'static str {
            "test-adapter"
        }

        fn supports(&self, unit: &BuildManifestDomainBuildUnit) -> bool {
            unit.domain_family == "network"
                && unit.backend_family.as_deref() == Some("urlsession")
        }
    }

    fn network_unit() -> BuildManifestDomainBuildUnit {
        BuildManifestDomainBuildUnit {
            package_id: "official.network".to_owned(),
            domain_family: "network".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("urlsession".to_owned()),
            selected_lowering_target: Some("urlsession".to_owned()),
            artifact_stub_path: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: Some("/tmp/network.bridge.stub.txt".to_owned()),
            artifact_payload_blob_path: Some("/tmp/network.payload.bin".to_owned()),
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            contract_family: "nustar.network".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
        }
    }

    #[test]
    fn registry_resolves_matching_adapter() {
        let mut registry = AdapterRegistry::new();
        registry.register(Box::new(TestAdapter));
        let adapter = registry.resolve(&network_unit()).unwrap();
        assert_eq!(adapter.adapter_id(), "test-adapter");
    }
}
