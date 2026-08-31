use sha2::{Digest, Sha256};

use crate::{
    compiler_component_attestation_registry::encode_hex, verify_compiler_component_build,
    verify_compiler_component_build_image, verify_compiler_component_transition, ArtifactError,
    CompilerComponentBuild, CompilerComponentTransition,
    CompilerComponentTransitionVerificationInput, COMPILER_COMPONENT_STAGE0_ROLE,
    COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE, COMPILER_COMPONENT_TRANSITION_PROTOCOL,
};

#[path = "compiler_component_dispatch_parse.rs"]
mod parse;

pub use parse::{
    parse_compiler_component_dispatch_receipt,
    parse_compiler_component_dispatch_receipt_from_source,
};

pub const COMPILER_COMPONENT_DISPATCH_PROTOCOL: &str = "nuis-compiler-component-dispatch-v1";
pub const COMPILER_COMPONENT_DISPATCH_FILE: &str = "nuis.compiler-component-dispatch.toml";
pub const COMPILER_COMPONENT_DISPATCH_DRIVER_CONTRACT: &str =
    "nuis-stage0-stage1-verified-dispatch-v1";
pub const COMPILER_COMPONENT_DISPATCH_AUTHORITY: &str =
    "verified-transition-runtime-evidence-no-state-mutation";
pub const COMPILER_COMPONENT_DISPATCH_INVENTORY_CONTRACT: &str =
    "unordered-exact-current-forward-build-images-v1";
pub const COMPILER_COMPONENT_DISPATCH_REQUEST_CONTRACT: &str =
    "nuis-help-frontdoor-closed-stdin-v1";
pub const COMPILER_COMPONENT_DISPATCH_REQUEST_ARGUMENT: &str = "help";
pub const COMPILER_COMPONENT_DISPATCH_VERDICT: &str = "current-executed-forward-retained";

const CURRENT_SELECTOR: &str = "current";
const FORWARD_SELECTOR: &str = "forward";
const CLOSED_STDIN_CONTRACT: &str = "closed-stdin-v1";

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentDispatchCandidate<'a> {
    pub component: &'a CompilerComponentBuild,
    pub compiler_image: &'a [u8],
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentDispatchResolution<'a> {
    current: CompilerComponentDispatchCandidate<'a>,
    forward: CompilerComponentDispatchCandidate<'a>,
}

impl<'a> CompilerComponentDispatchResolution<'a> {
    pub fn current(&self) -> CompilerComponentDispatchCandidate<'a> {
        self.current
    }

    pub fn forward(&self) -> CompilerComponentDispatchCandidate<'a> {
        self.forward
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentDispatchReceiptInput<'a, 'b> {
    pub transition: &'a CompilerComponentTransition,
    pub resolution: &'a CompilerComponentDispatchResolution<'b>,
    pub exit_code: usize,
    pub stdout: &'a [u8],
    pub stderr: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentDispatchReceipt {
    pub protocol: String,
    pub driver_contract: String,
    pub authority: String,
    pub inventory_contract: String,
    pub request_contract: String,
    pub transition_protocol: String,
    pub transition_generation: usize,
    pub transition_proof_sha256: String,
    pub component_id: String,
    pub selected_selector: String,
    pub selected_stage_role: String,
    pub selected_reproducible_build_sha256: String,
    pub selected_record_sha256: String,
    pub selected_compiler_image_bytes: usize,
    pub selected_compiler_image_sha256: String,
    pub forward_selector: String,
    pub forward_stage_role: String,
    pub forward_reproducible_build_sha256: String,
    pub forward_record_sha256: String,
    pub forward_compiler_image_bytes: usize,
    pub forward_compiler_image_sha256: String,
    pub argument_count: usize,
    pub argument_0: String,
    pub stdin_contract: String,
    pub exit_code: usize,
    pub stdout_bytes: usize,
    pub stdout_sha256: String,
    pub stderr_bytes: usize,
    pub stderr_sha256: String,
    pub verdict: String,
    pub dispatch_sha256: String,
}

pub fn resolve_compiler_component_dispatch<'a>(
    transition: &CompilerComponentTransition,
    verification: CompilerComponentTransitionVerificationInput<'_>,
    candidates: &'a [CompilerComponentDispatchCandidate<'a>],
) -> Result<CompilerComponentDispatchResolution<'a>, ArtifactError> {
    verify_compiler_component_transition(transition, verification)?;
    if candidates.len() != 2 {
        return Err(ArtifactError::new(
            "compiler component dispatch requires exactly current and forward build images",
        ));
    }

    let mut current = None;
    let mut forward = None;
    for candidate in candidates {
        verify_compiler_component_build(candidate.component)?;
        verify_compiler_component_build_image(candidate.component, candidate.compiler_image)?;
        if candidate.component.component_id != transition.component_id
            || candidate.component.native_binary_sha256 != transition.native_output_sha256
        {
            return Err(ArtifactError::new(
                "compiler component dispatch candidate does not match the signed transition component",
            ));
        }
        if candidate.component.stage_role == COMPILER_COMPONENT_STAGE0_ROLE
            && candidate.component.reproducible_build_sha256
                == transition.current_reproducible_build_sha256
        {
            if current.replace(*candidate).is_some() {
                return Err(ArtifactError::new(
                    "compiler component dispatch repeats the signed current target",
                ));
            }
        } else if candidate.component.stage_role == COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
            && candidate.component.reproducible_build_sha256
                == transition.forward_reproducible_build_sha256
        {
            if forward.replace(*candidate).is_some() {
                return Err(ArtifactError::new(
                    "compiler component dispatch repeats the signed forward target",
                ));
            }
        } else {
            return Err(ArtifactError::new(
                "compiler component dispatch candidate is not selected by the signed transition",
            ));
        }
    }

    let current = current.ok_or_else(|| {
        ArtifactError::new("compiler component dispatch is missing the signed current target")
    })?;
    let forward = forward.ok_or_else(|| {
        ArtifactError::new("compiler component dispatch is missing the signed forward target")
    })?;
    verify_dispatch_pair(transition, current.component, forward.component)?;
    Ok(CompilerComponentDispatchResolution { current, forward })
}

pub fn build_compiler_component_dispatch_receipt(
    input: CompilerComponentDispatchReceiptInput<'_, '_>,
) -> Result<CompilerComponentDispatchReceipt, ArtifactError> {
    if input.exit_code != 0 || input.stdout.is_empty() || !input.stderr.is_empty() {
        return Err(ArtifactError::new(
            "compiler component dispatch requires exit 0, non-empty stdout, and empty stderr",
        ));
    }
    let current = input.resolution.current();
    let forward = input.resolution.forward();
    verify_dispatch_pair(input.transition, current.component, forward.component)?;

    let mut receipt = CompilerComponentDispatchReceipt {
        protocol: COMPILER_COMPONENT_DISPATCH_PROTOCOL.to_owned(),
        driver_contract: COMPILER_COMPONENT_DISPATCH_DRIVER_CONTRACT.to_owned(),
        authority: COMPILER_COMPONENT_DISPATCH_AUTHORITY.to_owned(),
        inventory_contract: COMPILER_COMPONENT_DISPATCH_INVENTORY_CONTRACT.to_owned(),
        request_contract: COMPILER_COMPONENT_DISPATCH_REQUEST_CONTRACT.to_owned(),
        transition_protocol: input.transition.protocol.clone(),
        transition_generation: input.transition.generation,
        transition_proof_sha256: input.transition.proof_sha256.clone(),
        component_id: input.transition.component_id.clone(),
        selected_selector: input.transition.current_selector.clone(),
        selected_stage_role: current.component.stage_role.clone(),
        selected_reproducible_build_sha256: current.component.reproducible_build_sha256.clone(),
        selected_record_sha256: current.component.record_sha256.clone(),
        selected_compiler_image_bytes: current.compiler_image.len(),
        selected_compiler_image_sha256: current.component.compiler_image_sha256.clone(),
        forward_selector: input.transition.forward_selector.clone(),
        forward_stage_role: forward.component.stage_role.clone(),
        forward_reproducible_build_sha256: forward.component.reproducible_build_sha256.clone(),
        forward_record_sha256: forward.component.record_sha256.clone(),
        forward_compiler_image_bytes: forward.compiler_image.len(),
        forward_compiler_image_sha256: forward.component.compiler_image_sha256.clone(),
        argument_count: 1,
        argument_0: COMPILER_COMPONENT_DISPATCH_REQUEST_ARGUMENT.to_owned(),
        stdin_contract: CLOSED_STDIN_CONTRACT.to_owned(),
        exit_code: input.exit_code,
        stdout_bytes: input.stdout.len(),
        stdout_sha256: sha256_hex(input.stdout),
        stderr_bytes: input.stderr.len(),
        stderr_sha256: sha256_hex(input.stderr),
        verdict: COMPILER_COMPONENT_DISPATCH_VERDICT.to_owned(),
        dispatch_sha256: String::new(),
    };
    receipt.dispatch_sha256 = dispatch_identity(&receipt);
    validate_dispatch_receipt(&receipt)?;
    Ok(receipt)
}

pub fn render_compiler_component_dispatch_receipt(
    receipt: &CompilerComponentDispatchReceipt,
) -> String {
    format!(
        "protocol = \"{}\"\ndriver_contract = \"{}\"\nauthority = \"{}\"\ninventory_contract = \"{}\"\nrequest_contract = \"{}\"\ntransition_protocol = \"{}\"\ntransition_generation = {}\ntransition_proof_sha256 = \"{}\"\ncomponent_id = \"{}\"\nselected_selector = \"{}\"\nselected_stage_role = \"{}\"\nselected_reproducible_build_sha256 = \"{}\"\nselected_record_sha256 = \"{}\"\nselected_compiler_image_bytes = {}\nselected_compiler_image_sha256 = \"{}\"\nforward_selector = \"{}\"\nforward_stage_role = \"{}\"\nforward_reproducible_build_sha256 = \"{}\"\nforward_record_sha256 = \"{}\"\nforward_compiler_image_bytes = {}\nforward_compiler_image_sha256 = \"{}\"\nargument_count = {}\nargument_0 = \"{}\"\nstdin_contract = \"{}\"\nexit_code = {}\nstdout_bytes = {}\nstdout_sha256 = \"{}\"\nstderr_bytes = {}\nstderr_sha256 = \"{}\"\nverdict = \"{}\"\ndispatch_sha256 = \"{}\"\n",
        receipt.protocol,
        receipt.driver_contract,
        receipt.authority,
        receipt.inventory_contract,
        receipt.request_contract,
        receipt.transition_protocol,
        receipt.transition_generation,
        receipt.transition_proof_sha256,
        crate::toml::escape_toml_string(&receipt.component_id),
        receipt.selected_selector,
        receipt.selected_stage_role,
        receipt.selected_reproducible_build_sha256,
        receipt.selected_record_sha256,
        receipt.selected_compiler_image_bytes,
        receipt.selected_compiler_image_sha256,
        receipt.forward_selector,
        receipt.forward_stage_role,
        receipt.forward_reproducible_build_sha256,
        receipt.forward_record_sha256,
        receipt.forward_compiler_image_bytes,
        receipt.forward_compiler_image_sha256,
        receipt.argument_count,
        receipt.argument_0,
        receipt.stdin_contract,
        receipt.exit_code,
        receipt.stdout_bytes,
        receipt.stdout_sha256,
        receipt.stderr_bytes,
        receipt.stderr_sha256,
        receipt.verdict,
        receipt.dispatch_sha256,
    )
}

fn verify_dispatch_pair(
    transition: &CompilerComponentTransition,
    current: &CompilerComponentBuild,
    forward: &CompilerComponentBuild,
) -> Result<(), ArtifactError> {
    if transition.protocol != COMPILER_COMPONENT_TRANSITION_PROTOCOL
        || transition.current_selector != CURRENT_SELECTOR
        || transition.forward_selector != FORWARD_SELECTOR
        || current.component_id != transition.component_id
        || forward.component_id != transition.component_id
        || current.stage_role != COMPILER_COMPONENT_STAGE0_ROLE
        || forward.stage_role != COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        || current.reproducible_build_sha256 != transition.current_reproducible_build_sha256
        || forward.reproducible_build_sha256 != transition.forward_reproducible_build_sha256
        || forward.compiler_image_sha256 != transition.candidate_compiler_image_sha256
        || current.native_binary_sha256 != transition.native_output_sha256
        || forward.native_binary_sha256 != transition.native_output_sha256
    {
        return Err(ArtifactError::new(
            "compiler component dispatch pair does not match the signed generation-two targets",
        ));
    }
    if current.bootstrap_subset_protocol != forward.bootstrap_subset_protocol
        || current.component_domain != forward.component_domain
        || current.component_unit != forward.component_unit
        || current.stage_handoff_bundle_sha256 != forward.stage_handoff_bundle_sha256
        || current.dependency_closure_sha256 != forward.dependency_closure_sha256
        || current.record_sha256 == forward.record_sha256
    {
        return Err(ArtifactError::new(
            "compiler component dispatch current and forward builds are not one coherent component pair",
        ));
    }
    Ok(())
}

pub(super) fn validate_dispatch_receipt(
    receipt: &CompilerComponentDispatchReceipt,
) -> Result<(), ArtifactError> {
    if receipt.protocol != COMPILER_COMPONENT_DISPATCH_PROTOCOL
        || receipt.driver_contract != COMPILER_COMPONENT_DISPATCH_DRIVER_CONTRACT
        || receipt.authority != COMPILER_COMPONENT_DISPATCH_AUTHORITY
        || receipt.inventory_contract != COMPILER_COMPONENT_DISPATCH_INVENTORY_CONTRACT
        || receipt.request_contract != COMPILER_COMPONENT_DISPATCH_REQUEST_CONTRACT
        || receipt.transition_protocol != COMPILER_COMPONENT_TRANSITION_PROTOCOL
        || receipt.transition_generation != 2
        || receipt.selected_selector != CURRENT_SELECTOR
        || receipt.selected_stage_role != COMPILER_COMPONENT_STAGE0_ROLE
        || receipt.forward_selector != FORWARD_SELECTOR
        || receipt.forward_stage_role != COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        || receipt.argument_count != 1
        || receipt.argument_0 != COMPILER_COMPONENT_DISPATCH_REQUEST_ARGUMENT
        || receipt.stdin_contract != CLOSED_STDIN_CONTRACT
        || receipt.verdict != COMPILER_COMPONENT_DISPATCH_VERDICT
    {
        return Err(ArtifactError::new(
            "compiler component dispatch receipt declares an unsupported contract",
        ));
    }
    if receipt.component_id.is_empty()
        || receipt.component_id.trim() != receipt.component_id
        || receipt.component_id.chars().any(char::is_control)
        || receipt.selected_compiler_image_bytes == 0
        || receipt.forward_compiler_image_bytes == 0
        || receipt.selected_reproducible_build_sha256 == receipt.forward_reproducible_build_sha256
        || receipt.selected_record_sha256 == receipt.forward_record_sha256
    {
        return Err(ArtifactError::new(
            "compiler component dispatch receipt has invalid component targets",
        ));
    }
    if receipt.exit_code != 0
        || receipt.stdout_bytes == 0
        || receipt.stderr_bytes != 0
        || receipt.stderr_sha256 != sha256_hex(&[])
    {
        return Err(ArtifactError::new(
            "compiler component dispatch receipt has an invalid execution result",
        ));
    }
    for (label, value) in [
        ("transition proof", receipt.transition_proof_sha256.as_str()),
        (
            "selected reproducible build",
            receipt.selected_reproducible_build_sha256.as_str(),
        ),
        ("selected record", receipt.selected_record_sha256.as_str()),
        (
            "selected compiler image",
            receipt.selected_compiler_image_sha256.as_str(),
        ),
        (
            "forward reproducible build",
            receipt.forward_reproducible_build_sha256.as_str(),
        ),
        ("forward record", receipt.forward_record_sha256.as_str()),
        (
            "forward compiler image",
            receipt.forward_compiler_image_sha256.as_str(),
        ),
        ("stdout", receipt.stdout_sha256.as_str()),
        ("stderr", receipt.stderr_sha256.as_str()),
        ("dispatch", receipt.dispatch_sha256.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    if dispatch_identity(receipt) != receipt.dispatch_sha256 {
        return Err(ArtifactError::new(
            "compiler component dispatch receipt identity mismatch",
        ));
    }
    Ok(())
}

fn dispatch_identity(receipt: &CompilerComponentDispatchReceipt) -> String {
    let mut hash = Sha256::new();
    for value in [
        receipt.protocol.as_bytes(),
        receipt.driver_contract.as_bytes(),
        receipt.authority.as_bytes(),
        receipt.inventory_contract.as_bytes(),
        receipt.request_contract.as_bytes(),
        receipt.transition_protocol.as_bytes(),
        receipt.transition_proof_sha256.as_bytes(),
        receipt.component_id.as_bytes(),
        receipt.selected_selector.as_bytes(),
        receipt.selected_stage_role.as_bytes(),
        receipt.selected_reproducible_build_sha256.as_bytes(),
        receipt.selected_record_sha256.as_bytes(),
        receipt.selected_compiler_image_sha256.as_bytes(),
        receipt.forward_selector.as_bytes(),
        receipt.forward_stage_role.as_bytes(),
        receipt.forward_reproducible_build_sha256.as_bytes(),
        receipt.forward_record_sha256.as_bytes(),
        receipt.forward_compiler_image_sha256.as_bytes(),
        receipt.argument_0.as_bytes(),
        receipt.stdin_contract.as_bytes(),
        receipt.stdout_sha256.as_bytes(),
        receipt.stderr_sha256.as_bytes(),
        receipt.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        receipt.transition_generation,
        receipt.selected_compiler_image_bytes,
        receipt.forward_compiler_image_bytes,
        receipt.argument_count,
        receipt.exit_code,
        receipt.stdout_bytes,
        receipt.stderr_bytes,
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    encode_hex(&hash.finalize())
}

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}

fn validate_sha256(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(ArtifactError::new(format!(
            "compiler component dispatch {label} must be lowercase SHA-256"
        )))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "compiler_component_dispatch_tests.rs"]
mod tests;
