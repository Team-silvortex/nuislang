use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::{ExecutionPhaseAction, ExecutionPhaseContext, ExecutionPhaseOutcome, RuntimeError};

pub trait DomainAdapter: Send + Sync {
    fn adapter_id(&self) -> &'static str;
    fn supports(&self, unit: &BuildManifestDomainBuildUnit) -> bool;

    fn phase_hint(&self, _ctx: &ExecutionPhaseContext<'_>) -> Option<String> {
        None
    }

    fn phase_action(&self, ctx: &ExecutionPhaseContext<'_>) -> Option<ExecutionPhaseAction> {
        self.phase_hint(ctx).map(|hint| ExecutionPhaseAction {
            kind: format!("phase.{}", ctx.phase),
            input_handles: Vec::new(),
            resolved_inputs: Vec::new(),
            output_handles: Vec::new(),
            resource_bindings: Vec::new(),
            resolved_resources: Vec::new(),
            scheduler_keys: vec![ctx.scheduler_binding.to_owned(), ctx.phase.to_owned()],
            adapter_hint: Some(hint),
        })
    }

    fn phase_outcome(
        &self,
        _ctx: &ExecutionPhaseContext<'_>,
        _action: &ExecutionPhaseAction,
    ) -> Option<ExecutionPhaseOutcome> {
        None
    }
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
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{AdapterRegistry, DomainAdapter};

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("nuis_runtime_registry_{label}_{unique}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    struct TestAdapter;

    impl DomainAdapter for TestAdapter {
        fn adapter_id(&self) -> &'static str {
            "test-adapter"
        }

        fn supports(&self, unit: &BuildManifestDomainBuildUnit) -> bool {
            unit.domain_family == "network" && unit.backend_family.as_deref() == Some("urlsession")
        }
    }

    fn network_unit() -> BuildManifestDomainBuildUnit {
        let output_dir = temp_dir("network_unit");
        let bridge_stub_path = output_dir.join("network.bridge.stub.txt");
        let payload_blob_path = output_dir.join("network.payload.bin");

        BuildManifestDomainBuildUnit {
            package_id: "official.network".to_owned(),
            domain_family: "network".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("urlsession".to_owned()),
            vendor: None,
            device_class: None,
            target_device: Some("urlsession-stack".to_owned()),
            ir_format: Some("host-ffi-plan".to_owned()),
            dispatch_abi: Some("nuis-host-call".to_owned()),
            backend_priority: Some(700),
            verification: Some("contract-only".to_owned()),
            selected_lowering_target: Some("urlsession".to_owned()),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: Some(bridge_stub_path.display().to_string()),
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: Some(payload_blob_path.display().to_string()),
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
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
