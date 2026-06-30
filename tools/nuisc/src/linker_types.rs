pub const LINK_PLAN_SCHEMA: &str = "nuis-link-plan-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlan {
    pub schema: String,
    pub input: String,
    pub output_dir: String,
    pub packaging_mode: String,
    pub cpu_target: LinkPlanCpuTarget,
    pub lifecycle: LinkPlanLifecycle,
    pub envelope: LinkPlanEnvelope,
    pub compiled_artifact: LinkPlanArtifact,
    pub bridge_registry_path: Option<String>,
    pub host_bridge_plan_index_path: Option<String>,
    pub lowering_plan_index_path: Option<String>,
    pub domain_units: Vec<LinkPlanDomainUnit>,
    pub artifact_lowering_alignment: ArtifactLoweringAlignmentSummary,
    pub hetero_calculate: LinkPlanHeteroCalculate,
    pub final_stage: LinkPlanFinalStage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanCpuTarget {
    pub abi: String,
    pub machine_arch: String,
    pub machine_os: String,
    pub object_format: String,
    pub calling_abi: String,
    pub clang_target: String,
    pub cross_compile: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanLifecycle {
    pub bootstrap_entry: String,
    pub tick_policy: String,
    pub shutdown_policy: String,
    pub yalivia_rpc: String,
    pub hook_surface: Vec<String>,
    pub export_surface: Vec<String>,
    pub runtime_capability_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanEnvelope {
    pub schema: String,
    pub package_count: usize,
    pub contract_families: Vec<String>,
    pub domain_families: Vec<String>,
    pub function_kind: String,
    pub graph_kind: String,
    pub default_time_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanArtifact {
    pub path: String,
    pub binary_name: String,
    pub binary_path: String,
    pub binary_bytes: usize,
    pub build_manifest_bytes: usize,
    pub container_kind: Option<String>,
    pub container_version: Option<u16>,
    pub section_count: Option<usize>,
    pub section_names: Vec<String>,
    pub section_table_valid: Option<bool>,
    pub lowering_unit_count: Option<usize>,
    pub lowering_domain_families: Vec<String>,
    pub lowering_targets: Vec<String>,
    pub lowering_units: Vec<crate::aot::NuisCompiledArtifactLoweringUnitInspect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactLoweringAlignmentCheck {
    pub index: usize,
    pub package_id: String,
    pub domain_family: String,
    pub consistent: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactLoweringAlignmentSummary {
    pub checked: usize,
    pub mismatches: usize,
    pub consistent: bool,
    pub checks: Vec<ArtifactLoweringAlignmentCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanDomainUnit {
    pub kind: String,
    pub package_id: String,
    pub domain_family: String,
    pub abi: Option<String>,
    pub machine_arch: Option<String>,
    pub machine_os: Option<String>,
    pub backend_family: Option<String>,
    pub vendor: Option<String>,
    pub device_class: Option<String>,
    pub selected_lowering_target: Option<String>,
    pub contract_family: String,
    pub packaging_role: String,
    pub artifact_stub_path: Option<String>,
    pub artifact_stub_inline: Option<String>,
    pub artifact_payload_path: Option<String>,
    pub artifact_bridge_stub_path: Option<String>,
    pub artifact_ir_sidecar_path: Option<String>,
    pub artifact_bridge_stub_inline: Option<String>,
    pub artifact_payload_blob_path: Option<String>,
    pub artifact_payload_blob_bytes: Option<usize>,
    pub artifact_payload_format: Option<String>,
    pub artifact_payload_blob_inline: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanFinalStage {
    pub kind: String,
    pub driver: String,
    pub link_mode: String,
    pub output_path: String,
    pub inputs: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanHeteroCalculate {
    pub schema: String,
    pub mode: String,
    pub static_link: bool,
    pub lifecycle_driven: bool,
    pub time_order_model: String,
    pub data_order_model: String,
    pub c_world_policy: String,
    pub nodes: Vec<LinkPlanHeteroNode>,
    pub data_segments: Vec<LinkPlanDataSegment>,
    pub validation: LinkPlanHeteroValidationSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanHeteroNode {
    pub index: usize,
    pub timestamp: String,
    pub domain_family: String,
    pub package_id: String,
    pub lifecycle_hook: String,
    pub wait_on: Vec<String>,
    pub emits: Vec<String>,
    pub link_input: String,
    pub c_world_wrapper: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanDataSegment {
    pub index: usize,
    pub segment_id: String,
    pub domain_family: String,
    pub owner_package: String,
    pub order_key: String,
    pub access_phase: String,
    pub source_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlanHeteroValidationSummary {
    pub checked: usize,
    pub valid: bool,
    pub issues: Vec<String>,
}
