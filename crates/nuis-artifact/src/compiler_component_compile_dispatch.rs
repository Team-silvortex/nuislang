use std::{fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{
    compiler_component_attestation_registry::encode_hex,
    decode_nuis_compiled_artifact_binary,
    toml::{escape_toml_string, parse_required_toml_string, parse_required_toml_usize},
    verify_compiler_component_build, ArtifactError, CompilerComponentBuild,
    CompilerComponentDispatchResolution, CompilerComponentTransition, NuisCompiledArtifact,
    COMPILER_COMPONENT_DISPATCH_INVENTORY_CONTRACT, COMPILER_COMPONENT_STAGE0_ROLE,
    COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE, COMPILER_COMPONENT_TRANSITION_PROTOCOL,
};

pub const COMPILER_COMPONENT_COMPILE_DISPATCH_PROTOCOL: &str =
    "nuis-compiler-component-compile-dispatch-v1";
pub const COMPILER_COMPONENT_COMPILE_DISPATCH_FILE: &str =
    "nuis.compiler-component-compile-dispatch.toml";
pub const COMPILER_COMPONENT_COMPILE_DISPATCH_DRIVER_CONTRACT: &str =
    "nuis-stage0-stage1-verified-compile-dispatch-v1";
pub const COMPILER_COMPONENT_COMPILE_DISPATCH_AUTHORITY: &str =
    "verified-transition-current-rebuild-no-state-mutation";
pub const COMPILER_COMPONENT_COMPILE_REQUEST_CONTRACT: &str =
    "canonical-verified-stage0-component-rebuild-v1";
pub const COMPILER_COMPONENT_COMPILE_COMMAND: &str = "bootstrap-build";
pub const COMPILER_COMPONENT_COMPILE_ARGUMENT_CONTRACT: &str =
    "runtime-project-and-fresh-output-paths-not-persisted-v1";
pub const COMPILER_COMPONENT_COMPILED_ARTIFACT_IDENTITY_CONTRACT: &str =
    "nuis-compiled-artifact-semantic-identity-v1";
pub const COMPILER_COMPONENT_COMPILE_DISPATCH_VERDICT: &str = "current-compiled-forward-retained";

const CURRENT_SELECTOR: &str = "current";
const FORWARD_SELECTOR: &str = "forward";
const CLOSED_STDIN_CONTRACT: &str = "closed-stdin-v1";

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentCompileDispatchReceiptInput<'a, 'b> {
    pub transition: &'a CompilerComponentTransition,
    pub resolution: &'a CompilerComponentDispatchResolution<'b>,
    pub request: &'a CompilerComponentBuild,
    pub result: &'a CompilerComponentBuild,
    pub request_compiled_artifact: &'a [u8],
    pub result_compiled_artifact: &'a [u8],
    pub exit_code: usize,
    pub stdout: &'a [u8],
    pub stderr: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentCompileDispatchReceipt {
    pub protocol: String,
    pub driver_contract: String,
    pub authority: String,
    pub inventory_contract: String,
    pub request_contract: String,
    pub command: String,
    pub argument_contract: String,
    pub stdin_contract: String,
    pub transition_protocol: String,
    pub transition_generation: usize,
    pub transition_proof_sha256: String,
    pub bootstrap_subset_protocol: String,
    pub component_id: String,
    pub component_domain: String,
    pub component_unit: String,
    pub request_record_sha256: String,
    pub request_reproducible_build_sha256: String,
    pub request_dependency_count: usize,
    pub request_dependency_closure_sha256: String,
    pub request_stage_handoff_bundle_sha256: String,
    pub request_compiled_artifact_bytes: usize,
    pub request_compiled_artifact_sha256: String,
    pub request_native_binary_bytes: usize,
    pub request_native_binary_sha256: String,
    pub compiled_artifact_identity_contract: String,
    pub compiled_artifact_semantic_sha256: String,
    pub selected_selector: String,
    pub selected_stage_role: String,
    pub selected_record_sha256: String,
    pub selected_compiler_image_bytes: usize,
    pub selected_compiler_image_sha256: String,
    pub forward_selector: String,
    pub forward_stage_role: String,
    pub forward_reproducible_build_sha256: String,
    pub forward_record_sha256: String,
    pub forward_compiler_image_bytes: usize,
    pub forward_compiler_image_sha256: String,
    pub result_stage_role: String,
    pub result_record_sha256: String,
    pub result_compiler_image_bytes: usize,
    pub result_compiler_image_sha256: String,
    pub result_reproducible_build_sha256: String,
    pub result_dependency_count: usize,
    pub result_dependency_closure_sha256: String,
    pub result_stage_handoff_bundle_sha256: String,
    pub result_compiled_artifact_bytes: usize,
    pub result_compiled_artifact_sha256: String,
    pub result_native_binary_bytes: usize,
    pub result_native_binary_sha256: String,
    pub exit_code: usize,
    pub stdout_bytes: usize,
    pub stdout_sha256: String,
    pub stderr_bytes: usize,
    pub stderr_sha256: String,
    pub verdict: String,
    pub dispatch_sha256: String,
}

pub fn build_compiler_component_compile_dispatch_receipt(
    input: CompilerComponentCompileDispatchReceiptInput<'_, '_>,
) -> Result<CompilerComponentCompileDispatchReceipt, ArtifactError> {
    if input.exit_code != 0 || input.stdout.is_empty() || !input.stderr.is_empty() {
        return Err(ArtifactError::new(
            "compiler component compile dispatch requires exit 0, non-empty stdout, and empty stderr",
        ));
    }
    let current = input.resolution.current();
    let forward = input.resolution.forward();
    verify_compile_targets(input.transition, current.component, forward.component)?;
    verify_compile_result(input.request, current.component, input.result)?;
    verify_artifact_bytes(input.request, input.request_compiled_artifact)?;
    verify_artifact_bytes(input.result, input.result_compiled_artifact)?;
    let request_artifact = decode_nuis_compiled_artifact_binary(input.request_compiled_artifact)?;
    let result_artifact = decode_nuis_compiled_artifact_binary(input.result_compiled_artifact)?;
    let compiled_artifact_semantic_sha256 = compiled_artifact_semantic_identity(&request_artifact);
    if compiled_artifact_semantic_sha256 != compiled_artifact_semantic_identity(&result_artifact) {
        return Err(ArtifactError::new(
            "compiler component compile result changed the path-neutral compiled artifact semantics",
        ));
    }

    let mut receipt = CompilerComponentCompileDispatchReceipt {
        protocol: COMPILER_COMPONENT_COMPILE_DISPATCH_PROTOCOL.to_owned(),
        driver_contract: COMPILER_COMPONENT_COMPILE_DISPATCH_DRIVER_CONTRACT.to_owned(),
        authority: COMPILER_COMPONENT_COMPILE_DISPATCH_AUTHORITY.to_owned(),
        inventory_contract: COMPILER_COMPONENT_DISPATCH_INVENTORY_CONTRACT.to_owned(),
        request_contract: COMPILER_COMPONENT_COMPILE_REQUEST_CONTRACT.to_owned(),
        command: COMPILER_COMPONENT_COMPILE_COMMAND.to_owned(),
        argument_contract: COMPILER_COMPONENT_COMPILE_ARGUMENT_CONTRACT.to_owned(),
        stdin_contract: CLOSED_STDIN_CONTRACT.to_owned(),
        transition_protocol: input.transition.protocol.clone(),
        transition_generation: input.transition.generation,
        transition_proof_sha256: input.transition.proof_sha256.clone(),
        bootstrap_subset_protocol: input.request.bootstrap_subset_protocol.clone(),
        component_id: input.request.component_id.clone(),
        component_domain: input.request.component_domain.clone(),
        component_unit: input.request.component_unit.clone(),
        request_record_sha256: input.request.record_sha256.clone(),
        request_reproducible_build_sha256: input.request.reproducible_build_sha256.clone(),
        request_dependency_count: input.request.dependency_count,
        request_dependency_closure_sha256: input.request.dependency_closure_sha256.clone(),
        request_stage_handoff_bundle_sha256: input.request.stage_handoff_bundle_sha256.clone(),
        request_compiled_artifact_bytes: input.request.compiled_artifact_bytes,
        request_compiled_artifact_sha256: input.request.compiled_artifact_sha256.clone(),
        request_native_binary_bytes: input.request.native_binary_bytes,
        request_native_binary_sha256: input.request.native_binary_sha256.clone(),
        compiled_artifact_identity_contract: COMPILER_COMPONENT_COMPILED_ARTIFACT_IDENTITY_CONTRACT
            .to_owned(),
        compiled_artifact_semantic_sha256,
        selected_selector: input.transition.current_selector.clone(),
        selected_stage_role: current.component.stage_role.clone(),
        selected_record_sha256: current.component.record_sha256.clone(),
        selected_compiler_image_bytes: current.compiler_image.len(),
        selected_compiler_image_sha256: current.component.compiler_image_sha256.clone(),
        forward_selector: input.transition.forward_selector.clone(),
        forward_stage_role: forward.component.stage_role.clone(),
        forward_reproducible_build_sha256: forward.component.reproducible_build_sha256.clone(),
        forward_record_sha256: forward.component.record_sha256.clone(),
        forward_compiler_image_bytes: forward.compiler_image.len(),
        forward_compiler_image_sha256: forward.component.compiler_image_sha256.clone(),
        result_stage_role: input.result.stage_role.clone(),
        result_record_sha256: input.result.record_sha256.clone(),
        result_compiler_image_bytes: input.result.compiler_image_bytes,
        result_compiler_image_sha256: input.result.compiler_image_sha256.clone(),
        result_reproducible_build_sha256: input.result.reproducible_build_sha256.clone(),
        result_dependency_count: input.result.dependency_count,
        result_dependency_closure_sha256: input.result.dependency_closure_sha256.clone(),
        result_stage_handoff_bundle_sha256: input.result.stage_handoff_bundle_sha256.clone(),
        result_compiled_artifact_bytes: input.result.compiled_artifact_bytes,
        result_compiled_artifact_sha256: input.result.compiled_artifact_sha256.clone(),
        result_native_binary_bytes: input.result.native_binary_bytes,
        result_native_binary_sha256: input.result.native_binary_sha256.clone(),
        exit_code: input.exit_code,
        stdout_bytes: input.stdout.len(),
        stdout_sha256: sha256_hex(input.stdout),
        stderr_bytes: input.stderr.len(),
        stderr_sha256: sha256_hex(input.stderr),
        verdict: COMPILER_COMPONENT_COMPILE_DISPATCH_VERDICT.to_owned(),
        dispatch_sha256: String::new(),
    };
    receipt.dispatch_sha256 = compile_dispatch_identity(&receipt);
    validate_compiler_component_compile_dispatch_receipt(&receipt)?;
    Ok(receipt)
}

pub fn render_compiler_component_compile_dispatch_receipt(
    receipt: &CompilerComponentCompileDispatchReceipt,
) -> String {
    format!(
        "protocol = \"{}\"\ndriver_contract = \"{}\"\nauthority = \"{}\"\ninventory_contract = \"{}\"\nrequest_contract = \"{}\"\ncommand = \"{}\"\nargument_contract = \"{}\"\nstdin_contract = \"{}\"\ntransition_protocol = \"{}\"\ntransition_generation = {}\ntransition_proof_sha256 = \"{}\"\nbootstrap_subset_protocol = \"{}\"\ncomponent_id = \"{}\"\ncomponent_domain = \"{}\"\ncomponent_unit = \"{}\"\nrequest_record_sha256 = \"{}\"\nrequest_reproducible_build_sha256 = \"{}\"\nrequest_dependency_count = {}\nrequest_dependency_closure_sha256 = \"{}\"\nrequest_stage_handoff_bundle_sha256 = \"{}\"\nrequest_compiled_artifact_bytes = {}\nrequest_compiled_artifact_sha256 = \"{}\"\nrequest_native_binary_bytes = {}\nrequest_native_binary_sha256 = \"{}\"\ncompiled_artifact_identity_contract = \"{}\"\ncompiled_artifact_semantic_sha256 = \"{}\"\nselected_selector = \"{}\"\nselected_stage_role = \"{}\"\nselected_record_sha256 = \"{}\"\nselected_compiler_image_bytes = {}\nselected_compiler_image_sha256 = \"{}\"\nforward_selector = \"{}\"\nforward_stage_role = \"{}\"\nforward_reproducible_build_sha256 = \"{}\"\nforward_record_sha256 = \"{}\"\nforward_compiler_image_bytes = {}\nforward_compiler_image_sha256 = \"{}\"\nresult_stage_role = \"{}\"\nresult_record_sha256 = \"{}\"\nresult_compiler_image_bytes = {}\nresult_compiler_image_sha256 = \"{}\"\nresult_reproducible_build_sha256 = \"{}\"\nresult_dependency_count = {}\nresult_dependency_closure_sha256 = \"{}\"\nresult_stage_handoff_bundle_sha256 = \"{}\"\nresult_compiled_artifact_bytes = {}\nresult_compiled_artifact_sha256 = \"{}\"\nresult_native_binary_bytes = {}\nresult_native_binary_sha256 = \"{}\"\nexit_code = {}\nstdout_bytes = {}\nstdout_sha256 = \"{}\"\nstderr_bytes = {}\nstderr_sha256 = \"{}\"\nverdict = \"{}\"\ndispatch_sha256 = \"{}\"\n",
        receipt.protocol,
        receipt.driver_contract,
        receipt.authority,
        receipt.inventory_contract,
        receipt.request_contract,
        receipt.command,
        receipt.argument_contract,
        receipt.stdin_contract,
        receipt.transition_protocol,
        receipt.transition_generation,
        receipt.transition_proof_sha256,
        escape_toml_string(&receipt.bootstrap_subset_protocol),
        escape_toml_string(&receipt.component_id),
        escape_toml_string(&receipt.component_domain),
        escape_toml_string(&receipt.component_unit),
        receipt.request_record_sha256,
        receipt.request_reproducible_build_sha256,
        receipt.request_dependency_count,
        receipt.request_dependency_closure_sha256,
        receipt.request_stage_handoff_bundle_sha256,
        receipt.request_compiled_artifact_bytes,
        receipt.request_compiled_artifact_sha256,
        receipt.request_native_binary_bytes,
        receipt.request_native_binary_sha256,
        receipt.compiled_artifact_identity_contract,
        receipt.compiled_artifact_semantic_sha256,
        receipt.selected_selector,
        receipt.selected_stage_role,
        receipt.selected_record_sha256,
        receipt.selected_compiler_image_bytes,
        receipt.selected_compiler_image_sha256,
        receipt.forward_selector,
        receipt.forward_stage_role,
        receipt.forward_reproducible_build_sha256,
        receipt.forward_record_sha256,
        receipt.forward_compiler_image_bytes,
        receipt.forward_compiler_image_sha256,
        receipt.result_stage_role,
        receipt.result_record_sha256,
        receipt.result_compiler_image_bytes,
        receipt.result_compiler_image_sha256,
        receipt.result_reproducible_build_sha256,
        receipt.result_dependency_count,
        receipt.result_dependency_closure_sha256,
        receipt.result_stage_handoff_bundle_sha256,
        receipt.result_compiled_artifact_bytes,
        receipt.result_compiled_artifact_sha256,
        receipt.result_native_binary_bytes,
        receipt.result_native_binary_sha256,
        receipt.exit_code,
        receipt.stdout_bytes,
        receipt.stdout_sha256,
        receipt.stderr_bytes,
        receipt.stderr_sha256,
        receipt.verdict,
        receipt.dispatch_sha256,
    )
}

pub fn parse_compiler_component_compile_dispatch_receipt(
    path: &Path,
) -> Result<CompilerComponentCompileDispatchReceipt, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!("failed to read `{}`: {error}", path.display()))
    })?;
    parse_compiler_component_compile_dispatch_receipt_from_source(&source, path)
}

pub fn parse_compiler_component_compile_dispatch_receipt_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentCompileDispatchReceipt, ArtifactError> {
    if source.is_empty() || !source.ends_with('\n') || source.contains('\r') {
        return Err(ArtifactError::new(format!(
            "compiler component compile dispatch receipt `{}` must be canonical UTF-8 text ending in LF",
            path.display()
        )));
    }
    let string = |key| parse_required_toml_string(source, key, path);
    let number = |key| parse_required_toml_usize(source, key, path);
    let receipt = CompilerComponentCompileDispatchReceipt {
        protocol: string("protocol")?,
        driver_contract: string("driver_contract")?,
        authority: string("authority")?,
        inventory_contract: string("inventory_contract")?,
        request_contract: string("request_contract")?,
        command: string("command")?,
        argument_contract: string("argument_contract")?,
        stdin_contract: string("stdin_contract")?,
        transition_protocol: string("transition_protocol")?,
        transition_generation: number("transition_generation")?,
        transition_proof_sha256: string("transition_proof_sha256")?,
        bootstrap_subset_protocol: string("bootstrap_subset_protocol")?,
        component_id: string("component_id")?,
        component_domain: string("component_domain")?,
        component_unit: string("component_unit")?,
        request_record_sha256: string("request_record_sha256")?,
        request_reproducible_build_sha256: string("request_reproducible_build_sha256")?,
        request_dependency_count: number("request_dependency_count")?,
        request_dependency_closure_sha256: string("request_dependency_closure_sha256")?,
        request_stage_handoff_bundle_sha256: string("request_stage_handoff_bundle_sha256")?,
        request_compiled_artifact_bytes: number("request_compiled_artifact_bytes")?,
        request_compiled_artifact_sha256: string("request_compiled_artifact_sha256")?,
        request_native_binary_bytes: number("request_native_binary_bytes")?,
        request_native_binary_sha256: string("request_native_binary_sha256")?,
        compiled_artifact_identity_contract: string("compiled_artifact_identity_contract")?,
        compiled_artifact_semantic_sha256: string("compiled_artifact_semantic_sha256")?,
        selected_selector: string("selected_selector")?,
        selected_stage_role: string("selected_stage_role")?,
        selected_record_sha256: string("selected_record_sha256")?,
        selected_compiler_image_bytes: number("selected_compiler_image_bytes")?,
        selected_compiler_image_sha256: string("selected_compiler_image_sha256")?,
        forward_selector: string("forward_selector")?,
        forward_stage_role: string("forward_stage_role")?,
        forward_reproducible_build_sha256: string("forward_reproducible_build_sha256")?,
        forward_record_sha256: string("forward_record_sha256")?,
        forward_compiler_image_bytes: number("forward_compiler_image_bytes")?,
        forward_compiler_image_sha256: string("forward_compiler_image_sha256")?,
        result_stage_role: string("result_stage_role")?,
        result_record_sha256: string("result_record_sha256")?,
        result_compiler_image_bytes: number("result_compiler_image_bytes")?,
        result_compiler_image_sha256: string("result_compiler_image_sha256")?,
        result_reproducible_build_sha256: string("result_reproducible_build_sha256")?,
        result_dependency_count: number("result_dependency_count")?,
        result_dependency_closure_sha256: string("result_dependency_closure_sha256")?,
        result_stage_handoff_bundle_sha256: string("result_stage_handoff_bundle_sha256")?,
        result_compiled_artifact_bytes: number("result_compiled_artifact_bytes")?,
        result_compiled_artifact_sha256: string("result_compiled_artifact_sha256")?,
        result_native_binary_bytes: number("result_native_binary_bytes")?,
        result_native_binary_sha256: string("result_native_binary_sha256")?,
        exit_code: number("exit_code")?,
        stdout_bytes: number("stdout_bytes")?,
        stdout_sha256: string("stdout_sha256")?,
        stderr_bytes: number("stderr_bytes")?,
        stderr_sha256: string("stderr_sha256")?,
        verdict: string("verdict")?,
        dispatch_sha256: string("dispatch_sha256")?,
    };
    validate_compiler_component_compile_dispatch_receipt(&receipt)?;
    if render_compiler_component_compile_dispatch_receipt(&receipt) != source {
        return Err(ArtifactError::new(format!(
            "compiler component compile dispatch receipt `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(receipt)
}

fn verify_compile_targets(
    transition: &CompilerComponentTransition,
    current: &CompilerComponentBuild,
    forward: &CompilerComponentBuild,
) -> Result<(), ArtifactError> {
    if transition.protocol != COMPILER_COMPONENT_TRANSITION_PROTOCOL
        || transition.generation != 2
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
            "compiler component compile dispatch targets do not match the signed generation-two transition",
        ));
    }
    Ok(())
}

fn verify_compile_result(
    request: &CompilerComponentBuild,
    current: &CompilerComponentBuild,
    result: &CompilerComponentBuild,
) -> Result<(), ArtifactError> {
    verify_compiler_component_build(request)?;
    verify_compiler_component_build(result)?;
    if request.record_sha256 != current.record_sha256
        || request.stage_role != COMPILER_COMPONENT_STAGE0_ROLE
    {
        return Err(ArtifactError::new(
            "compiler component compile request must be the exact selected current stage0 record",
        ));
    }
    if result.stage_role != COMPILER_COMPONENT_STAGE0_ROLE
        || result.bootstrap_subset_protocol != request.bootstrap_subset_protocol
        || result.component_id != request.component_id
        || result.component_domain != request.component_domain
        || result.component_unit != request.component_unit
        || result.producer_id != request.producer_id
        || result.compiler_image_bytes != current.compiler_image_bytes
        || result.compiler_image_sha256 != current.compiler_image_sha256
        || result.stage_handoff_file != request.stage_handoff_file
        || result.stage_handoff_bundle_sha256 != request.stage_handoff_bundle_sha256
        || result.compiled_artifact_file != request.compiled_artifact_file
        || result.native_binary_file != request.native_binary_file
        || result.native_binary_bytes != request.native_binary_bytes
        || result.native_binary_sha256 != request.native_binary_sha256
        || result.dependency_closure_contract != request.dependency_closure_contract
        || result.dependency_count != request.dependency_count
        || result.dependency_closure_sha256 != request.dependency_closure_sha256
        || result.dependencies != request.dependencies
        || result.reproducible_identity_contract != request.reproducible_identity_contract
        || result.reproducible_build_sha256 != request.reproducible_build_sha256
    {
        return Err(ArtifactError::new(
            "compiler component compile result does not satisfy the canonical rebuild request",
        ));
    }
    Ok(())
}

fn verify_artifact_bytes(
    build: &CompilerComponentBuild,
    bytes: &[u8],
) -> Result<(), ArtifactError> {
    if bytes.len() != build.compiled_artifact_bytes
        || sha256_hex(bytes) != build.compiled_artifact_sha256
    {
        return Err(ArtifactError::new(
            "compiler component compile dispatch compiled artifact bytes do not match their build record",
        ));
    }
    Ok(())
}

fn compiled_artifact_semantic_identity(artifact: &NuisCompiledArtifact) -> String {
    let mut hash = Sha256::new();
    hash_field(
        &mut hash,
        COMPILER_COMPONENT_COMPILED_ARTIFACT_IDENTITY_CONTRACT.as_bytes(),
    );
    for value in [
        artifact.schema.as_bytes(),
        artifact.packaging_mode.as_bytes(),
        artifact.cpu_target_abi.as_bytes(),
        artifact.cpu_target_machine_arch.as_bytes(),
        artifact.cpu_target_machine_os.as_bytes(),
        artifact.cpu_target_object_format.as_bytes(),
        artifact.cpu_target_calling_abi.as_bytes(),
        artifact.binary_name.as_bytes(),
        artifact.envelope.schema.as_bytes(),
        artifact.envelope.executable_kind.as_bytes(),
        artifact.envelope.function_kind.as_bytes(),
        artifact.envelope.graph_kind.as_bytes(),
        artifact.envelope.default_time_mode.as_bytes(),
        artifact.lifecycle.schema.as_bytes(),
        artifact.lifecycle.bootstrap_entry.as_bytes(),
        artifact.lifecycle.tick_policy.as_bytes(),
        artifact.lifecycle.shutdown_policy.as_bytes(),
        artifact.lifecycle.yalivia_rpc.as_bytes(),
        artifact.binary_blob.as_slice(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [artifact.binary_bytes, artifact.envelope.package_count] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    for values in [
        &artifact.envelope.domain_families,
        &artifact.envelope.contract_families,
        &artifact.lifecycle.hook_surface,
        &artifact.lifecycle.export_surface,
        &artifact.lifecycle.runtime_capability_flags,
    ] {
        hash_field(&mut hash, &(values.len() as u64).to_le_bytes());
        for value in values {
            hash_field(&mut hash, value.as_bytes());
        }
    }
    hash_field(
        &mut hash,
        &(artifact.host_objects.len() as u64).to_le_bytes(),
    );
    for object in &artifact.host_objects {
        for value in [
            object.object_id.as_bytes(),
            object.role.as_bytes(),
            object.object_format.as_bytes(),
            object.bytes.as_slice(),
        ] {
            hash_field(&mut hash, value);
        }
    }
    encode_hex(&hash.finalize())
}

fn validate_compiler_component_compile_dispatch_receipt(
    receipt: &CompilerComponentCompileDispatchReceipt,
) -> Result<(), ArtifactError> {
    if receipt.protocol != COMPILER_COMPONENT_COMPILE_DISPATCH_PROTOCOL
        || receipt.driver_contract != COMPILER_COMPONENT_COMPILE_DISPATCH_DRIVER_CONTRACT
        || receipt.authority != COMPILER_COMPONENT_COMPILE_DISPATCH_AUTHORITY
        || receipt.inventory_contract != COMPILER_COMPONENT_DISPATCH_INVENTORY_CONTRACT
        || receipt.request_contract != COMPILER_COMPONENT_COMPILE_REQUEST_CONTRACT
        || receipt.command != COMPILER_COMPONENT_COMPILE_COMMAND
        || receipt.argument_contract != COMPILER_COMPONENT_COMPILE_ARGUMENT_CONTRACT
        || receipt.compiled_artifact_identity_contract
            != COMPILER_COMPONENT_COMPILED_ARTIFACT_IDENTITY_CONTRACT
        || receipt.stdin_contract != CLOSED_STDIN_CONTRACT
        || receipt.transition_protocol != COMPILER_COMPONENT_TRANSITION_PROTOCOL
        || receipt.transition_generation != 2
        || receipt.selected_selector != CURRENT_SELECTOR
        || receipt.selected_stage_role != COMPILER_COMPONENT_STAGE0_ROLE
        || receipt.forward_selector != FORWARD_SELECTOR
        || receipt.forward_stage_role != COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        || receipt.result_stage_role != COMPILER_COMPONENT_STAGE0_ROLE
        || receipt.verdict != COMPILER_COMPONENT_COMPILE_DISPATCH_VERDICT
    {
        return Err(ArtifactError::new(
            "compiler component compile dispatch receipt declares an unsupported contract",
        ));
    }
    for (label, value) in [
        (
            "bootstrap subset protocol",
            receipt.bootstrap_subset_protocol.as_str(),
        ),
        ("component id", receipt.component_id.as_str()),
        ("component domain", receipt.component_domain.as_str()),
        ("component unit", receipt.component_unit.as_str()),
    ] {
        if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
            return Err(ArtifactError::new(format!(
                "compiler component compile dispatch {label} is invalid"
            )));
        }
    }
    if receipt.request_record_sha256 != receipt.selected_record_sha256
        || receipt.request_reproducible_build_sha256 != receipt.result_reproducible_build_sha256
        || receipt.request_dependency_count != receipt.result_dependency_count
        || receipt.request_dependency_closure_sha256 != receipt.result_dependency_closure_sha256
        || receipt.request_stage_handoff_bundle_sha256 != receipt.result_stage_handoff_bundle_sha256
        || receipt.request_native_binary_bytes != receipt.result_native_binary_bytes
        || receipt.request_native_binary_sha256 != receipt.result_native_binary_sha256
        || receipt.selected_compiler_image_bytes != receipt.result_compiler_image_bytes
        || receipt.selected_compiler_image_sha256 != receipt.result_compiler_image_sha256
        || receipt.request_dependency_count == 0
        || receipt.request_compiled_artifact_bytes == 0
        || receipt.result_compiled_artifact_bytes == 0
        || receipt.request_native_binary_bytes == 0
        || receipt.selected_compiler_image_bytes == 0
        || receipt.forward_compiler_image_bytes == 0
        || receipt.selected_record_sha256 == receipt.forward_record_sha256
        || receipt.exit_code != 0
        || receipt.stdout_bytes == 0
        || receipt.stderr_bytes != 0
        || receipt.stderr_sha256 != sha256_hex(&[])
    {
        return Err(ArtifactError::new(
            "compiler component compile dispatch receipt has inconsistent request, targets, or result",
        ));
    }
    for (label, value) in receipt_hashes(receipt) {
        validate_sha256(value, label)?;
    }
    if compile_dispatch_identity(receipt) != receipt.dispatch_sha256 {
        return Err(ArtifactError::new(
            "compiler component compile dispatch receipt identity mismatch",
        ));
    }
    Ok(())
}

fn receipt_hashes(receipt: &CompilerComponentCompileDispatchReceipt) -> [(&str, &str); 20] {
    [
        ("transition proof", &receipt.transition_proof_sha256),
        ("request record", &receipt.request_record_sha256),
        (
            "request reproducible build",
            &receipt.request_reproducible_build_sha256,
        ),
        (
            "request dependency closure",
            &receipt.request_dependency_closure_sha256,
        ),
        (
            "request stage handoff",
            &receipt.request_stage_handoff_bundle_sha256,
        ),
        (
            "request compiled artifact",
            &receipt.request_compiled_artifact_sha256,
        ),
        (
            "request native binary",
            &receipt.request_native_binary_sha256,
        ),
        (
            "compiled artifact semantic identity",
            &receipt.compiled_artifact_semantic_sha256,
        ),
        ("selected record", &receipt.selected_record_sha256),
        (
            "selected compiler image",
            &receipt.selected_compiler_image_sha256,
        ),
        (
            "forward reproducible build",
            &receipt.forward_reproducible_build_sha256,
        ),
        ("forward record", &receipt.forward_record_sha256),
        (
            "forward compiler image",
            &receipt.forward_compiler_image_sha256,
        ),
        ("result record", &receipt.result_record_sha256),
        (
            "result compiler image",
            &receipt.result_compiler_image_sha256,
        ),
        (
            "result reproducible build",
            &receipt.result_reproducible_build_sha256,
        ),
        (
            "result dependency closure",
            &receipt.result_dependency_closure_sha256,
        ),
        (
            "result stage handoff",
            &receipt.result_stage_handoff_bundle_sha256,
        ),
        (
            "result compiled artifact",
            &receipt.result_compiled_artifact_sha256,
        ),
        ("result native binary", &receipt.result_native_binary_sha256),
    ]
}

fn compile_dispatch_identity(receipt: &CompilerComponentCompileDispatchReceipt) -> String {
    let mut hash = Sha256::new();
    for value in [
        receipt.protocol.as_bytes(),
        receipt.driver_contract.as_bytes(),
        receipt.authority.as_bytes(),
        receipt.inventory_contract.as_bytes(),
        receipt.request_contract.as_bytes(),
        receipt.command.as_bytes(),
        receipt.argument_contract.as_bytes(),
        receipt.stdin_contract.as_bytes(),
        receipt.transition_protocol.as_bytes(),
        receipt.transition_proof_sha256.as_bytes(),
        receipt.bootstrap_subset_protocol.as_bytes(),
        receipt.component_id.as_bytes(),
        receipt.component_domain.as_bytes(),
        receipt.component_unit.as_bytes(),
        receipt.request_record_sha256.as_bytes(),
        receipt.request_reproducible_build_sha256.as_bytes(),
        receipt.request_dependency_closure_sha256.as_bytes(),
        receipt.request_stage_handoff_bundle_sha256.as_bytes(),
        receipt.request_compiled_artifact_sha256.as_bytes(),
        receipt.request_native_binary_sha256.as_bytes(),
        receipt.compiled_artifact_identity_contract.as_bytes(),
        receipt.compiled_artifact_semantic_sha256.as_bytes(),
        receipt.selected_selector.as_bytes(),
        receipt.selected_stage_role.as_bytes(),
        receipt.selected_record_sha256.as_bytes(),
        receipt.selected_compiler_image_sha256.as_bytes(),
        receipt.forward_selector.as_bytes(),
        receipt.forward_stage_role.as_bytes(),
        receipt.forward_reproducible_build_sha256.as_bytes(),
        receipt.forward_record_sha256.as_bytes(),
        receipt.forward_compiler_image_sha256.as_bytes(),
        receipt.result_stage_role.as_bytes(),
        receipt.result_record_sha256.as_bytes(),
        receipt.result_compiler_image_sha256.as_bytes(),
        receipt.result_reproducible_build_sha256.as_bytes(),
        receipt.result_dependency_closure_sha256.as_bytes(),
        receipt.result_stage_handoff_bundle_sha256.as_bytes(),
        receipt.result_compiled_artifact_sha256.as_bytes(),
        receipt.result_native_binary_sha256.as_bytes(),
        receipt.stdout_sha256.as_bytes(),
        receipt.stderr_sha256.as_bytes(),
        receipt.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        receipt.transition_generation,
        receipt.request_dependency_count,
        receipt.request_compiled_artifact_bytes,
        receipt.request_native_binary_bytes,
        receipt.selected_compiler_image_bytes,
        receipt.forward_compiler_image_bytes,
        receipt.result_compiler_image_bytes,
        receipt.result_dependency_count,
        receipt.result_compiled_artifact_bytes,
        receipt.result_native_binary_bytes,
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
            "compiler component compile dispatch {label} must be lowercase SHA-256"
        )))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}
