mod artifact_chain;
mod assembly;
mod check;
mod cli;
mod closure;
mod commands;
mod container;
mod container_model;
mod container_pipeline;
mod container_pipeline_actual;
mod container_pipeline_mismatch;
mod container_pipeline_tables;
mod container_pipeline_verify;
mod container_render;
mod container_verify;
mod context;
mod display;
mod display_check;
mod display_container;
mod display_container_verify;
mod display_link_tables;
mod display_object;
mod display_object_emit;
mod display_object_image;
mod final_executable_emit;
mod final_executable_host;
mod final_executable_image;
mod final_executable_image_stage;
mod final_executable_layout;
mod final_executable_layout_stage;
mod final_executable_output;
mod final_executable_paths;
mod final_executable_render;
mod final_executable_summary;
mod final_executable_verify_helpers;
mod final_executable_writer;
mod final_executable_writer_input;
mod final_stage;
mod final_stage_plan;
mod json;
mod json_container;
mod json_fields;
mod json_fragments;
mod json_object;
mod json_object_emit;
mod json_object_image;
mod link_bundle_pipeline;
mod link_inputs_pipeline;
mod link_units;
#[cfg(test)]
mod main_cli_object_tests;
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
mod main_object_commands;
#[cfg(test)]
mod main_test_support;
#[cfg(test)]
mod main_tests;
mod object_byte_layout;
mod object_emit;
mod object_emit_render;
mod object_file_layout;
mod object_image_backend;
mod object_image_dry_run;
mod object_image_render;
mod object_layout;
mod object_macho_header;
mod object_macho_image;
mod object_macho_load_commands;
mod object_macho_relocations;
mod object_macho_symbols;
mod object_output;
mod object_plan;
mod object_plan_verify;
mod object_render;
mod object_writer_backend;
mod object_writer_input;
mod prepare;
mod protocol;
mod reports;
mod reports_object;
mod toml;
mod toml_read;

pub(crate) use protocol::*;

use artifact_chain::*;
use assembly::*;
use check::*;
use cli::{parse_args, Command};
use closure::*;
use commands::{run_plan_command, run_status_command};
use container_pipeline::*;
use context::load_link_input_context;
use display::*;
use final_stage::*;
use json::*;
use link_units::*;
use main_object_commands::run_object_command;
use prepare::*;
use std::{env, process};
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let command = parse_args(env::args().skip(1))?;
    if let Some(result) = run_object_command(&command) {
        return result;
    }

    match command {
        Command::Status => {
            run_status_command();
        }
        Command::Plan { input, json } => {
            run_plan_command(&input, json)?;
        }
        Command::Check { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_check_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", json::check_report_json(&report));
            } else {
                display::print_check_report(&report);
            }
            if !report.valid {
                return Err("nsld check failed".to_owned());
            }
        }
        Command::ArtifactChain { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_artifact_chain_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_artifact_chain_report_json(&report));
            } else {
                print_nsld_artifact_chain_report(&report);
            }
        }
        Command::Closure { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_closure_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_closure_report_json(&report));
            } else {
                print_nsld_closure_report(&report);
            }
        }
        Command::EmitClosure { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_closure_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_closure_emit_report_json(&report));
            } else {
                print_nsld_closure_emit_report(&report);
            }
        }
        Command::VerifyClosure { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_closure_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_closure_verify_report_json(&report));
            } else {
                print_nsld_closure_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld closure verification failed".to_owned());
            }
        }
        Command::FinalStagePlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_final_stage_plan_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_final_stage_plan_report_json(&report));
            } else {
                print_nsld_final_stage_plan_report(&report);
            }
        }
        Command::EmitFinalStagePlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_final_stage_plan_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_final_stage_plan_emit_report_json(&report));
            } else {
                print_nsld_final_stage_plan_emit_report(&report);
            }
        }
        Command::VerifyFinalStagePlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_final_stage_plan_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_final_stage_plan_verify_report_json(&report));
            } else {
                print_nsld_final_stage_plan_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld final-stage plan verification failed".to_owned());
            }
        }
        Command::FinalExecutableReadiness { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_final_executable_readiness_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_final_executable_readiness_report_json(&report));
            } else {
                print_nsld_final_executable_readiness_report(&report);
            }
        }
        Command::FinalExecutableWriterPlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_final_executable_writer_plan_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_final_executable_writer_plan_report_json(&report));
            } else {
                print_nsld_final_executable_writer_plan_report(&report);
            }
        }
        Command::EmitFinalExecutableWriterInput { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_final_executable_writer_input_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!(
                    "{}",
                    nsld_final_executable_writer_input_emit_report_json(&report)
                );
            } else {
                print_nsld_final_executable_writer_input_emit_report(&report);
            }
        }
        Command::VerifyFinalExecutableWriterInput { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_final_executable_writer_input_report(&ctx.manifest, &ctx.plan);
            if json {
                println!(
                    "{}",
                    nsld_final_executable_writer_input_verify_report_json(&report)
                );
            } else {
                print_nsld_final_executable_writer_input_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld final executable writer input verification failed".to_owned());
            }
        }
        Command::FinalExecutableHostDryRun { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_final_executable_host_dry_run_report(&ctx.manifest, &ctx.plan);
            if json {
                println!(
                    "{}",
                    nsld_final_executable_host_dry_run_report_json(&report)
                );
            } else {
                print_nsld_final_executable_host_dry_run_report(&report);
            }
        }
        Command::FinalExecutableHostInvokePlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_final_executable_host_invoke_plan_report(&ctx.manifest, &ctx.plan);
            if json {
                println!(
                    "{}",
                    nsld_final_executable_host_invoke_plan_report_json(&report)
                );
            } else {
                print_nsld_final_executable_host_invoke_plan_report(&report);
            }
        }
        Command::EmitFinalExecutableHostInvokePlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report =
                nsld_emit_final_executable_host_invoke_plan_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!(
                    "{}",
                    nsld_final_executable_host_invoke_plan_emit_report_json(&report)
                );
            } else {
                print_nsld_final_executable_host_invoke_plan_emit_report(&report);
            }
        }
        Command::VerifyFinalExecutableHostInvokePlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report =
                nsld_verify_final_executable_host_invoke_plan_report(&ctx.manifest, &ctx.plan);
            if json {
                println!(
                    "{}",
                    nsld_final_executable_host_invoke_plan_verify_report_json(&report)
                );
            } else {
                print_nsld_final_executable_host_invoke_plan_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld final executable host invoke plan verification failed".to_owned());
            }
        }
        Command::FinalExecutableLayout { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_final_executable_layout_plan_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_final_executable_layout_plan_report_json(&report));
            } else {
                print_nsld_final_executable_layout_plan_report(&report);
            }
        }
        Command::EmitFinalExecutableLayout { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_final_executable_layout_plan_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!(
                    "{}",
                    nsld_final_executable_layout_plan_emit_report_json(&report)
                );
            } else {
                print_nsld_final_executable_layout_plan_emit_report(&report);
            }
        }
        Command::VerifyFinalExecutableLayout { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_final_executable_layout_plan_report(&ctx.manifest, &ctx.plan);
            if json {
                println!(
                    "{}",
                    nsld_final_executable_layout_plan_verify_report_json(&report)
                );
            } else {
                print_nsld_final_executable_layout_plan_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld final executable layout verification failed".to_owned());
            }
        }
        Command::FinalExecutableImageDryRun { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_final_executable_image_dry_run_report(&ctx.manifest, &ctx.plan);
            if json {
                println!(
                    "{}",
                    nsld_final_executable_image_dry_run_report_json(&report)
                );
            } else {
                print_nsld_final_executable_image_dry_run_report(&report);
            }
        }
        Command::EmitFinalExecutableImageDryRun { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_final_executable_image_dry_run_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!(
                    "{}",
                    nsld_final_executable_image_dry_run_emit_report_json(&report)
                );
            } else {
                print_nsld_final_executable_image_dry_run_emit_report(&report);
            }
        }
        Command::VerifyFinalExecutableImageDryRun { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report =
                nsld_verify_final_executable_image_dry_run_report(&ctx.manifest, &ctx.plan);
            if json {
                println!(
                    "{}",
                    nsld_final_executable_image_dry_run_verify_report_json(&report)
                );
            } else {
                print_nsld_final_executable_image_dry_run_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld final executable image dry-run verification failed".to_owned());
            }
        }
        Command::EmitFinalExecutable { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_final_executable_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_final_executable_emit_report_json(&report));
            } else {
                print_nsld_final_executable_emit_report(&report);
            }
        }
        Command::VerifyFinalExecutableEmit { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_final_executable_emit_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_final_executable_emit_verify_report_json(&report));
            } else {
                print_nsld_final_executable_emit_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld final executable emit verification failed".to_owned());
            }
        }
        Command::FinalExecutableOutput { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_final_executable_output_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_final_executable_output_report_json(&report));
            } else {
                print_nsld_final_executable_output_report(&report);
            }
        }
        Command::Prepare { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_prepare_report(&ctx.manifest, &ctx.plan)?;
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
            let ctx = load_link_input_context(&input)?;
            let report = nsld_assemble_plan_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_assemble_plan_report_json(&report));
            } else {
                print_nsld_assemble_plan_report(&report);
            }
        }
        Command::EmitAssemblePlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_assemble_plan_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_assemble_plan_emit_report_json(&report));
            } else {
                print_nsld_assemble_plan_emit_report(&report);
            }
        }
        Command::VerifyAssemblePlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_assemble_plan_report(&ctx.manifest, &ctx.plan);
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
            let ctx = load_link_input_context(&input)?;
            let report = nsld_section_manifest_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_section_manifest_report_json(&report));
            } else {
                print_nsld_section_manifest_report(&report);
            }
        }
        Command::EmitSectionManifest { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_section_manifest_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_section_manifest_emit_report_json(&report));
            } else {
                print_nsld_section_manifest_emit_report(&report);
            }
        }
        Command::VerifySectionManifest { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_section_manifest_report(&ctx.manifest, &ctx.plan);
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
            let ctx = load_link_input_context(&input)?;
            let report = nsld_container_plan_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_container_plan_report_json(&report));
            } else {
                print_nsld_container_plan_report(&report);
            }
        }
        Command::EmitContainerPlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_container_plan_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_container_plan_emit_report_json(&report));
            } else {
                print_nsld_container_plan_emit_report(&report);
            }
        }
        Command::VerifyContainerPlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_container_plan_report(&ctx.manifest, &ctx.plan);
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
            let ctx = load_link_input_context(&input)?;
            let report = nsld_container_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_container_report_json(&report));
            } else {
                print_nsld_container_report(&report);
            }
        }
        Command::EmitContainer { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_container_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_container_emit_report_json(&report));
            } else {
                print_nsld_container_emit_report(&report);
            }
        }
        Command::VerifyContainer { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_container_report(&ctx.manifest, &ctx.plan);
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
            let ctx = load_link_input_context(&input)?;
            let report = nsld_link_bundle_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_link_bundle_report_json(&report));
            } else {
                print_nsld_link_bundle_report(&report);
            }
        }
        Command::EmitBundle { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_link_bundle_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_link_bundle_emit_report_json(&report));
            } else {
                print_nsld_link_bundle_emit_report(&report);
            }
        }
        Command::VerifyBundle { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_link_bundle_report(&ctx.manifest, &ctx.plan);
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
            let ctx = load_link_input_context(&input)?;
            let report = nsld_link_unit_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_link_unit_report_json(&report));
            } else {
                print_nsld_link_unit_report(&report);
            }
        }
        Command::EmitUnits { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_link_units_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_link_units_emit_report_json(&report));
            } else {
                print_nsld_link_units_emit_report(&report);
            }
        }
        Command::VerifyUnits { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_link_units_report(&ctx.manifest, &ctx.plan);
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
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_link_inputs_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_link_inputs_emit_report_json(&report));
            } else {
                print_nsld_link_inputs_emit_report(&report);
            }
        }
        Command::VerifyInputs { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_link_inputs_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_link_inputs_verify_report_json(&report));
            } else {
                print_nsld_link_inputs_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld link input verification failed".to_owned());
            }
        }
        _ => unreachable!("object commands are handled before main command dispatch"),
    }
    Ok(())
}
