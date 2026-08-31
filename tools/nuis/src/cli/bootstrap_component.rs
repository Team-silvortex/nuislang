use std::path::PathBuf;

use crate::bootstrap_component_compile_dispatch::BootstrapComponentCompileDispatchInput;
use crate::bootstrap_component_dispatch::BootstrapComponentDispatchInput;
use crate::bootstrap_component_replacement::BootstrapComponentTransitionVerificationInput;

use super::{support::parse_bootstrap_component_verification_prefix, CommandKind};

pub(super) fn parse_bootstrap_component_dispatch(
    command: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<CommandKind, String> {
    let usage = match command {
        "bootstrap-dispatch-component" => "usage: nuis bootstrap-dispatch-component <aggregate> <attestation> <attester-registry> <attester-registry-sha256> <attestation-challenge-sha256> <authorization> <authorizer-registry> <authorizer-registry-sha256> <authorization-challenge-sha256> <active-state> <transition> <transition-challenge-sha256> <current-component> <current-image> <forward-component> <forward-image> <output>",
        "bootstrap-dispatch-compile" => "usage: nuis bootstrap-dispatch-compile <aggregate> <attestation> <attester-registry> <attester-registry-sha256> <attestation-challenge-sha256> <authorization> <authorizer-registry> <authorizer-registry-sha256> <authorization-challenge-sha256> <active-state> <transition> <transition-challenge-sha256> <current-component> <current-image> <forward-component> <forward-image> <project-dir|nuis.toml> <fresh-build-output> <output>",
        _ => return Err(format!("unsupported bootstrap component dispatch `{command}`")),
    };
    let verification = parse_bootstrap_component_verification_prefix(args, usage)?;
    let active_state = path(args, usage)?;
    let transition = path(args, usage)?;
    let transition_challenge_sha256 = args.next().ok_or_else(|| usage.to_owned())?;
    let transition_verification = BootstrapComponentTransitionVerificationInput {
        verification,
        active_state,
        transition,
        transition_challenge_sha256,
    };
    let current_component = path(args, usage)?;
    let current_image = path(args, usage)?;
    let forward_component = path(args, usage)?;
    let forward_image = path(args, usage)?;

    let command = if command == "bootstrap-dispatch-component" {
        CommandKind::BootstrapDispatchComponent(BootstrapComponentDispatchInput {
            transition_verification,
            current_component,
            current_image,
            forward_component,
            forward_image,
            output: path(args, usage)?,
        })
    } else {
        CommandKind::BootstrapDispatchCompile(BootstrapComponentCompileDispatchInput {
            transition_verification,
            current_component,
            current_image,
            forward_component,
            forward_image,
            project_input: path(args, usage)?,
            build_output: path(args, usage)?,
            output: path(args, usage)?,
        })
    };
    if args.next().is_some() {
        return Err(usage.to_owned());
    }
    Ok(command)
}

fn path(args: &mut impl Iterator<Item = String>, usage: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| usage.to_owned())
}
