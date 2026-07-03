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
mod json;
mod json_container;
mod json_fields;
mod json_fragments;
mod json_object;
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
#[cfg(test)]
mod main_test_support;
#[cfg(test)]
mod main_tests;
mod object_byte_layout;
mod object_file_layout;
mod object_layout;
mod object_macho_header;
mod object_macho_load_commands;
mod object_macho_relocations;
mod object_macho_symbols;
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

use assembly::*;
use check::*;
use cli::{parse_args, Command};
use closure::*;
use commands::{run_plan_command, run_status_command};
use container_pipeline::*;
use context::load_link_input_context;
use display::*;
use json::*;
use link_units::*;
use object_byte_layout::*;
use object_file_layout::*;
use object_plan::*;
use object_writer_input::*;
use prepare::*;
use std::{env, process};
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match parse_args(env::args().skip(1))? {
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
        Command::Closure { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_closure_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_closure_report_json(&report));
            } else {
                print_nsld_closure_report(&report);
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
        Command::ObjectPlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_object_plan_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_object_plan_report_json(&report));
            } else {
                print_nsld_object_plan_report(&report);
            }
        }
        Command::EmitObjectPlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_object_plan_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_object_plan_emit_report_json(&report));
            } else {
                print_nsld_object_plan_emit_report(&report);
            }
        }
        Command::VerifyObjectPlan { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_object_plan_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_object_plan_verify_report_json(&report));
            } else {
                print_nsld_object_plan_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld object plan verification failed".to_owned());
            }
        }
        Command::ObjectWriterReadiness { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_object_writer_readiness_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_object_writer_readiness_report_json(&report));
            } else {
                print_nsld_object_writer_readiness_report(&report);
            }
        }
        Command::EmitObject { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_object_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_object_emit_report_json(&report));
            } else {
                print_nsld_object_emit_report(&report);
            }
            if !report.emitted {
                return Err("nsld object emission is not ready".to_owned());
            }
        }
        Command::VerifyObjectWriterInput { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_object_writer_input_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_object_writer_input_verify_report_json(&report));
            } else {
                print_nsld_object_writer_input_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld object writer input verification failed".to_owned());
            }
        }
        Command::ObjectWriterDryRun { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_object_writer_dry_run_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_object_writer_dry_run_report_json(&report));
            } else {
                print_nsld_object_writer_dry_run_report(&report);
            }
        }
        Command::EmitObjectWriterDryRun { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_object_writer_dry_run_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_object_writer_dry_run_emit_report_json(&report));
            } else {
                print_nsld_object_writer_dry_run_emit_report(&report);
            }
        }
        Command::VerifyObjectWriterDryRun { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_object_writer_dry_run_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_object_writer_dry_run_verify_report_json(&report));
            } else {
                print_nsld_object_writer_dry_run_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld object writer dry run verification failed".to_owned());
            }
        }
        Command::ObjectByteLayout { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_object_byte_layout_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_object_byte_layout_report_json(&report));
            } else {
                print_nsld_object_byte_layout_report(&report);
            }
        }
        Command::EmitObjectByteLayout { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_object_byte_layout_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_object_byte_layout_emit_report_json(&report));
            } else {
                print_nsld_object_byte_layout_emit_report(&report);
            }
        }
        Command::VerifyObjectByteLayout { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_object_byte_layout_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_object_byte_layout_verify_report_json(&report));
            } else {
                print_nsld_object_byte_layout_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld object byte layout verification failed".to_owned());
            }
        }
        Command::ObjectFileLayout { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_object_file_layout_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_object_file_layout_report_json(&report));
            } else {
                print_nsld_object_file_layout_report(&report);
            }
        }
        Command::EmitObjectFileLayout { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_emit_object_file_layout_report(&ctx.manifest, &ctx.plan)?;
            if json {
                println!("{}", nsld_object_file_layout_emit_report_json(&report));
            } else {
                print_nsld_object_file_layout_emit_report(&report);
            }
        }
        Command::VerifyObjectFileLayout { input, json } => {
            let ctx = load_link_input_context(&input)?;
            let report = nsld_verify_object_file_layout_report(&ctx.manifest, &ctx.plan);
            if json {
                println!("{}", nsld_object_file_layout_verify_report_json(&report));
            } else {
                print_nsld_object_file_layout_verify_report(&report);
            }
            if !report.valid {
                return Err("nsld object file layout verification failed".to_owned());
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
    }
    Ok(())
}
