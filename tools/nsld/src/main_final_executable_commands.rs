use super::{cli::Command, context::load_link_input_context, display::*, final_stage::*, json::*};

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
            let report = nsld_final_executable_output_report(&ctx.manifest, &ctx.plan);
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
