use crate::{
    provider_capability_registry::{
        select_provider_capability, ProviderCapabilitySelectionEvidence,
    },
    provider_sample_artifact::{fnv1a64_hex, push_toml_string},
};

pub const PROVIDER_CONFORMANCE_CAPSULE_CONTRACT: &str = "nuis-provider-conformance-capsule-v1";
pub const PROVIDER_CONFORMANCE_REPLAY_CONTRACT: &str = "nuis-provider-conformance-replay-v1";
pub const DATA_REFERENCE_SCENARIO_CONTRACT: &str = "nuis-data-reference-copy-conformance-v1";
pub const PROVIDER_CONFORMANCE_EXECUTION_AUTHORITY: &str = "conformance-only";

const DATA_REFERENCE_PROVIDER_FAMILY: &str = "data:host";
const DATA_REFERENCE_SCENARIO_ID: &str = "data.copy.binary-octets.v1";
const DATA_REFERENCE_BYTES: &[u8] = &[0x00, 0x01, 0x02, 0x03, 0x7f, 0x80, 0xfe, 0xff];
const DATA_REFERENCE_REQUIREMENTS: &[&str] = &[
    "clock.fabric-monotonic",
    "completion.verified",
    "execution.reference",
    "glm.owned-transfer",
    "memory.cpu",
    "movement.copy",
    "residency.host",
];
const CLOCK_ORDER: &str = "submission-before-completion-before-release";
const GLM_ORDER: &str = "owned-transfer-release-after-completion";
const COMPLETION_RULE: &str = "byte-length-and-fnv1a64-equal";
const REFERENCE_SUBMISSION_TICK: u64 = 1;
const REFERENCE_COMPLETION_TICK: u64 = 2;
const REFERENCE_RELEASE_TICK: u64 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderConformanceCapsule {
    pub contract: &'static str,
    pub scenario_contract: &'static str,
    pub scenario_id: &'static str,
    pub package_id: &'static str,
    pub provider_id: &'static str,
    pub bundle_id: &'static str,
    pub provider_family: &'static str,
    pub capability_selection_hash: String,
    pub requirements: String,
    pub input_hex: String,
    pub input_hash: String,
    pub input_byte_length: usize,
    pub expected_output_hex: String,
    pub expected_output_hash: String,
    pub expected_output_byte_length: usize,
    pub clock_order: &'static str,
    pub glm_order: &'static str,
    pub completion_rule: &'static str,
    pub execution_authority: &'static str,
    pub physical_execution_claimed: bool,
    pub capsule_hash: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ProviderConformanceObservation<'a> {
    pub output: &'a [u8],
    pub submission_tick: u64,
    pub completion_tick: u64,
    pub release_tick: u64,
    pub glm_released: bool,
    pub physical_execution_claimed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderConformanceReplayEvidence {
    pub contract: &'static str,
    pub capsule_hash: String,
    pub capability_selection_hash: String,
    pub output_hash: String,
    pub output_byte_length: usize,
    pub submission_tick: u64,
    pub completion_tick: u64,
    pub release_tick: u64,
    pub clock_status: &'static str,
    pub glm_status: &'static str,
    pub completion_status: &'static str,
    pub execution_authority: &'static str,
    pub physical_execution_claimed: bool,
    pub replay_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderConformanceLifecycleEvidence {
    pub capsule_contract: String,
    pub status: String,
    pub scenario_contract: String,
    pub scenario_id: String,
    pub package_id: String,
    pub provider_id: String,
    pub bundle_id: String,
    pub provider_family: String,
    pub capability_selection_hash: String,
    pub capsule_hash: String,
    pub replay_contract: String,
    pub replay_status: String,
    pub replay_hash: String,
    pub execution_authority: String,
    pub physical_execution_claimed: bool,
}

impl Default for ProviderConformanceLifecycleEvidence {
    fn default() -> Self {
        Self {
            capsule_contract: PROVIDER_CONFORMANCE_CAPSULE_CONTRACT.to_owned(),
            status: "not-applicable".to_owned(),
            scenario_contract: "none".to_owned(),
            scenario_id: "none".to_owned(),
            package_id: "none".to_owned(),
            provider_id: "none".to_owned(),
            bundle_id: "none".to_owned(),
            provider_family: "none".to_owned(),
            capability_selection_hash: "none".to_owned(),
            capsule_hash: "none".to_owned(),
            replay_contract: PROVIDER_CONFORMANCE_REPLAY_CONTRACT.to_owned(),
            replay_status: "not-applicable".to_owned(),
            replay_hash: "none".to_owned(),
            execution_authority: "none".to_owned(),
            physical_execution_claimed: false,
        }
    }
}

pub fn data_reference_conformance_capsule() -> Result<ProviderConformanceCapsule, String> {
    build_reference_copy_conformance_capsule(DATA_REFERENCE_PROVIDER_FAMILY)
}

fn data_reference_conformance_replay(
    capsule: &ProviderConformanceCapsule,
) -> Result<ProviderConformanceReplayEvidence, String> {
    replay_provider_conformance_capsule(
        capsule,
        ProviderConformanceObservation {
            output: DATA_REFERENCE_BYTES,
            submission_tick: REFERENCE_SUBMISSION_TICK,
            completion_tick: REFERENCE_COMPLETION_TICK,
            release_tick: REFERENCE_RELEASE_TICK,
            glm_released: true,
            physical_execution_claimed: false,
        },
    )
}

pub fn replay_provider_conformance_capsule(
    capsule: &ProviderConformanceCapsule,
    observation: ProviderConformanceObservation<'_>,
) -> Result<ProviderConformanceReplayEvidence, String> {
    let expected = data_reference_conformance_capsule()?;
    if capsule != &expected {
        return Err("provider-conformance-replay:capsule-drift".to_owned());
    }
    if observation.physical_execution_claimed {
        return Err("provider-conformance-replay:physical-authority-forbidden".to_owned());
    }
    if observation.output.len() != capsule.expected_output_byte_length
        || fnv1a64_digest(observation.output) != capsule.expected_output_hash
    {
        return Err("provider-conformance-replay:output-mismatch".to_owned());
    }
    if observation.submission_tick == 0
        || observation.submission_tick >= observation.completion_tick
        || observation.completion_tick >= observation.release_tick
    {
        return Err("provider-conformance-replay:clock-order-invalid".to_owned());
    }
    if !observation.glm_released {
        return Err("provider-conformance-replay:glm-release-missing".to_owned());
    }

    let output_hash = fnv1a64_digest(observation.output);
    let canonical = format!(
        "{PROVIDER_CONFORMANCE_REPLAY_CONTRACT}\n{}\n{}\noutput|{}|{}\nclock|{}|{}|{}|monotonic\nglm|released-after-completion\ncompletion|verified\nauthority|{PROVIDER_CONFORMANCE_EXECUTION_AUTHORITY}|physical-execution-claimed=false\n",
        capsule.capsule_hash,
        capsule.capability_selection_hash,
        observation.output.len(),
        output_hash,
        observation.submission_tick,
        observation.completion_tick,
        observation.release_tick,
    );
    Ok(ProviderConformanceReplayEvidence {
        contract: PROVIDER_CONFORMANCE_REPLAY_CONTRACT,
        capsule_hash: capsule.capsule_hash.clone(),
        capability_selection_hash: capsule.capability_selection_hash.clone(),
        output_hash,
        output_byte_length: observation.output.len(),
        submission_tick: observation.submission_tick,
        completion_tick: observation.completion_tick,
        release_tick: observation.release_tick,
        clock_status: "monotonic",
        glm_status: "released-after-completion",
        completion_status: "verified",
        execution_authority: PROVIDER_CONFORMANCE_EXECUTION_AUTHORITY,
        physical_execution_claimed: false,
        replay_hash: fnv1a64_digest(canonical.as_bytes()),
    })
}

pub(crate) fn append_provider_conformance_capsule_evidence(
    out: &mut String,
    provider_family: &str,
) {
    let Ok(capsule) = build_reference_copy_conformance_capsule(provider_family) else {
        return;
    };
    let replay = data_reference_conformance_replay(&capsule)
        .expect("generated provider conformance capsule must replay");
    push_toml_string(
        out,
        "provider_conformance_capsule_contract",
        capsule.contract,
    );
    push_toml_string(out, "provider_conformance_status", "verified");
    push_toml_string(
        out,
        "provider_conformance_scenario_contract",
        capsule.scenario_contract,
    );
    push_toml_string(out, "provider_conformance_scenario_id", capsule.scenario_id);
    push_toml_string(out, "provider_conformance_package_id", capsule.package_id);
    push_toml_string(out, "provider_conformance_provider_id", capsule.provider_id);
    push_toml_string(out, "provider_conformance_bundle_id", capsule.bundle_id);
    push_toml_string(
        out,
        "provider_conformance_provider_family",
        capsule.provider_family,
    );
    push_toml_string(
        out,
        "provider_conformance_capability_selection_hash",
        &capsule.capability_selection_hash,
    );
    push_toml_string(
        out,
        "provider_conformance_requirements",
        &capsule.requirements,
    );
    push_toml_string(out, "provider_conformance_input_hash", &capsule.input_hash);
    out.push_str(&format!(
        "provider_conformance_input_byte_length = {}\n",
        capsule.input_byte_length
    ));
    push_toml_string(
        out,
        "provider_conformance_expected_output_hash",
        &capsule.expected_output_hash,
    );
    out.push_str(&format!(
        "provider_conformance_expected_output_byte_length = {}\n",
        capsule.expected_output_byte_length
    ));
    push_toml_string(out, "provider_conformance_clock_order", capsule.clock_order);
    push_toml_string(out, "provider_conformance_glm_order", capsule.glm_order);
    push_toml_string(
        out,
        "provider_conformance_completion_rule",
        capsule.completion_rule,
    );
    push_toml_string(
        out,
        "provider_conformance_execution_authority",
        capsule.execution_authority,
    );
    out.push_str("provider_conformance_physical_execution_claimed = false\n");
    push_toml_string(
        out,
        "provider_conformance_capsule_hash",
        &capsule.capsule_hash,
    );
    push_toml_string(out, "provider_conformance_replay_contract", replay.contract);
    push_toml_string(out, "provider_conformance_replay_status", "verified");
    push_toml_string(
        out,
        "provider_conformance_replay_output_hash",
        &replay.output_hash,
    );
    out.push_str(&format!(
        "provider_conformance_replay_output_byte_length = {}\n",
        replay.output_byte_length
    ));
    out.push_str(&format!(
        "provider_conformance_replay_submission_tick = {}\n",
        replay.submission_tick
    ));
    out.push_str(&format!(
        "provider_conformance_replay_completion_tick = {}\n",
        replay.completion_tick
    ));
    out.push_str(&format!(
        "provider_conformance_replay_release_tick = {}\n",
        replay.release_tick
    ));
    push_toml_string(
        out,
        "provider_conformance_replay_clock_status",
        replay.clock_status,
    );
    push_toml_string(
        out,
        "provider_conformance_replay_glm_status",
        replay.glm_status,
    );
    push_toml_string(
        out,
        "provider_conformance_replay_completion_status",
        replay.completion_status,
    );
    push_toml_string(
        out,
        "provider_conformance_replay_execution_authority",
        replay.execution_authority,
    );
    out.push_str("provider_conformance_replay_physical_execution_claimed = false\n");
    push_toml_string(out, "provider_conformance_replay_hash", &replay.replay_hash);
}

pub(crate) fn completion_evidence_from_output(
    source: &str,
) -> Result<ProviderConformanceLifecycleEvidence, String> {
    let Some(_) = string_field(source, "provider_conformance_capsule_contract") else {
        return Ok(ProviderConformanceLifecycleEvidence::default());
    };
    let family = required_string(source, "provider_conformance_provider_family")?;
    let capsule = build_reference_copy_conformance_capsule(&family)?;
    let replay = data_reference_conformance_replay(&capsule)?;
    let expected = lifecycle_evidence(&capsule, &replay);
    if !output_capsule_matches(source, &capsule)
        || !output_replay_matches(source, &replay)
        || lifecycle_evidence_from_fields(source, "provider_conformance_") != expected
    {
        return Err("provider-conformance-completion:evidence-mismatch".to_owned());
    }
    Ok(expected)
}

pub(crate) fn render_completion_event_fields(
    out: &mut String,
    evidence: &ProviderConformanceLifecycleEvidence,
) {
    if evidence.status != "verified" {
        return;
    }
    for (key, value) in [
        (
            "conformance_capsule_contract",
            evidence.capsule_contract.as_str(),
        ),
        ("conformance_status", evidence.status.as_str()),
        (
            "conformance_scenario_contract",
            evidence.scenario_contract.as_str(),
        ),
        ("conformance_scenario_id", evidence.scenario_id.as_str()),
        ("conformance_package_id", evidence.package_id.as_str()),
        ("conformance_provider_id", evidence.provider_id.as_str()),
        ("conformance_bundle_id", evidence.bundle_id.as_str()),
        (
            "conformance_provider_family",
            evidence.provider_family.as_str(),
        ),
        (
            "conformance_capability_selection_hash",
            evidence.capability_selection_hash.as_str(),
        ),
        ("conformance_capsule_hash", evidence.capsule_hash.as_str()),
        (
            "conformance_replay_contract",
            evidence.replay_contract.as_str(),
        ),
        ("conformance_replay_status", evidence.replay_status.as_str()),
        ("conformance_replay_hash", evidence.replay_hash.as_str()),
        (
            "conformance_execution_authority",
            evidence.execution_authority.as_str(),
        ),
    ] {
        push_toml_string(out, key, value);
    }
    out.push_str(&format!(
        "conformance_physical_execution_claimed = {}\n",
        evidence.physical_execution_claimed
    ));
}

pub(crate) fn parse_completion_event_fields(source: &str) -> ProviderConformanceLifecycleEvidence {
    if string_field(source, "conformance_status").as_deref() != Some("verified") {
        return ProviderConformanceLifecycleEvidence::default();
    }
    lifecycle_evidence_from_fields(source, "conformance_")
}

pub(crate) fn append_completion_hash_material(
    material: &mut String,
    evidence: &ProviderConformanceLifecycleEvidence,
) {
    if evidence.status == "verified" {
        material.push_str(&format!(
            "\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}",
            evidence.capsule_contract,
            evidence.status,
            evidence.scenario_contract,
            evidence.scenario_id,
            evidence.package_id,
            evidence.provider_id,
            evidence.bundle_id,
            evidence.provider_family,
            evidence.capability_selection_hash,
            evidence.capsule_hash,
            evidence.replay_contract,
            evidence.replay_status,
            evidence.replay_hash,
            evidence.execution_authority,
            evidence.physical_execution_claimed,
        ));
    }
}

fn lifecycle_evidence(
    capsule: &ProviderConformanceCapsule,
    replay: &ProviderConformanceReplayEvidence,
) -> ProviderConformanceLifecycleEvidence {
    ProviderConformanceLifecycleEvidence {
        capsule_contract: capsule.contract.to_owned(),
        status: "verified".to_owned(),
        scenario_contract: capsule.scenario_contract.to_owned(),
        scenario_id: capsule.scenario_id.to_owned(),
        package_id: capsule.package_id.to_owned(),
        provider_id: capsule.provider_id.to_owned(),
        bundle_id: capsule.bundle_id.to_owned(),
        provider_family: capsule.provider_family.to_owned(),
        capability_selection_hash: capsule.capability_selection_hash.clone(),
        capsule_hash: capsule.capsule_hash.clone(),
        replay_contract: replay.contract.to_owned(),
        replay_status: "verified".to_owned(),
        replay_hash: replay.replay_hash.clone(),
        execution_authority: PROVIDER_CONFORMANCE_EXECUTION_AUTHORITY.to_owned(),
        physical_execution_claimed: false,
    }
}

fn lifecycle_evidence_from_fields(
    source: &str,
    prefix: &str,
) -> ProviderConformanceLifecycleEvidence {
    let value = |suffix: &str, fallback: &str| {
        string_field(source, &format!("{prefix}{suffix}")).unwrap_or_else(|| fallback.to_owned())
    };
    ProviderConformanceLifecycleEvidence {
        capsule_contract: value("capsule_contract", PROVIDER_CONFORMANCE_CAPSULE_CONTRACT),
        status: value("status", "not-applicable"),
        scenario_contract: value("scenario_contract", "none"),
        scenario_id: value("scenario_id", "none"),
        package_id: value("package_id", "none"),
        provider_id: value("provider_id", "none"),
        bundle_id: value("bundle_id", "none"),
        provider_family: value("provider_family", "none"),
        capability_selection_hash: value("capability_selection_hash", "none"),
        capsule_hash: value("capsule_hash", "none"),
        replay_contract: value("replay_contract", PROVIDER_CONFORMANCE_REPLAY_CONTRACT),
        replay_status: value("replay_status", "not-applicable"),
        replay_hash: value("replay_hash", "none"),
        execution_authority: value("execution_authority", "none"),
        physical_execution_claimed: bool_field(
            source,
            &format!("{prefix}physical_execution_claimed"),
        )
        .unwrap_or(false),
    }
}

fn output_capsule_matches(source: &str, capsule: &ProviderConformanceCapsule) -> bool {
    string_field(source, "provider_conformance_requirements").as_deref()
        == Some(capsule.requirements.as_str())
        && string_field(source, "provider_conformance_input_hash").as_deref()
            == Some(capsule.input_hash.as_str())
        && usize_field(source, "provider_conformance_input_byte_length")
            == Some(capsule.input_byte_length)
        && string_field(source, "provider_conformance_expected_output_hash").as_deref()
            == Some(capsule.expected_output_hash.as_str())
        && usize_field(source, "provider_conformance_expected_output_byte_length")
            == Some(capsule.expected_output_byte_length)
        && string_field(source, "provider_conformance_clock_order").as_deref()
            == Some(capsule.clock_order)
        && string_field(source, "provider_conformance_glm_order").as_deref()
            == Some(capsule.glm_order)
        && string_field(source, "provider_conformance_completion_rule").as_deref()
            == Some(capsule.completion_rule)
        && bool_field(source, "provider_conformance_physical_execution_claimed") == Some(false)
}

fn output_replay_matches(source: &str, replay: &ProviderConformanceReplayEvidence) -> bool {
    string_field(source, "provider_conformance_replay_output_hash").as_deref()
        == Some(replay.output_hash.as_str())
        && usize_field(source, "provider_conformance_replay_output_byte_length")
            == Some(replay.output_byte_length)
        && u64_field(source, "provider_conformance_replay_submission_tick")
            == Some(replay.submission_tick)
        && u64_field(source, "provider_conformance_replay_completion_tick")
            == Some(replay.completion_tick)
        && u64_field(source, "provider_conformance_replay_release_tick")
            == Some(replay.release_tick)
        && string_field(source, "provider_conformance_replay_clock_status").as_deref()
            == Some(replay.clock_status)
        && string_field(source, "provider_conformance_replay_glm_status").as_deref()
            == Some(replay.glm_status)
        && string_field(source, "provider_conformance_replay_completion_status").as_deref()
            == Some(replay.completion_status)
        && string_field(source, "provider_conformance_replay_execution_authority").as_deref()
            == Some(replay.execution_authority)
        && bool_field(
            source,
            "provider_conformance_replay_physical_execution_claimed",
        ) == Some(false)
        && string_field(source, "provider_conformance_replay_hash").as_deref()
            == Some(replay.replay_hash.as_str())
}

fn required_string(source: &str, key: &str) -> Result<String, String> {
    string_field(source, key)
        .filter(|value| !value.is_empty() && value != "none")
        .ok_or_else(|| format!("provider-conformance-completion:missing-{key}"))
}

fn string_field(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    source.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)?
            .trim()
            .strip_prefix('"')?
            .strip_suffix('"')
            .map(str::to_owned)
    })
}

fn usize_field(source: &str, key: &str) -> Option<usize> {
    raw_field(source, key)?.parse().ok()
}

fn u64_field(source: &str, key: &str) -> Option<u64> {
    raw_field(source, key)?.parse().ok()
}

fn bool_field(source: &str, key: &str) -> Option<bool> {
    match raw_field(source, key)? {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn raw_field<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key} = ");
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(str::trim))
}

fn build_reference_copy_conformance_capsule(
    provider_family: &str,
) -> Result<ProviderConformanceCapsule, String> {
    let selection = select_provider_capability(provider_family, DATA_REFERENCE_REQUIREMENTS)?;
    build_capsule(&selection)
}

fn build_capsule(
    selection: &ProviderCapabilitySelectionEvidence,
) -> Result<ProviderConformanceCapsule, String> {
    if selection.availability_status != "available"
        || selection.requirements != DATA_REFERENCE_REQUIREMENTS.join(",")
    {
        return Err("provider-conformance-capsule:selection-incompatible".to_owned());
    }
    let input_hex = hex_encode(DATA_REFERENCE_BYTES);
    let input_hash = fnv1a64_digest(DATA_REFERENCE_BYTES);
    let expected_output_hex = input_hex.clone();
    let expected_output_hash = input_hash.clone();
    let mut capsule = ProviderConformanceCapsule {
        contract: PROVIDER_CONFORMANCE_CAPSULE_CONTRACT,
        scenario_contract: DATA_REFERENCE_SCENARIO_CONTRACT,
        scenario_id: DATA_REFERENCE_SCENARIO_ID,
        package_id: selection.package_id,
        provider_id: selection.provider_id,
        bundle_id: selection.bundle_id,
        provider_family: selection.provider_family,
        capability_selection_hash: selection.selection_hash.clone(),
        requirements: selection.requirements.clone(),
        input_hex,
        input_hash,
        input_byte_length: DATA_REFERENCE_BYTES.len(),
        expected_output_hex,
        expected_output_hash,
        expected_output_byte_length: DATA_REFERENCE_BYTES.len(),
        clock_order: CLOCK_ORDER,
        glm_order: GLM_ORDER,
        completion_rule: COMPLETION_RULE,
        execution_authority: PROVIDER_CONFORMANCE_EXECUTION_AUTHORITY,
        physical_execution_claimed: false,
        capsule_hash: String::new(),
    };
    capsule.capsule_hash = fnv1a64_digest(capsule_canonical(&capsule).as_bytes());
    Ok(capsule)
}

fn capsule_canonical(capsule: &ProviderConformanceCapsule) -> String {
    format!(
        "{PROVIDER_CONFORMANCE_CAPSULE_CONTRACT}\nscenario|{}|{}\nprovider|{}|{}|{}|{}\nselection|{}|{}\ninput|{}|{}|{}\nexpected|{}|{}|{}\nclock|{}\nglm|{}\ncompletion|{}\nauthority|{}|physical-execution-claimed={}\n",
        capsule.scenario_contract,
        capsule.scenario_id,
        capsule.package_id,
        capsule.provider_id,
        capsule.bundle_id,
        capsule.provider_family,
        capsule.capability_selection_hash,
        capsule.requirements,
        capsule.input_byte_length,
        capsule.input_hex,
        capsule.input_hash,
        capsule.expected_output_byte_length,
        capsule.expected_output_hex,
        capsule.expected_output_hash,
        capsule.clock_order,
        capsule.glm_order,
        capsule.completion_rule,
        capsule.execution_authority,
        capsule.physical_execution_claimed,
    )
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn fnv1a64_digest(bytes: &[u8]) -> String {
    format!("fnv1a64:{}", fnv1a64_hex(bytes).trim_start_matches("0x"))
}
