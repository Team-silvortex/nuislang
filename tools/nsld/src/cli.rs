use super::cli_command_parse::parse_named_input_command;
use super::cli_usage::usage;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
    Status,
    Plan {
        input: PathBuf,
        json: bool,
    },
    Check {
        input: PathBuf,
        json: bool,
    },
    CheckNextAction {
        input: PathBuf,
        json: bool,
    },
    Drive {
        input: PathBuf,
        json: bool,
        apply: bool,
        until_clean: bool,
    },
    ArtifactChain {
        input: PathBuf,
        json: bool,
    },
    Closure {
        input: PathBuf,
        json: bool,
    },
    EmitClosure {
        input: PathBuf,
        json: bool,
    },
    VerifyClosure {
        input: PathBuf,
        json: bool,
    },
    FinalStagePlan {
        input: PathBuf,
        json: bool,
    },
    EmitFinalStagePlan {
        input: PathBuf,
        json: bool,
    },
    VerifyFinalStagePlan {
        input: PathBuf,
        json: bool,
    },
    FinalExecutableReadiness {
        input: PathBuf,
        json: bool,
    },
    FinalExecutableWriterPlan {
        input: PathBuf,
        json: bool,
    },
    EmitFinalExecutableWriterInput {
        input: PathBuf,
        json: bool,
    },
    VerifyFinalExecutableWriterInput {
        input: PathBuf,
        json: bool,
    },
    FinalExecutableHostDryRun {
        input: PathBuf,
        json: bool,
    },
    FinalExecutableHostInvokePlan {
        input: PathBuf,
        json: bool,
    },
    EmitFinalExecutableHostInvokePlan {
        input: PathBuf,
        json: bool,
    },
    VerifyFinalExecutableHostInvokePlan {
        input: PathBuf,
        json: bool,
    },
    FinalExecutableLayout {
        input: PathBuf,
        json: bool,
    },
    EmitFinalExecutableLayout {
        input: PathBuf,
        json: bool,
    },
    VerifyFinalExecutableLayout {
        input: PathBuf,
        json: bool,
    },
    FinalExecutableImageDryRun {
        input: PathBuf,
        json: bool,
    },
    EmitFinalExecutableImageDryRun {
        input: PathBuf,
        json: bool,
    },
    VerifyFinalExecutableImageDryRun {
        input: PathBuf,
        json: bool,
    },
    EmitFinalExecutablePipeline {
        input: PathBuf,
        json: bool,
    },
    VerifyFinalExecutablePipeline {
        input: PathBuf,
        json: bool,
    },
    EmitFinalExecutable {
        input: PathBuf,
        json: bool,
    },
    VerifyFinalExecutableEmit {
        input: PathBuf,
        json: bool,
    },
    FinalExecutableOutput {
        input: PathBuf,
        json: bool,
    },
    FinalExecutableLauncherManifest {
        input: PathBuf,
        json: bool,
    },
    EmitFinalExecutableLauncherManifest {
        input: PathBuf,
        json: bool,
    },
    VerifyFinalExecutableLauncherManifest {
        input: PathBuf,
        json: bool,
    },
    FinalExecutableLauncherDryRun {
        input: PathBuf,
        json: bool,
    },
    EmitFinalExecutableLauncherDryRun {
        input: PathBuf,
        json: bool,
    },
    VerifyFinalExecutableLauncherDryRun {
        input: PathBuf,
        json: bool,
    },
    Prepare {
        input: PathBuf,
        json: bool,
    },
    AssemblePlan {
        input: PathBuf,
        json: bool,
    },
    EmitAssemblePlan {
        input: PathBuf,
        json: bool,
    },
    VerifyAssemblePlan {
        input: PathBuf,
        json: bool,
    },
    SectionManifest {
        input: PathBuf,
        json: bool,
    },
    EmitSectionManifest {
        input: PathBuf,
        json: bool,
    },
    VerifySectionManifest {
        input: PathBuf,
        json: bool,
    },
    ObjectPlan {
        input: PathBuf,
        json: bool,
    },
    EmitObjectPlan {
        input: PathBuf,
        json: bool,
    },
    VerifyObjectPlan {
        input: PathBuf,
        json: bool,
    },
    ObjectWriterReadiness {
        input: PathBuf,
        json: bool,
    },
    EmitObject {
        input: PathBuf,
        json: bool,
    },
    VerifyObjectEmit {
        input: PathBuf,
        json: bool,
    },
    VerifyObjectOutput {
        input: PathBuf,
        json: bool,
    },
    VerifyObjectWriterInput {
        input: PathBuf,
        json: bool,
    },
    ObjectWriterDryRun {
        input: PathBuf,
        json: bool,
    },
    EmitObjectWriterDryRun {
        input: PathBuf,
        json: bool,
    },
    VerifyObjectWriterDryRun {
        input: PathBuf,
        json: bool,
    },
    ObjectByteLayout {
        input: PathBuf,
        json: bool,
    },
    EmitObjectByteLayout {
        input: PathBuf,
        json: bool,
    },
    VerifyObjectByteLayout {
        input: PathBuf,
        json: bool,
    },
    ObjectFileLayout {
        input: PathBuf,
        json: bool,
    },
    EmitObjectFileLayout {
        input: PathBuf,
        json: bool,
    },
    VerifyObjectFileLayout {
        input: PathBuf,
        json: bool,
    },
    ObjectImageDryRun {
        input: PathBuf,
        json: bool,
    },
    EmitObjectImageDryRun {
        input: PathBuf,
        json: bool,
    },
    VerifyObjectImageDryRun {
        input: PathBuf,
        json: bool,
    },
    ContainerPlan {
        input: PathBuf,
        json: bool,
    },
    EmitContainerPlan {
        input: PathBuf,
        json: bool,
    },
    VerifyContainerPlan {
        input: PathBuf,
        json: bool,
    },
    Container {
        input: PathBuf,
        json: bool,
    },
    EmitContainer {
        input: PathBuf,
        json: bool,
    },
    VerifyContainer {
        input: PathBuf,
        json: bool,
    },
    Bundle {
        input: PathBuf,
        json: bool,
    },
    EmitBundle {
        input: PathBuf,
        json: bool,
    },
    VerifyBundle {
        input: PathBuf,
        json: bool,
    },
    Units {
        input: PathBuf,
        json: bool,
    },
    EmitUnits {
        input: PathBuf,
        json: bool,
    },
    VerifyUnits {
        input: PathBuf,
        json: bool,
    },
    Inputs {
        input: PathBuf,
        json: bool,
    },
    EmitInputs {
        input: PathBuf,
        json: bool,
    },
    VerifyInputs {
        input: PathBuf,
        json: bool,
    },
}

pub(crate) fn parse_args<I>(mut args: I) -> Result<Command, String>
where
    I: Iterator<Item = String>,
{
    let Some(command) = args.next() else {
        return Ok(Command::Status);
    };
    match command.as_str() {
        "status" => Ok(Command::Status),
        "--help" | "-h" | "help" => Err(usage().to_owned()),
        other => parse_named_input_command(other, args)
            .unwrap_or_else(|| Err(format!("unknown nsld command `{other}`\n{}", usage()))),
    }
}

pub(crate) fn resolve_manifest_input(input: &Path) -> Result<PathBuf, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Command, String> {
        parse_args(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn parses_emit_inputs_as_explicit_materialization_command() {
        assert_eq!(
            parse(&["emit-inputs", "out", "--json"]).unwrap(),
            Command::EmitInputs {
                input: PathBuf::from("out"),
                json: true,
            }
        );
    }

    #[test]
    fn keeps_inputs_as_legacy_materialization_alias() {
        assert_eq!(
            parse(&["inputs", "out"]).unwrap(),
            Command::Inputs {
                input: PathBuf::from("out"),
                json: false,
            }
        );
    }
}
