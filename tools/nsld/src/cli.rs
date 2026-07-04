use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
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
    ObjectPlan { input: PathBuf, json: bool },
    EmitObjectPlan { input: PathBuf, json: bool },
    VerifyObjectPlan { input: PathBuf, json: bool },
    ObjectWriterReadiness { input: PathBuf, json: bool },
    EmitObject { input: PathBuf, json: bool },
    VerifyObjectEmit { input: PathBuf, json: bool },
    VerifyObjectOutput { input: PathBuf, json: bool },
    VerifyObjectWriterInput { input: PathBuf, json: bool },
    ObjectWriterDryRun { input: PathBuf, json: bool },
    EmitObjectWriterDryRun { input: PathBuf, json: bool },
    VerifyObjectWriterDryRun { input: PathBuf, json: bool },
    ObjectByteLayout { input: PathBuf, json: bool },
    EmitObjectByteLayout { input: PathBuf, json: bool },
    VerifyObjectByteLayout { input: PathBuf, json: bool },
    ObjectFileLayout { input: PathBuf, json: bool },
    EmitObjectFileLayout { input: PathBuf, json: bool },
    VerifyObjectFileLayout { input: PathBuf, json: bool },
    ObjectImageDryRun { input: PathBuf, json: bool },
    EmitObjectImageDryRun { input: PathBuf, json: bool },
    VerifyObjectImageDryRun { input: PathBuf, json: bool },
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
    EmitInputs { input: PathBuf, json: bool },
    VerifyInputs { input: PathBuf, json: bool },
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
        | "object-plan"
        | "emit-object-plan"
        | "verify-object-plan"
        | "object-writer-readiness"
        | "emit-object"
        | "verify-object-emit"
        | "verify-object-output"
        | "verify-object-writer-input"
        | "object-writer-dry-run"
        | "emit-object-writer-dry-run"
        | "verify-object-writer-dry-run"
        | "object-byte-layout"
        | "emit-object-byte-layout"
        | "verify-object-byte-layout"
        | "object-file-layout"
        | "emit-object-file-layout"
        | "verify-object-file-layout"
        | "object-image-dry-run"
        | "emit-object-image-dry-run"
        | "verify-object-image-dry-run"
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
        | "emit-inputs"
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
            let is_object_plan = command == "object-plan";
            let is_emit_object_plan = command == "emit-object-plan";
            let is_verify_object_plan = command == "verify-object-plan";
            let is_object_writer_readiness = command == "object-writer-readiness";
            let is_emit_object = command == "emit-object";
            let is_verify_object_emit = command == "verify-object-emit";
            let is_verify_object_output = command == "verify-object-output";
            let is_verify_object_writer_input = command == "verify-object-writer-input";
            let is_object_writer_dry_run = command == "object-writer-dry-run";
            let is_emit_object_writer_dry_run = command == "emit-object-writer-dry-run";
            let is_verify_object_writer_dry_run = command == "verify-object-writer-dry-run";
            let is_object_byte_layout = command == "object-byte-layout";
            let is_emit_object_byte_layout = command == "emit-object-byte-layout";
            let is_verify_object_byte_layout = command == "verify-object-byte-layout";
            let is_object_file_layout = command == "object-file-layout";
            let is_emit_object_file_layout = command == "emit-object-file-layout";
            let is_verify_object_file_layout = command == "verify-object-file-layout";
            let is_object_image_dry_run = command == "object-image-dry-run";
            let is_emit_object_image_dry_run = command == "emit-object-image-dry-run";
            let is_verify_object_image_dry_run = command == "verify-object-image-dry-run";
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
            let is_emit_inputs = command == "emit-inputs";
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
            } else if is_object_plan {
                Ok(Command::ObjectPlan { input, json })
            } else if is_emit_object_plan {
                Ok(Command::EmitObjectPlan { input, json })
            } else if is_verify_object_plan {
                Ok(Command::VerifyObjectPlan { input, json })
            } else if is_object_writer_readiness {
                Ok(Command::ObjectWriterReadiness { input, json })
            } else if is_emit_object {
                Ok(Command::EmitObject { input, json })
            } else if is_verify_object_emit {
                Ok(Command::VerifyObjectEmit { input, json })
            } else if is_verify_object_output {
                Ok(Command::VerifyObjectOutput { input, json })
            } else if is_verify_object_writer_input {
                Ok(Command::VerifyObjectWriterInput { input, json })
            } else if is_object_writer_dry_run {
                Ok(Command::ObjectWriterDryRun { input, json })
            } else if is_emit_object_writer_dry_run {
                Ok(Command::EmitObjectWriterDryRun { input, json })
            } else if is_verify_object_writer_dry_run {
                Ok(Command::VerifyObjectWriterDryRun { input, json })
            } else if is_object_byte_layout {
                Ok(Command::ObjectByteLayout { input, json })
            } else if is_emit_object_byte_layout {
                Ok(Command::EmitObjectByteLayout { input, json })
            } else if is_verify_object_byte_layout {
                Ok(Command::VerifyObjectByteLayout { input, json })
            } else if is_object_file_layout {
                Ok(Command::ObjectFileLayout { input, json })
            } else if is_emit_object_file_layout {
                Ok(Command::EmitObjectFileLayout { input, json })
            } else if is_verify_object_file_layout {
                Ok(Command::VerifyObjectFileLayout { input, json })
            } else if is_object_image_dry_run {
                Ok(Command::ObjectImageDryRun { input, json })
            } else if is_emit_object_image_dry_run {
                Ok(Command::EmitObjectImageDryRun { input, json })
            } else if is_verify_object_image_dry_run {
                Ok(Command::VerifyObjectImageDryRun { input, json })
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
            } else if is_emit_inputs {
                Ok(Command::EmitInputs { input, json })
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

fn usage() -> &'static str {
    concat!(
        "usage:\n",
        "  nsld status\n",
        "  nsld plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld check <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld closure <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld prepare <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld assemble-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-assemble-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-assemble-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld section-manifest <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-section-manifest <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-section-manifest <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld object-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-object-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-object-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld object-writer-readiness <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-object <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-object-emit <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-object-output <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-object-writer-input <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld object-writer-dry-run <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-object-writer-dry-run <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-object-writer-dry-run <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld object-byte-layout <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-object-byte-layout <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-object-byte-layout <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld object-file-layout <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-object-file-layout <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-object-file-layout <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld object-image-dry-run <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-object-image-dry-run <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-object-image-dry-run <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld container-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-container-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-container-plan <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld container <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-container <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-container <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld bundle <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-bundle <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-bundle <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld units <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-units <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-units <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld inputs <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld emit-inputs <nuis.build.manifest.toml|artifact-output-dir> [--json]\n",
        "  nsld verify-inputs <nuis.build.manifest.toml|artifact-output-dir> [--json]"
    )
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
