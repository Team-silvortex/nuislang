use super::cli::Command;
use super::cli_usage::usage;
use std::path::PathBuf;

pub(crate) fn parse_named_input_command<I>(
    command: &str,
    args: I,
) -> Option<Result<Command, String>>
where
    I: Iterator<Item = String>,
{
    let (input, json, apply, until_clean) = match parse_input_flags(command, args) {
        Ok(parsed) => parsed,
        Err(err) => return Some(Err(err)),
    };
    let parsed = match command {
        "plan" => Command::Plan { input, json },
        "check" => Command::Check { input, json },
        "check-next-action" => Command::CheckNextAction { input, json },
        "drive" => Command::Drive {
            input,
            json,
            apply,
            until_clean,
        },
        "artifact-chain" => Command::ArtifactChain { input, json },
        "closure" => Command::Closure { input, json },
        "emit-closure" => Command::EmitClosure { input, json },
        "verify-closure" => Command::VerifyClosure { input, json },
        "final-stage-plan" => Command::FinalStagePlan { input, json },
        "emit-final-stage-plan" => Command::EmitFinalStagePlan { input, json },
        "verify-final-stage-plan" => Command::VerifyFinalStagePlan { input, json },
        "final-executable-readiness" => Command::FinalExecutableReadiness { input, json },
        "final-executable-writer-plan" => Command::FinalExecutableWriterPlan { input, json },
        "emit-final-executable-writer-input" => {
            Command::EmitFinalExecutableWriterInput { input, json }
        }
        "verify-final-executable-writer-input" => {
            Command::VerifyFinalExecutableWriterInput { input, json }
        }
        "final-executable-host-dry-run" => Command::FinalExecutableHostDryRun { input, json },
        "final-executable-host-invoke-plan" => {
            Command::FinalExecutableHostInvokePlan { input, json }
        }
        "emit-final-executable-host-invoke-plan" => {
            Command::EmitFinalExecutableHostInvokePlan { input, json }
        }
        "verify-final-executable-host-invoke-plan" => {
            Command::VerifyFinalExecutableHostInvokePlan { input, json }
        }
        "final-executable-layout" => Command::FinalExecutableLayout { input, json },
        "emit-final-executable-layout" => Command::EmitFinalExecutableLayout { input, json },
        "verify-final-executable-layout" => Command::VerifyFinalExecutableLayout { input, json },
        "final-executable-image-dry-run" => Command::FinalExecutableImageDryRun { input, json },
        "emit-final-executable-image-dry-run" => {
            Command::EmitFinalExecutableImageDryRun { input, json }
        }
        "verify-final-executable-image-dry-run" => {
            Command::VerifyFinalExecutableImageDryRun { input, json }
        }
        "emit-final-executable-pipeline" => Command::EmitFinalExecutablePipeline { input, json },
        "verify-final-executable-pipeline" => {
            Command::VerifyFinalExecutablePipeline { input, json }
        }
        "emit-final-executable" => Command::EmitFinalExecutable { input, json },
        "verify-final-executable-emit" => Command::VerifyFinalExecutableEmit { input, json },
        "final-executable-output" => Command::FinalExecutableOutput { input, json },
        "final-executable-launcher-manifest" => {
            Command::FinalExecutableLauncherManifest { input, json }
        }
        "emit-final-executable-launcher-manifest" => {
            Command::EmitFinalExecutableLauncherManifest { input, json }
        }
        "verify-final-executable-launcher-manifest" => {
            Command::VerifyFinalExecutableLauncherManifest { input, json }
        }
        "final-executable-launcher-dry-run" => {
            Command::FinalExecutableLauncherDryRun { input, json }
        }
        "emit-final-executable-launcher-dry-run" => {
            Command::EmitFinalExecutableLauncherDryRun { input, json }
        }
        "verify-final-executable-launcher-dry-run" => {
            Command::VerifyFinalExecutableLauncherDryRun { input, json }
        }
        "prepare" => Command::Prepare { input, json },
        "assemble-plan" => Command::AssemblePlan { input, json },
        "emit-assemble-plan" => Command::EmitAssemblePlan { input, json },
        "verify-assemble-plan" => Command::VerifyAssemblePlan { input, json },
        "section-manifest" => Command::SectionManifest { input, json },
        "emit-section-manifest" => Command::EmitSectionManifest { input, json },
        "verify-section-manifest" => Command::VerifySectionManifest { input, json },
        "object-plan" => Command::ObjectPlan { input, json },
        "emit-object-plan" => Command::EmitObjectPlan { input, json },
        "verify-object-plan" => Command::VerifyObjectPlan { input, json },
        "object-writer-readiness" => Command::ObjectWriterReadiness { input, json },
        "emit-object" | "emit-native-object" => Command::EmitObject { input, json },
        "verify-object-emit" => Command::VerifyObjectEmit { input, json },
        "verify-object-output" => Command::VerifyObjectOutput { input, json },
        "verify-object-writer-input" => Command::VerifyObjectWriterInput { input, json },
        "object-writer-dry-run" => Command::ObjectWriterDryRun { input, json },
        "emit-object-writer-dry-run" => Command::EmitObjectWriterDryRun { input, json },
        "verify-object-writer-dry-run" => Command::VerifyObjectWriterDryRun { input, json },
        "object-byte-layout" => Command::ObjectByteLayout { input, json },
        "emit-object-byte-layout" => Command::EmitObjectByteLayout { input, json },
        "verify-object-byte-layout" => Command::VerifyObjectByteLayout { input, json },
        "object-file-layout" => Command::ObjectFileLayout { input, json },
        "emit-object-file-layout" => Command::EmitObjectFileLayout { input, json },
        "verify-object-file-layout" => Command::VerifyObjectFileLayout { input, json },
        "object-image-dry-run" => Command::ObjectImageDryRun { input, json },
        "emit-object-image-dry-run" => Command::EmitObjectImageDryRun { input, json },
        "verify-object-image-dry-run" => Command::VerifyObjectImageDryRun { input, json },
        "container-plan" => Command::ContainerPlan { input, json },
        "emit-container-plan" => Command::EmitContainerPlan { input, json },
        "verify-container-plan" => Command::VerifyContainerPlan { input, json },
        "container" => Command::Container { input, json },
        "emit-container" => Command::EmitContainer { input, json },
        "verify-container" => Command::VerifyContainer { input, json },
        "bundle" => Command::Bundle { input, json },
        "emit-bundle" => Command::EmitBundle { input, json },
        "verify-bundle" => Command::VerifyBundle { input, json },
        "units" => Command::Units { input, json },
        "emit-units" => Command::EmitUnits { input, json },
        "verify-units" => Command::VerifyUnits { input, json },
        "inputs" => Command::Inputs { input, json },
        "emit-inputs" => Command::EmitInputs { input, json },
        "verify-inputs" => Command::VerifyInputs { input, json },
        _ => return None,
    };
    Some(Ok(parsed))
}

fn parse_input_flags<I>(command: &str, args: I) -> Result<(PathBuf, bool, bool, bool), String>
where
    I: Iterator<Item = String>,
{
    let mut json = false;
    let mut apply = false;
    let mut until_clean = false;
    let mut input = None;
    for arg in args {
        if arg == "--json" {
            json = true;
        } else if arg == "--apply" {
            if command != "drive" {
                return Err(format!("unexpected argument `{arg}`"));
            }
            apply = true;
        } else if arg == "--until-clean" {
            if command != "drive" {
                return Err(format!("unexpected argument `{arg}`"));
            }
            until_clean = true;
        } else if input.is_none() {
            input = Some(PathBuf::from(arg));
        } else {
            return Err(format!("unexpected argument `{arg}`"));
        }
    }
    if until_clean && !apply {
        return Err("`--until-clean` requires `--apply`".to_owned());
    }
    let input = input.ok_or_else(|| usage().to_owned())?;
    Ok((input, json, apply, until_clean))
}
