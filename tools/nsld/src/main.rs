mod assembly;
mod check;
mod cli;
mod closure;
mod container;
mod container_model;
mod container_pipeline;
mod container_pipeline_actual;
mod container_pipeline_mismatch;
mod container_pipeline_tables;
mod container_pipeline_verify;
mod container_render;
mod container_verify;
mod display;
mod display_check;
mod display_container;
mod display_container_verify;
mod display_link_tables;
mod json;
mod json_container;
mod json_fields;
mod json_fragments;
mod link_bundle_pipeline;
mod link_inputs_pipeline;
mod link_units;
#[cfg(test)]
mod main_cli_tests;
#[cfg(test)]
mod main_container_tests;
#[cfg(test)]
mod main_container_verify_tamper;
#[cfg(test)]
mod main_container_verify_tests;
#[cfg(test)]
mod main_link_pipeline_tests;
#[cfg(test)]
mod main_link_table_tests;
#[cfg(test)]
mod main_test_support;
#[cfg(test)]
mod main_tests;
mod prepare;
mod reports;
mod toml;
mod toml_read;

use assembly::*;
use check::*;
use cli::{parse_args, resolve_manifest_input, Command};
use closure::*;
use container_pipeline::*;
use display::*;
use json::*;
use link_units::*;
use prepare::*;
use std::{env, process};

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
