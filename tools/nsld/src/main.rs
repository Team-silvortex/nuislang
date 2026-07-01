use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const NSLD_LINK_INPUT_TABLE_SCHEMA: &str = "nuis-nsld-link-input-table-v1";
const NSLD_LINK_INPUT_TABLE_SCHEMA_VERSION: usize = 1;
const NSLD_LINK_INPUT_TABLE_KIND: &str = "lowering-sidecar-link-inputs";
const NSLD_LINK_INPUT_TABLE_PRODUCER: &str = "nsld";
const NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE: &str = "alpha-0.6.0";
const NSLD_LINK_UNIT_TABLE_SCHEMA: &str = "nuis-nsld-link-unit-table-v1";
const NSLD_LINK_UNIT_TABLE_SCHEMA_VERSION: usize = 1;
const NSLD_LINK_UNIT_TABLE_KIND: &str = "deterministic-link-units";
const NSLD_LINK_BUNDLE_SCHEMA: &str = "nuis-nsld-link-bundle-v1";
const NSLD_LINK_BUNDLE_SCHEMA_VERSION: usize = 1;
const NSLD_LINK_BUNDLE_KIND: &str = "hetero-static-link-bundle";
const NSLD_ASSEMBLE_PLAN_SCHEMA: &str = "nuis-nsld-assemble-plan-v1";
const NSLD_ASSEMBLE_PLAN_SCHEMA_VERSION: usize = 1;
const NSLD_ASSEMBLE_PLAN_KIND: &str = "deterministic-section-assembly-plan";
const NSLD_SECTION_MANIFEST_SCHEMA: &str = "nuis-nsld-section-manifest-v1";
const NSLD_SECTION_MANIFEST_SCHEMA_VERSION: usize = 1;
const NSLD_SECTION_MANIFEST_KIND: &str = "deterministic-section-manifest";
const NSLD_CONTAINER_PLAN_SCHEMA: &str = "nuis-nsld-container-plan-v1";
const NSLD_CONTAINER_PLAN_SCHEMA_VERSION: usize = 1;
const NSLD_CONTAINER_PLAN_KIND: &str = "deterministic-container-layout-plan";
const NSLD_CONTAINER_SCHEMA: &str = "nuis-nsld-container-v1";
const NSLD_CONTAINER_SCHEMA_VERSION: usize = 1;
const NSLD_CONTAINER_KIND: &str = "deterministic-hetero-container";
const NSLD_CONTAINER_MAGIC: &str = "NUISNSLD";
const NSLD_CONTAINER_VERSION: usize = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Status,
    Plan { input: PathBuf, json: bool },
    Check { input: PathBuf, json: bool },
    Closure { input: PathBuf, json: bool },
    Prepare { input: PathBuf, json: bool },
    AssemblePlan { input: PathBuf, json: bool },
    EmitAssemblePlan { input: PathBuf, json: bool },
    VerifyAssemblePlan { input: PathBuf, json: bool },
    SectionManifest { input: PathBuf, json: bool },
    EmitSectionManifest { input: PathBuf, json: bool },
    VerifySectionManifest { input: PathBuf, json: bool },
    ContainerPlan { input: PathBuf, json: bool },
    EmitContainerPlan { input: PathBuf, json: bool },
    VerifyContainerPlan { input: PathBuf, json: bool },
    Container { input: PathBuf, json: bool },
    EmitContainer { input: PathBuf, json: bool },
    VerifyContainer { input: PathBuf, json: bool },
    Bundle { input: PathBuf, json: bool },
    EmitBundle { input: PathBuf, json: bool },
    VerifyBundle { input: PathBuf, json: bool },
    Units { input: PathBuf, json: bool },
    EmitUnits { input: PathBuf, json: bool },
    VerifyUnits { input: PathBuf, json: bool },
    Inputs { input: PathBuf, json: bool },
    VerifyInputs { input: PathBuf, json: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldCheckReport {
    manifest: String,
    valid: bool,
    checks: usize,
    failures: usize,
    artifact_lowering_alignment_consistent: bool,
    artifact_lowering_alignment_mismatches: usize,
    clock_protocol_valid: bool,
    clock_protocol_issues: Vec<String>,
    hetero_calculate_valid: bool,
    hetero_calculate_issues: Vec<String>,
    static_link: bool,
    lifecycle_driven: bool,
    sidecar_capability_valid: bool,
    sidecar_capability_issues: Vec<String>,
    link_input_table_present: bool,
    link_input_table_valid: Option<bool>,
    link_input_table_issues: Vec<String>,
    link_unit_table_present: bool,
    link_unit_table_valid: Option<bool>,
    link_unit_table_issues: Vec<String>,
    link_bundle_present: bool,
    link_bundle_valid: Option<bool>,
    link_bundle_issues: Vec<String>,
    assemble_plan_present: bool,
    assemble_plan_valid: Option<bool>,
    assemble_plan_issues: Vec<String>,
    section_manifest_present: bool,
    section_manifest_valid: Option<bool>,
    section_manifest_issues: Vec<String>,
    container_plan_present: bool,
    container_plan_valid: Option<bool>,
    container_plan_issues: Vec<String>,
    container_present: bool,
    container_valid: Option<bool>,
    container_issues: Vec<String>,
    final_stage_link_mode: String,
    domains: Vec<NsldDomainDiagnostic>,
    sidecar_capabilities: Vec<NsldSidecarCapabilityDiagnostic>,
    clock_edges: Vec<NsldClockEdgeDiagnostic>,
    data_segments: Vec<NsldDataSegmentDiagnostic>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldDomainDiagnostic {
    domain_family: String,
    package_id: String,
    kind: String,
    packaging_role: String,
    lowering_target: String,
    backend_family: String,
    alignment_consistent: bool,
    alignment_issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldSidecarCapabilityDiagnostic {
    domain_family: String,
    package_id: String,
    path: String,
    content_bytes: usize,
    content_hash: String,
    valid: bool,
    capability_owner: String,
    frontend_ir: String,
    native_ir: String,
    dispatch_lowering: String,
    validation_contracts: Vec<String>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldClockEdgeDiagnostic {
    index: usize,
    from: String,
    to: String,
    relation: String,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldDataSegmentDiagnostic {
    index: usize,
    segment_id: String,
    domain_family: String,
    owner_package: String,
    order_key: String,
    access_phase: String,
    source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldClosureReport {
    manifest: String,
    closed: bool,
    internal_contracts: Vec<String>,
    link_inputs: Vec<NsldLinkInputDiagnostic>,
    link_input_count: usize,
    link_input_total_bytes: usize,
    link_input_table_hash: String,
    link_input_table_present: bool,
    link_input_table_valid: Option<bool>,
    external_dependencies: Vec<String>,
    unresolved: Vec<String>,
    host_wrapper_required: bool,
    domain_count: usize,
    hetero_domain_count: usize,
    sidecar_capability_count: usize,
    clock_edge_count: usize,
    data_segment_count: usize,
    final_stage_link_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkUnitReport {
    manifest: String,
    unit_count: usize,
    hetero_unit_count: usize,
    link_input_count: usize,
    clock_edge_count: usize,
    data_segment_count: usize,
    unit_table_hash: String,
    units: Vec<NsldLinkUnitDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkUnitDiagnostic {
    order_index: usize,
    unit_id: String,
    unit_kind: String,
    domain_family: String,
    package_id: String,
    backend_family: String,
    lowering_target: String,
    packaging_role: String,
    link_input_ids: Vec<String>,
    clock_edge_count: usize,
    data_segment_count: usize,
    requires_host_wrapper: bool,
    deterministic_order_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkUnitsEmitReport {
    manifest: String,
    output_path: String,
    unit_count: usize,
    hetero_unit_count: usize,
    link_input_count: usize,
    unit_table_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkUnitsVerifyReport {
    manifest: String,
    input_path: String,
    valid: bool,
    expected_unit_count: usize,
    expected_hetero_unit_count: usize,
    expected_link_input_count: usize,
    expected_unit_table_hash: String,
    actual_unit_count: Option<usize>,
    actual_hetero_unit_count: Option<usize>,
    actual_link_input_count: Option<usize>,
    actual_unit_table_hash: Option<String>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkBundleReport {
    manifest: String,
    bundle_id: String,
    bundle_hash: String,
    bundle_ready: bool,
    unit_count: usize,
    hetero_unit_count: usize,
    link_input_count: usize,
    link_input_total_bytes: usize,
    link_input_table_hash: String,
    unit_table_hash: String,
    clock_edge_count: usize,
    data_segment_count: usize,
    final_stage_link_mode: String,
    host_wrapper_required: bool,
    compiled_artifact_path: String,
    native_output_path: String,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkBundleEmitReport {
    manifest: String,
    output_path: String,
    bundle_id: String,
    bundle_hash: String,
    bundle_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkBundleVerifyReport {
    manifest: String,
    input_path: String,
    valid: bool,
    expected_bundle_id: String,
    expected_bundle_hash: String,
    actual_bundle_id: Option<String>,
    actual_bundle_hash: Option<String>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldPrepareReport {
    manifest: String,
    valid: bool,
    output_dir: String,
    link_input_table_path: String,
    link_unit_table_path: String,
    link_bundle_path: String,
    assemble_plan_path: String,
    section_manifest_path: String,
    container_plan_path: String,
    container_path: String,
    container_payload_path: String,
    link_input_count: usize,
    link_input_table_hash: String,
    unit_count: usize,
    unit_table_hash: String,
    bundle_id: String,
    bundle_hash: String,
    bundle_ready: bool,
    assemble_plan_hash: String,
    section_table_hash: String,
    container_layout_hash: String,
    container_hash: String,
    payload_size_bytes: usize,
    payload_hash: String,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldAssemblePlanReport {
    manifest: String,
    ready: bool,
    bundle_id: String,
    bundle_hash: String,
    assemble_plan_hash: String,
    section_count: usize,
    sections: Vec<NsldAssembleSectionDiagnostic>,
    blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldAssembleSectionDiagnostic {
    order_index: usize,
    section_id: String,
    section_kind: String,
    source_path: String,
    source_hash: String,
    required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldAssemblePlanEmitReport {
    manifest: String,
    output_path: String,
    ready: bool,
    assemble_plan_hash: String,
    section_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldAssemblePlanVerifyReport {
    manifest: String,
    input_path: String,
    valid: bool,
    expected_assemble_plan_hash: String,
    expected_section_count: usize,
    actual_assemble_plan_hash: Option<String>,
    actual_section_count: Option<usize>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldSectionManifestReport {
    manifest: String,
    ready: bool,
    assemble_plan_hash: String,
    section_count: usize,
    section_table_hash: String,
    sections: Vec<NsldAssembleSectionDiagnostic>,
    blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldSectionManifestEmitReport {
    manifest: String,
    output_path: String,
    ready: bool,
    section_count: usize,
    section_table_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldSectionManifestVerifyReport {
    manifest: String,
    input_path: String,
    valid: bool,
    expected_section_count: usize,
    expected_section_table_hash: String,
    actual_section_count: Option<usize>,
    actual_section_table_hash: Option<String>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldContainerPlanReport {
    manifest: String,
    ready: bool,
    container_magic: String,
    container_version: usize,
    section_count: usize,
    section_table_hash: String,
    container_layout_hash: String,
    output_path: String,
    sections: Vec<NsldAssembleSectionDiagnostic>,
    blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldContainerPlanEmitReport {
    manifest: String,
    output_path: String,
    ready: bool,
    container_layout_hash: String,
    section_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldContainerPlanVerifyReport {
    manifest: String,
    input_path: String,
    valid: bool,
    expected_container_layout_hash: String,
    expected_section_count: usize,
    actual_container_layout_hash: Option<String>,
    actual_section_count: Option<usize>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldContainerReport {
    manifest: String,
    ready: bool,
    container_magic: String,
    container_version: usize,
    container_layout_hash: String,
    container_hash: String,
    payload_size_bytes: usize,
    payload_hash: String,
    output_path: String,
    payload_path: String,
    section_count: usize,
    sections: Vec<NsldContainerSectionEntry>,
    blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldContainerSectionEntry {
    order_index: usize,
    section_id: String,
    section_kind: String,
    source_path: String,
    source_hash: String,
    payload_hash: String,
    required: bool,
    offset: usize,
    size_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldContainerEmitReport {
    manifest: String,
    output_path: String,
    payload_path: String,
    ready: bool,
    container_layout_hash: String,
    container_hash: String,
    payload_size_bytes: usize,
    payload_hash: String,
    section_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldContainerVerifyReport {
    manifest: String,
    input_path: String,
    valid: bool,
    expected_container_layout_hash: String,
    expected_container_hash: String,
    expected_payload_size_bytes: usize,
    expected_payload_hash: String,
    expected_payload_path: String,
    expected_section_count: usize,
    actual_container_layout_hash: Option<String>,
    actual_container_hash: Option<String>,
    actual_payload_size_bytes: Option<usize>,
    actual_payload_hash: Option<String>,
    actual_section_count: Option<usize>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkInputDiagnostic {
    order_index: usize,
    input_id: String,
    input_kind: String,
    domain_family: String,
    package_id: String,
    path: String,
    native_ir: String,
    dispatch_lowering: String,
    contract_count: usize,
    content_bytes: usize,
    content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkInputSummary {
    inputs: Vec<NsldLinkInputDiagnostic>,
    count: usize,
    total_bytes: usize,
    table_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkInputsEmitReport {
    manifest: String,
    output_path: String,
    link_input_count: usize,
    link_input_total_bytes: usize,
    link_input_table_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldLinkInputsVerifyReport {
    manifest: String,
    input_path: String,
    valid: bool,
    expected_link_input_count: usize,
    expected_link_input_total_bytes: usize,
    expected_link_input_table_hash: String,
    actual_link_input_count: Option<usize>,
    actual_link_input_total_bytes: Option<usize>,
    actual_link_input_table_hash: Option<String>,
    issues: Vec<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match parse_args(env::args().skip(1))? {
        Command::Status => {
            println!("Nsld linker front-door");
            println!("  tool: nsld");
            println!("  phase: alpha-0.6.0 linker boundary");
            println!(
                "  current_role: link-plan inspection and hetero clock/link contract surfacing"
            );
            println!("  implementation: reuses nuisc::linker while linker ownership is split out");
            println!("  final_link_status: host-toolchain wrapper is still used for native launcher finalization");
        }
        Command::Plan { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            if json {
                println!("{}", nuisc::linker::render_link_plan_json(&plan));
            } else {
                println!("Nsld link plan");
                println!("  input: {}", input.display());
                println!("  manifest: {}", manifest.display());
                println!("  role: alpha-0.6.0 linker front-door");
                for line in nuisc::linker::render_link_plan_summary(&plan) {
                    println!("  {line}");
                }
            }
        }
        Command::Check { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_check_report(&manifest, &plan);
            if json {
                println!("{}", nsld_check_report_json(&report));
            } else {
                print_nsld_check_report(&report);
            }
            if !report.valid {
                return Err("nsld check failed".to_owned());
            }
        }
        Command::Closure { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_closure_report(&manifest, &plan);
            if json {
                println!("{}", nsld_closure_report_json(&report));
            } else {
                print_nsld_closure_report(&report);
            }
        }
        Command::Prepare { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_prepare_report(&manifest, &plan)?;
            if json {
                println!("{}", nsld_prepare_report_json(&report));
            } else {
                print_nsld_prepare_report(&report);
            }
            if !report.valid {
                return Err("nsld prepare failed".to_owned());
            }
        }
        Command::AssemblePlan { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_assemble_plan_report(&manifest, &plan);
            if json {
                println!("{}", nsld_assemble_plan_report_json(&report));
            } else {
                print_nsld_assemble_plan_report(&report);
            }
        }
        Command::EmitAssemblePlan { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_emit_assemble_plan_report(&manifest, &plan)?;
            if json {
                println!("{}", nsld_assemble_plan_emit_report_json(&report));
            } else {
                print_nsld_assemble_plan_emit_report(&report);
            }
        }
        Command::VerifyAssemblePlan { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_verify_assemble_plan_report(&manifest, &plan);
            if json {
                println!("{}", nsld_assemble_plan_verify_report_json(&report));
            } else {
                print_nsld_assemble_plan_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld assemble plan verification failed".to_owned());
            }
        }
        Command::SectionManifest { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_section_manifest_report(&manifest, &plan);
            if json {
                println!("{}", nsld_section_manifest_report_json(&report));
            } else {
                print_nsld_section_manifest_report(&report);
            }
        }
        Command::EmitSectionManifest { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_emit_section_manifest_report(&manifest, &plan)?;
            if json {
                println!("{}", nsld_section_manifest_emit_report_json(&report));
            } else {
                print_nsld_section_manifest_emit_report(&report);
            }
        }
        Command::VerifySectionManifest { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_verify_section_manifest_report(&manifest, &plan);
            if json {
                println!("{}", nsld_section_manifest_verify_report_json(&report));
            } else {
                print_nsld_section_manifest_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld section manifest verification failed".to_owned());
            }
        }
        Command::ContainerPlan { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_container_plan_report(&manifest, &plan);
            if json {
                println!("{}", nsld_container_plan_report_json(&report));
            } else {
                print_nsld_container_plan_report(&report);
            }
        }
        Command::EmitContainerPlan { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_emit_container_plan_report(&manifest, &plan)?;
            if json {
                println!("{}", nsld_container_plan_emit_report_json(&report));
            } else {
                print_nsld_container_plan_emit_report(&report);
            }
        }
        Command::VerifyContainerPlan { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_verify_container_plan_report(&manifest, &plan);
            if json {
                println!("{}", nsld_container_plan_verify_report_json(&report));
            } else {
                print_nsld_container_plan_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld container plan verification failed".to_owned());
            }
        }
        Command::Container { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_container_report(&manifest, &plan);
            if json {
                println!("{}", nsld_container_report_json(&report));
            } else {
                print_nsld_container_report(&report);
            }
        }
        Command::EmitContainer { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_emit_container_report(&manifest, &plan)?;
            if json {
                println!("{}", nsld_container_emit_report_json(&report));
            } else {
                print_nsld_container_emit_report(&report);
            }
        }
        Command::VerifyContainer { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_verify_container_report(&manifest, &plan);
            if json {
                println!("{}", nsld_container_verify_report_json(&report));
            } else {
                print_nsld_container_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld container verification failed".to_owned());
            }
        }
        Command::Bundle { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_link_bundle_report(&manifest, &plan);
            if json {
                println!("{}", nsld_link_bundle_report_json(&report));
            } else {
                print_nsld_link_bundle_report(&report);
            }
        }
        Command::EmitBundle { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_emit_link_bundle_report(&manifest, &plan)?;
            if json {
                println!("{}", nsld_link_bundle_emit_report_json(&report));
            } else {
                print_nsld_link_bundle_emit_report(&report);
            }
        }
        Command::VerifyBundle { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_verify_link_bundle_report(&manifest, &plan);
            if json {
                println!("{}", nsld_link_bundle_verify_report_json(&report));
            } else {
                print_nsld_link_bundle_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld link bundle verification failed".to_owned());
            }
        }
        Command::Units { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_link_unit_report(&manifest, &plan);
            if json {
                println!("{}", nsld_link_unit_report_json(&report));
            } else {
                print_nsld_link_unit_report(&report);
            }
        }
        Command::EmitUnits { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_emit_link_units_report(&manifest, &plan)?;
            if json {
                println!("{}", nsld_link_units_emit_report_json(&report));
            } else {
                print_nsld_link_units_emit_report(&report);
            }
        }
        Command::VerifyUnits { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_verify_link_units_report(&manifest, &plan);
            if json {
                println!("{}", nsld_link_units_verify_report_json(&report));
            } else {
                print_nsld_link_units_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld link unit verification failed".to_owned());
            }
        }
        Command::Inputs { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_emit_link_inputs_report(&manifest, &plan)?;
            if json {
                println!("{}", nsld_link_inputs_emit_report_json(&report));
            } else {
                print_nsld_link_inputs_emit_report(&report);
            }
        }
        Command::VerifyInputs { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsld_verify_link_inputs_report(&manifest, &plan);
            if json {
                println!("{}", nsld_link_inputs_verify_report_json(&report));
            } else {
                print_nsld_link_inputs_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld link input verification failed".to_owned());
            }
        }
    }
    Ok(())
}

fn parse_args<I>(mut args: I) -> Result<Command, String>
where
    I: Iterator<Item = String>,
{
    let Some(command) = args.next() else {
        return Ok(Command::Status);
    };
    match command.as_str() {
        "status" => Ok(Command::Status),
        "plan"
        | "check"
        | "closure"
        | "prepare"
        | "assemble-plan"
        | "emit-assemble-plan"
        | "verify-assemble-plan"
        | "section-manifest"
        | "emit-section-manifest"
        | "verify-section-manifest"
        | "container-plan"
        | "emit-container-plan"
        | "verify-container-plan"
        | "container"
        | "emit-container"
        | "verify-container"
        | "bundle"
        | "emit-bundle"
        | "verify-bundle"
        | "units"
        | "emit-units"
        | "verify-units"
        | "inputs"
        | "verify-inputs" => {
            let is_check = command == "check";
            let is_closure = command == "closure";
            let is_prepare = command == "prepare";
            let is_assemble_plan = command == "assemble-plan";
            let is_emit_assemble_plan = command == "emit-assemble-plan";
            let is_verify_assemble_plan = command == "verify-assemble-plan";
            let is_section_manifest = command == "section-manifest";
            let is_emit_section_manifest = command == "emit-section-manifest";
            let is_verify_section_manifest = command == "verify-section-manifest";
            let is_container_plan = command == "container-plan";
            let is_emit_container_plan = command == "emit-container-plan";
            let is_verify_container_plan = command == "verify-container-plan";
            let is_container = command == "container";
            let is_emit_container = command == "emit-container";
            let is_verify_container = command == "verify-container";
            let is_bundle = command == "bundle";
            let is_emit_bundle = command == "emit-bundle";
            let is_verify_bundle = command == "verify-bundle";
            let is_units = command == "units";
            let is_emit_units = command == "emit-units";
            let is_verify_units = command == "verify-units";
            let is_inputs = command == "inputs";
            let is_verify_inputs = command == "verify-inputs";
            let mut json = false;
            let mut input = None;
            for arg in args {
                if arg == "--json" {
                    json = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(format!("unexpected argument `{arg}`"));
                }
            }
            let input = input.ok_or_else(|| usage().to_owned())?;
            if is_check {
                Ok(Command::Check { input, json })
            } else if is_closure {
                Ok(Command::Closure { input, json })
            } else if is_prepare {
                Ok(Command::Prepare { input, json })
            } else if is_assemble_plan {
                Ok(Command::AssemblePlan { input, json })
            } else if is_emit_assemble_plan {
                Ok(Command::EmitAssemblePlan { input, json })
            } else if is_verify_assemble_plan {
                Ok(Command::VerifyAssemblePlan { input, json })
            } else if is_section_manifest {
                Ok(Command::SectionManifest { input, json })
            } else if is_emit_section_manifest {
                Ok(Command::EmitSectionManifest { input, json })
            } else if is_verify_section_manifest {
                Ok(Command::VerifySectionManifest { input, json })
            } else if is_container_plan {
                Ok(Command::ContainerPlan { input, json })
            } else if is_emit_container_plan {
                Ok(Command::EmitContainerPlan { input, json })
            } else if is_verify_container_plan {
                Ok(Command::VerifyContainerPlan { input, json })
            } else if is_container {
                Ok(Command::Container { input, json })
            } else if is_emit_container {
                Ok(Command::EmitContainer { input, json })
            } else if is_verify_container {
                Ok(Command::VerifyContainer { input, json })
            } else if is_bundle {
                Ok(Command::Bundle { input, json })
            } else if is_emit_bundle {
                Ok(Command::EmitBundle { input, json })
            } else if is_verify_bundle {
                Ok(Command::VerifyBundle { input, json })
            } else if is_units {
                Ok(Command::Units { input, json })
            } else if is_emit_units {
                Ok(Command::EmitUnits { input, json })
            } else if is_verify_units {
                Ok(Command::VerifyUnits { input, json })
            } else if is_inputs {
                Ok(Command::Inputs { input, json })
            } else if is_verify_inputs {
                Ok(Command::VerifyInputs { input, json })
            } else {
                Ok(Command::Plan { input, json })
            }
        }
        "--help" | "-h" | "help" => Err(usage().to_owned()),
        other => Err(format!("unknown nsld command `{other}`\n{}", usage())),
    }
}

fn resolve_manifest_input(input: &Path) -> Result<PathBuf, String> {
    if input.is_dir() {
        let candidate = input.join("nuis.build.manifest.toml");
        if candidate.exists() {
            return Ok(candidate);
        }
        return Err(format!(
            "directory `{}` does not contain `nuis.build.manifest.toml`",
            input.display()
        ));
    }
    Ok(input.to_path_buf())
}

fn usage() -> &'static str {
    "usage:\n  nsld status\n  nsld plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld check <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld closure <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld prepare <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld assemble-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld emit-assemble-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld verify-assemble-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld section-manifest <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld emit-section-manifest <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld verify-section-manifest <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld container-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld emit-container-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld verify-container-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld container <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld emit-container <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld verify-container <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld bundle <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld emit-bundle <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld verify-bundle <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld units <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld emit-units <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld verify-units <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld inputs <nuis.build.manifest.toml|artifact-output-dir> [--json]\n  nsld verify-inputs <nuis.build.manifest.toml|artifact-output-dir> [--json]"
}

fn nsld_check_report(manifest: &Path, plan: &nuisc::linker::LinkPlan) -> NsldCheckReport {
    let artifact_lowering_alignment_consistent = plan.artifact_lowering_alignment.consistent;
    let artifact_lowering_alignment_mismatches = plan.artifact_lowering_alignment.mismatches;
    let clock_protocol_valid = plan.clock_protocol.validation.valid;
    let clock_protocol_issues = plan.clock_protocol.validation.issues.clone();
    let hetero_calculate_valid = plan.hetero_calculate.validation.valid;
    let hetero_calculate_issues = plan.hetero_calculate.validation.issues.clone();
    let static_link = plan.hetero_calculate.static_link;
    let lifecycle_driven = plan.hetero_calculate.lifecycle_driven;
    let domains = nsld_domain_diagnostics(plan);
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let sidecar_capability_issues = sidecar_capabilities
        .iter()
        .flat_map(|capability| {
            capability
                .issues
                .iter()
                .map(|issue| {
                    format!(
                        "{}:{}: {}",
                        capability.package_id, capability.domain_family, issue
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let sidecar_capability_valid = sidecar_capability_issues.is_empty();
    let link_input_table_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-inputs.toml");
    let link_input_table_present = link_input_table_path.exists();
    let link_input_verify_report =
        link_input_table_present.then(|| nsld_verify_link_inputs_report(manifest, plan));
    let link_input_table_valid = link_input_verify_report.as_ref().map(|report| report.valid);
    let link_input_table_issues = link_input_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let link_unit_table_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-units.toml");
    let link_unit_table_present = link_unit_table_path.exists();
    let link_unit_verify_report =
        link_unit_table_present.then(|| nsld_verify_link_units_report(manifest, plan));
    let link_unit_table_valid = link_unit_verify_report.as_ref().map(|report| report.valid);
    let link_unit_table_issues = link_unit_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let link_bundle_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-bundle.toml");
    let link_bundle_present = link_bundle_path.exists();
    let link_bundle_verify_report =
        link_bundle_present.then(|| nsld_verify_link_bundle_report(manifest, plan));
    let link_bundle_valid = link_bundle_verify_report
        .as_ref()
        .map(|report| report.valid);
    let link_bundle_issues = link_bundle_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let assemble_plan_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.assemble-plan.toml");
    let assemble_plan_present = assemble_plan_path.exists();
    let assemble_plan_verify_report =
        assemble_plan_present.then(|| nsld_verify_assemble_plan_report(manifest, plan));
    let assemble_plan_valid = assemble_plan_verify_report
        .as_ref()
        .map(|report| report.valid);
    let assemble_plan_issues = assemble_plan_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let section_manifest_path =
        PathBuf::from(&plan.output_dir).join("nuis.nsld.section-manifest.toml");
    let section_manifest_present = section_manifest_path.exists();
    let section_manifest_verify_report =
        section_manifest_present.then(|| nsld_verify_section_manifest_report(manifest, plan));
    let section_manifest_valid = section_manifest_verify_report
        .as_ref()
        .map(|report| report.valid);
    let section_manifest_issues = section_manifest_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let container_plan_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.container-plan.toml");
    let container_plan_present = container_plan_path.exists();
    let container_plan_verify_report =
        container_plan_present.then(|| nsld_verify_container_plan_report(manifest, plan));
    let container_plan_valid = container_plan_verify_report
        .as_ref()
        .map(|report| report.valid);
    let container_plan_issues = container_plan_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let container_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.container");
    let container_present = container_path.exists();
    let container_verify_report =
        container_present.then(|| nsld_verify_container_report(manifest, plan));
    let container_valid = container_verify_report.as_ref().map(|report| report.valid);
    let container_issues = container_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let clock_edges = plan
        .clock_protocol
        .edges
        .iter()
        .map(|edge| NsldClockEdgeDiagnostic {
            index: edge.index,
            from: edge.from.clone(),
            to: edge.to.clone(),
            relation: edge.relation.clone(),
            source: edge.source.clone(),
        })
        .collect::<Vec<_>>();
    let data_segments = plan
        .hetero_calculate
        .data_segments
        .iter()
        .map(|segment| NsldDataSegmentDiagnostic {
            index: segment.index,
            segment_id: segment.segment_id.clone(),
            domain_family: segment.domain_family.clone(),
            owner_package: segment.owner_package.clone(),
            order_key: segment.order_key.clone(),
            access_phase: segment.access_phase.clone(),
            source_path: segment
                .source_path
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
        })
        .collect::<Vec<_>>();
    let mut issues = Vec::new();

    if !artifact_lowering_alignment_consistent {
        issues.push(format!(
            "artifact lowering alignment has {} mismatch(es)",
            artifact_lowering_alignment_mismatches
        ));
        for check in &plan.artifact_lowering_alignment.checks {
            for issue in &check.issues {
                issues.push(format!(
                    "{}:{}: {}",
                    check.package_id, check.domain_family, issue
                ));
            }
        }
    }
    if !clock_protocol_valid {
        issues.push("clock protocol validation failed".to_owned());
        issues.extend(clock_protocol_issues.iter().cloned());
    }
    if !hetero_calculate_valid {
        issues.push("hetero calculate validation failed".to_owned());
        issues.extend(hetero_calculate_issues.iter().cloned());
    }
    if !static_link {
        issues.push("hetero calculate plan is not static-link".to_owned());
    }
    if !lifecycle_driven {
        issues.push("hetero calculate plan is not lifecycle-driven".to_owned());
    }
    if !sidecar_capability_valid {
        issues.push("sidecar capability validation failed".to_owned());
        issues.extend(sidecar_capability_issues.iter().cloned());
    }
    if link_input_table_valid == Some(false) {
        issues.push("link input table verification failed".to_owned());
        issues.extend(link_input_table_issues.iter().cloned());
    }
    if link_unit_table_valid == Some(false) {
        issues.push("link unit table verification failed".to_owned());
        issues.extend(link_unit_table_issues.iter().cloned());
    }
    if link_bundle_valid == Some(false) {
        issues.push("link bundle verification failed".to_owned());
        issues.extend(link_bundle_issues.iter().cloned());
    }
    if assemble_plan_valid == Some(false) {
        issues.push("assemble plan verification failed".to_owned());
        issues.extend(assemble_plan_issues.iter().cloned());
    }
    if section_manifest_valid == Some(false) {
        issues.push("section manifest verification failed".to_owned());
        issues.extend(section_manifest_issues.iter().cloned());
    }
    if container_plan_valid == Some(false) {
        issues.push("container plan verification failed".to_owned());
        issues.extend(container_plan_issues.iter().cloned());
    }
    if container_valid == Some(false) {
        issues.push("container verification failed".to_owned());
        issues.extend(container_issues.iter().cloned());
    }

    let checks = 6 + usize::from(link_input_table_present) + usize::from(link_unit_table_present);
    let checks = checks + usize::from(link_bundle_present);
    let checks = checks + usize::from(assemble_plan_present);
    let checks = checks + usize::from(section_manifest_present);
    let checks = checks + usize::from(container_plan_present);
    let checks = checks + usize::from(container_present);
    let failures = issues.len();
    NsldCheckReport {
        manifest: manifest.display().to_string(),
        valid: failures == 0,
        checks,
        failures,
        artifact_lowering_alignment_consistent,
        artifact_lowering_alignment_mismatches,
        clock_protocol_valid,
        clock_protocol_issues,
        hetero_calculate_valid,
        hetero_calculate_issues,
        static_link,
        lifecycle_driven,
        sidecar_capability_valid,
        sidecar_capability_issues,
        link_input_table_present,
        link_input_table_valid,
        link_input_table_issues,
        link_unit_table_present,
        link_unit_table_valid,
        link_unit_table_issues,
        link_bundle_present,
        link_bundle_valid,
        link_bundle_issues,
        assemble_plan_present,
        assemble_plan_valid,
        assemble_plan_issues,
        section_manifest_present,
        section_manifest_valid,
        section_manifest_issues,
        container_plan_present,
        container_plan_valid,
        container_plan_issues,
        container_present,
        container_valid,
        container_issues,
        final_stage_link_mode: plan.final_stage.link_mode.clone(),
        domains,
        sidecar_capabilities,
        clock_edges,
        data_segments,
        issues,
    }
}

fn nsld_closure_report(manifest: &Path, plan: &nuisc::linker::LinkPlan) -> NsldClosureReport {
    let mut internal_contracts = vec![
        "build-manifest".to_owned(),
        "compiled-artifact-envelope".to_owned(),
        "artifact-lowering-alignment".to_owned(),
        "clock-protocol".to_owned(),
        "hetero-calculate-plan".to_owned(),
        "deterministic-data-segment-order".to_owned(),
    ];
    if plan.bridge_registry_path.is_some() {
        internal_contracts.push("bridge-registry".to_owned());
    }
    if plan.host_bridge_plan_index_path.is_some() {
        internal_contracts.push("host-bridge-plan-index".to_owned());
    }
    if plan.lowering_plan_index_path.is_some() {
        internal_contracts.push("lowering-plan-index".to_owned());
    }
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    if !sidecar_capabilities.is_empty()
        && sidecar_capabilities
            .iter()
            .all(|capability| capability.valid)
    {
        internal_contracts.push("lowering-sidecar-capabilities".to_owned());
        internal_contracts.push("link-input-sidecar-table".to_owned());
    }
    let link_input_summary = nsld_link_input_summary(&sidecar_capabilities);
    let link_input_table_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-inputs.toml");
    let link_input_verify_report = link_input_table_path
        .exists()
        .then(|| nsld_verify_link_inputs_report(manifest, plan));
    let link_input_table_present = link_input_verify_report.is_some();
    let link_input_table_valid = link_input_verify_report.as_ref().map(|report| report.valid);
    if link_input_table_valid == Some(true) {
        internal_contracts.push("verified-link-input-table".to_owned());
    }

    let host_wrapper_required = matches!(
        plan.final_stage.link_mode.as_str(),
        "host-toolchain-finalize" | "bundle-packaging"
    );
    let mut external_dependencies = Vec::new();
    if host_wrapper_required {
        external_dependencies.push(format!("final-stage:{}", plan.final_stage.driver));
    }
    if !plan.cpu_target.clang_target.is_empty() {
        external_dependencies.push(format!("clang-target:{}", plan.cpu_target.clang_target));
    }
    if plan.final_stage.link_mode == "bundle-packaging" {
        external_dependencies.push("host-launcher-wrapper".to_owned());
    }

    let mut unresolved = Vec::new();
    if host_wrapper_required {
        unresolved.push("self-owned-final-native-linker".to_owned());
    }
    if plan.compiled_artifact.container_kind.is_none() {
        unresolved.push("nuis-owned-container-kind".to_owned());
    }
    if !plan.artifact_lowering_alignment.consistent {
        unresolved.push("artifact-lowering-alignment-mismatch".to_owned());
    }
    if !plan.clock_protocol.validation.valid {
        unresolved.push("clock-protocol-validation".to_owned());
    }
    if !plan.hetero_calculate.validation.valid {
        unresolved.push("hetero-calculate-validation".to_owned());
    }
    for capability in &sidecar_capabilities {
        for issue in &capability.issues {
            unresolved.push(format!(
                "sidecar-capability:{}:{}:{}",
                capability.package_id, capability.domain_family, issue
            ));
        }
    }
    if let Some(report) = &link_input_verify_report {
        for issue in &report.issues {
            unresolved.push(format!("link-input-table:{issue}"));
        }
    }

    NsldClosureReport {
        manifest: manifest.display().to_string(),
        closed: unresolved.is_empty(),
        internal_contracts,
        link_inputs: link_input_summary.inputs,
        link_input_count: link_input_summary.count,
        link_input_total_bytes: link_input_summary.total_bytes,
        link_input_table_hash: link_input_summary.table_hash,
        link_input_table_present,
        link_input_table_valid,
        external_dependencies,
        unresolved,
        host_wrapper_required,
        domain_count: plan.domain_units.len(),
        hetero_domain_count: plan
            .domain_units
            .iter()
            .filter(|unit| unit.kind == "heterogeneous")
            .count(),
        sidecar_capability_count: sidecar_capabilities.len(),
        clock_edge_count: plan.clock_protocol.edges.len(),
        data_segment_count: plan.hetero_calculate.data_segments.len(),
        final_stage_link_mode: plan.final_stage.link_mode.clone(),
    }
}

fn nsld_emit_link_inputs_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldLinkInputsEmitReport, String> {
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let invalid = sidecar_capabilities
        .iter()
        .filter(|capability| !capability.valid)
        .flat_map(|capability| {
            capability.issues.iter().map(|issue| {
                format!(
                    "{}:{}:{}",
                    capability.package_id, capability.domain_family, issue
                )
            })
        })
        .collect::<Vec<_>>();
    if !invalid.is_empty() {
        return Err(format!(
            "cannot emit nsld link inputs while sidecar capabilities are invalid: {}",
            invalid.join(", ")
        ));
    }
    let link_input_summary = nsld_link_input_summary(&sidecar_capabilities);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-inputs.toml");
    fs::write(
        &output_path,
        render_nsld_link_input_table_toml(
            &link_input_summary.inputs,
            link_input_summary.total_bytes,
            &link_input_summary.table_hash,
        ),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld link input table `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldLinkInputsEmitReport {
        manifest: manifest.display().to_string(),
        output_path: output_path.display().to_string(),
        link_input_count: link_input_summary.count,
        link_input_total_bytes: link_input_summary.total_bytes,
        link_input_table_hash: link_input_summary.table_hash,
    })
}

fn nsld_prepare_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldPrepareReport, String> {
    let input_emit = nsld_emit_link_inputs_report(manifest, plan)?;
    let input_verify = nsld_verify_link_inputs_report(manifest, plan);
    let unit_emit = nsld_emit_link_units_report(manifest, plan)?;
    let unit_verify = nsld_verify_link_units_report(manifest, plan);
    let bundle_emit = nsld_emit_link_bundle_report(manifest, plan)?;
    let bundle_verify = nsld_verify_link_bundle_report(manifest, plan);
    let assemble_emit = nsld_emit_assemble_plan_report(manifest, plan)?;
    let assemble_verify = nsld_verify_assemble_plan_report(manifest, plan);
    let section_emit = nsld_emit_section_manifest_report(manifest, plan)?;
    let section_verify = nsld_verify_section_manifest_report(manifest, plan);
    let container_emit = nsld_emit_container_plan_report(manifest, plan)?;
    let container_verify = nsld_verify_container_plan_report(manifest, plan);
    let container_file_emit = nsld_emit_container_report(manifest, plan)?;
    let container_file_verify = nsld_verify_container_report(manifest, plan);

    let mut issues = Vec::new();
    if !input_verify.valid {
        issues.extend(
            input_verify
                .issues
                .iter()
                .map(|issue| format!("link-inputs:{issue}")),
        );
    }
    if !unit_verify.valid {
        issues.extend(
            unit_verify
                .issues
                .iter()
                .map(|issue| format!("link-units:{issue}")),
        );
    }
    if !bundle_verify.valid {
        issues.extend(
            bundle_verify
                .issues
                .iter()
                .map(|issue| format!("link-bundle:{issue}")),
        );
    }
    if !assemble_verify.valid {
        issues.extend(
            assemble_verify
                .issues
                .iter()
                .map(|issue| format!("assemble-plan:{issue}")),
        );
    }
    if !section_verify.valid {
        issues.extend(
            section_verify
                .issues
                .iter()
                .map(|issue| format!("section-manifest:{issue}")),
        );
    }
    if !container_verify.valid {
        issues.extend(
            container_verify
                .issues
                .iter()
                .map(|issue| format!("container-plan:{issue}")),
        );
    }
    if !container_file_verify.valid {
        issues.extend(
            container_file_verify
                .issues
                .iter()
                .map(|issue| format!("container:{issue}")),
        );
    }

    Ok(NsldPrepareReport {
        manifest: manifest.display().to_string(),
        valid: issues.is_empty(),
        output_dir: plan.output_dir.clone(),
        link_input_table_path: input_emit.output_path,
        link_unit_table_path: unit_emit.output_path,
        link_bundle_path: bundle_emit.output_path,
        assemble_plan_path: assemble_emit.output_path,
        section_manifest_path: section_emit.output_path,
        container_plan_path: container_emit.output_path,
        container_path: container_file_emit.output_path,
        container_payload_path: container_file_emit.payload_path,
        link_input_count: input_emit.link_input_count,
        link_input_table_hash: input_emit.link_input_table_hash,
        unit_count: unit_emit.unit_count,
        unit_table_hash: unit_emit.unit_table_hash,
        bundle_id: bundle_emit.bundle_id,
        bundle_hash: bundle_emit.bundle_hash,
        bundle_ready: bundle_emit.bundle_ready,
        assemble_plan_hash: assemble_emit.assemble_plan_hash,
        section_table_hash: section_emit.section_table_hash,
        container_layout_hash: container_emit.container_layout_hash,
        container_hash: container_file_emit.container_hash,
        payload_size_bytes: container_file_emit.payload_size_bytes,
        payload_hash: container_file_emit.payload_hash,
        issues,
    })
}

fn nsld_verify_link_inputs_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldLinkInputsVerifyReport {
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let link_input_summary = nsld_link_input_summary(&sidecar_capabilities);
    let expected = render_nsld_link_input_table_toml(
        &link_input_summary.inputs,
        link_input_summary.total_bytes,
        &link_input_summary.table_hash,
    );
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-inputs.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_link_input_table `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_link_input_count, actual_link_input_total_bytes, actual_link_input_table_hash) =
        match actual.as_ref() {
            Ok(source) => (
                toml_usize_value(source, "link_input_count"),
                toml_usize_value(source, "link_input_total_bytes"),
                toml_string_value(source, "link_input_table_hash"),
            ),
            Err(error) => {
                issues.push(error.clone());
                (None, None, None)
            }
        };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("link-input-table-content-mismatch".to_owned());
        }
        if actual_link_input_count != Some(link_input_summary.count) {
            issues.push(format!(
                "link_input_count mismatch: expected {}, found {}",
                link_input_summary.count,
                actual_link_input_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_link_input_total_bytes != Some(link_input_summary.total_bytes) {
            issues.push(format!(
                "link_input_total_bytes mismatch: expected {}, found {}",
                link_input_summary.total_bytes,
                actual_link_input_total_bytes
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_link_input_table_hash.as_deref() != Some(link_input_summary.table_hash.as_str()) {
            issues.push(format!(
                "link_input_table_hash mismatch: expected {}, found {}",
                link_input_summary.table_hash,
                actual_link_input_table_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldLinkInputsVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_link_input_count: link_input_summary.count,
        expected_link_input_total_bytes: link_input_summary.total_bytes,
        expected_link_input_table_hash: link_input_summary.table_hash,
        actual_link_input_count,
        actual_link_input_total_bytes,
        actual_link_input_table_hash,
        issues,
    }
}

fn nsld_emit_link_units_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldLinkUnitsEmitReport, String> {
    let report = nsld_link_unit_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-units.toml");
    fs::write(&output_path, render_nsld_link_unit_table_toml(&report)).map_err(|error| {
        format!(
            "failed to write nsld link unit table `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldLinkUnitsEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        unit_count: report.unit_count,
        hetero_unit_count: report.hetero_unit_count,
        link_input_count: report.link_input_count,
        unit_table_hash: report.unit_table_hash,
    })
}

fn nsld_verify_link_units_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldLinkUnitsVerifyReport {
    let expected_report = nsld_link_unit_report(manifest, plan);
    let expected = render_nsld_link_unit_table_toml(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-units.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_link_unit_table `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_unit_count,
        actual_hetero_unit_count,
        actual_link_input_count,
        actual_unit_table_hash,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml_usize_value(source, "unit_count"),
            toml_usize_value(source, "hetero_unit_count"),
            toml_usize_value(source, "link_input_count"),
            toml_string_value(source, "unit_table_hash"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("link-unit-table-content-mismatch".to_owned());
        }
        if actual_unit_count != Some(expected_report.unit_count) {
            issues.push(format!(
                "unit_count mismatch: expected {}, found {}",
                expected_report.unit_count,
                actual_unit_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_hetero_unit_count != Some(expected_report.hetero_unit_count) {
            issues.push(format!(
                "hetero_unit_count mismatch: expected {}, found {}",
                expected_report.hetero_unit_count,
                actual_hetero_unit_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_link_input_count != Some(expected_report.link_input_count) {
            issues.push(format!(
                "link_input_count mismatch: expected {}, found {}",
                expected_report.link_input_count,
                actual_link_input_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_unit_table_hash.as_deref() != Some(expected_report.unit_table_hash.as_str()) {
            issues.push(format!(
                "unit_table_hash mismatch: expected {}, found {}",
                expected_report.unit_table_hash,
                actual_unit_table_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldLinkUnitsVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_unit_count: expected_report.unit_count,
        expected_hetero_unit_count: expected_report.hetero_unit_count,
        expected_link_input_count: expected_report.link_input_count,
        expected_unit_table_hash: expected_report.unit_table_hash,
        actual_unit_count,
        actual_hetero_unit_count,
        actual_link_input_count,
        actual_unit_table_hash,
        issues,
    }
}

fn nsld_link_bundle_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldLinkBundleReport {
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let link_input_summary = nsld_link_input_summary(&sidecar_capabilities);
    let unit_report = nsld_link_unit_report(manifest, plan);
    let host_wrapper_required = matches!(
        plan.final_stage.link_mode.as_str(),
        "host-toolchain-finalize" | "bundle-packaging"
    );
    let mut issues = Vec::new();
    if !plan.artifact_lowering_alignment.consistent {
        issues.push("artifact-lowering-alignment-mismatch".to_owned());
    }
    if !plan.clock_protocol.validation.valid {
        issues.push("clock-protocol-invalid".to_owned());
    }
    if !plan.hetero_calculate.validation.valid {
        issues.push("hetero-calculate-invalid".to_owned());
    }
    if !plan.hetero_calculate.static_link {
        issues.push("hetero-calculate-not-static-link".to_owned());
    }
    if !plan.hetero_calculate.lifecycle_driven {
        issues.push("hetero-calculate-not-lifecycle-driven".to_owned());
    }
    for capability in &sidecar_capabilities {
        for issue in &capability.issues {
            issues.push(format!(
                "sidecar-capability:{}:{}:{}",
                capability.package_id, capability.domain_family, issue
            ));
        }
    }

    let bundle_ready = issues.is_empty();
    let bundle_hash = nsld_link_bundle_hash(
        &unit_report,
        &link_input_summary,
        plan,
        host_wrapper_required,
        bundle_ready,
    );
    let bundle_id = format!("lb.{}", bundle_hash.trim_start_matches("0x"));

    NsldLinkBundleReport {
        manifest: manifest.display().to_string(),
        bundle_id,
        bundle_hash,
        bundle_ready,
        unit_count: unit_report.unit_count,
        hetero_unit_count: unit_report.hetero_unit_count,
        link_input_count: link_input_summary.count,
        link_input_total_bytes: link_input_summary.total_bytes,
        link_input_table_hash: link_input_summary.table_hash,
        unit_table_hash: unit_report.unit_table_hash,
        clock_edge_count: plan.clock_protocol.edges.len(),
        data_segment_count: plan.hetero_calculate.data_segments.len(),
        final_stage_link_mode: plan.final_stage.link_mode.clone(),
        host_wrapper_required,
        compiled_artifact_path: plan.compiled_artifact.path.clone(),
        native_output_path: plan.final_stage.output_path.clone(),
        issues,
    }
}

fn nsld_assemble_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldAssemblePlanReport {
    let bundle = nsld_link_bundle_report(manifest, plan);
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let link_input_summary = nsld_link_input_summary(&sidecar_capabilities);
    let mut blockers = bundle.issues.clone();
    let mut sections = Vec::new();

    push_assemble_section(
        &mut sections,
        "compiled-artifact",
        &plan.compiled_artifact.path,
        true,
    );
    push_assemble_section(
        &mut sections,
        "nsld-link-input-table",
        &PathBuf::from(&plan.output_dir)
            .join("nuis.nsld.link-inputs.toml")
            .display()
            .to_string(),
        true,
    );
    push_assemble_section(
        &mut sections,
        "nsld-link-unit-table",
        &PathBuf::from(&plan.output_dir)
            .join("nuis.nsld.link-units.toml")
            .display()
            .to_string(),
        true,
    );
    push_assemble_section(
        &mut sections,
        "nsld-link-bundle",
        &PathBuf::from(&plan.output_dir)
            .join("nuis.nsld.link-bundle.toml")
            .display()
            .to_string(),
        true,
    );
    for input in &link_input_summary.inputs {
        push_assemble_section(&mut sections, "lowering-sidecar-input", &input.path, true);
    }
    for segment in &plan.hetero_calculate.data_segments {
        if let Some(source_path) = &segment.source_path {
            push_assemble_section(&mut sections, "hetero-data-segment", source_path, true);
        } else {
            blockers.push(format!(
                "data-segment:{}:{}:missing-source-path",
                segment.owner_package, segment.segment_id
            ));
        }
    }

    for section in &sections {
        if section.required && section.source_hash == "missing" {
            blockers.push(format!(
                "section:{}:{}:missing-source",
                section.section_kind, section.source_path
            ));
        }
    }

    let assemble_plan_hash =
        nsld_assemble_plan_hash(&bundle.bundle_id, &bundle.bundle_hash, &sections, &blockers);

    NsldAssemblePlanReport {
        manifest: manifest.display().to_string(),
        ready: bundle.bundle_ready && blockers.is_empty(),
        bundle_id: bundle.bundle_id,
        bundle_hash: bundle.bundle_hash,
        assemble_plan_hash,
        section_count: sections.len(),
        sections,
        blockers,
    }
}

fn nsld_emit_assemble_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldAssemblePlanEmitReport, String> {
    let report = nsld_assemble_plan_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.assemble-plan.toml");
    fs::write(&output_path, render_nsld_assemble_plan_toml(&report)).map_err(|error| {
        format!(
            "failed to write nsld assemble plan `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldAssemblePlanEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        ready: report.ready,
        assemble_plan_hash: report.assemble_plan_hash,
        section_count: report.section_count,
    })
}

fn nsld_verify_assemble_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldAssemblePlanVerifyReport {
    let expected_report = nsld_assemble_plan_report(manifest, plan);
    let expected = render_nsld_assemble_plan_toml(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.assemble-plan.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_assemble_plan `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_assemble_plan_hash, actual_section_count) = match actual.as_ref() {
        Ok(source) => (
            toml_string_value(source, "assemble_plan_hash"),
            toml_usize_value(source, "section_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("assemble-plan-content-mismatch".to_owned());
        }
        if actual_assemble_plan_hash.as_deref() != Some(expected_report.assemble_plan_hash.as_str())
        {
            issues.push(format!(
                "assemble_plan_hash mismatch: expected {}, found {}",
                expected_report.assemble_plan_hash,
                actual_assemble_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_section_count != Some(expected_report.section_count) {
            issues.push(format!(
                "section_count mismatch: expected {}, found {}",
                expected_report.section_count,
                actual_section_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldAssemblePlanVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_assemble_plan_hash: expected_report.assemble_plan_hash,
        expected_section_count: expected_report.section_count,
        actual_assemble_plan_hash,
        actual_section_count,
        issues,
    }
}

fn nsld_section_manifest_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldSectionManifestReport {
    let assemble_plan = nsld_assemble_plan_report(manifest, plan);
    let section_table_hash = nsld_section_table_hash(&assemble_plan.sections);
    NsldSectionManifestReport {
        manifest: manifest.display().to_string(),
        ready: assemble_plan.ready,
        assemble_plan_hash: assemble_plan.assemble_plan_hash,
        section_count: assemble_plan.section_count,
        section_table_hash,
        sections: assemble_plan.sections,
        blockers: assemble_plan.blockers,
    }
}

fn nsld_emit_section_manifest_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldSectionManifestEmitReport, String> {
    let report = nsld_section_manifest_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.section-manifest.toml");
    fs::write(&output_path, render_nsld_section_manifest_toml(&report)).map_err(|error| {
        format!(
            "failed to write nsld section manifest `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldSectionManifestEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        ready: report.ready,
        section_count: report.section_count,
        section_table_hash: report.section_table_hash,
    })
}

fn nsld_verify_section_manifest_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldSectionManifestVerifyReport {
    let expected_report = nsld_section_manifest_report(manifest, plan);
    let expected = render_nsld_section_manifest_toml(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.section-manifest.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_section_manifest `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_section_count, actual_section_table_hash) = match actual.as_ref() {
        Ok(source) => (
            toml_usize_value(source, "section_count"),
            toml_string_value(source, "section_table_hash"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("section-manifest-content-mismatch".to_owned());
        }
        if actual_section_count != Some(expected_report.section_count) {
            issues.push(format!(
                "section_count mismatch: expected {}, found {}",
                expected_report.section_count,
                actual_section_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_section_table_hash.as_deref() != Some(expected_report.section_table_hash.as_str())
        {
            issues.push(format!(
                "section_table_hash mismatch: expected {}, found {}",
                expected_report.section_table_hash,
                actual_section_table_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldSectionManifestVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_section_count: expected_report.section_count,
        expected_section_table_hash: expected_report.section_table_hash,
        actual_section_count,
        actual_section_table_hash,
        issues,
    }
}

fn nsld_section_table_hash(sections: &[NsldAssembleSectionDiagnostic]) -> String {
    let mut material = String::new();
    for section in sections {
        material.push_str(&section.order_index.to_string());
        material.push('\t');
        material.push_str(&section.section_id);
        material.push('\t');
        material.push_str(&section.section_kind);
        material.push('\t');
        material.push_str(&section.source_hash);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

fn nsld_container_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldContainerPlanReport {
    let section_manifest = nsld_section_manifest_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir)
        .join("nuis.nsld.container")
        .display()
        .to_string();
    let container_layout_hash = nsld_container_layout_hash(
        NSLD_CONTAINER_MAGIC,
        NSLD_CONTAINER_VERSION,
        section_manifest.section_count,
        &section_manifest.section_table_hash,
        &output_path,
    );
    NsldContainerPlanReport {
        manifest: manifest.display().to_string(),
        ready: section_manifest.ready,
        container_magic: NSLD_CONTAINER_MAGIC.to_owned(),
        container_version: NSLD_CONTAINER_VERSION,
        section_count: section_manifest.section_count,
        section_table_hash: section_manifest.section_table_hash,
        container_layout_hash,
        output_path,
        sections: section_manifest.sections,
        blockers: section_manifest.blockers,
    }
}

fn nsld_emit_container_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldContainerPlanEmitReport, String> {
    let report = nsld_container_plan_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.container-plan.toml");
    fs::write(&output_path, render_nsld_container_plan_toml(&report)).map_err(|error| {
        format!(
            "failed to write nsld container plan `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldContainerPlanEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        ready: report.ready,
        container_layout_hash: report.container_layout_hash,
        section_count: report.section_count,
    })
}

fn nsld_verify_container_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldContainerPlanVerifyReport {
    let expected_report = nsld_container_plan_report(manifest, plan);
    let expected = render_nsld_container_plan_toml(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.container-plan.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_container_plan `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_container_layout_hash, actual_section_count) = match actual.as_ref() {
        Ok(source) => (
            toml_string_value(source, "container_layout_hash"),
            toml_usize_value(source, "section_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("container-plan-content-mismatch".to_owned());
        }
        if actual_container_layout_hash.as_deref()
            != Some(expected_report.container_layout_hash.as_str())
        {
            issues.push(format!(
                "container_layout_hash mismatch: expected {}, found {}",
                expected_report.container_layout_hash,
                actual_container_layout_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_section_count != Some(expected_report.section_count) {
            issues.push(format!(
                "section_count mismatch: expected {}, found {}",
                expected_report.section_count,
                actual_section_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldContainerPlanVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_container_layout_hash: expected_report.container_layout_hash,
        expected_section_count: expected_report.section_count,
        actual_container_layout_hash,
        actual_section_count,
        issues,
    }
}

fn nsld_container_layout_hash(
    container_magic: &str,
    container_version: usize,
    section_count: usize,
    section_table_hash: &str,
    output_path: &str,
) -> String {
    let material = format!(
        "{container_magic}\t{container_version}\t{section_count}\t{section_table_hash}\t{output_path}\n"
    );
    fnv1a64_hex(material.as_bytes())
}

fn nsld_container_report(manifest: &Path, plan: &nuisc::linker::LinkPlan) -> NsldContainerReport {
    let container_plan = nsld_container_plan_report(manifest, plan);
    let sections = nsld_container_section_entries(&container_plan.sections);
    let payload_size_bytes = nsld_container_payload_size(&sections);
    let payload_hash = nsld_container_payload_hash(&sections);
    let container_hash = nsld_container_file_hash(
        &container_plan,
        &sections,
        payload_size_bytes,
        &payload_hash,
    );
    NsldContainerReport {
        manifest: manifest.display().to_string(),
        ready: container_plan.ready,
        container_magic: container_plan.container_magic,
        container_version: container_plan.container_version,
        container_layout_hash: container_plan.container_layout_hash,
        container_hash,
        payload_size_bytes,
        payload_hash,
        payload_path: format!("{}.payload", container_plan.output_path),
        output_path: container_plan.output_path,
        section_count: container_plan.section_count,
        sections,
        blockers: container_plan.blockers,
    }
}

fn nsld_emit_container_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldContainerEmitReport, String> {
    let report = nsld_container_report(manifest, plan);
    let output_path = PathBuf::from(&report.output_path);
    let payload_path = PathBuf::from(&report.payload_path);
    fs::write(
        &payload_path,
        nsld_container_payload_bytes(&report.sections),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld container payload `{}`: {error}",
            payload_path.display()
        )
    })?;
    fs::write(&output_path, render_nsld_container_toml(&report)).map_err(|error| {
        format!(
            "failed to write nsld container `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldContainerEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        payload_path: payload_path.display().to_string(),
        ready: report.ready,
        container_layout_hash: report.container_layout_hash,
        container_hash: report.container_hash,
        payload_size_bytes: report.payload_size_bytes,
        payload_hash: report.payload_hash,
        section_count: report.section_count,
    })
}

fn nsld_verify_container_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldContainerVerifyReport {
    let expected_report = nsld_container_report(manifest, plan);
    let expected = render_nsld_container_toml(&expected_report);
    let input_path = PathBuf::from(&expected_report.output_path);
    let payload_path = PathBuf::from(&expected_report.payload_path);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_container `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_container_layout_hash,
        actual_container_hash,
        actual_payload_size_bytes,
        actual_payload_hash,
        actual_section_count,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml_string_value(source, "container_layout_hash"),
            toml_string_value(source, "container_hash"),
            toml_usize_value(source, "payload_size_bytes"),
            toml_string_value(source, "payload_hash"),
            toml_usize_value(source, "section_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("container-content-mismatch".to_owned());
        }
        if actual_container_layout_hash.as_deref()
            != Some(expected_report.container_layout_hash.as_str())
        {
            issues.push(format!(
                "container_layout_hash mismatch: expected {}, found {}",
                expected_report.container_layout_hash,
                actual_container_layout_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_container_hash.as_deref() != Some(expected_report.container_hash.as_str()) {
            issues.push(format!(
                "container_hash mismatch: expected {}, found {}",
                expected_report.container_hash,
                actual_container_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_payload_size_bytes != Some(expected_report.payload_size_bytes) {
            issues.push(format!(
                "payload_size_bytes mismatch: expected {}, found {}",
                expected_report.payload_size_bytes,
                actual_payload_size_bytes
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_payload_hash.as_deref() != Some(expected_report.payload_hash.as_str()) {
            issues.push(format!(
                "payload_hash mismatch: expected {}, found {}",
                expected_report.payload_hash,
                actual_payload_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_section_count != Some(expected_report.section_count) {
            issues.push(format!(
                "section_count mismatch: expected {}, found {}",
                expected_report.section_count,
                actual_section_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }
    let (actual_payload_file_size, actual_payload_file_hash) = match fs::read(&payload_path)
        .map_err(|error| {
            format!(
                "missing_or_unreadable_container_payload `{}`: {error}",
                payload_path.display()
            )
        }) {
        Ok(bytes) => (Some(bytes.len()), Some(fnv1a64_hex(&bytes))),
        Err(error) => {
            issues.push(error);
            (None, None)
        }
    };
    if actual_payload_file_size != Some(expected_report.payload_size_bytes) {
        issues.push(format!(
            "payload_file_size mismatch: expected {}, found {}",
            expected_report.payload_size_bytes,
            actual_payload_file_size
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
    if actual_payload_file_hash.as_deref() != Some(expected_report.payload_hash.as_str()) {
        issues.push(format!(
            "payload_file_hash mismatch: expected {}, found {}",
            expected_report.payload_hash,
            actual_payload_file_hash
                .clone()
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }

    NsldContainerVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_container_layout_hash: expected_report.container_layout_hash,
        expected_container_hash: expected_report.container_hash,
        expected_payload_size_bytes: expected_report.payload_size_bytes,
        expected_payload_hash: expected_report.payload_hash,
        expected_payload_path: expected_report.payload_path,
        expected_section_count: expected_report.section_count,
        actual_container_layout_hash,
        actual_container_hash,
        actual_payload_size_bytes,
        actual_payload_hash,
        actual_section_count,
        issues,
    }
}

fn nsld_container_section_entries(
    sections: &[NsldAssembleSectionDiagnostic],
) -> Vec<NsldContainerSectionEntry> {
    let mut offset = 0usize;
    sections
        .iter()
        .map(|section| {
            let size_bytes = fs::metadata(&section.source_path)
                .map(|metadata| metadata.len() as usize)
                .unwrap_or(0);
            let payload_hash = fs::read(&section.source_path)
                .map(|bytes| fnv1a64_hex(&bytes))
                .unwrap_or_else(|_| "missing".to_owned());
            let entry = NsldContainerSectionEntry {
                order_index: section.order_index,
                section_id: section.section_id.clone(),
                section_kind: section.section_kind.clone(),
                source_path: section.source_path.clone(),
                source_hash: section.source_hash.clone(),
                payload_hash,
                required: section.required,
                offset,
                size_bytes,
            };
            offset = offset.saturating_add(size_bytes);
            entry
        })
        .collect()
}

fn nsld_container_payload_size(sections: &[NsldContainerSectionEntry]) -> usize {
    sections
        .iter()
        .map(|section| section.size_bytes)
        .fold(0usize, usize::saturating_add)
}

fn nsld_container_payload_hash(sections: &[NsldContainerSectionEntry]) -> String {
    fnv1a64_hex(&nsld_container_payload_bytes(sections))
}

fn nsld_container_payload_bytes(sections: &[NsldContainerSectionEntry]) -> Vec<u8> {
    let mut payload = Vec::new();
    for section in sections {
        if let Ok(bytes) = fs::read(&section.source_path) {
            payload.extend_from_slice(&bytes);
        }
    }
    payload
}

fn nsld_container_file_hash(
    container_plan: &NsldContainerPlanReport,
    sections: &[NsldContainerSectionEntry],
    payload_size_bytes: usize,
    payload_hash: &str,
) -> String {
    let mut material = String::new();
    material.push_str(&container_plan.container_magic);
    material.push('\t');
    material.push_str(&container_plan.container_version.to_string());
    material.push('\t');
    material.push_str(&container_plan.container_layout_hash);
    material.push('\t');
    material.push_str(&payload_size_bytes.to_string());
    material.push('\t');
    material.push_str(payload_hash);
    material.push('\n');
    for section in sections {
        material.push_str(&section.order_index.to_string());
        material.push('\t');
        material.push_str(&section.section_id);
        material.push('\t');
        material.push_str(&section.section_kind);
        material.push('\t');
        material.push_str(&section.source_hash);
        material.push('\t');
        material.push_str(&section.payload_hash);
        material.push('\t');
        material.push_str(&section.source_path);
        material.push('\t');
        material.push_str(&section.offset.to_string());
        material.push('\t');
        material.push_str(&section.size_bytes.to_string());
        material.push('\n');
    }
    for blocker in &container_plan.blockers {
        material.push_str("blocker\t");
        material.push_str(blocker);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

fn nsld_assemble_plan_hash(
    bundle_id: &str,
    bundle_hash: &str,
    sections: &[NsldAssembleSectionDiagnostic],
    blockers: &[String],
) -> String {
    let mut material = String::new();
    material.push_str(bundle_id);
    material.push('\t');
    material.push_str(bundle_hash);
    material.push('\n');
    for section in sections {
        material.push_str(&section.order_index.to_string());
        material.push('\t');
        material.push_str(&section.section_id);
        material.push('\t');
        material.push_str(&section.section_kind);
        material.push('\t');
        material.push_str(&section.source_path);
        material.push('\t');
        material.push_str(&section.source_hash);
        material.push('\t');
        material.push_str(if section.required {
            "required"
        } else {
            "optional"
        });
        material.push('\n');
    }
    for blocker in blockers {
        material.push_str("blocker\t");
        material.push_str(blocker);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

fn push_assemble_section(
    sections: &mut Vec<NsldAssembleSectionDiagnostic>,
    section_kind: &str,
    source_path: &str,
    required: bool,
) {
    let order_index = sections.len();
    let source_hash = fs::read(source_path)
        .map(|bytes| fnv1a64_hex(&bytes))
        .unwrap_or_else(|_| "missing".to_owned());
    sections.push(NsldAssembleSectionDiagnostic {
        order_index,
        section_id: format!("sec{order_index:04}.{section_kind}"),
        section_kind: section_kind.to_owned(),
        source_path: source_path.to_owned(),
        source_hash,
        required,
    });
}

fn nsld_emit_link_bundle_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldLinkBundleEmitReport, String> {
    let report = nsld_link_bundle_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-bundle.toml");
    fs::write(&output_path, render_nsld_link_bundle_toml(&report)).map_err(|error| {
        format!(
            "failed to write nsld link bundle `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldLinkBundleEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        bundle_id: report.bundle_id,
        bundle_hash: report.bundle_hash,
        bundle_ready: report.bundle_ready,
    })
}

fn nsld_verify_link_bundle_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldLinkBundleVerifyReport {
    let expected_report = nsld_link_bundle_report(manifest, plan);
    let expected = render_nsld_link_bundle_toml(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-bundle.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_link_bundle `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_bundle_id, actual_bundle_hash) = match actual.as_ref() {
        Ok(source) => (
            toml_string_value(source, "bundle_id"),
            toml_string_value(source, "bundle_hash"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("link-bundle-content-mismatch".to_owned());
        }
        if actual_bundle_id.as_deref() != Some(expected_report.bundle_id.as_str()) {
            issues.push(format!(
                "bundle_id mismatch: expected {}, found {}",
                expected_report.bundle_id,
                actual_bundle_id
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_bundle_hash.as_deref() != Some(expected_report.bundle_hash.as_str()) {
            issues.push(format!(
                "bundle_hash mismatch: expected {}, found {}",
                expected_report.bundle_hash,
                actual_bundle_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldLinkBundleVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_bundle_id: expected_report.bundle_id,
        expected_bundle_hash: expected_report.bundle_hash,
        actual_bundle_id,
        actual_bundle_hash,
        issues,
    }
}

fn nsld_link_bundle_hash(
    unit_report: &NsldLinkUnitReport,
    link_input_summary: &NsldLinkInputSummary,
    plan: &nuisc::linker::LinkPlan,
    host_wrapper_required: bool,
    bundle_ready: bool,
) -> String {
    let material = format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        unit_report.unit_count,
        unit_report.hetero_unit_count,
        link_input_summary.count,
        link_input_summary.total_bytes,
        link_input_summary.table_hash,
        unit_report.unit_table_hash,
        plan.clock_protocol.edges.len(),
        plan.hetero_calculate.data_segments.len(),
        plan.final_stage.link_mode,
        host_wrapper_required,
        bundle_ready
    );
    fnv1a64_hex(material.as_bytes())
}

fn nsld_link_unit_report(manifest: &Path, plan: &nuisc::linker::LinkPlan) -> NsldLinkUnitReport {
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let link_input_summary = nsld_link_input_summary(&sidecar_capabilities);
    let host_wrapper_required = matches!(
        plan.final_stage.link_mode.as_str(),
        "host-toolchain-finalize" | "bundle-packaging"
    );
    let mut units = plan.domain_units.iter().collect::<Vec<_>>();
    units.sort_by(|left, right| {
        left.domain_family
            .cmp(&right.domain_family)
            .then_with(|| left.package_id.cmp(&right.package_id))
            .then_with(|| left.packaging_role.cmp(&right.packaging_role))
    });
    let units = units
        .into_iter()
        .enumerate()
        .map(|(index, unit)| {
            let link_input_ids = link_input_summary
                .inputs
                .iter()
                .filter(|input| {
                    input.domain_family == unit.domain_family && input.package_id == unit.package_id
                })
                .map(|input| input.input_id.clone())
                .collect::<Vec<_>>();
            let clock_edge_count = plan
                .clock_protocol
                .edges
                .iter()
                .filter(|edge| {
                    edge.from.contains(&unit.domain_family) || edge.to.contains(&unit.domain_family)
                })
                .count();
            let data_segment_count = plan
                .hetero_calculate
                .data_segments
                .iter()
                .filter(|segment| {
                    segment.domain_family == unit.domain_family
                        && segment.owner_package == unit.package_id
                })
                .count();
            let unit_kind = if unit.kind == "heterogeneous" {
                "hetero-domain"
            } else {
                "native-domain"
            }
            .to_owned();
            let deterministic_order_key =
                format!("{index:04}.{}.{}", unit.domain_family, unit.package_id);

            NsldLinkUnitDiagnostic {
                order_index: index,
                unit_id: format!("lu{index:04}.{}.{}", unit.domain_family, unit.package_id),
                unit_kind,
                domain_family: unit.domain_family.clone(),
                package_id: unit.package_id.clone(),
                backend_family: unit
                    .backend_family
                    .clone()
                    .unwrap_or_else(|| "none".to_owned()),
                lowering_target: unit
                    .selected_lowering_target
                    .clone()
                    .unwrap_or_else(|| "none".to_owned()),
                packaging_role: unit.packaging_role.clone(),
                link_input_ids,
                clock_edge_count,
                data_segment_count,
                requires_host_wrapper: host_wrapper_required
                    && (unit.domain_family == "cpu" || unit.packaging_role.contains("launcher")),
                deterministic_order_key,
            }
        })
        .collect::<Vec<_>>();
    let unit_table_hash = nsld_link_unit_table_hash(&units);

    NsldLinkUnitReport {
        manifest: manifest.display().to_string(),
        unit_count: units.len(),
        hetero_unit_count: units
            .iter()
            .filter(|unit| unit.unit_kind == "hetero-domain")
            .count(),
        link_input_count: link_input_summary.count,
        clock_edge_count: plan.clock_protocol.edges.len(),
        data_segment_count: plan.hetero_calculate.data_segments.len(),
        unit_table_hash,
        units,
    }
}

fn nsld_link_unit_table_hash(units: &[NsldLinkUnitDiagnostic]) -> String {
    let mut material = String::new();
    for unit in units {
        material.push_str(&unit.order_index.to_string());
        material.push('\t');
        material.push_str(&unit.unit_id);
        material.push('\t');
        material.push_str(&unit.unit_kind);
        material.push('\t');
        material.push_str(&unit.domain_family);
        material.push('\t');
        material.push_str(&unit.package_id);
        material.push('\t');
        material.push_str(&unit.backend_family);
        material.push('\t');
        material.push_str(&unit.lowering_target);
        material.push('\t');
        material.push_str(&unit.packaging_role);
        material.push('\t');
        material.push_str(&unit.link_input_ids.join("|"));
        material.push('\t');
        material.push_str(&unit.clock_edge_count.to_string());
        material.push('\t');
        material.push_str(&unit.data_segment_count.to_string());
        material.push('\t');
        material.push_str(if unit.requires_host_wrapper {
            "host-wrapper"
        } else {
            "self-contained"
        });
        material.push('\t');
        material.push_str(&unit.deterministic_order_key);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

fn print_nsld_closure_report(report: &NsldClosureReport) {
    println!("Nsld linker closure");
    println!("  manifest: {}", report.manifest);
    println!("  closed: {}", report.closed);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!("  domain_count: {}", report.domain_count);
    println!("  hetero_domain_count: {}", report.hetero_domain_count);
    println!(
        "  sidecar_capability_count: {}",
        report.sidecar_capability_count
    );
    println!("  clock_edge_count: {}", report.clock_edge_count);
    println!("  data_segment_count: {}", report.data_segment_count);
    println!("  internal_contracts: {}", report.internal_contracts.len());
    for contract in &report.internal_contracts {
        println!("  internal_contract: {contract}");
    }
    println!("  link_inputs: {}", report.link_inputs.len());
    println!("  link_input_count: {}", report.link_input_count);
    println!(
        "  link_input_total_bytes: {}",
        report.link_input_total_bytes
    );
    println!("  link_input_table_hash: {}", report.link_input_table_hash);
    println!(
        "  link_input_table: present={} valid={}",
        report.link_input_table_present,
        optional_bool_text(report.link_input_table_valid)
    );
    for input in &report.link_inputs {
        println!(
            "  link_input: order={} id={} kind={} domain={} package={} native={} dispatch={} contracts={} bytes={} hash={} path={}",
            input.order_index,
            input.input_id,
            input.input_kind,
            input.domain_family,
            input.package_id,
            input.native_ir,
            input.dispatch_lowering,
            input.contract_count,
            input.content_bytes,
            input.content_hash,
            input.path
        );
    }
    println!(
        "  external_dependencies: {}",
        report.external_dependencies.len()
    );
    for dependency in &report.external_dependencies {
        println!("  external_dependency: {dependency}");
    }
    println!("  unresolved: {}", report.unresolved.len());
    for item in &report.unresolved {
        println!("  unresolved_item: {item}");
    }
}

fn print_nsld_link_unit_report(report: &NsldLinkUnitReport) {
    println!("Nsld link units");
    println!("  manifest: {}", report.manifest);
    println!("  unit_count: {}", report.unit_count);
    println!("  hetero_unit_count: {}", report.hetero_unit_count);
    println!("  link_input_count: {}", report.link_input_count);
    println!("  clock_edge_count: {}", report.clock_edge_count);
    println!("  data_segment_count: {}", report.data_segment_count);
    println!("  unit_table_hash: {}", report.unit_table_hash);
    for unit in &report.units {
        println!(
            "  link_unit: order={} id={} kind={} domain={} package={} backend={} target={} role={} inputs={} clock_edges={} data_segments={} host_wrapper={} order_key={}",
            unit.order_index,
            unit.unit_id,
            unit.unit_kind,
            unit.domain_family,
            unit.package_id,
            unit.backend_family,
            unit.lowering_target,
            unit.packaging_role,
            unit.link_input_ids.join(","),
            unit.clock_edge_count,
            unit.data_segment_count,
            unit.requires_host_wrapper,
            unit.deterministic_order_key
        );
    }
}

fn print_nsld_link_bundle_report(report: &NsldLinkBundleReport) {
    println!("Nsld link bundle");
    println!("  manifest: {}", report.manifest);
    println!("  bundle_id: {}", report.bundle_id);
    println!("  bundle_hash: {}", report.bundle_hash);
    println!("  bundle_ready: {}", report.bundle_ready);
    println!("  unit_count: {}", report.unit_count);
    println!("  hetero_unit_count: {}", report.hetero_unit_count);
    println!("  link_input_count: {}", report.link_input_count);
    println!(
        "  link_input_total_bytes: {}",
        report.link_input_total_bytes
    );
    println!("  link_input_table_hash: {}", report.link_input_table_hash);
    println!("  unit_table_hash: {}", report.unit_table_hash);
    println!("  clock_edge_count: {}", report.clock_edge_count);
    println!("  data_segment_count: {}", report.data_segment_count);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!(
        "  compiled_artifact_path: {}",
        report.compiled_artifact_path
    );
    println!("  native_output_path: {}", report.native_output_path);
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn print_nsld_link_bundle_emit_report(report: &NsldLinkBundleEmitReport) {
    println!("Nsld link bundle emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  bundle_id: {}", report.bundle_id);
    println!("  bundle_hash: {}", report.bundle_hash);
    println!("  bundle_ready: {}", report.bundle_ready);
}

fn print_nsld_link_bundle_verify_report(report: &NsldLinkBundleVerifyReport) {
    println!("Nsld link bundle verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!("  expected_bundle_id: {}", report.expected_bundle_id);
    println!("  expected_bundle_hash: {}", report.expected_bundle_hash);
    println!(
        "  actual_bundle_id: {}",
        report.actual_bundle_id.as_deref().unwrap_or("missing")
    );
    println!(
        "  actual_bundle_hash: {}",
        report.actual_bundle_hash.as_deref().unwrap_or("missing")
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn print_nsld_prepare_report(report: &NsldPrepareReport) {
    println!("Nsld prepare");
    println!("  manifest: {}", report.manifest);
    println!("  valid: {}", report.valid);
    println!("  output_dir: {}", report.output_dir);
    println!("  link_input_table: {}", report.link_input_table_path);
    println!("  link_unit_table: {}", report.link_unit_table_path);
    println!("  link_bundle: {}", report.link_bundle_path);
    println!("  assemble_plan: {}", report.assemble_plan_path);
    println!("  section_manifest: {}", report.section_manifest_path);
    println!("  container_plan: {}", report.container_plan_path);
    println!("  container: {}", report.container_path);
    println!("  container_payload: {}", report.container_payload_path);
    println!("  link_input_count: {}", report.link_input_count);
    println!("  link_input_table_hash: {}", report.link_input_table_hash);
    println!("  unit_count: {}", report.unit_count);
    println!("  unit_table_hash: {}", report.unit_table_hash);
    println!("  bundle_id: {}", report.bundle_id);
    println!("  bundle_hash: {}", report.bundle_hash);
    println!("  bundle_ready: {}", report.bundle_ready);
    println!("  assemble_plan_hash: {}", report.assemble_plan_hash);
    println!("  section_table_hash: {}", report.section_table_hash);
    println!("  container_layout_hash: {}", report.container_layout_hash);
    println!("  container_hash: {}", report.container_hash);
    println!("  payload_size_bytes: {}", report.payload_size_bytes);
    println!("  payload_hash: {}", report.payload_hash);
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn print_nsld_assemble_plan_report(report: &NsldAssemblePlanReport) {
    println!("Nsld assemble plan");
    println!("  manifest: {}", report.manifest);
    println!("  ready: {}", report.ready);
    println!("  bundle_id: {}", report.bundle_id);
    println!("  bundle_hash: {}", report.bundle_hash);
    println!("  assemble_plan_hash: {}", report.assemble_plan_hash);
    println!("  section_count: {}", report.section_count);
    for section in &report.sections {
        println!(
            "  section: order={} id={} kind={} required={} hash={} source={}",
            section.order_index,
            section.section_id,
            section.section_kind,
            section.required,
            section.source_hash,
            section.source_path
        );
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

fn print_nsld_assemble_plan_emit_report(report: &NsldAssemblePlanEmitReport) {
    println!("Nsld assemble plan emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  ready: {}", report.ready);
    println!("  assemble_plan_hash: {}", report.assemble_plan_hash);
    println!("  section_count: {}", report.section_count);
}

fn print_nsld_assemble_plan_verify_report(report: &NsldAssemblePlanVerifyReport) {
    println!("Nsld assemble plan verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_assemble_plan_hash: {}",
        report.expected_assemble_plan_hash
    );
    println!(
        "  expected_section_count: {}",
        report.expected_section_count
    );
    println!(
        "  actual_assemble_plan_hash: {}",
        report
            .actual_assemble_plan_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_section_count: {}",
        optional_usize_text(report.actual_section_count)
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn print_nsld_section_manifest_report(report: &NsldSectionManifestReport) {
    println!("Nsld section manifest");
    println!("  manifest: {}", report.manifest);
    println!("  ready: {}", report.ready);
    println!("  assemble_plan_hash: {}", report.assemble_plan_hash);
    println!("  section_count: {}", report.section_count);
    println!("  section_table_hash: {}", report.section_table_hash);
    for section in &report.sections {
        println!(
            "  section: order={} id={} kind={} required={} hash={} source={}",
            section.order_index,
            section.section_id,
            section.section_kind,
            section.required,
            section.source_hash,
            section.source_path
        );
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

fn print_nsld_section_manifest_emit_report(report: &NsldSectionManifestEmitReport) {
    println!("Nsld section manifest emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  ready: {}", report.ready);
    println!("  section_count: {}", report.section_count);
    println!("  section_table_hash: {}", report.section_table_hash);
}

fn print_nsld_section_manifest_verify_report(report: &NsldSectionManifestVerifyReport) {
    println!("Nsld section manifest verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_section_count: {}",
        report.expected_section_count
    );
    println!(
        "  expected_section_table_hash: {}",
        report.expected_section_table_hash
    );
    println!(
        "  actual_section_count: {}",
        optional_usize_text(report.actual_section_count)
    );
    println!(
        "  actual_section_table_hash: {}",
        report
            .actual_section_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn print_nsld_container_plan_report(report: &NsldContainerPlanReport) {
    println!("Nsld container plan");
    println!("  manifest: {}", report.manifest);
    println!("  ready: {}", report.ready);
    println!("  container_magic: {}", report.container_magic);
    println!("  container_version: {}", report.container_version);
    println!("  section_count: {}", report.section_count);
    println!("  section_table_hash: {}", report.section_table_hash);
    println!("  container_layout_hash: {}", report.container_layout_hash);
    println!("  output_path: {}", report.output_path);
    for section in &report.sections {
        println!(
            "  section: order={} id={} kind={} required={} hash={} source={}",
            section.order_index,
            section.section_id,
            section.section_kind,
            section.required,
            section.source_hash,
            section.source_path
        );
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

fn print_nsld_container_plan_emit_report(report: &NsldContainerPlanEmitReport) {
    println!("Nsld container plan emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  ready: {}", report.ready);
    println!("  section_count: {}", report.section_count);
    println!("  container_layout_hash: {}", report.container_layout_hash);
}

fn print_nsld_container_plan_verify_report(report: &NsldContainerPlanVerifyReport) {
    println!("Nsld container plan verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_container_layout_hash: {}",
        report.expected_container_layout_hash
    );
    println!(
        "  expected_section_count: {}",
        report.expected_section_count
    );
    println!(
        "  actual_container_layout_hash: {}",
        report
            .actual_container_layout_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_section_count: {}",
        optional_usize_text(report.actual_section_count)
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn print_nsld_container_report(report: &NsldContainerReport) {
    println!("Nsld container");
    println!("  manifest: {}", report.manifest);
    println!("  ready: {}", report.ready);
    println!("  container_magic: {}", report.container_magic);
    println!("  container_version: {}", report.container_version);
    println!("  container_layout_hash: {}", report.container_layout_hash);
    println!("  container_hash: {}", report.container_hash);
    println!("  payload_size_bytes: {}", report.payload_size_bytes);
    println!("  payload_hash: {}", report.payload_hash);
    println!("  output_path: {}", report.output_path);
    println!("  payload_path: {}", report.payload_path);
    println!("  section_count: {}", report.section_count);
    for section in &report.sections {
        println!(
            "  section: order={} id={} kind={} required={} offset={} size={} payload_hash={} hash={} source={}",
            section.order_index,
            section.section_id,
            section.section_kind,
            section.required,
            section.offset,
            section.size_bytes,
            section.payload_hash,
            section.source_hash,
            section.source_path
        );
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

fn print_nsld_container_emit_report(report: &NsldContainerEmitReport) {
    println!("Nsld container emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  payload_path: {}", report.payload_path);
    println!("  ready: {}", report.ready);
    println!("  section_count: {}", report.section_count);
    println!("  container_layout_hash: {}", report.container_layout_hash);
    println!("  container_hash: {}", report.container_hash);
    println!("  payload_size_bytes: {}", report.payload_size_bytes);
    println!("  payload_hash: {}", report.payload_hash);
}

fn print_nsld_container_verify_report(report: &NsldContainerVerifyReport) {
    println!("Nsld container verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_container_layout_hash: {}",
        report.expected_container_layout_hash
    );
    println!(
        "  expected_container_hash: {}",
        report.expected_container_hash
    );
    println!(
        "  expected_payload_size_bytes: {}",
        report.expected_payload_size_bytes
    );
    println!("  expected_payload_hash: {}", report.expected_payload_hash);
    println!("  expected_payload_path: {}", report.expected_payload_path);
    println!(
        "  expected_section_count: {}",
        report.expected_section_count
    );
    println!(
        "  actual_container_layout_hash: {}",
        report
            .actual_container_layout_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_container_hash: {}",
        report.actual_container_hash.as_deref().unwrap_or("missing")
    );
    println!(
        "  actual_payload_size_bytes: {}",
        optional_usize_text(report.actual_payload_size_bytes)
    );
    println!(
        "  actual_payload_hash: {}",
        report.actual_payload_hash.as_deref().unwrap_or("missing")
    );
    println!(
        "  actual_section_count: {}",
        optional_usize_text(report.actual_section_count)
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn print_nsld_link_units_emit_report(report: &NsldLinkUnitsEmitReport) {
    println!("Nsld link units emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  unit_count: {}", report.unit_count);
    println!("  hetero_unit_count: {}", report.hetero_unit_count);
    println!("  link_input_count: {}", report.link_input_count);
    println!("  unit_table_hash: {}", report.unit_table_hash);
}

fn nsld_link_units_emit_report_json(report: &NsldLinkUnitsEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_units_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_usize_field("unit_count", report.unit_count),
        json_usize_field("hetero_unit_count", report.hetero_unit_count),
        json_usize_field("link_input_count", report.link_input_count),
        json_string_field("unit_table_hash", &report.unit_table_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

fn print_nsld_link_units_verify_report(report: &NsldLinkUnitsVerifyReport) {
    println!("Nsld link units verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!("  expected_unit_count: {}", report.expected_unit_count);
    println!(
        "  expected_hetero_unit_count: {}",
        report.expected_hetero_unit_count
    );
    println!(
        "  expected_link_input_count: {}",
        report.expected_link_input_count
    );
    println!(
        "  expected_unit_table_hash: {}",
        report.expected_unit_table_hash
    );
    println!(
        "  actual_unit_count: {}",
        optional_usize_text(report.actual_unit_count)
    );
    println!(
        "  actual_hetero_unit_count: {}",
        optional_usize_text(report.actual_hetero_unit_count)
    );
    println!(
        "  actual_link_input_count: {}",
        optional_usize_text(report.actual_link_input_count)
    );
    println!(
        "  actual_unit_table_hash: {}",
        report
            .actual_unit_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn nsld_link_units_verify_report_json(report: &NsldLinkUnitsVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_units_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_usize_field("expected_unit_count", report.expected_unit_count),
        json_usize_field(
            "expected_hetero_unit_count",
            report.expected_hetero_unit_count,
        ),
        json_usize_field(
            "expected_link_input_count",
            report.expected_link_input_count,
        ),
        json_string_field("expected_unit_table_hash", &report.expected_unit_table_hash),
        json_optional_usize_field("actual_unit_count", report.actual_unit_count),
        json_optional_usize_field("actual_hetero_unit_count", report.actual_hetero_unit_count),
        json_optional_usize_field("actual_link_input_count", report.actual_link_input_count),
        json_optional_string_field(
            "actual_unit_table_hash",
            report.actual_unit_table_hash.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_link_bundle_report_json(report: &NsldLinkBundleReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_bundle"),
        json_string_field("manifest", &report.manifest),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_bool_field("bundle_ready", report.bundle_ready),
        json_usize_field("unit_count", report.unit_count),
        json_usize_field("hetero_unit_count", report.hetero_unit_count),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
        json_string_field("unit_table_hash", &report.unit_table_hash),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_string_field("compiled_artifact_path", &report.compiled_artifact_path),
        json_string_field("native_output_path", &report.native_output_path),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_link_bundle_emit_report_json(report: &NsldLinkBundleEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_bundle_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_bool_field("bundle_ready", report.bundle_ready),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_link_bundle_verify_report_json(report: &NsldLinkBundleVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_bundle_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field("expected_bundle_id", &report.expected_bundle_id),
        json_string_field("expected_bundle_hash", &report.expected_bundle_hash),
        json_optional_string_field("actual_bundle_id", report.actual_bundle_id.as_deref()),
        json_optional_string_field("actual_bundle_hash", report.actual_bundle_hash.as_deref()),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_prepare_report_json(report: &NsldPrepareReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_prepare"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("valid", report.valid),
        json_string_field("output_dir", &report.output_dir),
        json_string_field("link_input_table_path", &report.link_input_table_path),
        json_string_field("link_unit_table_path", &report.link_unit_table_path),
        json_string_field("link_bundle_path", &report.link_bundle_path),
        json_string_field("assemble_plan_path", &report.assemble_plan_path),
        json_string_field("section_manifest_path", &report.section_manifest_path),
        json_string_field("container_plan_path", &report.container_plan_path),
        json_string_field("container_path", &report.container_path),
        json_string_field("container_payload_path", &report.container_payload_path),
        json_usize_field("link_input_count", report.link_input_count),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
        json_usize_field("unit_count", report.unit_count),
        json_string_field("unit_table_hash", &report.unit_table_hash),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_bool_field("bundle_ready", report.bundle_ready),
        json_string_field("assemble_plan_hash", &report.assemble_plan_hash),
        json_string_field("section_table_hash", &report.section_table_hash),
        json_string_field("container_layout_hash", &report.container_layout_hash),
        json_string_field("container_hash", &report.container_hash),
        json_usize_field("payload_size_bytes", report.payload_size_bytes),
        json_string_field("payload_hash", &report.payload_hash),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_assemble_plan_report_json(report: &NsldAssemblePlanReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_assemble_plan"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_string_field("assemble_plan_hash", &report.assemble_plan_hash),
        json_usize_field("section_count", report.section_count),
        format!(
            "\"sections\":[{}]",
            nsld_assemble_sections_json(&report.sections)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_assemble_plan_emit_report_json(report: &NsldAssemblePlanEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_assemble_plan_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("ready", report.ready),
        json_string_field("assemble_plan_hash", &report.assemble_plan_hash),
        json_usize_field("section_count", report.section_count),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_assemble_plan_verify_report_json(report: &NsldAssemblePlanVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_assemble_plan_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_assemble_plan_hash",
            &report.expected_assemble_plan_hash,
        ),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_optional_string_field(
            "actual_assemble_plan_hash",
            report.actual_assemble_plan_hash.as_deref(),
        ),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_section_manifest_report_json(report: &NsldSectionManifestReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_section_manifest"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("assemble_plan_hash", &report.assemble_plan_hash),
        json_usize_field("section_count", report.section_count),
        json_string_field("section_table_hash", &report.section_table_hash),
        format!(
            "\"sections\":[{}]",
            nsld_assemble_sections_json(&report.sections)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_section_manifest_emit_report_json(report: &NsldSectionManifestEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_section_manifest_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("ready", report.ready),
        json_usize_field("section_count", report.section_count),
        json_string_field("section_table_hash", &report.section_table_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_section_manifest_verify_report_json(report: &NsldSectionManifestVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_section_manifest_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_string_field(
            "expected_section_table_hash",
            &report.expected_section_table_hash,
        ),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_optional_string_field(
            "actual_section_table_hash",
            report.actual_section_table_hash.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_container_plan_report_json(report: &NsldContainerPlanReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_container_plan"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("container_magic", &report.container_magic),
        json_usize_field("container_version", report.container_version),
        json_usize_field("section_count", report.section_count),
        json_string_field("section_table_hash", &report.section_table_hash),
        json_string_field("container_layout_hash", &report.container_layout_hash),
        json_string_field("output_path", &report.output_path),
        format!(
            "\"sections\":[{}]",
            nsld_assemble_sections_json(&report.sections)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_container_plan_emit_report_json(report: &NsldContainerPlanEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_container_plan_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("ready", report.ready),
        json_usize_field("section_count", report.section_count),
        json_string_field("container_layout_hash", &report.container_layout_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_container_plan_verify_report_json(report: &NsldContainerPlanVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_container_plan_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_container_layout_hash",
            &report.expected_container_layout_hash,
        ),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_optional_string_field(
            "actual_container_layout_hash",
            report.actual_container_layout_hash.as_deref(),
        ),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_container_report_json(report: &NsldContainerReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_container"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("container_magic", &report.container_magic),
        json_usize_field("container_version", report.container_version),
        json_string_field("container_layout_hash", &report.container_layout_hash),
        json_string_field("container_hash", &report.container_hash),
        json_usize_field("payload_size_bytes", report.payload_size_bytes),
        json_string_field("payload_hash", &report.payload_hash),
        json_string_field("output_path", &report.output_path),
        json_string_field("payload_path", &report.payload_path),
        json_usize_field("section_count", report.section_count),
        format!(
            "\"sections\":[{}]",
            nsld_container_sections_json(&report.sections)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_container_emit_report_json(report: &NsldContainerEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_container_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("payload_path", &report.payload_path),
        json_bool_field("ready", report.ready),
        json_string_field("container_layout_hash", &report.container_layout_hash),
        json_string_field("container_hash", &report.container_hash),
        json_usize_field("payload_size_bytes", report.payload_size_bytes),
        json_string_field("payload_hash", &report.payload_hash),
        json_usize_field("section_count", report.section_count),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_container_verify_report_json(report: &NsldContainerVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_container_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_container_layout_hash",
            &report.expected_container_layout_hash,
        ),
        json_string_field("expected_container_hash", &report.expected_container_hash),
        json_usize_field(
            "expected_payload_size_bytes",
            report.expected_payload_size_bytes,
        ),
        json_string_field("expected_payload_hash", &report.expected_payload_hash),
        json_string_field("expected_payload_path", &report.expected_payload_path),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_optional_string_field(
            "actual_container_layout_hash",
            report.actual_container_layout_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_container_hash",
            report.actual_container_hash.as_deref(),
        ),
        json_optional_usize_field(
            "actual_payload_size_bytes",
            report.actual_payload_size_bytes,
        ),
        json_optional_string_field("actual_payload_hash", report.actual_payload_hash.as_deref()),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn print_nsld_link_inputs_emit_report(report: &NsldLinkInputsEmitReport) {
    println!("Nsld link inputs");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  link_input_count: {}", report.link_input_count);
    println!(
        "  link_input_total_bytes: {}",
        report.link_input_total_bytes
    );
    println!("  link_input_table_hash: {}", report.link_input_table_hash);
}

fn nsld_link_inputs_emit_report_json(report: &NsldLinkInputsEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_inputs_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

fn print_nsld_link_inputs_verify_report(report: &NsldLinkInputsVerifyReport) {
    println!("Nsld link inputs verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_link_input_count: {}",
        report.expected_link_input_count
    );
    println!(
        "  expected_link_input_total_bytes: {}",
        report.expected_link_input_total_bytes
    );
    println!(
        "  expected_link_input_table_hash: {}",
        report.expected_link_input_table_hash
    );
    println!(
        "  actual_link_input_count: {}",
        optional_usize_text(report.actual_link_input_count)
    );
    println!(
        "  actual_link_input_total_bytes: {}",
        optional_usize_text(report.actual_link_input_total_bytes)
    );
    println!(
        "  actual_link_input_table_hash: {}",
        report
            .actual_link_input_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn nsld_link_inputs_verify_report_json(report: &NsldLinkInputsVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_inputs_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_usize_field(
            "expected_link_input_count",
            report.expected_link_input_count,
        ),
        json_usize_field(
            "expected_link_input_total_bytes",
            report.expected_link_input_total_bytes,
        ),
        json_string_field(
            "expected_link_input_table_hash",
            &report.expected_link_input_table_hash,
        ),
        json_optional_usize_field("actual_link_input_count", report.actual_link_input_count),
        json_optional_usize_field(
            "actual_link_input_total_bytes",
            report.actual_link_input_total_bytes,
        ),
        json_optional_string_field(
            "actual_link_input_table_hash",
            report.actual_link_input_table_hash.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_closure_report_json(report: &NsldClosureReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_closure"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("closed", report.closed),
        json_string_array_field("internal_contracts", &report.internal_contracts),
        format!(
            "\"link_inputs\":[{}]",
            nsld_link_inputs_json(&report.link_inputs)
        ),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
        json_bool_field("link_input_table_present", report.link_input_table_present),
        json_optional_bool_field("link_input_table_valid", report.link_input_table_valid),
        json_string_array_field("external_dependencies", &report.external_dependencies),
        json_string_array_field("unresolved", &report.unresolved),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_usize_field("domain_count", report.domain_count),
        json_usize_field("hetero_domain_count", report.hetero_domain_count),
        json_usize_field("sidecar_capability_count", report.sidecar_capability_count),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_link_unit_report_json(report: &NsldLinkUnitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_units"),
        json_string_field("manifest", &report.manifest),
        json_usize_field("unit_count", report.unit_count),
        json_usize_field("hetero_unit_count", report.hetero_unit_count),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_string_field("unit_table_hash", &report.unit_table_hash),
        format!("\"units\":[{}]", nsld_link_units_json(&report.units)),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_domain_diagnostics(plan: &nuisc::linker::LinkPlan) -> Vec<NsldDomainDiagnostic> {
    plan.domain_units
        .iter()
        .map(|unit| {
            let alignment = plan
                .artifact_lowering_alignment
                .checks
                .iter()
                .find(|check| {
                    check.package_id == unit.package_id && check.domain_family == unit.domain_family
                });
            NsldDomainDiagnostic {
                domain_family: unit.domain_family.clone(),
                package_id: unit.package_id.clone(),
                kind: unit.kind.clone(),
                packaging_role: unit.packaging_role.clone(),
                lowering_target: unit
                    .selected_lowering_target
                    .clone()
                    .unwrap_or_else(|| "none".to_owned()),
                backend_family: unit
                    .backend_family
                    .clone()
                    .unwrap_or_else(|| "none".to_owned()),
                alignment_consistent: alignment.map(|check| check.consistent).unwrap_or(true),
                alignment_issues: alignment
                    .map(|check| check.issues.clone())
                    .unwrap_or_default(),
            }
        })
        .collect()
}

fn nsld_sidecar_capability_diagnostics(
    plan: &nuisc::linker::LinkPlan,
) -> Vec<NsldSidecarCapabilityDiagnostic> {
    plan.domain_units
        .iter()
        .filter(|unit| unit.kind == "heterogeneous")
        .filter(|unit| unit.artifact_ir_sidecar_path.is_some())
        .map(|unit| {
            let path = unit
                .artifact_ir_sidecar_path
                .clone()
                .unwrap_or_else(|| "none".to_owned());
            let Some(source) = unit
                .artifact_ir_sidecar_path
                .as_deref()
                .and_then(|path| fs::read_to_string(path).ok())
            else {
                return NsldSidecarCapabilityDiagnostic {
                    domain_family: unit.domain_family.clone(),
                    package_id: unit.package_id.clone(),
                    path,
                    content_bytes: 0,
                    content_hash: "missing".to_owned(),
                    valid: false,
                    capability_owner: "missing".to_owned(),
                    frontend_ir: "missing".to_owned(),
                    native_ir: "missing".to_owned(),
                    dispatch_lowering: "missing".to_owned(),
                    validation_contracts: Vec::new(),
                    issues: vec!["missing_or_unreadable_ir_sidecar".to_owned()],
                };
            };

            let capability_owner =
                toml_string_value(&source, "capability_owner").unwrap_or_else(|| "missing".to_owned());
            let frontend_ir =
                toml_string_value(&source, "frontend_ir").unwrap_or_else(|| "missing".to_owned());
            let native_ir =
                toml_string_value(&source, "native_ir").unwrap_or_else(|| "missing".to_owned());
            let dispatch_lowering =
                toml_string_value(&source, "dispatch_lowering").unwrap_or_else(|| "missing".to_owned());
            let validation_contracts = toml_string_array_value(&source, "validation_contracts");
            let mut issues = Vec::new();
            let expected_owner = format!("{}-nustar", unit.domain_family);
            if capability_owner != expected_owner {
                issues.push(format!(
                    "capability_owner mismatch: expected `{expected_owner}`, found `{capability_owner}`"
                ));
            }
            let expected_frontend = format!("nuis-yir.{}", unit.domain_family);
            if frontend_ir != expected_frontend {
                issues.push(format!(
                    "frontend_ir mismatch: expected `{expected_frontend}`, found `{frontend_ir}`"
                ));
            }
            if native_ir == "missing" || native_ir == "unknown" || native_ir == "unimplemented" {
                issues.push(format!("native_ir is not link-ready: `{native_ir}`"));
            }
            if dispatch_lowering == "missing" || dispatch_lowering == "unimplemented" {
                issues.push(format!(
                    "dispatch_lowering is not link-ready: `{dispatch_lowering}`"
                ));
            }
            if validation_contracts.is_empty() {
                issues.push("validation_contracts is empty".to_owned());
            }

            NsldSidecarCapabilityDiagnostic {
                domain_family: unit.domain_family.clone(),
                package_id: unit.package_id.clone(),
                path,
                content_bytes: source.len(),
                content_hash: fnv1a64_hex(source.as_bytes()),
                valid: issues.is_empty(),
                capability_owner,
                frontend_ir,
                native_ir,
                dispatch_lowering,
                validation_contracts,
                issues,
            }
        })
        .collect()
}

fn nsld_link_input_diagnostics(
    capabilities: &[NsldSidecarCapabilityDiagnostic],
) -> Vec<NsldLinkInputDiagnostic> {
    let mut capabilities = capabilities
        .iter()
        .filter(|capability| capability.valid)
        .collect::<Vec<_>>();
    capabilities.sort_by(|left, right| {
        left.domain_family
            .cmp(&right.domain_family)
            .then_with(|| left.package_id.cmp(&right.package_id))
            .then_with(|| left.path.cmp(&right.path))
    });
    capabilities
        .into_iter()
        .enumerate()
        .map(|(index, capability)| NsldLinkInputDiagnostic {
            order_index: index,
            input_id: format!(
                "li{:04}.{}.{}",
                index, capability.domain_family, capability.package_id
            ),
            input_kind: "lowering-ir-sidecar".to_owned(),
            domain_family: capability.domain_family.clone(),
            package_id: capability.package_id.clone(),
            path: capability.path.clone(),
            native_ir: capability.native_ir.clone(),
            dispatch_lowering: capability.dispatch_lowering.clone(),
            contract_count: capability.validation_contracts.len(),
            content_bytes: capability.content_bytes,
            content_hash: capability.content_hash.clone(),
        })
        .collect()
}

fn nsld_link_input_summary(
    capabilities: &[NsldSidecarCapabilityDiagnostic],
) -> NsldLinkInputSummary {
    let inputs = nsld_link_input_diagnostics(capabilities);
    let count = inputs.len();
    let total_bytes = inputs
        .iter()
        .map(|input| input.content_bytes)
        .sum::<usize>();
    let table_hash = nsld_link_input_table_hash(&inputs);
    NsldLinkInputSummary {
        inputs,
        count,
        total_bytes,
        table_hash,
    }
}

fn nsld_link_input_table_hash(inputs: &[NsldLinkInputDiagnostic]) -> String {
    let mut material = String::new();
    for input in inputs {
        material.push_str(&input.order_index.to_string());
        material.push('\t');
        material.push_str(&input.input_id);
        material.push('\t');
        material.push_str(&input.input_kind);
        material.push('\t');
        material.push_str(&input.domain_family);
        material.push('\t');
        material.push_str(&input.package_id);
        material.push('\t');
        material.push_str(&input.native_ir);
        material.push('\t');
        material.push_str(&input.dispatch_lowering);
        material.push('\t');
        material.push_str(&input.contract_count.to_string());
        material.push('\t');
        material.push_str(&input.content_bytes.to_string());
        material.push('\t');
        material.push_str(&input.content_hash);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

fn render_nsld_link_input_table_toml(
    inputs: &[NsldLinkInputDiagnostic],
    total_bytes: usize,
    table_hash: &str,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_LINK_INPUT_TABLE_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "table_kind = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!("link_input_count = {}\n", inputs.len()));
    out.push_str(&format!("link_input_total_bytes = {total_bytes}\n"));
    out.push_str(&format!(
        "link_input_table_hash = \"{}\"\n",
        escape_toml_string(table_hash)
    ));
    for input in inputs {
        out.push_str("\n[[link_input]]\n");
        out.push_str(&format!("order_index = {}\n", input.order_index));
        out.push_str(&format!(
            "input_id = \"{}\"\n",
            escape_toml_string(&input.input_id)
        ));
        out.push_str(&format!(
            "input_kind = \"{}\"\n",
            escape_toml_string(&input.input_kind)
        ));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&input.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&input.package_id)
        ));
        out.push_str(&format!("path = \"{}\"\n", escape_toml_string(&input.path)));
        out.push_str(&format!(
            "native_ir = \"{}\"\n",
            escape_toml_string(&input.native_ir)
        ));
        out.push_str(&format!(
            "dispatch_lowering = \"{}\"\n",
            escape_toml_string(&input.dispatch_lowering)
        ));
        out.push_str(&format!("contract_count = {}\n", input.contract_count));
        out.push_str(&format!("content_bytes = {}\n", input.content_bytes));
        out.push_str(&format!(
            "content_hash = \"{}\"\n",
            escape_toml_string(&input.content_hash)
        ));
    }
    out
}

fn render_nsld_link_unit_table_toml(report: &NsldLinkUnitReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_LINK_UNIT_TABLE_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_LINK_UNIT_TABLE_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "table_kind = \"{}\"\n",
        escape_toml_string(NSLD_LINK_UNIT_TABLE_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!("unit_count = {}\n", report.unit_count));
    out.push_str(&format!(
        "hetero_unit_count = {}\n",
        report.hetero_unit_count
    ));
    out.push_str(&format!("link_input_count = {}\n", report.link_input_count));
    out.push_str(&format!("clock_edge_count = {}\n", report.clock_edge_count));
    out.push_str(&format!(
        "data_segment_count = {}\n",
        report.data_segment_count
    ));
    out.push_str(&format!(
        "unit_table_hash = \"{}\"\n",
        escape_toml_string(&report.unit_table_hash)
    ));
    for unit in &report.units {
        out.push_str("\n[[link_unit]]\n");
        out.push_str(&format!("order_index = {}\n", unit.order_index));
        out.push_str(&format!(
            "unit_id = \"{}\"\n",
            escape_toml_string(&unit.unit_id)
        ));
        out.push_str(&format!(
            "unit_kind = \"{}\"\n",
            escape_toml_string(&unit.unit_kind)
        ));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&unit.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&unit.package_id)
        ));
        out.push_str(&format!(
            "backend_family = \"{}\"\n",
            escape_toml_string(&unit.backend_family)
        ));
        out.push_str(&format!(
            "lowering_target = \"{}\"\n",
            escape_toml_string(&unit.lowering_target)
        ));
        out.push_str(&format!(
            "packaging_role = \"{}\"\n",
            escape_toml_string(&unit.packaging_role)
        ));
        out.push_str(&format!(
            "link_input_ids = [{}]\n",
            toml_string_array_literal(&unit.link_input_ids)
        ));
        out.push_str(&format!("clock_edge_count = {}\n", unit.clock_edge_count));
        out.push_str(&format!(
            "data_segment_count = {}\n",
            unit.data_segment_count
        ));
        out.push_str(&format!(
            "requires_host_wrapper = {}\n",
            unit.requires_host_wrapper
        ));
        out.push_str(&format!(
            "deterministic_order_key = \"{}\"\n",
            escape_toml_string(&unit.deterministic_order_key)
        ));
    }
    out
}

fn render_nsld_link_bundle_toml(report: &NsldLinkBundleReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_LINK_BUNDLE_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_LINK_BUNDLE_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "bundle_kind = \"{}\"\n",
        escape_toml_string(NSLD_LINK_BUNDLE_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!(
        "bundle_id = \"{}\"\n",
        escape_toml_string(&report.bundle_id)
    ));
    out.push_str(&format!(
        "bundle_hash = \"{}\"\n",
        escape_toml_string(&report.bundle_hash)
    ));
    out.push_str(&format!("bundle_ready = {}\n", report.bundle_ready));
    out.push_str(&format!("unit_count = {}\n", report.unit_count));
    out.push_str(&format!(
        "hetero_unit_count = {}\n",
        report.hetero_unit_count
    ));
    out.push_str(&format!("link_input_count = {}\n", report.link_input_count));
    out.push_str(&format!(
        "link_input_total_bytes = {}\n",
        report.link_input_total_bytes
    ));
    out.push_str(&format!(
        "link_input_table_hash = \"{}\"\n",
        escape_toml_string(&report.link_input_table_hash)
    ));
    out.push_str(&format!(
        "unit_table_hash = \"{}\"\n",
        escape_toml_string(&report.unit_table_hash)
    ));
    out.push_str(&format!("clock_edge_count = {}\n", report.clock_edge_count));
    out.push_str(&format!(
        "data_segment_count = {}\n",
        report.data_segment_count
    ));
    out.push_str(&format!(
        "final_stage_link_mode = \"{}\"\n",
        escape_toml_string(&report.final_stage_link_mode)
    ));
    out.push_str(&format!(
        "host_wrapper_required = {}\n",
        report.host_wrapper_required
    ));
    out.push_str(&format!(
        "compiled_artifact_path = \"{}\"\n",
        escape_toml_string(&report.compiled_artifact_path)
    ));
    out.push_str(&format!(
        "native_output_path = \"{}\"\n",
        escape_toml_string(&report.native_output_path)
    ));
    out.push_str(&format!(
        "issues = [{}]\n",
        toml_string_array_literal(&report.issues)
    ));
    out
}

fn render_nsld_assemble_plan_toml(report: &NsldAssemblePlanReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_ASSEMBLE_PLAN_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_ASSEMBLE_PLAN_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "plan_kind = \"{}\"\n",
        escape_toml_string(NSLD_ASSEMBLE_PLAN_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!("ready = {}\n", report.ready));
    out.push_str(&format!(
        "bundle_id = \"{}\"\n",
        escape_toml_string(&report.bundle_id)
    ));
    out.push_str(&format!(
        "bundle_hash = \"{}\"\n",
        escape_toml_string(&report.bundle_hash)
    ));
    out.push_str(&format!(
        "assemble_plan_hash = \"{}\"\n",
        escape_toml_string(&report.assemble_plan_hash)
    ));
    out.push_str(&format!("section_count = {}\n", report.section_count));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml_string_array_literal(&report.blockers)
    ));
    for section in &report.sections {
        out.push_str("\n[[section]]\n");
        out.push_str(&format!("order_index = {}\n", section.order_index));
        out.push_str(&format!(
            "section_id = \"{}\"\n",
            escape_toml_string(&section.section_id)
        ));
        out.push_str(&format!(
            "section_kind = \"{}\"\n",
            escape_toml_string(&section.section_kind)
        ));
        out.push_str(&format!(
            "source_path = \"{}\"\n",
            escape_toml_string(&section.source_path)
        ));
        out.push_str(&format!(
            "source_hash = \"{}\"\n",
            escape_toml_string(&section.source_hash)
        ));
        out.push_str(&format!("required = {}\n", section.required));
    }
    out
}

fn render_nsld_section_manifest_toml(report: &NsldSectionManifestReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_SECTION_MANIFEST_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_SECTION_MANIFEST_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "manifest_kind = \"{}\"\n",
        escape_toml_string(NSLD_SECTION_MANIFEST_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!("ready = {}\n", report.ready));
    out.push_str(&format!(
        "assemble_plan_hash = \"{}\"\n",
        escape_toml_string(&report.assemble_plan_hash)
    ));
    out.push_str(&format!("section_count = {}\n", report.section_count));
    out.push_str(&format!(
        "section_table_hash = \"{}\"\n",
        escape_toml_string(&report.section_table_hash)
    ));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml_string_array_literal(&report.blockers)
    ));
    for section in &report.sections {
        out.push_str("\n[[section]]\n");
        out.push_str(&format!("order_index = {}\n", section.order_index));
        out.push_str(&format!(
            "section_id = \"{}\"\n",
            escape_toml_string(&section.section_id)
        ));
        out.push_str(&format!(
            "section_kind = \"{}\"\n",
            escape_toml_string(&section.section_kind)
        ));
        out.push_str(&format!(
            "source_path = \"{}\"\n",
            escape_toml_string(&section.source_path)
        ));
        out.push_str(&format!(
            "source_hash = \"{}\"\n",
            escape_toml_string(&section.source_hash)
        ));
        out.push_str(&format!("required = {}\n", section.required));
    }
    out
}

fn render_nsld_container_plan_toml(report: &NsldContainerPlanReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_CONTAINER_PLAN_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_CONTAINER_PLAN_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "plan_kind = \"{}\"\n",
        escape_toml_string(NSLD_CONTAINER_PLAN_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!("ready = {}\n", report.ready));
    out.push_str(&format!(
        "container_magic = \"{}\"\n",
        escape_toml_string(&report.container_magic)
    ));
    out.push_str(&format!(
        "container_version = {}\n",
        report.container_version
    ));
    out.push_str(&format!("section_count = {}\n", report.section_count));
    out.push_str(&format!(
        "section_table_hash = \"{}\"\n",
        escape_toml_string(&report.section_table_hash)
    ));
    out.push_str(&format!(
        "container_layout_hash = \"{}\"\n",
        escape_toml_string(&report.container_layout_hash)
    ));
    out.push_str(&format!(
        "output_path = \"{}\"\n",
        escape_toml_string(&report.output_path)
    ));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml_string_array_literal(&report.blockers)
    ));
    for section in &report.sections {
        out.push_str("\n[[section]]\n");
        out.push_str(&format!("order_index = {}\n", section.order_index));
        out.push_str(&format!(
            "section_id = \"{}\"\n",
            escape_toml_string(&section.section_id)
        ));
        out.push_str(&format!(
            "section_kind = \"{}\"\n",
            escape_toml_string(&section.section_kind)
        ));
        out.push_str(&format!(
            "source_path = \"{}\"\n",
            escape_toml_string(&section.source_path)
        ));
        out.push_str(&format!(
            "source_hash = \"{}\"\n",
            escape_toml_string(&section.source_hash)
        ));
        out.push_str(&format!("required = {}\n", section.required));
    }
    out
}

fn render_nsld_container_toml(report: &NsldContainerReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_CONTAINER_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_CONTAINER_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "container_kind = \"{}\"\n",
        escape_toml_string(NSLD_CONTAINER_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!("ready = {}\n", report.ready));
    out.push_str(&format!(
        "container_magic = \"{}\"\n",
        escape_toml_string(&report.container_magic)
    ));
    out.push_str(&format!(
        "container_version = {}\n",
        report.container_version
    ));
    out.push_str(&format!("section_count = {}\n", report.section_count));
    out.push_str(&format!(
        "container_layout_hash = \"{}\"\n",
        escape_toml_string(&report.container_layout_hash)
    ));
    out.push_str(&format!(
        "container_hash = \"{}\"\n",
        escape_toml_string(&report.container_hash)
    ));
    out.push_str(&format!(
        "payload_size_bytes = {}\n",
        report.payload_size_bytes
    ));
    out.push_str(&format!(
        "payload_hash = \"{}\"\n",
        escape_toml_string(&report.payload_hash)
    ));
    out.push_str(&format!(
        "payload_path = \"{}\"\n",
        escape_toml_string(&report.payload_path)
    ));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml_string_array_literal(&report.blockers)
    ));
    for section in &report.sections {
        out.push_str("\n[[section]]\n");
        out.push_str(&format!("order_index = {}\n", section.order_index));
        out.push_str(&format!(
            "section_id = \"{}\"\n",
            escape_toml_string(&section.section_id)
        ));
        out.push_str(&format!(
            "section_kind = \"{}\"\n",
            escape_toml_string(&section.section_kind)
        ));
        out.push_str(&format!(
            "source_path = \"{}\"\n",
            escape_toml_string(&section.source_path)
        ));
        out.push_str(&format!(
            "source_hash = \"{}\"\n",
            escape_toml_string(&section.source_hash)
        ));
        out.push_str(&format!(
            "payload_hash = \"{}\"\n",
            escape_toml_string(&section.payload_hash)
        ));
        out.push_str(&format!("required = {}\n", section.required));
        out.push_str(&format!("offset = {}\n", section.offset));
        out.push_str(&format!("size_bytes = {}\n", section.size_bytes));
    }
    out
}

fn toml_string_array_literal(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", escape_toml_string(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn escape_toml_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("0x{hash:016x}")
}

fn toml_string_value(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|raw| {
        let line = raw.trim();
        let (found_key, value) = line.split_once('=')?;
        if found_key.trim() != key {
            return None;
        }
        let value = value.trim();
        value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .map(|value| {
                value
                    .replace("\\n", "\n")
                    .replace("\\\"", "\"")
                    .replace("\\\\", "\\")
            })
    })
}

fn toml_string_array_value(source: &str, key: &str) -> Vec<String> {
    let Some(value) = source.lines().find_map(|raw| {
        let line = raw.trim();
        let (found_key, value) = line.split_once('=')?;
        (found_key.trim() == key).then(|| value.trim().to_owned())
    }) else {
        return Vec::new();
    };
    let Some(body) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };
    body.split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            entry
                .strip_prefix('"')
                .and_then(|entry| entry.strip_suffix('"'))
                .map(str::to_owned)
        })
        .collect()
}

fn toml_usize_value(source: &str, key: &str) -> Option<usize> {
    source.lines().find_map(|raw| {
        let line = raw.trim();
        let (found_key, value) = line.split_once('=')?;
        (found_key.trim() == key)
            .then(|| value.trim().parse::<usize>().ok())
            .flatten()
    })
}

fn print_nsld_check_report(report: &NsldCheckReport) {
    println!("Nsld linker check");
    println!("  manifest: {}", report.manifest);
    println!("  valid: {}", report.valid);
    println!("  checks: {}", report.checks);
    println!("  failures: {}", report.failures);
    println!(
        "  artifact_lowering_alignment: consistent={} mismatches={}",
        report.artifact_lowering_alignment_consistent,
        report.artifact_lowering_alignment_mismatches
    );
    println!("  clock_protocol: valid={}", report.clock_protocol_valid);
    println!(
        "  hetero_calculate: valid={}",
        report.hetero_calculate_valid
    );
    println!(
        "  hetero_static_lifecycle: static_link={} lifecycle_driven={}",
        report.static_link, report.lifecycle_driven
    );
    println!(
        "  sidecar_capabilities: valid={} issues={}",
        report.sidecar_capability_valid,
        report.sidecar_capability_issues.len()
    );
    println!(
        "  link_input_table: present={} valid={}",
        report.link_input_table_present,
        optional_bool_text(report.link_input_table_valid)
    );
    println!(
        "  link_unit_table: present={} valid={}",
        report.link_unit_table_present,
        optional_bool_text(report.link_unit_table_valid)
    );
    println!(
        "  link_bundle: present={} valid={}",
        report.link_bundle_present,
        optional_bool_text(report.link_bundle_valid)
    );
    println!(
        "  assemble_plan: present={} valid={}",
        report.assemble_plan_present,
        optional_bool_text(report.assemble_plan_valid)
    );
    println!(
        "  section_manifest: present={} valid={}",
        report.section_manifest_present,
        optional_bool_text(report.section_manifest_valid)
    );
    println!(
        "  container_plan: present={} valid={}",
        report.container_plan_present,
        optional_bool_text(report.container_plan_valid)
    );
    println!(
        "  container: present={} valid={}",
        report.container_present,
        optional_bool_text(report.container_valid)
    );
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  domains: {}", report.domains.len());
    for domain in &report.domains {
        println!(
            "  domain: {} package={} kind={} lowering={} backend={} alignment_consistent={}",
            domain.domain_family,
            domain.package_id,
            domain.kind,
            domain.lowering_target,
            domain.backend_family,
            domain.alignment_consistent
        );
        for issue in &domain.alignment_issues {
            println!("    domain_issue: {issue}");
        }
    }
    println!(
        "  sidecar_capabilities: {}",
        report.sidecar_capabilities.len()
    );
    for capability in &report.sidecar_capabilities {
        println!(
            "  sidecar_capability: {} package={} owner={} frontend={} native={} dispatch={} valid={} contracts={}",
            capability.domain_family,
            capability.package_id,
            capability.capability_owner,
            capability.frontend_ir,
            capability.native_ir,
            capability.dispatch_lowering,
            capability.valid,
            capability.validation_contracts.len()
        );
        for issue in &capability.issues {
            println!("    sidecar_capability_issue: {issue}");
        }
    }
    println!("  clock_edges: {}", report.clock_edges.len());
    for edge in &report.clock_edges {
        println!(
            "  clock_edge: index={} from={} to={} relation={} source={}",
            edge.index, edge.from, edge.to, edge.relation, edge.source
        );
    }
    println!("  data_segments: {}", report.data_segments.len());
    for segment in &report.data_segments {
        println!(
            "  data_segment: index={} id={} domain={} owner={} order={} phase={} source={}",
            segment.index,
            segment.segment_id,
            segment.domain_family,
            segment.owner_package,
            segment.order_key,
            segment.access_phase,
            segment.source_path
        );
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
    for issue in &report.link_input_table_issues {
        println!("  link_input_table_issue: {issue}");
    }
    for issue in &report.link_unit_table_issues {
        println!("  link_unit_table_issue: {issue}");
    }
    for issue in &report.link_bundle_issues {
        println!("  link_bundle_issue: {issue}");
    }
    for issue in &report.assemble_plan_issues {
        println!("  assemble_plan_issue: {issue}");
    }
    for issue in &report.section_manifest_issues {
        println!("  section_manifest_issue: {issue}");
    }
    for issue in &report.container_plan_issues {
        println!("  container_plan_issue: {issue}");
    }
    for issue in &report.container_issues {
        println!("  container_issue: {issue}");
    }
}

fn nsld_check_report_json(report: &NsldCheckReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_check"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("valid", report.valid),
        json_usize_field("checks", report.checks),
        json_usize_field("failures", report.failures),
        json_bool_field(
            "artifact_lowering_alignment_consistent",
            report.artifact_lowering_alignment_consistent,
        ),
        json_usize_field(
            "artifact_lowering_alignment_mismatches",
            report.artifact_lowering_alignment_mismatches,
        ),
        json_bool_field("clock_protocol_valid", report.clock_protocol_valid),
        json_string_array_field("clock_protocol_issues", &report.clock_protocol_issues),
        json_bool_field("hetero_calculate_valid", report.hetero_calculate_valid),
        json_string_array_field("hetero_calculate_issues", &report.hetero_calculate_issues),
        json_bool_field("static_link", report.static_link),
        json_bool_field("lifecycle_driven", report.lifecycle_driven),
        json_bool_field("sidecar_capability_valid", report.sidecar_capability_valid),
        json_string_array_field(
            "sidecar_capability_issues",
            &report.sidecar_capability_issues,
        ),
        json_bool_field("link_input_table_present", report.link_input_table_present),
        json_optional_bool_field("link_input_table_valid", report.link_input_table_valid),
        json_string_array_field("link_input_table_issues", &report.link_input_table_issues),
        json_bool_field("link_unit_table_present", report.link_unit_table_present),
        json_optional_bool_field("link_unit_table_valid", report.link_unit_table_valid),
        json_string_array_field("link_unit_table_issues", &report.link_unit_table_issues),
        json_bool_field("link_bundle_present", report.link_bundle_present),
        json_optional_bool_field("link_bundle_valid", report.link_bundle_valid),
        json_string_array_field("link_bundle_issues", &report.link_bundle_issues),
        json_bool_field("assemble_plan_present", report.assemble_plan_present),
        json_optional_bool_field("assemble_plan_valid", report.assemble_plan_valid),
        json_string_array_field("assemble_plan_issues", &report.assemble_plan_issues),
        json_bool_field("section_manifest_present", report.section_manifest_present),
        json_optional_bool_field("section_manifest_valid", report.section_manifest_valid),
        json_string_array_field("section_manifest_issues", &report.section_manifest_issues),
        json_bool_field("container_plan_present", report.container_plan_present),
        json_optional_bool_field("container_plan_valid", report.container_plan_valid),
        json_string_array_field("container_plan_issues", &report.container_plan_issues),
        json_bool_field("container_present", report.container_present),
        json_optional_bool_field("container_valid", report.container_valid),
        json_string_array_field("container_issues", &report.container_issues),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        format!("\"domains\":[{}]", nsld_domains_json(&report.domains)),
        format!(
            "\"sidecar_capabilities\":[{}]",
            nsld_sidecar_capabilities_json(&report.sidecar_capabilities)
        ),
        format!(
            "\"clock_edges\":[{}]",
            nsld_clock_edges_json(&report.clock_edges)
        ),
        format!(
            "\"data_segments\":[{}]",
            nsld_data_segments_json(&report.data_segments)
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_domains_json(domains: &[NsldDomainDiagnostic]) -> String {
    domains
        .iter()
        .map(|domain| {
            let fields = vec![
                json_string_field("domain_family", &domain.domain_family),
                json_string_field("package_id", &domain.package_id),
                json_string_field("kind", &domain.kind),
                json_string_field("packaging_role", &domain.packaging_role),
                json_string_field("lowering_target", &domain.lowering_target),
                json_string_field("backend_family", &domain.backend_family),
                json_bool_field("alignment_consistent", domain.alignment_consistent),
                json_string_array_field("alignment_issues", &domain.alignment_issues),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_sidecar_capabilities_json(capabilities: &[NsldSidecarCapabilityDiagnostic]) -> String {
    capabilities
        .iter()
        .map(|capability| {
            let fields = vec![
                json_string_field("domain_family", &capability.domain_family),
                json_string_field("package_id", &capability.package_id),
                json_string_field("path", &capability.path),
                json_bool_field("valid", capability.valid),
                json_string_field("capability_owner", &capability.capability_owner),
                json_string_field("frontend_ir", &capability.frontend_ir),
                json_string_field("native_ir", &capability.native_ir),
                json_string_field("dispatch_lowering", &capability.dispatch_lowering),
                json_usize_field("content_bytes", capability.content_bytes),
                json_string_field("content_hash", &capability.content_hash),
                json_string_array_field("validation_contracts", &capability.validation_contracts),
                json_string_array_field("issues", &capability.issues),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_link_inputs_json(inputs: &[NsldLinkInputDiagnostic]) -> String {
    inputs
        .iter()
        .map(|input| {
            let fields = vec![
                json_usize_field("order_index", input.order_index),
                json_string_field("input_id", &input.input_id),
                json_string_field("input_kind", &input.input_kind),
                json_string_field("domain_family", &input.domain_family),
                json_string_field("package_id", &input.package_id),
                json_string_field("path", &input.path),
                json_string_field("native_ir", &input.native_ir),
                json_string_field("dispatch_lowering", &input.dispatch_lowering),
                json_usize_field("contract_count", input.contract_count),
                json_usize_field("content_bytes", input.content_bytes),
                json_string_field("content_hash", &input.content_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_link_units_json(units: &[NsldLinkUnitDiagnostic]) -> String {
    units
        .iter()
        .map(|unit| {
            let fields = vec![
                json_usize_field("order_index", unit.order_index),
                json_string_field("unit_id", &unit.unit_id),
                json_string_field("unit_kind", &unit.unit_kind),
                json_string_field("domain_family", &unit.domain_family),
                json_string_field("package_id", &unit.package_id),
                json_string_field("backend_family", &unit.backend_family),
                json_string_field("lowering_target", &unit.lowering_target),
                json_string_field("packaging_role", &unit.packaging_role),
                json_string_array_field("link_input_ids", &unit.link_input_ids),
                json_usize_field("clock_edge_count", unit.clock_edge_count),
                json_usize_field("data_segment_count", unit.data_segment_count),
                json_bool_field("requires_host_wrapper", unit.requires_host_wrapper),
                json_string_field("deterministic_order_key", &unit.deterministic_order_key),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_assemble_sections_json(sections: &[NsldAssembleSectionDiagnostic]) -> String {
    sections
        .iter()
        .map(|section| {
            let fields = vec![
                json_usize_field("order_index", section.order_index),
                json_string_field("section_id", &section.section_id),
                json_string_field("section_kind", &section.section_kind),
                json_string_field("source_path", &section.source_path),
                json_string_field("source_hash", &section.source_hash),
                json_bool_field("required", section.required),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_container_sections_json(sections: &[NsldContainerSectionEntry]) -> String {
    sections
        .iter()
        .map(|section| {
            let fields = vec![
                json_usize_field("order_index", section.order_index),
                json_string_field("section_id", &section.section_id),
                json_string_field("section_kind", &section.section_kind),
                json_string_field("source_path", &section.source_path),
                json_string_field("source_hash", &section.source_hash),
                json_string_field("payload_hash", &section.payload_hash),
                json_bool_field("required", section.required),
                json_usize_field("offset", section.offset),
                json_usize_field("size_bytes", section.size_bytes),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_clock_edges_json(edges: &[NsldClockEdgeDiagnostic]) -> String {
    edges
        .iter()
        .map(|edge| {
            let fields = vec![
                json_usize_field("index", edge.index),
                json_string_field("from", &edge.from),
                json_string_field("to", &edge.to),
                json_string_field("relation", &edge.relation),
                json_string_field("source", &edge.source),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn nsld_data_segments_json(segments: &[NsldDataSegmentDiagnostic]) -> String {
    segments
        .iter()
        .map(|segment| {
            let fields = vec![
                json_usize_field("index", segment.index),
                json_string_field("segment_id", &segment.segment_id),
                json_string_field("domain_family", &segment.domain_family),
                json_string_field("owner_package", &segment.owner_package),
                json_string_field("order_key", &segment.order_key),
                json_string_field("access_phase", &segment.access_phase),
                json_string_field("source_path", &segment.source_path),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{name}\":{value}")
}

fn json_optional_bool_field(name: &str, value: Option<bool>) -> String {
    match value {
        Some(value) => json_bool_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_string_field(name: &str, value: &str) -> String {
    format!("\"{name}\":\"{}\"", json_escape(value))
}

fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{name}\":{value}")
}

fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => json_usize_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => json_string_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_string_array_field(name: &str, values: &[String]) -> String {
    let body = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{name}\":[{body}]")
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn optional_usize_text(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "missing".to_owned())
}

fn optional_bool_text(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "absent".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        fnv1a64_hex, nsld_assemble_plan_report, nsld_link_bundle_report,
        nsld_link_input_diagnostics, nsld_link_input_table_hash, nsld_link_unit_report,
        nsld_link_unit_table_hash, nsld_prepare_report, nsld_sidecar_capability_diagnostics,
        nsld_verify_assemble_plan_report, nsld_verify_container_plan_report,
        nsld_verify_container_report, nsld_verify_link_bundle_report,
        nsld_verify_link_inputs_report, nsld_verify_link_units_report,
        nsld_verify_section_manifest_report, parse_args, render_nsld_assemble_plan_toml,
        render_nsld_link_bundle_toml, render_nsld_link_input_table_toml,
        render_nsld_link_unit_table_toml, Command,
    };
    use nuisc::linker::{
        ArtifactLoweringAlignmentSummary, LinkPlan, LinkPlanArtifact, LinkPlanClockProtocol,
        LinkPlanCpuTarget, LinkPlanEnvelope, LinkPlanFinalStage, LinkPlanHeteroCalculate,
        LinkPlanLifecycle,
    };
    use std::{
        env, fs,
        path::{Path, PathBuf},
    };

    #[test]
    fn parses_status_by_default() {
        assert_eq!(
            parse_args(Vec::<String>::new().into_iter()),
            Ok(Command::Status)
        );
    }

    #[test]
    fn parses_plan_input_and_json_flag() {
        let command =
            parse_args(vec!["plan".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter());
        assert_eq!(
            command,
            Ok(Command::Plan {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_check_input_and_json_flag() {
        let command = parse_args(
            vec![
                "check".to_owned(),
                "nuis.build.manifest.toml".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Check {
                input: PathBuf::from("nuis.build.manifest.toml"),
                json: true
            })
        );
    }

    #[test]
    fn parses_closure_input_and_json_flag() {
        let command = parse_args(
            vec!["closure".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Closure {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_prepare_input_and_json_flag() {
        let command = parse_args(
            vec!["prepare".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Prepare {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_assemble_plan_input_and_json_flag() {
        let command = parse_args(
            vec![
                "assemble-plan".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::AssemblePlan {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_emit_assemble_plan_input_and_json_flag() {
        let command = parse_args(
            vec![
                "emit-assemble-plan".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::EmitAssemblePlan {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_verify_assemble_plan_input_and_json_flag() {
        let command = parse_args(
            vec![
                "verify-assemble-plan".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::VerifyAssemblePlan {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_section_manifest_input_and_json_flag() {
        let command = parse_args(
            vec![
                "section-manifest".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::SectionManifest {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_emit_section_manifest_input_and_json_flag() {
        let command = parse_args(
            vec![
                "emit-section-manifest".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::EmitSectionManifest {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_verify_section_manifest_input_and_json_flag() {
        let command = parse_args(
            vec![
                "verify-section-manifest".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::VerifySectionManifest {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_container_plan_input_and_json_flag() {
        let command = parse_args(
            vec![
                "container-plan".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::ContainerPlan {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_emit_container_plan_input_and_json_flag() {
        let command = parse_args(
            vec![
                "emit-container-plan".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::EmitContainerPlan {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_verify_container_plan_input_and_json_flag() {
        let command = parse_args(
            vec![
                "verify-container-plan".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::VerifyContainerPlan {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_container_input_and_json_flag() {
        let command = parse_args(
            vec![
                "container".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Container {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_emit_container_input_and_json_flag() {
        let command = parse_args(
            vec![
                "emit-container".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::EmitContainer {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_verify_container_input_and_json_flag() {
        let command = parse_args(
            vec![
                "verify-container".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::VerifyContainer {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_bundle_input_and_json_flag() {
        let command = parse_args(
            vec!["bundle".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Bundle {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_emit_bundle_input_and_json_flag() {
        let command = parse_args(
            vec![
                "emit-bundle".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::EmitBundle {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_verify_bundle_input_and_json_flag() {
        let command = parse_args(
            vec![
                "verify-bundle".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::VerifyBundle {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_units_input_and_json_flag() {
        let command =
            parse_args(vec!["units".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter());
        assert_eq!(
            command,
            Ok(Command::Units {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_emit_units_input_and_json_flag() {
        let command = parse_args(
            vec![
                "emit-units".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::EmitUnits {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_verify_units_input_and_json_flag() {
        let command = parse_args(
            vec![
                "verify-units".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::VerifyUnits {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_inputs_input_and_json_flag() {
        let command = parse_args(
            vec!["inputs".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Inputs {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn parses_verify_inputs_input_and_json_flag() {
        let command = parse_args(
            vec![
                "verify-inputs".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::VerifyInputs {
                input: PathBuf::from("out"),
                json: true
            })
        );
    }

    #[test]
    fn sidecar_capability_check_skips_hetero_domains_without_ir_sidecars() {
        let path = env::temp_dir().join(format!("nsld-sidecar-cap-{}.toml", std::process::id()));
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.data".to_owned(),
            domain_family: "data".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: None,
            vendor: None,
            device_class: None,
            selected_lowering_target: None,
            contract_family: "nustar.data".to_owned(),
            packaging_role: "domain-sidecar".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });

        let diagnostics = nsld_sidecar_capability_diagnostics(&plan);
        fs::remove_file(path).unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].domain_family, "shader");
        assert_eq!(diagnostics[0].content_bytes, sidecar_source.len());
        assert_eq!(
            diagnostics[0].content_hash,
            fnv1a64_hex(sidecar_source.as_bytes())
        );
        assert!(diagnostics[0].valid);
        let link_inputs = nsld_link_input_diagnostics(&diagnostics);
        assert_eq!(link_inputs.len(), 1);
        assert_eq!(link_inputs[0].order_index, 0);
        assert_eq!(link_inputs[0].input_id, "li0000.shader.official.shader");
        assert_eq!(link_inputs[0].input_kind, "lowering-ir-sidecar");
        assert_eq!(link_inputs[0].native_ir, "msl2.4");
        assert_eq!(
            link_inputs[0].dispatch_lowering,
            "command-encoder-draw-dispatch"
        );
        assert_eq!(link_inputs[0].content_bytes, sidecar_source.len());
        assert_eq!(
            link_inputs[0].content_hash,
            fnv1a64_hex(sidecar_source.as_bytes())
        );
        let expected_material = format!(
            "0\tli0000.shader.official.shader\tlowering-ir-sidecar\tshader\tofficial.shader\tmsl2.4\tcommand-encoder-draw-dispatch\t1\t{}\t{}\n",
            sidecar_source.len(),
            fnv1a64_hex(sidecar_source.as_bytes())
        );
        assert_eq!(
            nsld_link_input_table_hash(&link_inputs),
            fnv1a64_hex(expected_material.as_bytes())
        );
        let table = render_nsld_link_input_table_toml(
            &link_inputs,
            link_inputs
                .iter()
                .map(|input| input.content_bytes)
                .sum::<usize>(),
            &nsld_link_input_table_hash(&link_inputs),
        );
        assert!(table.contains("schema = \"nuis-nsld-link-input-table-v1\""));
        assert!(table.contains("schema_version = 1"));
        assert!(table.contains("table_kind = \"lowering-sidecar-link-inputs\""));
        assert!(table.contains("producer = \"nsld\""));
        assert!(table.contains("producer_phase = \"alpha-0.6.0\""));
        assert!(table.contains("link_input_count = 1"));
        assert!(table.contains("input_id = \"li0000.shader.official.shader\""));
        assert!(table.contains("native_ir = \"msl2.4\""));
        assert!(table.contains("content_hash = \""));
    }

    fn empty_link_plan() -> LinkPlan {
        LinkPlan {
            schema: "nuis-link-plan-v1".to_owned(),
            input: "in".to_owned(),
            output_dir: "out".to_owned(),
            packaging_mode: "executable".to_owned(),
            cpu_target: LinkPlanCpuTarget {
                abi: "nuis".to_owned(),
                machine_arch: "arm64".to_owned(),
                machine_os: "macos".to_owned(),
                object_format: "mach-o".to_owned(),
                calling_abi: "aapcs64".to_owned(),
                clang_target: "arm64-apple-macos".to_owned(),
                cross_compile: false,
            },
            lifecycle: LinkPlanLifecycle {
                bootstrap_entry: "main".to_owned(),
                tick_policy: "single".to_owned(),
                shutdown_policy: "return".to_owned(),
                yalivia_rpc: "none".to_owned(),
                hook_surface: Vec::new(),
                export_surface: Vec::new(),
                runtime_capability_flags: Vec::new(),
            },
            envelope: LinkPlanEnvelope {
                schema: "nuis-artifact-envelope-v1".to_owned(),
                package_count: 0,
                contract_families: Vec::new(),
                domain_families: Vec::new(),
                function_kind: "function".to_owned(),
                graph_kind: "static".to_owned(),
                default_time_mode: "logical".to_owned(),
            },
            compiled_artifact: LinkPlanArtifact {
                path: "out/nuis.compiled.artifact".to_owned(),
                binary_name: "demo".to_owned(),
                binary_path: "out/demo".to_owned(),
                binary_bytes: 0,
                build_manifest_bytes: 0,
                container_kind: Some("compiled-artifact-section-table-v2".to_owned()),
                container_version: Some(2),
                section_count: Some(0),
                section_names: Vec::new(),
                section_table_valid: Some(true),
                lowering_unit_count: Some(0),
                lowering_domain_families: Vec::new(),
                lowering_targets: Vec::new(),
                lowering_units: Vec::new(),
            },
            bridge_registry_path: None,
            host_bridge_plan_index_path: None,
            lowering_plan_index_path: None,
            domain_units: Vec::new(),
            artifact_lowering_alignment: ArtifactLoweringAlignmentSummary {
                checked: 0,
                mismatches: 0,
                consistent: true,
                checks: Vec::new(),
            },
            clock_protocol: LinkPlanClockProtocol {
                schema: "nuis-clock-protocol-v1".to_owned(),
                mode: "static".to_owned(),
                source: "test".to_owned(),
                default_time_mode: "logical".to_owned(),
                lifecycle_tick_policy: "single".to_owned(),
                domains: Vec::new(),
                edges: Vec::new(),
                validation: nuisc::linker::LinkPlanClockValidationSummary {
                    checked: 0,
                    valid: true,
                    issues: Vec::new(),
                },
            },
            hetero_calculate: LinkPlanHeteroCalculate {
                schema: "nuis-hetero-calculate-link-v1".to_owned(),
                mode: "static".to_owned(),
                static_link: true,
                lifecycle_driven: true,
                time_order_model: "partial-order".to_owned(),
                data_order_model: "deterministic".to_owned(),
                c_world_policy: "wrapped".to_owned(),
                nodes: Vec::new(),
                data_segments: Vec::new(),
                validation: nuisc::linker::LinkPlanHeteroValidationSummary {
                    checked: 0,
                    valid: true,
                    issues: Vec::new(),
                },
            },
            final_stage: LinkPlanFinalStage {
                kind: "native-executable".to_owned(),
                driver: "clang".to_owned(),
                link_mode: "host-toolchain-finalize".to_owned(),
                output_path: "out/demo".to_owned(),
                inputs: Vec::new(),
                notes: Vec::new(),
            },
        }
    }

    #[test]
    fn verify_link_inputs_accepts_matching_emitted_table() {
        let dir = env::temp_dir().join(format!("nsld-link-input-verify-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let sidecar_path = dir.join("shader.sidecar.toml");
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&sidecar_path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });
        let diagnostics = nsld_sidecar_capability_diagnostics(&plan);
        let inputs = nsld_link_input_diagnostics(&diagnostics);
        let total_bytes = inputs
            .iter()
            .map(|input| input.content_bytes)
            .sum::<usize>();
        let table_hash = nsld_link_input_table_hash(&inputs);
        fs::write(
            dir.join("nuis.nsld.link-inputs.toml"),
            render_nsld_link_input_table_toml(&inputs, total_bytes, &table_hash),
        )
        .unwrap();

        let report = nsld_verify_link_inputs_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(report.valid);
        assert!(report.issues.is_empty());
        assert_eq!(report.actual_link_input_count, Some(1));
        assert_eq!(
            report.actual_link_input_total_bytes,
            Some(sidecar_source.len())
        );
        assert_eq!(report.actual_link_input_table_hash, Some(table_hash));
    }

    #[test]
    fn link_unit_report_attaches_registered_sidecar_inputs() {
        let dir = env::temp_dir().join(format!("nsld-link-unit-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let sidecar_path = dir.join("shader.sidecar.toml");
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&sidecar_path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });

        let report = nsld_link_unit_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert_eq!(report.unit_count, 1);
        assert_eq!(report.hetero_unit_count, 1);
        assert_eq!(report.link_input_count, 1);
        assert_eq!(report.units[0].unit_id, "lu0000.shader.official.shader");
        assert_eq!(report.units[0].unit_kind, "hetero-domain");
        assert_eq!(report.units[0].backend_family, "metal");
        assert_eq!(report.units[0].link_input_ids.len(), 1);
        assert_eq!(
            report.units[0].link_input_ids[0],
            "li0000.shader.official.shader"
        );
        assert_eq!(
            report.unit_table_hash,
            nsld_link_unit_table_hash(&report.units)
        );
    }

    #[test]
    fn verify_link_units_accepts_matching_emitted_table() {
        let dir = env::temp_dir().join(format!("nsld-link-unit-verify-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let sidecar_path = dir.join("shader.sidecar.toml");
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&sidecar_path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });
        let unit_report = nsld_link_unit_report(Path::new("manifest.toml"), &plan);
        fs::write(
            dir.join("nuis.nsld.link-units.toml"),
            render_nsld_link_unit_table_toml(&unit_report),
        )
        .unwrap();

        let report = nsld_verify_link_units_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(report.valid);
        assert!(report.issues.is_empty());
        assert_eq!(report.actual_unit_count, Some(1));
        assert_eq!(report.actual_hetero_unit_count, Some(1));
        assert_eq!(report.actual_link_input_count, Some(1));
        assert_eq!(
            report.actual_unit_table_hash,
            Some(unit_report.unit_table_hash)
        );
    }

    #[test]
    fn verify_link_bundle_accepts_matching_emitted_bundle() {
        let dir = env::temp_dir().join(format!("nsld-link-bundle-verify-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let sidecar_path = dir.join("shader.sidecar.toml");
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&sidecar_path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });
        let bundle_report = nsld_link_bundle_report(Path::new("manifest.toml"), &plan);
        fs::write(
            dir.join("nuis.nsld.link-bundle.toml"),
            render_nsld_link_bundle_toml(&bundle_report),
        )
        .unwrap();

        let report = nsld_verify_link_bundle_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(report.valid);
        assert!(report.issues.is_empty());
        assert_eq!(report.actual_bundle_id, Some(bundle_report.bundle_id));
        assert_eq!(report.actual_bundle_hash, Some(bundle_report.bundle_hash));
    }

    #[test]
    fn prepare_emits_and_verifies_all_linker_artifacts() {
        let dir = env::temp_dir().join(format!("nsld-prepare-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let sidecar_path = dir.join("shader.sidecar.toml");
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&sidecar_path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });

        let report = nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();

        assert!(report.valid);
        assert!(report.issues.is_empty());
        assert!(Path::new(&report.link_input_table_path).exists());
        assert!(Path::new(&report.link_unit_table_path).exists());
        assert!(Path::new(&report.link_bundle_path).exists());
        assert!(Path::new(&report.assemble_plan_path).exists());
        assert!(Path::new(&report.section_manifest_path).exists());
        assert!(Path::new(&report.container_plan_path).exists());
        assert!(Path::new(&report.container_path).exists());
        assert_eq!(report.link_input_count, 1);
        assert_eq!(report.unit_count, 1);
        assert!(report.bundle_ready);
        assert_ne!(report.assemble_plan_hash, "missing");
        assert_ne!(report.section_table_hash, "missing");
        assert_ne!(report.container_layout_hash, "missing");
        assert_ne!(report.container_hash, "missing");
        assert!(report.payload_size_bytes > 0);
        assert_ne!(report.payload_hash, "missing");

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn assemble_plan_lists_prepared_linker_sections() {
        let dir = env::temp_dir().join(format!("nsld-assemble-plan-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let sidecar_path = dir.join("shader.sidecar.toml");
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&sidecar_path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });
        nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();

        let report = nsld_assemble_plan_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(report.ready);
        assert!(report.blockers.is_empty());
        assert_eq!(report.section_count, 5);
        assert_eq!(report.sections[0].section_kind, "compiled-artifact");
        assert_eq!(report.sections[1].section_kind, "nsld-link-input-table");
        assert_eq!(report.sections[2].section_kind, "nsld-link-unit-table");
        assert_eq!(report.sections[3].section_kind, "nsld-link-bundle");
        assert_eq!(report.sections[4].section_kind, "lowering-sidecar-input");
        assert!(report
            .sections
            .iter()
            .all(|section| section.source_hash != "missing"));
    }

    #[test]
    fn verify_assemble_plan_accepts_matching_emitted_plan() {
        let dir = env::temp_dir().join(format!("nsld-assemble-plan-verify-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let sidecar_path = dir.join("shader.sidecar.toml");
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&sidecar_path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });
        nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
        let assemble_plan = nsld_assemble_plan_report(Path::new("manifest.toml"), &plan);
        fs::write(
            dir.join("nuis.nsld.assemble-plan.toml"),
            render_nsld_assemble_plan_toml(&assemble_plan),
        )
        .unwrap();

        let report = nsld_verify_assemble_plan_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(report.valid);
        assert!(report.issues.is_empty());
        assert_eq!(
            report.actual_assemble_plan_hash,
            Some(assemble_plan.assemble_plan_hash)
        );
        assert_eq!(
            report.actual_section_count,
            Some(assemble_plan.section_count)
        );
    }

    #[test]
    fn verify_section_manifest_accepts_matching_emitted_manifest() {
        let dir = env::temp_dir().join(format!(
            "nsld-section-manifest-verify-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let sidecar_path = dir.join("shader.sidecar.toml");
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&sidecar_path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });
        let prepare = nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
        let source = fs::read_to_string(&prepare.section_manifest_path).unwrap();
        fs::write(dir.join("nuis.nsld.section-manifest.toml"), source).unwrap();

        let report = nsld_verify_section_manifest_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(report.valid);
        assert!(report.issues.is_empty());
        assert_eq!(report.actual_section_count, Some(5));
        assert_eq!(
            report.actual_section_table_hash,
            Some(prepare.section_table_hash)
        );
    }

    #[test]
    fn verify_container_plan_accepts_matching_emitted_plan() {
        let dir =
            env::temp_dir().join(format!("nsld-container-plan-verify-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let sidecar_path = dir.join("shader.sidecar.toml");
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&sidecar_path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });
        let prepare = nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();

        let report = nsld_verify_container_plan_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(report.valid);
        assert!(report.issues.is_empty());
        assert_eq!(report.actual_section_count, Some(5));
        assert_eq!(
            report.actual_container_layout_hash,
            Some(prepare.container_layout_hash)
        );
    }

    #[test]
    fn verify_container_accepts_matching_emitted_container() {
        let dir = env::temp_dir().join(format!("nsld-container-verify-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let sidecar_path = dir.join("shader.sidecar.toml");
        let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
        fs::write(&sidecar_path, sidecar_source).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
            kind: "heterogeneous".to_owned(),
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        });
        let prepare = nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
        let container_source = fs::read_to_string(&prepare.container_path).unwrap();
        let payload_bytes = fs::read(&prepare.container_payload_path).unwrap();

        let report = nsld_verify_container_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert_eq!(payload_bytes.len(), prepare.payload_size_bytes);
        assert_eq!(fnv1a64_hex(&payload_bytes), prepare.payload_hash);
        assert!(container_source.contains("offset = 0"));
        assert!(container_source.contains("size_bytes = 17"));
        assert!(container_source.contains("payload_size_bytes = "));
        assert!(container_source.contains("payload_hash = \"0x"));
        assert!(report.valid);
        assert!(report.issues.is_empty());
        assert_eq!(report.actual_section_count, Some(5));
        assert_eq!(
            report.actual_container_layout_hash,
            Some(prepare.container_layout_hash)
        );
        assert_eq!(report.actual_container_hash, Some(prepare.container_hash));
    }
}
