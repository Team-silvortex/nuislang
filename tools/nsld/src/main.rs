mod cli;
mod container;
mod display;
mod json;
mod link_units;
mod reports;
mod toml;

use cli::{parse_args, resolve_manifest_input, Command};
use container::{
    NsldContainerEmitReport, NsldContainerPlanEmitReport, NsldContainerPlanReport,
    NsldContainerPlanVerifyReport, NsldContainerReport, NsldContainerVerifyReport,
};
use display::*;
use json::*;
use link_units::*;
use reports::*;
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
const NSLD_CONTAINER_MAGIC: &str = "NUISNSLD";
const NSLD_CONTAINER_VERSION: usize = 1;

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
                println!("{}", json::check_report_json(&report));
            } else {
                display::print_check_report(&report);
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
        Command::Inputs { input, json } | Command::EmitInputs { input, json } => {
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
    let expected_container_report =
        container_present.then(|| nsld_container_report(manifest, plan));
    let container_loader_readiness = expected_container_report
        .as_ref()
        .map(|report| report.loader_readiness.clone());
    let container_loader_blockers = expected_container_report
        .as_ref()
        .map(|report| report.loader_blockers.clone())
        .unwrap_or_default();
    let container_metadata_table_hash = expected_container_report
        .as_ref()
        .map(|report| report.metadata_table_hash.clone());
    let container_external_import_count = expected_container_report
        .as_ref()
        .map(|report| report.external_imports.len());
    let container_payload_path =
        PathBuf::from(&plan.output_dir).join("nuis.nsld.container.payload");
    let container_payload_present = container_payload_path.exists();
    let mut container_payload_issues = Vec::new();
    if container_payload_present && !container_present {
        container_payload_issues.push("container payload is present without container".to_owned());
    }
    if container_present && !container_payload_present {
        container_payload_issues
            .push("container payload is missing for present container".to_owned());
    }
    let artifact_chain_issues = nsld_artifact_chain_issues(&[
        ("nuis.nsld.link-inputs.toml", link_input_table_present),
        ("nuis.nsld.link-units.toml", link_unit_table_present),
        ("nuis.nsld.link-bundle.toml", link_bundle_present),
        ("nuis.nsld.assemble-plan.toml", assemble_plan_present),
        ("nuis.nsld.section-manifest.toml", section_manifest_present),
        ("nuis.nsld.container-plan.toml", container_plan_present),
        ("nuis.nsld.container", container_present),
        ("nuis.nsld.container.payload", container_payload_present),
    ]);
    let artifact_chain_valid = artifact_chain_issues.is_empty();
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
    if container_loader_readiness.as_deref() == Some("blocked") {
        issues.push("container loader readiness is blocked".to_owned());
        issues.extend(container_loader_blockers.iter().cloned());
    }
    if !container_payload_issues.is_empty() {
        issues.push("container payload state is inconsistent".to_owned());
        issues.extend(container_payload_issues.iter().cloned());
    }
    if !artifact_chain_valid {
        issues.push("nsld artifact chain is incomplete".to_owned());
        issues.extend(artifact_chain_issues.iter().cloned());
    }

    let checks = 6 + usize::from(link_input_table_present) + usize::from(link_unit_table_present);
    let checks = checks + usize::from(link_bundle_present);
    let checks = checks + usize::from(assemble_plan_present);
    let checks = checks + usize::from(section_manifest_present);
    let checks = checks + usize::from(container_plan_present);
    let checks = checks + usize::from(container_present);
    let checks = checks + usize::from(container_present || container_payload_present);
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
        container_payload_present,
        container_payload_issues,
        container_loader_readiness,
        container_loader_blockers,
        container_metadata_table_hash,
        container_external_import_count,
        artifact_chain_valid,
        artifact_chain_issues,
        final_stage_link_mode: plan.final_stage.link_mode.clone(),
        domains,
        sidecar_capabilities,
        clock_edges,
        data_segments,
        issues,
    }
}

fn nsld_artifact_chain_issues(stages: &[(&str, bool)]) -> Vec<String> {
    let mut first_missing_before_present = None;
    let mut issues = Vec::new();

    for (name, present) in stages {
        if *present {
            if let Some(missing) = first_missing_before_present {
                issues.push(format!(
                    "artifact `{name}` is present but prerequisite `{missing}` is missing"
                ));
            }
        } else if first_missing_before_present.is_none() {
            first_missing_before_present = Some(*name);
        }
    }

    issues
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
        toml::render_link_input_table(
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
        metadata_table_hash: container_file_emit.metadata_table_hash,
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
    let expected = toml::render_link_input_table(
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
    fs::write(&output_path, toml::render_link_unit_table(&report)).map_err(|error| {
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
    let expected = toml::render_link_unit_table(&expected_report);
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
    fs::write(&output_path, toml::render_assemble_plan(&report)).map_err(|error| {
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
    let expected = toml::render_assemble_plan(&expected_report);
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
    fs::write(&output_path, toml::render_section_manifest(&report)).map_err(|error| {
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
    let expected = toml::render_section_manifest(&expected_report);
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
    let container_layout_hash = container::layout_hash(
        NSLD_CONTAINER_MAGIC,
        NSLD_CONTAINER_VERSION,
        section_manifest.section_count,
        &section_manifest.section_table_hash,
        &output_path,
        fnv1a64_hex,
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
    fs::write(&output_path, container::render_container_plan_toml(&report)).map_err(|error| {
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
    let expected = container::render_container_plan_toml(&expected_report);
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

fn nsld_container_report(manifest: &Path, plan: &nuisc::linker::LinkPlan) -> NsldContainerReport {
    let container_plan = nsld_container_plan_report(manifest, plan);
    let sections = container::section_entries(&container_plan.sections, fnv1a64_hex);
    let container_section_table_hash =
        container::container_section_table_hash(&sections, fnv1a64_hex);
    let loader_entry_kind = "lifecycle-bootstrap".to_owned();
    let loader_entry_symbol = plan.lifecycle.bootstrap_entry.clone();
    let loader_entry_section_id = sections
        .iter()
        .find(|section| section.section_kind == "compiled-artifact")
        .map(|section| section.section_id.clone())
        .unwrap_or_else(|| "missing".to_owned());
    let loader_symbols = nsld_container_loader_symbols(
        &loader_entry_kind,
        &loader_entry_symbol,
        &loader_entry_section_id,
        &sections,
    );
    let loader_symbol_table_hash =
        container::loader_symbol_table_hash(&loader_symbols, fnv1a64_hex);
    let relocations = Vec::new();
    let external_imports = nsld_container_external_imports(plan);
    let external_import_table_hash =
        container::external_import_table_hash(&external_imports, fnv1a64_hex);
    let metadata_table_hash = container::metadata_table_hash(
        &container_section_table_hash,
        &loader_symbol_table_hash,
        relocations.len(),
        &external_import_table_hash,
        fnv1a64_hex,
    );
    let loader_blockers =
        nsld_container_loader_blockers(&external_imports, &container_plan.blockers);
    let loader_readiness = if !container_plan.ready || !container_plan.blockers.is_empty() {
        "blocked"
    } else if external_imports
        .iter()
        .any(|external_import| external_import.required)
    {
        "host-assisted"
    } else {
        "self-contained"
    }
    .to_owned();
    let payload_size_bytes = container::payload_size(&sections);
    let payload_hash = container::payload_hash(&sections, fnv1a64_hex);
    let container_hash = container::file_hash(
        &container_plan,
        &sections,
        &loader_entry_kind,
        &loader_entry_symbol,
        &loader_entry_section_id,
        &loader_symbols,
        &relocations,
        &external_imports,
        &loader_readiness,
        &loader_blockers,
        payload_size_bytes,
        &payload_hash,
        fnv1a64_hex,
    );
    NsldContainerReport {
        manifest: manifest.display().to_string(),
        ready: container_plan.ready,
        container_magic: container_plan.container_magic,
        container_version: container_plan.container_version,
        metadata_table_hash,
        container_layout_hash: container_plan.container_layout_hash,
        container_hash,
        loader_readiness,
        loader_blockers,
        loader_entry_kind,
        loader_entry_symbol,
        loader_entry_section_id,
        loader_symbol_table_hash,
        loader_symbols,
        relocations,
        external_import_table_hash,
        external_imports,
        payload_size_bytes,
        payload_hash,
        payload_path: format!("{}.payload", container_plan.output_path),
        output_path: container_plan.output_path,
        section_count: container_plan.section_count,
        container_section_table_hash,
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
    fs::write(&payload_path, container::payload_bytes(&report.sections)).map_err(|error| {
        format!(
            "failed to write nsld container payload `{}`: {error}",
            payload_path.display()
        )
    })?;
    fs::write(&output_path, container::render_container_toml(&report)).map_err(|error| {
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
        metadata_table_hash: report.metadata_table_hash,
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
    let expected = container::render_container_toml(&expected_report);
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
        actual_metadata_table_hash,
        actual_payload_size_bytes,
        actual_payload_hash,
        actual_section_count,
        actual_container_section_table_hash,
        actual_loader_readiness,
        actual_loader_entry_kind,
        actual_loader_entry_symbol,
        actual_loader_entry_section_id,
        actual_loader_symbol_count,
        actual_loader_symbol_id,
        actual_loader_symbol_kind,
        actual_loader_symbol_name,
        actual_loader_symbol_section_id,
        actual_loader_symbol_table_hash,
        actual_relocation_count,
        actual_external_import_count,
        actual_external_import_table_hash,
        actual_external_import_id,
        actual_external_import_kind,
        actual_external_import_name,
        actual_external_import_provider,
        actual_external_import_required,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml_string_value(source, "container_layout_hash"),
            toml_string_value(source, "container_hash"),
            toml_string_value(source, "metadata_table_hash"),
            toml_usize_value(source, "payload_size_bytes"),
            toml_string_value(source, "payload_hash"),
            toml_usize_value(source, "section_count"),
            toml_string_value(source, "container_section_table_hash"),
            toml_string_value(source, "loader_readiness"),
            toml_string_value(source, "loader_entry_kind"),
            toml_string_value(source, "loader_entry_symbol"),
            toml_string_value(source, "loader_entry_section_id"),
            toml_usize_value(source, "loader_symbol_count"),
            toml_first_table_string_value(source, "loader_symbol", "symbol_id"),
            toml_first_table_string_value(source, "loader_symbol", "symbol_kind"),
            toml_first_table_string_value(source, "loader_symbol", "symbol_name"),
            toml_first_table_string_value(source, "loader_symbol", "section_id"),
            toml_string_value(source, "loader_symbol_table_hash"),
            toml_usize_value(source, "relocation_count"),
            toml_usize_value(source, "external_import_count"),
            toml_string_value(source, "external_import_table_hash"),
            toml_first_table_string_value(source, "external_import", "import_id"),
            toml_first_table_string_value(source, "external_import", "import_kind"),
            toml_first_table_string_value(source, "external_import", "import_name"),
            toml_first_table_string_value(source, "external_import", "provider"),
            toml_first_table_bool_value(source, "external_import", "required"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None,
            )
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
        if actual_metadata_table_hash.as_deref()
            != Some(expected_report.metadata_table_hash.as_str())
        {
            issues.push(format!(
                "metadata_table_hash mismatch: expected {}, found {}",
                expected_report.metadata_table_hash,
                actual_metadata_table_hash
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
        if actual_container_section_table_hash.as_deref()
            != Some(expected_report.container_section_table_hash.as_str())
        {
            issues.push(format!(
                "container_section_table_hash mismatch: expected {}, found {}",
                expected_report.container_section_table_hash,
                actual_container_section_table_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_loader_readiness.as_deref() != Some(expected_report.loader_readiness.as_str()) {
            issues.push(format!(
                "loader_readiness mismatch: expected {}, found {}",
                expected_report.loader_readiness,
                actual_loader_readiness
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_loader_entry_kind.as_deref() != Some(expected_report.loader_entry_kind.as_str()) {
            issues.push(format!(
                "loader_entry_kind mismatch: expected {}, found {}",
                expected_report.loader_entry_kind,
                actual_loader_entry_kind
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_loader_entry_symbol.as_deref()
            != Some(expected_report.loader_entry_symbol.as_str())
        {
            issues.push(format!(
                "loader_entry_symbol mismatch: expected {}, found {}",
                expected_report.loader_entry_symbol,
                actual_loader_entry_symbol
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_loader_entry_section_id.as_deref()
            != Some(expected_report.loader_entry_section_id.as_str())
        {
            issues.push(format!(
                "loader_entry_section_id mismatch: expected {}, found {}",
                expected_report.loader_entry_section_id,
                actual_loader_entry_section_id
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_loader_symbol_count != Some(expected_report.loader_symbols.len()) {
            issues.push(format!(
                "loader_symbol_count mismatch: expected {}, found {}",
                expected_report.loader_symbols.len(),
                actual_loader_symbol_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if let Some(expected_symbol) = expected_report.loader_symbols.first() {
            if actual_loader_symbol_id.as_deref() != Some(expected_symbol.symbol_id.as_str()) {
                issues.push(format!(
                    "loader_symbol_id mismatch: expected {}, found {}",
                    expected_symbol.symbol_id,
                    actual_loader_symbol_id
                        .clone()
                        .unwrap_or_else(|| "missing".to_owned())
                ));
            }
            if actual_loader_symbol_kind.as_deref() != Some(expected_symbol.symbol_kind.as_str()) {
                issues.push(format!(
                    "loader_symbol_kind mismatch: expected {}, found {}",
                    expected_symbol.symbol_kind,
                    actual_loader_symbol_kind
                        .clone()
                        .unwrap_or_else(|| "missing".to_owned())
                ));
            }
            if actual_loader_symbol_name.as_deref() != Some(expected_symbol.symbol_name.as_str()) {
                issues.push(format!(
                    "loader_symbol_name mismatch: expected {}, found {}",
                    expected_symbol.symbol_name,
                    actual_loader_symbol_name
                        .clone()
                        .unwrap_or_else(|| "missing".to_owned())
                ));
            }
            if actual_loader_symbol_section_id.as_deref()
                != Some(expected_symbol.section_id.as_str())
            {
                issues.push(format!(
                    "loader_symbol_section_id mismatch: expected {}, found {}",
                    expected_symbol.section_id,
                    actual_loader_symbol_section_id
                        .clone()
                        .unwrap_or_else(|| "missing".to_owned())
                ));
            }
        }
        if actual_loader_symbol_table_hash.as_deref()
            != Some(expected_report.loader_symbol_table_hash.as_str())
        {
            issues.push(format!(
                "loader_symbol_table_hash mismatch: expected {}, found {}",
                expected_report.loader_symbol_table_hash,
                actual_loader_symbol_table_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_relocation_count != Some(expected_report.relocations.len()) {
            issues.push(format!(
                "relocation_count mismatch: expected {}, found {}",
                expected_report.relocations.len(),
                actual_relocation_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_external_import_count != Some(expected_report.external_imports.len()) {
            issues.push(format!(
                "external_import_count mismatch: expected {}, found {}",
                expected_report.external_imports.len(),
                actual_external_import_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_external_import_table_hash.as_deref()
            != Some(expected_report.external_import_table_hash.as_str())
        {
            issues.push(format!(
                "external_import_table_hash mismatch: expected {}, found {}",
                expected_report.external_import_table_hash,
                actual_external_import_table_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if let Some(expected_import) = expected_report.external_imports.first() {
            if actual_external_import_id.as_deref() != Some(expected_import.import_id.as_str()) {
                issues.push(format!(
                    "external_import_id mismatch: expected {}, found {}",
                    expected_import.import_id,
                    actual_external_import_id
                        .clone()
                        .unwrap_or_else(|| "missing".to_owned())
                ));
            }
            if actual_external_import_kind.as_deref() != Some(expected_import.import_kind.as_str())
            {
                issues.push(format!(
                    "external_import_kind mismatch: expected {}, found {}",
                    expected_import.import_kind,
                    actual_external_import_kind
                        .clone()
                        .unwrap_or_else(|| "missing".to_owned())
                ));
            }
            if actual_external_import_name.as_deref() != Some(expected_import.import_name.as_str())
            {
                issues.push(format!(
                    "external_import_name mismatch: expected {}, found {}",
                    expected_import.import_name,
                    actual_external_import_name
                        .clone()
                        .unwrap_or_else(|| "missing".to_owned())
                ));
            }
            if actual_external_import_provider.as_deref() != Some(expected_import.provider.as_str())
            {
                issues.push(format!(
                    "external_import_provider mismatch: expected {}, found {}",
                    expected_import.provider,
                    actual_external_import_provider
                        .clone()
                        .unwrap_or_else(|| "missing".to_owned())
                ));
            }
            if actual_external_import_required != Some(expected_import.required) {
                issues.push(format!(
                    "external_import_required mismatch: expected {}, found {}",
                    expected_import.required,
                    actual_external_import_required
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "missing".to_owned())
                ));
            }
        }
    }
    let mut section_range_issues = Vec::new();
    let (actual_payload_file_size, actual_payload_file_hash) = match fs::read(&payload_path)
        .map_err(|error| {
            format!(
                "missing_or_unreadable_container_payload `{}`: {error}",
                payload_path.display()
            )
        }) {
        Ok(bytes) => {
            section_range_issues =
                container::payload_range_issues(&expected_report, &bytes, fnv1a64_hex);
            (Some(bytes.len()), Some(fnv1a64_hex(&bytes)))
        }
        Err(error) => {
            issues.push(error);
            (None, None)
        }
    };
    issues.extend(section_range_issues.iter().cloned());
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
        expected_metadata_table_hash: expected_report.metadata_table_hash,
        expected_payload_size_bytes: expected_report.payload_size_bytes,
        expected_payload_hash: expected_report.payload_hash,
        expected_payload_path: expected_report.payload_path,
        expected_section_count: expected_report.section_count,
        expected_container_section_table_hash: expected_report.container_section_table_hash,
        expected_loader_readiness: expected_report.loader_readiness,
        expected_loader_entry_kind: expected_report.loader_entry_kind,
        expected_loader_entry_symbol: expected_report.loader_entry_symbol,
        expected_loader_entry_section_id: expected_report.loader_entry_section_id,
        expected_loader_symbol_count: expected_report.loader_symbols.len(),
        expected_loader_symbol_id: expected_report
            .loader_symbols
            .first()
            .map(|symbol| symbol.symbol_id.clone())
            .unwrap_or_default(),
        expected_loader_symbol_kind: expected_report
            .loader_symbols
            .first()
            .map(|symbol| symbol.symbol_kind.clone())
            .unwrap_or_default(),
        expected_loader_symbol_name: expected_report
            .loader_symbols
            .first()
            .map(|symbol| symbol.symbol_name.clone())
            .unwrap_or_default(),
        expected_loader_symbol_section_id: expected_report
            .loader_symbols
            .first()
            .map(|symbol| symbol.section_id.clone())
            .unwrap_or_default(),
        expected_loader_symbol_table_hash: expected_report.loader_symbol_table_hash,
        expected_relocation_count: expected_report.relocations.len(),
        expected_external_import_count: expected_report.external_imports.len(),
        expected_external_import_table_hash: expected_report.external_import_table_hash,
        expected_external_import_id: expected_report
            .external_imports
            .first()
            .map(|external_import| external_import.import_id.clone())
            .unwrap_or_default(),
        expected_external_import_kind: expected_report
            .external_imports
            .first()
            .map(|external_import| external_import.import_kind.clone())
            .unwrap_or_default(),
        expected_external_import_name: expected_report
            .external_imports
            .first()
            .map(|external_import| external_import.import_name.clone())
            .unwrap_or_default(),
        expected_external_import_provider: expected_report
            .external_imports
            .first()
            .map(|external_import| external_import.provider.clone())
            .unwrap_or_default(),
        expected_external_import_required: expected_report
            .external_imports
            .first()
            .map(|external_import| external_import.required)
            .unwrap_or(false),
        actual_container_layout_hash,
        actual_container_hash,
        actual_metadata_table_hash,
        actual_payload_size_bytes,
        actual_payload_hash,
        actual_section_count,
        actual_container_section_table_hash,
        actual_loader_readiness,
        actual_loader_entry_kind,
        actual_loader_entry_symbol,
        actual_loader_entry_section_id,
        actual_loader_symbol_count,
        actual_loader_symbol_id,
        actual_loader_symbol_kind,
        actual_loader_symbol_name,
        actual_loader_symbol_section_id,
        actual_loader_symbol_table_hash,
        actual_relocation_count,
        actual_external_import_count,
        actual_external_import_table_hash,
        actual_external_import_id,
        actual_external_import_kind,
        actual_external_import_name,
        actual_external_import_provider,
        actual_external_import_required,
        section_range_issues,
        issues,
    }
}

fn nsld_container_external_imports(
    plan: &nuisc::linker::LinkPlan,
) -> Vec<container::NsldContainerExternalImport> {
    let mut imports = Vec::new();
    let mut push_import = |import_kind: &str, import_name: String, provider: &str| {
        let index = imports.len();
        imports.push(container::NsldContainerExternalImport {
            import_id: format!("imp{index:04}.{import_kind}"),
            import_kind: import_kind.to_owned(),
            import_name,
            provider: provider.to_owned(),
            required: true,
        });
    };

    if matches!(
        plan.final_stage.link_mode.as_str(),
        "host-toolchain-finalize" | "bundle-packaging"
    ) {
        push_import(
            "final-stage-driver",
            plan.final_stage.driver.clone(),
            "host-toolchain",
        );
    }
    if !plan.cpu_target.clang_target.is_empty() {
        push_import(
            "clang-target",
            plan.cpu_target.clang_target.clone(),
            "host-toolchain",
        );
    }
    if plan.final_stage.link_mode == "bundle-packaging" {
        push_import(
            "host-launcher-wrapper",
            "host-launcher-wrapper".to_owned(),
            "host-toolchain",
        );
    }
    if !plan.hetero_calculate.c_world_policy.is_empty()
        && plan.hetero_calculate.c_world_policy != "none"
    {
        push_import(
            "c-world-policy",
            plan.hetero_calculate.c_world_policy.clone(),
            "c-world-wrapper",
        );
    }

    imports
}

fn nsld_container_loader_blockers(
    external_imports: &[container::NsldContainerExternalImport],
    container_blockers: &[String],
) -> Vec<String> {
    let mut blockers = container_blockers.to_vec();
    blockers.extend(
        external_imports
            .iter()
            .filter(|external_import| external_import.required)
            .map(|external_import| {
                format!(
                    "external-import:{}:{}",
                    external_import.import_kind, external_import.import_name
                )
            }),
    );
    blockers
}

fn nsld_container_loader_symbols(
    loader_entry_kind: &str,
    loader_entry_symbol: &str,
    loader_entry_section_id: &str,
    sections: &[container::NsldContainerSectionEntry],
) -> Vec<container::NsldContainerLoaderSymbol> {
    sections
        .iter()
        .find(|section| section.section_id == loader_entry_section_id)
        .map(|section| {
            vec![container::NsldContainerLoaderSymbol {
                symbol_id: "sym0000.loader-entry".to_owned(),
                symbol_kind: loader_entry_kind.to_owned(),
                symbol_name: loader_entry_symbol.to_owned(),
                section_id: section.section_id.clone(),
                offset: section.offset,
                size_bytes: section.size_bytes,
                payload_hash: section.payload_hash.clone(),
            }]
        })
        .unwrap_or_default()
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
    fs::write(&output_path, toml::render_link_bundle(&report)).map_err(|error| {
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
    let expected = toml::render_link_bundle(&expected_report);
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
        toml_decode_string_value(value.trim())
    })
}

fn toml_first_table_string_value(source: &str, table: &str, key: &str) -> Option<String> {
    toml_first_table_value(source, table, key).and_then(toml_decode_string_value)
}

fn toml_first_table_bool_value(source: &str, table: &str, key: &str) -> Option<bool> {
    toml_first_table_value(source, table, key).and_then(|value| value.parse::<bool>().ok())
}

fn toml_first_table_value<'a>(source: &'a str, table: &str, key: &str) -> Option<&'a str> {
    let header = format!("[[{table}]]");
    let mut in_target_table = false;

    for raw in source.lines() {
        let line = raw.trim();
        if line.starts_with("[[") && line.ends_with("]]") {
            if in_target_table {
                return None;
            }
            in_target_table = line == header;
            continue;
        }
        if !in_target_table {
            continue;
        }
        if let Some((found_key, value)) = line.split_once('=') {
            if found_key.trim() == key {
                return Some(value.trim());
            }
        }
    }

    None
}

fn toml_decode_string_value(value: &str) -> Option<String> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(|value| {
            value
                .replace("\\n", "\n")
                .replace("\\\"", "\"")
                .replace("\\\\", "\\")
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

#[cfg(test)]
mod tests {
    use super::{
        fnv1a64_hex, nsld_artifact_chain_issues, nsld_assemble_plan_report, nsld_check_report,
        nsld_emit_container_report, nsld_link_bundle_report, nsld_link_input_diagnostics,
        nsld_link_input_table_hash, nsld_link_unit_report, nsld_link_unit_table_hash,
        nsld_prepare_report, nsld_sidecar_capability_diagnostics, nsld_verify_assemble_plan_report,
        nsld_verify_container_plan_report, nsld_verify_container_report,
        nsld_verify_link_bundle_report, nsld_verify_link_inputs_report,
        nsld_verify_link_units_report, nsld_verify_section_manifest_report, parse_args, toml,
        toml_first_table_bool_value, toml_first_table_string_value, Command,
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
    fn parses_emit_inputs_input_and_json_flag() {
        let command = parse_args(
            vec![
                "emit-inputs".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::EmitInputs {
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
    fn scoped_toml_helpers_read_the_first_matching_table_only() {
        let source = r#"
[[loader_symbol]]
symbol_id = "sym0000.loader-entry"
section_id = "sec0000.compiled-artifact"

[[external_import]]
import_id = "imp0000.final-stage-driver"
required = true

[[section]]
section_id = "sec9999.section-table"

[[external_import]]
import_id = "imp0001.clang-target"
required = false
"#;

        assert_eq!(
            toml_first_table_string_value(source, "loader_symbol", "section_id").as_deref(),
            Some("sec0000.compiled-artifact")
        );
        assert_eq!(
            toml_first_table_string_value(source, "external_import", "import_id").as_deref(),
            Some("imp0000.final-stage-driver")
        );
        assert_eq!(
            toml_first_table_bool_value(source, "external_import", "required"),
            Some(true)
        );
        assert_eq!(
            toml_first_table_string_value(source, "missing", "section_id"),
            None
        );
    }

    #[test]
    fn artifact_chain_accepts_contiguous_prepared_prefix() {
        let issues = nsld_artifact_chain_issues(&[
            ("inputs", true),
            ("units", true),
            ("bundle", true),
            ("assemble", false),
            ("section", false),
        ]);
        assert!(issues.is_empty());
    }

    #[test]
    fn artifact_chain_rejects_later_artifact_without_prerequisite() {
        let issues = nsld_artifact_chain_issues(&[
            ("inputs", true),
            ("units", false),
            ("bundle", true),
            ("assemble", true),
        ]);
        assert_eq!(
            issues,
            vec![
                "artifact `bundle` is present but prerequisite `units` is missing".to_owned(),
                "artifact `assemble` is present but prerequisite `units` is missing".to_owned(),
            ]
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
        let table = toml::render_link_input_table(
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
            toml::render_link_input_table(&inputs, total_bytes, &table_hash),
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
            toml::render_link_unit_table(&unit_report),
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
            toml::render_link_bundle(&bundle_report),
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
        assert_ne!(report.metadata_table_hash, "missing");
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
            toml::render_assemble_plan(&assemble_plan),
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
    fn emit_container_reports_metadata_table_hash() {
        let dir = env::temp_dir().join(format!("nsld-container-emit-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();

        let report = nsld_emit_container_report(Path::new("manifest.toml"), &plan).unwrap();
        let container_source = fs::read_to_string(&report.output_path).unwrap();
        fs::remove_dir_all(dir).unwrap();

        assert!(report.metadata_table_hash.starts_with("0x"));
        assert!(container_source.contains(&format!(
            "metadata_table_hash = \"{}\"",
            report.metadata_table_hash
        )));
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

        assert_eq!(payload_bytes.len(), prepare.payload_size_bytes);
        assert_eq!(fnv1a64_hex(&payload_bytes), prepare.payload_hash);
        assert!(container_source.contains("offset = 0"));
        assert!(container_source.contains("size_bytes = 17"));
        assert!(container_source.contains("loader_readiness = \"host-assisted\""));
        assert!(container_source.contains("external-import:final-stage-driver:clang"));
        assert!(container_source.contains("external-import:clang-target:"));
        assert!(container_source.contains("external-import:c-world-policy:wrapped"));
        assert!(container_source.contains("loader_entry_kind = \"lifecycle-bootstrap\""));
        assert!(container_source.contains("loader_entry_symbol = \"main\""));
        assert!(
            container_source.contains("loader_entry_section_id = \"sec0000.compiled-artifact\"")
        );
        assert!(container_source.contains("loader_symbol_count = 1"));
        assert!(container_source.contains("loader_symbol_table_hash = \"0x"));
        assert!(container_source.contains("[[loader_symbol]]"));
        assert!(container_source.contains("symbol_id = \"sym0000.loader-entry\""));
        assert!(container_source.contains("symbol_name = \"main\""));
        assert!(container_source.contains("section_id = \"sec0000.compiled-artifact\""));
        assert!(container_source.contains("relocation_count = 0"));
        assert!(container_source.contains("external_import_count = 3"));
        assert!(container_source.contains("external_import_table_hash = \"0x"));
        assert!(container_source.contains("[[external_import]]"));
        assert!(container_source.contains("import_kind = \"final-stage-driver\""));
        assert!(container_source.contains("import_kind = \"clang-target\""));
        assert!(container_source.contains("import_kind = \"c-world-policy\""));
        assert!(container_source.contains("payload_size_bytes = "));
        assert!(container_source.contains("payload_hash = \"0x"));
        assert!(container_source.contains("container_section_table_hash = \"0x"));
        assert!(container_source.contains("metadata_table_hash = \"0x"));
        assert!(report.valid);
        assert!(report.issues.is_empty());
        assert_eq!(report.actual_section_count, Some(5));
        assert_eq!(
            report.actual_container_section_table_hash.as_deref(),
            Some(report.expected_container_section_table_hash.as_str())
        );
        assert_eq!(
            report.actual_container_layout_hash,
            Some(prepare.container_layout_hash)
        );
        assert_eq!(report.actual_container_hash, Some(prepare.container_hash));
        assert_eq!(
            report.actual_metadata_table_hash.as_deref(),
            Some(report.expected_metadata_table_hash.as_str())
        );
        assert_eq!(report.expected_loader_readiness, "host-assisted");
        assert_eq!(
            report.actual_loader_readiness.as_deref(),
            Some("host-assisted")
        );
        assert_eq!(report.expected_loader_entry_kind, "lifecycle-bootstrap");
        assert_eq!(
            report.actual_loader_entry_kind.as_deref(),
            Some("lifecycle-bootstrap")
        );
        assert_eq!(report.expected_loader_entry_symbol, "main");
        assert_eq!(report.actual_loader_entry_symbol.as_deref(), Some("main"));
        assert_eq!(
            report.expected_loader_entry_section_id,
            "sec0000.compiled-artifact"
        );
        assert_eq!(
            report.actual_loader_entry_section_id.as_deref(),
            Some("sec0000.compiled-artifact")
        );
        assert_eq!(report.expected_external_import_count, 3);
        assert_eq!(report.actual_external_import_count, Some(3));
        assert_eq!(report.expected_loader_symbol_count, 1);
        assert_eq!(report.actual_loader_symbol_count, Some(1));
        assert_eq!(
            report.actual_loader_symbol_table_hash.as_deref(),
            Some(report.expected_loader_symbol_table_hash.as_str())
        );
        assert_eq!(report.expected_loader_symbol_id, "sym0000.loader-entry");
        assert_eq!(
            report.actual_loader_symbol_id.as_deref(),
            Some("sym0000.loader-entry")
        );
        assert_eq!(report.expected_loader_symbol_kind, "lifecycle-bootstrap");
        assert_eq!(
            report.actual_loader_symbol_kind.as_deref(),
            Some("lifecycle-bootstrap")
        );
        assert_eq!(report.expected_loader_symbol_name, "main");
        assert_eq!(report.actual_loader_symbol_name.as_deref(), Some("main"));
        assert_eq!(
            report.expected_loader_symbol_section_id,
            "sec0000.compiled-artifact"
        );
        assert_eq!(
            report.actual_loader_symbol_section_id.as_deref(),
            Some("sec0000.compiled-artifact")
        );
        assert_eq!(report.expected_relocation_count, 0);
        assert_eq!(report.actual_relocation_count, Some(0));
        assert_eq!(
            report.actual_external_import_table_hash.as_deref(),
            Some(report.expected_external_import_table_hash.as_str())
        );
        assert_eq!(
            report.expected_external_import_id,
            "imp0000.final-stage-driver"
        );
        assert_eq!(
            report.actual_external_import_id.as_deref(),
            Some("imp0000.final-stage-driver")
        );
        assert_eq!(report.expected_external_import_kind, "final-stage-driver");
        assert_eq!(
            report.actual_external_import_kind.as_deref(),
            Some("final-stage-driver")
        );
        assert_eq!(report.expected_external_import_name, "clang");
        assert_eq!(report.actual_external_import_name.as_deref(), Some("clang"));
        assert_eq!(report.expected_external_import_provider, "host-toolchain");
        assert_eq!(
            report.actual_external_import_provider.as_deref(),
            Some("host-toolchain")
        );
        assert!(report.expected_external_import_required);
        assert_eq!(report.actual_external_import_required, Some(true));

        let tampered_container_source = container_source
            .replace(
                "loader_readiness = \"host-assisted\"",
                "loader_readiness = \"self-contained\"",
            )
            .replace(
                &format!(
                    "metadata_table_hash = \"{}\"",
                    report.expected_metadata_table_hash
                ),
                "metadata_table_hash = \"0x0000000000000000\"",
            )
            .replace(
                &format!(
                    "container_section_table_hash = \"{}\"",
                    report.expected_container_section_table_hash
                ),
                "container_section_table_hash = \"0x0000000000000000\"",
            )
            .replace(
                "loader_entry_kind = \"lifecycle-bootstrap\"",
                "loader_entry_kind = \"manual-entry\"",
            )
            .replace(
                "loader_entry_symbol = \"main\"",
                "loader_entry_symbol = \"alt\"",
            )
            .replace(
                "loader_entry_section_id = \"sec0000.compiled-artifact\"",
                "loader_entry_section_id = \"sec9999.missing\"",
            )
            .replace("loader_symbol_count = 1", "loader_symbol_count = 0")
            .replace(
                &format!(
                    "loader_symbol_table_hash = \"{}\"",
                    report.expected_loader_symbol_table_hash
                ),
                "loader_symbol_table_hash = \"0x0000000000000000\"",
            )
            .replace(
                "symbol_id = \"sym0000.loader-entry\"",
                "symbol_id = \"sym9999.manual\"",
            )
            .replace(
                "symbol_kind = \"lifecycle-bootstrap\"",
                "symbol_kind = \"manual-symbol\"",
            )
            .replace("symbol_name = \"main\"", "symbol_name = \"alt\"")
            .replace(
                "section_id = \"sec0000.compiled-artifact\"",
                "section_id = \"sec9999.missing\"",
            )
            .replace("relocation_count = 0", "relocation_count = 1")
            .replace("external_import_count = 3", "external_import_count = 0")
            .replace(
                &format!(
                    "external_import_table_hash = \"{}\"",
                    report.expected_external_import_table_hash
                ),
                "external_import_table_hash = \"0x0000000000000000\"",
            )
            .replace(
                "import_id = \"imp0000.final-stage-driver\"",
                "import_id = \"imp9999.manual\"",
            )
            .replace(
                "import_kind = \"final-stage-driver\"",
                "import_kind = \"manual-driver\"",
            )
            .replace("import_name = \"clang\"", "import_name = \"manual-clang\"")
            .replace(
                "provider = \"host-toolchain\"",
                "provider = \"manual-provider\"",
            )
            .replace("required = true", "required = false");
        fs::write(&prepare.container_path, tampered_container_source).unwrap();
        let tampered_report = nsld_verify_container_report(Path::new("manifest.toml"), &plan);
        assert!(!tampered_report.valid);
        assert_eq!(
            tampered_report.actual_loader_readiness.as_deref(),
            Some("self-contained")
        );
        assert_eq!(
            tampered_report
                .actual_container_section_table_hash
                .as_deref(),
            Some("0x0000000000000000")
        );
        assert_eq!(
            tampered_report.actual_metadata_table_hash.as_deref(),
            Some("0x0000000000000000")
        );
        assert_eq!(
            tampered_report.actual_loader_entry_kind.as_deref(),
            Some("manual-entry")
        );
        assert_eq!(
            tampered_report.actual_loader_entry_symbol.as_deref(),
            Some("alt")
        );
        assert_eq!(
            tampered_report.actual_loader_entry_section_id.as_deref(),
            Some("sec9999.missing")
        );
        assert_eq!(tampered_report.actual_external_import_count, Some(0));
        assert_eq!(tampered_report.actual_loader_symbol_count, Some(0));
        assert_eq!(
            tampered_report.actual_loader_symbol_table_hash.as_deref(),
            Some("0x0000000000000000")
        );
        assert_eq!(
            tampered_report.actual_loader_symbol_id.as_deref(),
            Some("sym9999.manual")
        );
        assert_eq!(
            tampered_report.actual_loader_symbol_kind.as_deref(),
            Some("manual-symbol")
        );
        assert_eq!(
            tampered_report.actual_loader_symbol_name.as_deref(),
            Some("alt")
        );
        assert_eq!(
            tampered_report.actual_loader_symbol_section_id.as_deref(),
            Some("sec9999.missing")
        );
        assert_eq!(tampered_report.actual_relocation_count, Some(1));
        assert_eq!(
            tampered_report.actual_external_import_table_hash.as_deref(),
            Some("0x0000000000000000")
        );
        assert_eq!(
            tampered_report.actual_external_import_id.as_deref(),
            Some("imp9999.manual")
        );
        assert_eq!(
            tampered_report.actual_external_import_kind.as_deref(),
            Some("manual-driver")
        );
        assert_eq!(
            tampered_report.actual_external_import_name.as_deref(),
            Some("manual-clang")
        );
        assert_eq!(
            tampered_report.actual_external_import_provider.as_deref(),
            Some("manual-provider")
        );
        assert_eq!(tampered_report.actual_external_import_required, Some(false));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("loader_readiness mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("container_section_table_hash mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("metadata_table_hash mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("loader_entry_kind mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("loader_entry_symbol mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("loader_entry_section_id mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("loader_symbol_count mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("loader_symbol_table_hash mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("loader_symbol_id mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("loader_symbol_kind mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("loader_symbol_name mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("loader_symbol_section_id mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("relocation_count mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("external_import_count mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("external_import_table_hash mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("external_import_id mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("external_import_kind mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("external_import_name mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("external_import_provider mismatch")));
        assert!(tampered_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("external_import_required mismatch")));
        fs::write(&prepare.container_path, container_source).unwrap();

        let mut corrupted_payload = payload_bytes;
        corrupted_payload[0] ^= 0xff;
        fs::write(&prepare.container_payload_path, corrupted_payload).unwrap();
        let corrupted_report = nsld_verify_container_report(Path::new("manifest.toml"), &plan);
        assert!(!corrupted_report.valid);
        assert!(corrupted_report
            .issues
            .iter()
            .any(|issue| issue.starts_with("payload_file_hash mismatch")));
        assert!(corrupted_report
            .section_range_issues
            .iter()
            .any(|issue| issue.starts_with("section_payload_hash mismatch")));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn check_reports_container_loader_readiness_without_failing_host_assisted_state() {
        let dir = env::temp_dir().join(format!("nsld-check-loader-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();

        nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
        let report = nsld_check_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(report.valid);
        assert_eq!(
            report.container_loader_readiness.as_deref(),
            Some("host-assisted")
        );
        assert!(report
            .container_metadata_table_hash
            .as_deref()
            .is_some_and(|hash| hash.starts_with("0x")));
        assert_eq!(report.container_external_import_count, Some(3));
        assert!(report
            .container_loader_blockers
            .iter()
            .any(|blocker| blocker == "external-import:final-stage-driver:clang"));
        assert!(report
            .issues
            .iter()
            .all(|issue| !issue.contains("container loader readiness is blocked")));
    }
}
