use crate::cli::Command;
use crate::context::load_link_input_context;
use crate::display::*;
use crate::json::*;
use crate::object_byte_layout::*;
use crate::object_emit::*;
use crate::object_file_layout::*;
use crate::object_image_dry_run::*;
use crate::object_plan::*;
use crate::object_writer_input::*;

pub(crate) fn run_object_command(command: &Command) -> Option<Result<(), String>> {
    match command {
        Command::ObjectPlan { input, json } => Some(run_object_plan_command(input, *json)),
        Command::EmitObjectPlan { input, json } => Some(run_emit_object_plan_command(input, *json)),
        Command::VerifyObjectPlan { input, json } => {
            Some(run_verify_object_plan_command(input, *json))
        }
        Command::ObjectWriterReadiness { input, json } => {
            Some(run_object_writer_readiness_command(input, *json))
        }
        Command::EmitObject { input, json } => Some(run_emit_object_command(input, *json)),
        Command::VerifyObjectEmit { input, json } => {
            Some(run_verify_object_emit_command(input, *json))
        }
        Command::VerifyObjectWriterInput { input, json } => {
            Some(run_verify_object_writer_input_command(input, *json))
        }
        Command::ObjectWriterDryRun { input, json } => {
            Some(run_object_writer_dry_run_command(input, *json))
        }
        Command::EmitObjectWriterDryRun { input, json } => {
            Some(run_emit_object_writer_dry_run_command(input, *json))
        }
        Command::VerifyObjectWriterDryRun { input, json } => {
            Some(run_verify_object_writer_dry_run_command(input, *json))
        }
        Command::ObjectByteLayout { input, json } => {
            Some(run_object_byte_layout_command(input, *json))
        }
        Command::EmitObjectByteLayout { input, json } => {
            Some(run_emit_object_byte_layout_command(input, *json))
        }
        Command::VerifyObjectByteLayout { input, json } => {
            Some(run_verify_object_byte_layout_command(input, *json))
        }
        Command::ObjectFileLayout { input, json } => {
            Some(run_object_file_layout_command(input, *json))
        }
        Command::EmitObjectFileLayout { input, json } => {
            Some(run_emit_object_file_layout_command(input, *json))
        }
        Command::VerifyObjectFileLayout { input, json } => {
            Some(run_verify_object_file_layout_command(input, *json))
        }
        Command::ObjectImageDryRun { input, json } => {
            Some(run_object_image_dry_run_command(input, *json))
        }
        Command::EmitObjectImageDryRun { input, json } => {
            Some(run_emit_object_image_dry_run_command(input, *json))
        }
        Command::VerifyObjectImageDryRun { input, json } => {
            Some(run_verify_object_image_dry_run_command(input, *json))
        }
        _ => None,
    }
}

fn run_object_plan_command(input: &std::path::Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_object_plan_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_plan_report_json(&report));
    } else {
        print_nsld_object_plan_report(&report);
    }
    Ok(())
}

fn run_emit_object_plan_command(input: &std::path::Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_emit_object_plan_report(&ctx.manifest, &ctx.plan)?;
    if json {
        println!("{}", nsld_object_plan_emit_report_json(&report));
    } else {
        print_nsld_object_plan_emit_report(&report);
    }
    Ok(())
}

fn run_verify_object_plan_command(input: &std::path::Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_verify_object_plan_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_plan_verify_report_json(&report));
    } else {
        print_nsld_object_plan_verify_report(&report);
    }
    if !report.valid {
        return Err("nsld object plan verification failed".to_owned());
    }
    Ok(())
}

fn run_object_writer_readiness_command(input: &std::path::Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_object_writer_readiness_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_writer_readiness_report_json(&report));
    } else {
        print_nsld_object_writer_readiness_report(&report);
    }
    Ok(())
}

fn run_emit_object_command(input: &std::path::Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_emit_object_report(&ctx.manifest, &ctx.plan)?;
    if json {
        println!("{}", nsld_object_emit_report_json(&report));
    } else {
        print_nsld_object_emit_report(&report);
    }
    Ok(())
}

fn run_verify_object_emit_command(input: &std::path::Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_verify_object_emit_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_emit_verify_report_json(&report));
    } else {
        print_nsld_object_emit_verify_report(&report);
    }
    if !report.valid {
        return Err("nsld object emit verification failed".to_owned());
    }
    Ok(())
}

fn run_verify_object_writer_input_command(
    input: &std::path::Path,
    json: bool,
) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_verify_object_writer_input_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_writer_input_verify_report_json(&report));
    } else {
        print_nsld_object_writer_input_verify_report(&report);
    }
    if !report.valid {
        return Err("nsld object writer input verification failed".to_owned());
    }
    Ok(())
}

fn run_object_writer_dry_run_command(input: &std::path::Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_object_writer_dry_run_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_writer_dry_run_report_json(&report));
    } else {
        print_nsld_object_writer_dry_run_report(&report);
    }
    Ok(())
}

fn run_emit_object_writer_dry_run_command(
    input: &std::path::Path,
    json: bool,
) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_emit_object_writer_dry_run_report(&ctx.manifest, &ctx.plan)?;
    if json {
        println!("{}", nsld_object_writer_dry_run_emit_report_json(&report));
    } else {
        print_nsld_object_writer_dry_run_emit_report(&report);
    }
    Ok(())
}

fn run_verify_object_writer_dry_run_command(
    input: &std::path::Path,
    json: bool,
) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_verify_object_writer_dry_run_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_writer_dry_run_verify_report_json(&report));
    } else {
        print_nsld_object_writer_dry_run_verify_report(&report);
    }
    if !report.valid {
        return Err("nsld object writer dry run verification failed".to_owned());
    }
    Ok(())
}

fn run_object_byte_layout_command(input: &std::path::Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_object_byte_layout_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_byte_layout_report_json(&report));
    } else {
        print_nsld_object_byte_layout_report(&report);
    }
    Ok(())
}

fn run_emit_object_byte_layout_command(input: &std::path::Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_emit_object_byte_layout_report(&ctx.manifest, &ctx.plan)?;
    if json {
        println!("{}", nsld_object_byte_layout_emit_report_json(&report));
    } else {
        print_nsld_object_byte_layout_emit_report(&report);
    }
    Ok(())
}

fn run_verify_object_byte_layout_command(
    input: &std::path::Path,
    json: bool,
) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_verify_object_byte_layout_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_byte_layout_verify_report_json(&report));
    } else {
        print_nsld_object_byte_layout_verify_report(&report);
    }
    if !report.valid {
        return Err("nsld object byte layout verification failed".to_owned());
    }
    Ok(())
}

fn run_object_file_layout_command(input: &std::path::Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_object_file_layout_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_file_layout_report_json(&report));
    } else {
        print_nsld_object_file_layout_report(&report);
    }
    Ok(())
}

fn run_emit_object_file_layout_command(input: &std::path::Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_emit_object_file_layout_report(&ctx.manifest, &ctx.plan)?;
    if json {
        println!("{}", nsld_object_file_layout_emit_report_json(&report));
    } else {
        print_nsld_object_file_layout_emit_report(&report);
    }
    Ok(())
}

fn run_verify_object_file_layout_command(
    input: &std::path::Path,
    json: bool,
) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_verify_object_file_layout_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_file_layout_verify_report_json(&report));
    } else {
        print_nsld_object_file_layout_verify_report(&report);
    }
    if !report.valid {
        return Err("nsld object file layout verification failed".to_owned());
    }
    Ok(())
}

fn run_object_image_dry_run_command(input: &std::path::Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_object_image_dry_run_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_image_dry_run_report_json(&report));
    } else {
        print_nsld_object_image_dry_run_report(&report);
    }
    Ok(())
}

fn run_emit_object_image_dry_run_command(
    input: &std::path::Path,
    json: bool,
) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_emit_object_image_dry_run_report(&ctx.manifest, &ctx.plan)?;
    if json {
        println!("{}", nsld_object_image_dry_run_emit_report_json(&report));
    } else {
        print_nsld_object_image_dry_run_emit_report(&report);
    }
    Ok(())
}

fn run_verify_object_image_dry_run_command(
    input: &std::path::Path,
    json: bool,
) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_verify_object_image_dry_run_report(&ctx.manifest, &ctx.plan);
    if json {
        println!("{}", nsld_object_image_dry_run_verify_report_json(&report));
    } else {
        print_nsld_object_image_dry_run_verify_report(&report);
    }
    if !report.valid {
        return Err("nsld object image dry run verification failed".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run_object_command;
    use crate::cli::Command;
    use std::path::PathBuf;

    fn missing_input() -> PathBuf {
        PathBuf::from("__nsld_missing_object_command_input__")
    }

    #[test]
    fn object_command_dispatcher_claims_every_object_command_variant() {
        let input = missing_input();
        let commands = vec![
            Command::ObjectPlan {
                input: input.clone(),
                json: false,
            },
            Command::EmitObjectPlan {
                input: input.clone(),
                json: false,
            },
            Command::VerifyObjectPlan {
                input: input.clone(),
                json: false,
            },
            Command::ObjectWriterReadiness {
                input: input.clone(),
                json: false,
            },
            Command::EmitObject {
                input: input.clone(),
                json: false,
            },
            Command::VerifyObjectEmit {
                input: input.clone(),
                json: false,
            },
            Command::VerifyObjectWriterInput {
                input: input.clone(),
                json: false,
            },
            Command::ObjectWriterDryRun {
                input: input.clone(),
                json: false,
            },
            Command::EmitObjectWriterDryRun {
                input: input.clone(),
                json: false,
            },
            Command::VerifyObjectWriterDryRun {
                input: input.clone(),
                json: false,
            },
            Command::ObjectByteLayout {
                input: input.clone(),
                json: false,
            },
            Command::EmitObjectByteLayout {
                input: input.clone(),
                json: false,
            },
            Command::VerifyObjectByteLayout {
                input: input.clone(),
                json: false,
            },
            Command::ObjectFileLayout {
                input: input.clone(),
                json: false,
            },
            Command::EmitObjectFileLayout {
                input: input.clone(),
                json: false,
            },
            Command::VerifyObjectFileLayout {
                input: input.clone(),
                json: false,
            },
            Command::ObjectImageDryRun {
                input: input.clone(),
                json: false,
            },
            Command::EmitObjectImageDryRun {
                input: input.clone(),
                json: false,
            },
            Command::VerifyObjectImageDryRun { input, json: false },
        ];

        for command in commands {
            assert!(
                run_object_command(&command).is_some(),
                "object command was not claimed by object dispatcher: {command:?}"
            );
        }
    }

    #[test]
    fn object_command_dispatcher_ignores_non_object_commands() {
        assert!(run_object_command(&Command::Status).is_none());
        assert!(run_object_command(&Command::ContainerPlan {
            input: missing_input(),
            json: false
        })
        .is_none());
    }
}
