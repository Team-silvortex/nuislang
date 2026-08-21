use super::{
    cli::Command,
    context::load_link_input_context,
    display::*,
    display_final_macho_admission::print_macho_arm64_publication_admission_verify_report,
    display_final_macho_loader_probe::print_macho_arm64_loader_probe_report,
    final_executable_macho_admission::{
        build_macho_arm64_publication_admission_receipt,
        verify_macho_arm64_publication_admission_receipt,
    },
    final_executable_macho_admission_receipt::{
        persist_macho_arm64_publication_admission_receipt, MACHO_ARM64_PUBLICATION_ADMISSION_FILE,
    },
    final_executable_macho_artifact::macho_artifact_private_shell_product,
    final_executable_macho_loader_probe::{
        probe_macho_arm64_signed_shell_image, MachOArm64LoaderProbeInput,
    },
    final_executable_output_nsdb_handoff::{
        attach_final_output_nsdb_handoff_summary, persist_final_output_nsdb_handoff,
    },
    final_stage::*,
    json::*,
    json_final_macho_admission::macho_arm64_publication_admission_verify_report_json,
    json_final_macho_loader_probe::macho_arm64_loader_probe_report_json,
};
use std::path::Path;

pub(crate) fn run_final_executable_command(command: &Command) -> Result<bool, String> {
    match command {
        Command::FinalExecutableReadiness { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_final_executable_readiness_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!("{}", nsld_final_executable_readiness_report_json(&report));
            } else {
                print_nsld_final_executable_readiness_report(&report);
            }
            Ok(true)
        }
        Command::FinalExecutableWriterPlan { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_final_executable_writer_plan_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!("{}", nsld_final_executable_writer_plan_report_json(&report));
            } else {
                print_nsld_final_executable_writer_plan_report(&report);
            }
            Ok(true)
        }
        Command::EmitFinalExecutableWriterInput { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_emit_final_executable_writer_input_report(&ctx.manifest, &ctx.plan)?;
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_writer_input_emit_report_json(&report)
                );
            } else {
                print_nsld_final_executable_writer_input_emit_report(&report);
            }
            Ok(true)
        }
        Command::VerifyFinalExecutableWriterInput { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_verify_final_executable_writer_input_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_writer_input_verify_report_json(&report)
                );
            } else {
                print_nsld_final_executable_writer_input_verify_report(&report);
            }
            if report.valid {
                Ok(true)
            } else {
                Err("nsld final executable writer input verification failed".to_owned())
            }
        }
        Command::FinalExecutableHostDryRun { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_final_executable_host_dry_run_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_host_dry_run_report_json(&report)
                );
            } else {
                print_nsld_final_executable_host_dry_run_report(&report);
            }
            Ok(true)
        }
        Command::FinalExecutablePrivateImageLoaderProbe { input, json, apply } => {
            let ctx = load_link_input_context(input)?;
            let product = macho_artifact_private_shell_product(&ctx.plan)?;
            let mut report = probe_macho_arm64_signed_shell_image(
                MachOArm64LoaderProbeInput {
                    bytes: &product.bytes,
                    serialization: &product.summary.shell_image_serialization,
                    unresolved_external_symbol_count: product
                        .summary
                        .unresolved_external_symbol_count,
                    bind_count: product.summary.shell_layout_plan.binds.len(),
                },
                Path::new(&ctx.plan.output_dir),
                *apply,
            )?;
            if *apply && report.publication_eligible {
                let receipt =
                    build_macho_arm64_publication_admission_receipt(&ctx.plan, &product, &report)?;
                persist_macho_arm64_publication_admission_receipt(&ctx.plan, &receipt)?;
                let verification =
                    verify_macho_arm64_publication_admission_receipt(&ctx.plan, &product);
                report.admission_receipt_file =
                    Some(MACHO_ARM64_PUBLICATION_ADMISSION_FILE.to_owned());
                report.admission_receipt_persisted = true;
                report.admission_receipt_hash_sha256 = Some(receipt.receipt_hash_sha256.clone());
                report.admission_receipt_validation_status = verification.status.clone();
            }
            if *json {
                println!("{}", macho_arm64_loader_probe_report_json(&report));
            } else {
                print_macho_arm64_loader_probe_report(&report);
            }
            if *apply
                && (!report.publication_eligible
                    || !report.admission_receipt_persisted
                    || report.admission_receipt_validation_status
                        != "publication-admission-replay-verified")
            {
                Err(
                    "nsld private-image loader probe did not produce a verified admission receipt"
                        .to_owned(),
                )
            } else {
                Ok(true)
            }
        }
        Command::VerifyFinalExecutablePrivateImageAdmission { input, json } => {
            let ctx = load_link_input_context(input)?;
            let product = macho_artifact_private_shell_product(&ctx.plan)?;
            let report = verify_macho_arm64_publication_admission_receipt(&ctx.plan, &product);
            if *json {
                println!(
                    "{}",
                    macho_arm64_publication_admission_verify_report_json(&report)
                );
            } else {
                print_macho_arm64_publication_admission_verify_report(&report);
            }
            if report.valid {
                Ok(true)
            } else {
                Err("nsld private-image publication admission verification failed".to_owned())
            }
        }
        Command::FinalExecutableHostInvokePlan { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_final_executable_host_invoke_plan_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_host_invoke_plan_report_json(&report)
                );
            } else {
                print_nsld_final_executable_host_invoke_plan_report(&report);
            }
            Ok(true)
        }
        Command::EmitFinalExecutableHostInvokePlan { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report =
                nsld_emit_final_executable_host_invoke_plan_report(&ctx.manifest, &ctx.plan)?;
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_host_invoke_plan_emit_report_json(&report)
                );
            } else {
                print_nsld_final_executable_host_invoke_plan_emit_report(&report);
            }
            Ok(true)
        }
        Command::VerifyFinalExecutableHostInvokePlan { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report =
                nsld_verify_final_executable_host_invoke_plan_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_host_invoke_plan_verify_report_json(&report)
                );
            } else {
                print_nsld_final_executable_host_invoke_plan_verify_report(&report);
            }
            if report.valid {
                Ok(true)
            } else {
                Err("nsld final executable host invoke plan verification failed".to_owned())
            }
        }
        Command::FinalExecutableLayout { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_final_executable_layout_plan_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!("{}", nsld_final_executable_layout_plan_report_json(&report));
            } else {
                print_nsld_final_executable_layout_plan_report(&report);
            }
            Ok(true)
        }
        Command::EmitFinalExecutableLayout { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_emit_final_executable_layout_plan_report(&ctx.manifest, &ctx.plan)?;
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_layout_plan_emit_report_json(&report)
                );
            } else {
                print_nsld_final_executable_layout_plan_emit_report(&report);
            }
            Ok(true)
        }
        Command::VerifyFinalExecutableLayout { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_verify_final_executable_layout_plan_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_layout_plan_verify_report_json(&report)
                );
            } else {
                print_nsld_final_executable_layout_plan_verify_report(&report);
            }
            if report.valid {
                Ok(true)
            } else {
                Err("nsld final executable layout verification failed".to_owned())
            }
        }
        Command::FinalExecutableImageDryRun { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_final_executable_image_dry_run_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_image_dry_run_report_json(&report)
                );
            } else {
                print_nsld_final_executable_image_dry_run_report(&report);
            }
            Ok(true)
        }
        Command::EmitFinalExecutableImageDryRun { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_emit_final_executable_image_dry_run_report(&ctx.manifest, &ctx.plan)?;
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_image_dry_run_emit_report_json(&report)
                );
            } else {
                print_nsld_final_executable_image_dry_run_emit_report(&report);
            }
            Ok(true)
        }
        Command::VerifyFinalExecutableImageDryRun { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report =
                nsld_verify_final_executable_image_dry_run_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_image_dry_run_verify_report_json(&report)
                );
            } else {
                print_nsld_final_executable_image_dry_run_verify_report(&report);
            }
            if report.valid {
                Ok(true)
            } else {
                Err("nsld final executable image dry-run verification failed".to_owned())
            }
        }
        Command::EmitFinalExecutablePipeline { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_emit_final_executable_pipeline_report(&ctx.manifest, &ctx.plan)?;
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_pipeline_emit_report_json(&report)
                );
            } else {
                print_nsld_final_executable_pipeline_emit_report(&report);
            }
            if report.valid {
                Ok(true)
            } else {
                Err("nsld final executable pipeline emit completed with blockers".to_owned())
            }
        }
        Command::VerifyFinalExecutablePipeline { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_verify_final_executable_pipeline_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_pipeline_verify_report_json(&report)
                );
            } else {
                print_nsld_final_executable_pipeline_verify_report(&report);
            }
            if report.valid {
                Ok(true)
            } else {
                Err("nsld final executable pipeline verification failed".to_owned())
            }
        }
        Command::EmitFinalExecutable { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_emit_final_executable_report(&ctx.manifest, &ctx.plan)?;
            if *json {
                println!("{}", nsld_final_executable_emit_report_json(&report));
            } else {
                print_nsld_final_executable_emit_report(&report);
            }
            Ok(true)
        }
        Command::VerifyFinalExecutableEmit { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_verify_final_executable_emit_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!("{}", nsld_final_executable_emit_verify_report_json(&report));
            } else {
                print_nsld_final_executable_emit_verify_report(&report);
            }
            if report.valid {
                Ok(true)
            } else {
                Err("nsld final executable emit verification failed".to_owned())
            }
        }
        Command::FinalExecutableOutput { input, json } => {
            let ctx = load_link_input_context(input)?;
            let mut report = nsld_final_executable_output_report(&ctx.manifest, &ctx.plan);
            let summary = persist_final_output_nsdb_handoff(
                std::path::Path::new(&ctx.plan.output_dir),
                &report,
            );
            attach_final_output_nsdb_handoff_summary(&mut report, summary);
            if *json {
                println!("{}", nsld_final_executable_output_report_json(&report));
            } else {
                print_nsld_final_executable_output_report(&report);
            }
            Ok(true)
        }
        Command::FinalExecutableLauncherManifest { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_final_executable_launcher_manifest_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_launcher_manifest_report_json(&report)
                );
            } else {
                print_nsld_final_executable_launcher_manifest_report(&report);
            }
            Ok(true)
        }
        Command::EmitFinalExecutableLauncherManifest { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report =
                nsld_emit_final_executable_launcher_manifest_report(&ctx.manifest, &ctx.plan)?;
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_launcher_manifest_emit_report_json(&report)
                );
            } else {
                print_nsld_final_executable_launcher_manifest_emit_report(&report);
            }
            Ok(true)
        }
        Command::VerifyFinalExecutableLauncherManifest { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report =
                nsld_verify_final_executable_launcher_manifest_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_launcher_manifest_verify_report_json(&report)
                );
            } else {
                print_nsld_final_executable_launcher_manifest_verify_report(&report);
            }
            if report.valid {
                Ok(true)
            } else {
                Err("nsld final executable launcher manifest verification failed".to_owned())
            }
        }
        Command::FinalExecutableLauncherDryRun { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report = nsld_final_executable_launcher_dry_run_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_launcher_dry_run_report_json(&report)
                );
            } else {
                print_nsld_final_executable_launcher_dry_run_report(&report);
            }
            Ok(true)
        }
        Command::EmitFinalExecutableLauncherDryRun { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report =
                nsld_emit_final_executable_launcher_dry_run_report(&ctx.manifest, &ctx.plan)?;
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_launcher_dry_run_emit_report_json(&report)
                );
            } else {
                print_nsld_final_executable_launcher_dry_run_emit_report(&report);
            }
            Ok(true)
        }
        Command::VerifyFinalExecutableLauncherDryRun { input, json } => {
            let ctx = load_link_input_context(input)?;
            let report =
                nsld_verify_final_executable_launcher_dry_run_report(&ctx.manifest, &ctx.plan);
            if *json {
                println!(
                    "{}",
                    nsld_final_executable_launcher_dry_run_verify_report_json(&report)
                );
            } else {
                print_nsld_final_executable_launcher_dry_run_verify_report(&report);
            }
            if report.valid {
                Ok(true)
            } else {
                Err("nsld final executable launcher dry-run verification failed".to_owned())
            }
        }
        _ => Ok(false),
    }
}
