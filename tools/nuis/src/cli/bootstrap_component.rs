use std::path::PathBuf;

use crate::bootstrap_candidate_compile_capability::BootstrapCandidateCompileCapabilityInput;
use crate::bootstrap_candidate_direct_compile::BootstrapCandidateDirectCompileInput;
use crate::bootstrap_candidate_preselection::BootstrapCandidatePreselectionInput;
use crate::bootstrap_candidate_successor::BootstrapCandidateSuccessorInput;
use crate::bootstrap_component_compile_dispatch::BootstrapComponentCompileDispatchInput;
use crate::bootstrap_component_dispatch::BootstrapComponentDispatchInput;
use crate::bootstrap_component_replacement::BootstrapComponentTransitionVerificationInput;

use super::{support::parse_bootstrap_component_verification_prefix, CommandKind};

pub(super) fn parse_bootstrap_candidate_compile_capability(
    args: &mut impl Iterator<Item = String>,
) -> Result<CommandKind, String> {
    let usage = "usage: nuis bootstrap-candidate-compile-capability <candidate-build-root> <stage0-provider-image> <project-dir|nuis.toml> <fresh-build-output> <output>";
    let command = CommandKind::BootstrapCandidateCompileCapability(
        BootstrapCandidateCompileCapabilityInput {
            candidate_root: path(args, usage)?,
            provider_image: path(args, usage)?,
            project_input: path(args, usage)?,
            build_output: path(args, usage)?,
            output: path(args, usage)?,
        },
    );
    finish(args, usage, command)
}

pub(super) fn parse_bootstrap_candidate_direct_compile(
    args: &mut impl Iterator<Item = String>,
) -> Result<CommandKind, String> {
    let usage = "usage: nuis bootstrap-candidate-direct-compile <candidate-build-root> <front-end-result-output> <capability-output>";
    let command =
        CommandKind::BootstrapCandidateDirectCompile(BootstrapCandidateDirectCompileInput {
            candidate_root: path(args, usage)?,
            result_output: path(args, usage)?,
            capability_output: path(args, usage)?,
        });
    finish(args, usage, command)
}

pub(super) fn parse_bootstrap_candidate_preselection(
    args: &mut impl Iterator<Item = String>,
) -> Result<CommandKind, String> {
    let usage = "usage: nuis bootstrap-preselect-candidate <aggregate> <attestation> <attester-registry> <attester-registry-sha256> <attestation-challenge-sha256> <authorization> <authorizer-registry> <authorizer-registry-sha256> <authorization-challenge-sha256> <active-state> <transition> <transition-challenge-sha256> <candidate-build-root> <candidate-compile-capability> <preselection-challenge-sha256> <authorizer-id> <environment-id> <preselection-id> <output>";
    let verification = parse_bootstrap_component_verification_prefix(args, usage)?;
    let active_state = path(args, usage)?;
    let transition = path(args, usage)?;
    let transition_challenge_sha256 = args.next().ok_or_else(|| usage.to_owned())?;
    let command = CommandKind::BootstrapPreselectCandidate(BootstrapCandidatePreselectionInput {
        transition_verification: BootstrapComponentTransitionVerificationInput {
            verification,
            active_state,
            transition,
            transition_challenge_sha256,
        },
        candidate_root: path(args, usage)?,
        capability: path(args, usage)?,
        challenge_sha256: args.next().ok_or_else(|| usage.to_owned())?,
        authorizer_id: args.next().ok_or_else(|| usage.to_owned())?,
        environment_id: args.next().ok_or_else(|| usage.to_owned())?,
        preselection_id: args.next().ok_or_else(|| usage.to_owned())?,
        output: path(args, usage)?,
    });
    finish(args, usage, command)
}

pub(super) fn parse_bootstrap_candidate_successor(
    args: &mut impl Iterator<Item = String>,
) -> Result<CommandKind, String> {
    let usage = "usage: nuis bootstrap-sign-candidate-successor <aggregate> <attestation> <attester-registry> <attester-registry-sha256> <attestation-challenge-sha256> <authorization> <authorizer-registry> <authorizer-registry-sha256> <authorization-challenge-sha256> <active-state> <transition> <transition-challenge-sha256> <candidate-build-root> <candidate-compile-capability-v1> <preselection> <preselection-challenge-sha256> <direct-compile-capability-v2> <front-end-result> <successor-challenge-sha256> <authorizer-id> <environment-id> <successor-id> <output>";
    let verification = parse_bootstrap_component_verification_prefix(args, usage)?;
    let transition_verification = BootstrapComponentTransitionVerificationInput {
        verification,
        active_state: path(args, usage)?,
        transition: path(args, usage)?,
        transition_challenge_sha256: args.next().ok_or_else(|| usage.to_owned())?,
    };
    let command = CommandKind::BootstrapSignCandidateSuccessor(BootstrapCandidateSuccessorInput {
        transition_verification,
        candidate_root: path(args, usage)?,
        delegated_capability: path(args, usage)?,
        preselection: path(args, usage)?,
        preselection_challenge_sha256: args.next().ok_or_else(|| usage.to_owned())?,
        direct_capability: path(args, usage)?,
        frontend_result: path(args, usage)?,
        challenge_sha256: args.next().ok_or_else(|| usage.to_owned())?,
        authorizer_id: args.next().ok_or_else(|| usage.to_owned())?,
        environment_id: args.next().ok_or_else(|| usage.to_owned())?,
        successor_id: args.next().ok_or_else(|| usage.to_owned())?,
        output: path(args, usage)?,
    });
    finish(args, usage, command)
}

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

fn finish(
    args: &mut impl Iterator<Item = String>,
    usage: &str,
    command: CommandKind,
) -> Result<CommandKind, String> {
    if args.next().is_some() {
        Err(usage.to_owned())
    } else {
        Ok(command)
    }
}
