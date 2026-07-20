use crate::model::{
    NsdbDeviceProviderSampleRecordInfo, NsdbPayloadExecutionEvent, NsdbPayloadExecutionHandoffInfo,
    PayloadExecutionHandoffPersistSummary, PayloadExecutionHandoffRecord,
};
use crate::provider_completion_integrity::{
    legacy_set_hash as legacy_provider_completion_set_hash,
    set_hash as provider_completion_set_hash, signature_message,
    CLAIM_AUTHORITY as PROVIDER_COMPLETION_CLAIM_AUTHORITY,
    CLAIM_AUTHORITY_CONTRACT as PROVIDER_COMPLETION_CLAIM_AUTHORITY_CONTRACT,
    DIGEST_FNV1A64_CONTRACT as PROVIDER_COMPLETION_DIGEST_FNV1A64_CONTRACT,
    DIGEST_SHA256_AUTHORITY_CONTRACT as PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT,
    DIGEST_SHA256_CONTRACT as PROVIDER_COMPLETION_DIGEST_SHA256_CONTRACT,
    DIGEST_SHA256_SIGNED_CONTRACT as PROVIDER_COMPLETION_DIGEST_SHA256_SIGNED_CONTRACT,
};
use crate::provider_completion_signature::{
    handoff_error_status as provider_completion_signature_error, sign_from_environment,
    signing_key_configured, verify_from_environment,
    SIGNATURE_CONTRACT as PROVIDER_COMPLETION_SIGNATURE_CONTRACT,
};
use std::{fs, path::Path};

const HANDOFF_FILE_NAME: &str = "nuis.nsdb.payload-execution-handoff.toml";
const HANDOFF_PROTOCOL: &str = "nuis-nsdb-payload-execution-handoff-v1";
const DEBUGGER_CONTRACT: &str = "nsdb-yir-payload-execution-trace-v1";

pub(crate) fn persist_provider_completion_handoff(
    output_dir: &Path,
    records: &[NsdbDeviceProviderSampleRecordInfo],
) -> Result<usize, String> {
    let completions = records
        .iter()
        .filter(|record| {
            record.materialization_status == "provider-sample-materialized"
                && record.sample_status == "provider-execution-ready"
                && record.validation_status == "provider-execution-validated"
        })
        .map(provider_completion_event)
        .collect::<Vec<_>>();
    if completions.is_empty() {
        return Ok(0);
    }
    for completion in completions {
        persist_payload_execution_handoff_record(
            output_dir,
            "nsdb-provider-sample-materialization",
            public_handoff_record(completion),
        )?;
    }
    Ok(read_payload_execution_handoff(output_dir)
        .events
        .iter()
        .filter(|event| event.execution_phase == "provider-device-completion")
        .count())
}

pub(crate) fn persist_payload_execution_handoff_record(
    output_dir: &Path,
    source: &str,
    record: PayloadExecutionHandoffRecord,
) -> Result<PayloadExecutionHandoffPersistSummary, String> {
    let existing = read_payload_execution_handoff(output_dir);
    if existing.available
        && matches!(
            existing
                .provider_completion_set_hash_validation_status
                .as_str(),
            "mismatch" | "unsupported-digest-contract"
        )
    {
        return Err(format!(
            "provider completion digest validation failed in existing handoff: {}",
            existing.provider_completion_set_hash_validation_status
        ));
    }
    if existing.available
        && matches!(
            existing.provider_completion_claim_authority_status.as_str(),
            "authority-missing" | "unsupported-authority-contract" | "authority-untrusted"
        )
    {
        return Err(format!(
            "provider completion authority validation failed in existing handoff: {}",
            existing.provider_completion_claim_authority_status
        ));
    }
    if existing.available
        && !matches!(
            existing.provider_completion_signature_status.as_str(),
            "not-applicable" | "legacy-unsigned" | "signature-verified"
        )
    {
        return Err(format!(
            "provider completion signature validation failed in existing handoff: {}",
            existing.provider_completion_signature_status
        ));
    }
    let mut events = if existing.available
        && existing.protocol == HANDOFF_PROTOCOL
        && existing.debugger_contract == DEBUGGER_CONTRACT
    {
        existing.events.clone()
    } else {
        Vec::new()
    };
    let replacement = internal_handoff_event(record);
    if let Some(index) = events.iter().position(|event| {
        event.trace_id == replacement.trace_id
            && event.execution_phase == replacement.execution_phase
    }) {
        events[index] = replacement;
    } else {
        events.push(replacement);
    }
    for (index, event) in events.iter_mut().enumerate() {
        event.index = index;
    }
    let content = render_payload_execution_handoff(&events, &existing, source)?;
    let path = output_dir.join(HANDOFF_FILE_NAME);
    fs::write(&path, content).map_err(|error| {
        format!(
            "failed to persist payload execution handoff `{}`: {error}",
            path.display()
        )
    })?;
    Ok(PayloadExecutionHandoffPersistSummary {
        record_count: events.len(),
        ready_record_count: events
            .iter()
            .filter(|event| event.status == "ready")
            .count(),
        first_trace_id: events.first().map(|event| event.trace_id.clone()),
    })
}

fn public_handoff_record(event: NsdbPayloadExecutionEvent) -> PayloadExecutionHandoffRecord {
    PayloadExecutionHandoffRecord {
        trace_id: event.trace_id,
        status: event.status,
        execution_phase: event.execution_phase,
        target: event.target,
        entry_symbol: event.entry_symbol,
        entry_kind: event.entry_kind,
        entry_section_id: event.entry_section_id,
        provider_family: event.provider_family,
        output_contract: event.output_contract,
        output_evidence: event.output_evidence,
        first_blocker: event.first_blocker,
        next_action: event.next_action,
    }
}

fn internal_handoff_event(record: PayloadExecutionHandoffRecord) -> NsdbPayloadExecutionEvent {
    NsdbPayloadExecutionEvent {
        index: 0,
        trace_id: record.trace_id,
        status: record.status,
        execution_phase: record.execution_phase,
        target: record.target,
        entry_symbol: record.entry_symbol,
        entry_kind: record.entry_kind,
        entry_section_id: record.entry_section_id,
        provider_family: record.provider_family,
        output_contract: record.output_contract,
        output_evidence: record.output_evidence,
        first_blocker: record.first_blocker,
        next_action: record.next_action,
    }
}

fn provider_completion_event(
    record: &NsdbDeviceProviderSampleRecordInfo,
) -> NsdbPayloadExecutionEvent {
    let output_evidence = if !matches!(
        record.provider_output_payload_evidence.as_str(),
        "none" | "not-materialized"
    ) {
        record.provider_output_payload_evidence.clone()
    } else {
        record.output_evidence.clone()
    };
    let output_contract = if record.provider_output_payload_contract == "none" {
        "nsdb-yir-provider-sample-artifact-v1".to_owned()
    } else {
        record.provider_output_payload_contract.clone()
    };
    NsdbPayloadExecutionEvent {
        index: 0,
        trace_id: record.trace_id.clone(),
        status: "ready".to_owned(),
        execution_phase: "provider-device-completion".to_owned(),
        target: record.provider_family.clone(),
        entry_symbol: record.provider.clone(),
        entry_kind: output_contract.clone(),
        entry_section_id: output_evidence.clone(),
        provider_family: record.provider_family.clone(),
        output_contract,
        output_evidence,
        first_blocker: "none".to_owned(),
        next_action: "replay-provider-completion".to_owned(),
    }
}

fn render_payload_execution_handoff(
    events: &[NsdbPayloadExecutionEvent],
    existing: &NsdbPayloadExecutionHandoffInfo,
    source: &str,
) -> Result<String, String> {
    let ready_count = events
        .iter()
        .filter(|event| event.status == "ready")
        .count();
    let first = events.first();
    let mut out = String::new();
    push_toml_string(&mut out, "protocol", HANDOFF_PROTOCOL);
    push_toml_string(&mut out, "debugger_contract", DEBUGGER_CONTRACT);
    push_toml_string(&mut out, "source", source);
    out.push_str(&format!("record_count = {}\n", events.len()));
    out.push_str(&format!("ready_record_count = {ready_count}\n"));
    push_toml_string(
        &mut out,
        "first_trace_id",
        first.map(|event| event.trace_id.as_str()).unwrap_or("none"),
    );
    push_toml_string(
        &mut out,
        "first_status",
        first.map(|event| event.status.as_str()).unwrap_or("none"),
    );
    push_toml_string(
        &mut out,
        "first_next_action",
        first
            .map(|event| event.next_action.as_str())
            .unwrap_or("none"),
    );
    push_toml_string(
        &mut out,
        "provider_completion_claim_authority_contract",
        PROVIDER_COMPLETION_CLAIM_AUTHORITY_CONTRACT,
    );
    push_toml_string(
        &mut out,
        "provider_completion_claim_authority",
        PROVIDER_COMPLETION_CLAIM_AUTHORITY,
    );
    push_toml_string(
        &mut out,
        "provider_completion_digest_contract",
        if signing_key_configured() {
            PROVIDER_COMPLETION_DIGEST_SHA256_SIGNED_CONTRACT
        } else {
            PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT
        },
    );
    let digest_contract = if signing_key_configured() {
        PROVIDER_COMPLETION_DIGEST_SHA256_SIGNED_CONTRACT
    } else {
        PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT
    };
    let set_hash = provider_completion_set_hash(
        events,
        HANDOFF_PROTOCOL,
        events.len(),
        digest_contract,
        PROVIDER_COMPLETION_CLAIM_AUTHORITY_CONTRACT,
        PROVIDER_COMPLETION_CLAIM_AUTHORITY,
    )
    .unwrap_or_else(|| "none".to_owned());
    push_toml_string(&mut out, "provider_completion_set_hash", &set_hash);
    let signature_message = signature_message(
        HANDOFF_PROTOCOL,
        digest_contract,
        PROVIDER_COMPLETION_CLAIM_AUTHORITY_CONTRACT,
        PROVIDER_COMPLETION_CLAIM_AUTHORITY,
        &set_hash,
    );
    if let Some(signature) = sign_from_environment(&signature_message)? {
        push_toml_string(
            &mut out,
            "provider_completion_signature_contract",
            PROVIDER_COMPLETION_SIGNATURE_CONTRACT,
        );
        push_toml_string(
            &mut out,
            "provider_completion_signature_public_key_id",
            &signature.public_key_id,
        );
        push_toml_string(
            &mut out,
            "provider_completion_signature",
            &signature.signature_hex,
        );
    }
    push_toml_string(
        &mut out,
        "hetero_execution_closure_protocol",
        &existing.hetero_execution_closure_protocol,
    );
    push_toml_string(
        &mut out,
        "hetero_execution_closure_status",
        &existing.hetero_execution_closure_status,
    );
    push_toml_string(
        &mut out,
        "hetero_execution_closure_ready",
        &existing.hetero_execution_closure_ready,
    );
    push_toml_string(
        &mut out,
        "hetero_execution_closure_first_blocker",
        if existing.hetero_execution_closure_first_blocker == "none" {
            ""
        } else {
            &existing.hetero_execution_closure_first_blocker
        },
    );
    push_toml_string(
        &mut out,
        "hetero_execution_closure_next_action",
        &existing.hetero_execution_closure_next_action,
    );
    for event in events {
        out.push_str("\n[[records]]\n");
        push_toml_string(&mut out, "trace_id", &event.trace_id);
        push_toml_string(&mut out, "status", &event.status);
        push_toml_string(&mut out, "execution_phase", &event.execution_phase);
        push_toml_string(&mut out, "target", &event.target);
        push_toml_string(&mut out, "entry_symbol", &event.entry_symbol);
        push_toml_string(&mut out, "entry_kind", &event.entry_kind);
        push_toml_string(&mut out, "entry_section_id", &event.entry_section_id);
        push_toml_string(&mut out, "provider_family", &event.provider_family);
        push_toml_string(&mut out, "output_contract", &event.output_contract);
        push_toml_string(&mut out, "output_evidence", &event.output_evidence);
        push_toml_string(
            &mut out,
            "first_blocker",
            if event.first_blocker == "none" {
                ""
            } else {
                &event.first_blocker
            },
        );
        push_toml_string(&mut out, "next_action", &event.next_action);
    }
    Ok(out)
}

pub(crate) fn read_payload_execution_handoff(output_dir: &Path) -> NsdbPayloadExecutionHandoffInfo {
    let path = output_dir.join(HANDOFF_FILE_NAME);
    let Ok(source) = fs::read_to_string(&path) else {
        return NsdbPayloadExecutionHandoffInfo {
            available: false,
            path: path.display().to_string(),
            protocol: "none".to_owned(),
            debugger_contract: "none".to_owned(),
            status: "missing".to_owned(),
            record_count: 0,
            ready_record_count: 0,
            first_trace_id: "none".to_owned(),
            first_status: "none".to_owned(),
            first_next_action: "none".to_owned(),
            first_entry_symbol: "none".to_owned(),
            first_execution_phase: "none".to_owned(),
            provider_completion_claim_authority_contract: "none".to_owned(),
            provider_completion_claim_authority: "none".to_owned(),
            provider_completion_claim_authority_status: "not-applicable".to_owned(),
            provider_completion_signature_contract: "none".to_owned(),
            provider_completion_signature_public_key_id: "none".to_owned(),
            provider_completion_signature: "none".to_owned(),
            provider_completion_signature_status: "not-applicable".to_owned(),
            provider_completion_digest_contract: "none".to_owned(),
            provider_completion_set_hash_claim: "none".to_owned(),
            provider_completion_set_hash_actual: "none".to_owned(),
            provider_completion_set_hash_validation_status: "not-applicable".to_owned(),
            hetero_execution_closure_protocol: "none".to_owned(),
            hetero_execution_closure_status: "none".to_owned(),
            hetero_execution_closure_ready: "false".to_owned(),
            hetero_execution_closure_first_blocker: "none".to_owned(),
            hetero_execution_closure_next_action: "none".to_owned(),
            events: Vec::new(),
        };
    };
    let protocol =
        parse_string_toml_field(&source, "protocol").unwrap_or_else(|| "none".to_owned());
    let debugger_contract =
        parse_string_toml_field(&source, "debugger_contract").unwrap_or_else(|| "none".to_owned());
    let record_count = parse_usize_toml_field(&source, "record_count").unwrap_or(0);
    let ready_record_count = parse_usize_toml_field(&source, "ready_record_count").unwrap_or(0);
    let events = parse_payload_execution_events(&source);
    let provider_completion_claim_authority_contract =
        parse_string_toml_field(&source, "provider_completion_claim_authority_contract")
            .unwrap_or_else(|| "none".to_owned());
    let provider_completion_claim_authority =
        parse_string_toml_field(&source, "provider_completion_claim_authority")
            .unwrap_or_else(|| "none".to_owned());
    let provider_completion_signature_contract =
        parse_string_toml_field(&source, "provider_completion_signature_contract")
            .unwrap_or_else(|| "none".to_owned());
    let provider_completion_signature_public_key_id =
        parse_string_toml_field(&source, "provider_completion_signature_public_key_id")
            .unwrap_or_else(|| "none".to_owned());
    let provider_completion_signature =
        parse_string_toml_field(&source, "provider_completion_signature")
            .unwrap_or_else(|| "none".to_owned());
    let provider_completion_digest_contract =
        parse_string_toml_field(&source, "provider_completion_digest_contract")
            .unwrap_or_else(|| "none".to_owned());
    let provider_completion_set_hash_claim =
        parse_string_toml_field(&source, "provider_completion_set_hash")
            .unwrap_or_else(|| "none".to_owned());
    let has_provider_completions = events
        .iter()
        .any(|event| event.execution_phase == "provider-device-completion");
    let provider_completion_set_hash_actual = match provider_completion_digest_contract.as_str() {
        "none" => legacy_provider_completion_set_hash(&events),
        PROVIDER_COMPLETION_DIGEST_FNV1A64_CONTRACT
        | PROVIDER_COMPLETION_DIGEST_SHA256_CONTRACT
        | PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT
        | PROVIDER_COMPLETION_DIGEST_SHA256_SIGNED_CONTRACT => provider_completion_set_hash(
            &events,
            &protocol,
            record_count,
            &provider_completion_digest_contract,
            &provider_completion_claim_authority_contract,
            &provider_completion_claim_authority,
        ),
        _ => None,
    }
    .unwrap_or_else(|| "none".to_owned());
    let provider_completion_set_hash_validation_status = if !has_provider_completions {
        "not-applicable"
    } else if provider_completion_digest_contract != "none"
        && provider_completion_digest_contract != PROVIDER_COMPLETION_DIGEST_FNV1A64_CONTRACT
        && provider_completion_digest_contract != PROVIDER_COMPLETION_DIGEST_SHA256_CONTRACT
        && provider_completion_digest_contract
            != PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT
        && provider_completion_digest_contract != PROVIDER_COMPLETION_DIGEST_SHA256_SIGNED_CONTRACT
    {
        "unsupported-digest-contract"
    } else if provider_completion_set_hash_claim == "none" {
        "legacy-unclaimed"
    } else if provider_completion_set_hash_claim == provider_completion_set_hash_actual {
        if provider_completion_digest_contract == "none" {
            "legacy-verified"
        } else {
            "verified"
        }
    } else {
        "mismatch"
    }
    .to_owned();
    let provider_completion_claim_authority_status = if !has_provider_completions {
        "not-applicable"
    } else if provider_completion_digest_contract
        != PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT
        && provider_completion_digest_contract != PROVIDER_COMPLETION_DIGEST_SHA256_SIGNED_CONTRACT
    {
        "legacy-unattributed"
    } else if provider_completion_claim_authority_contract == "none"
        || provider_completion_claim_authority == "none"
    {
        "authority-missing"
    } else if provider_completion_claim_authority_contract
        != PROVIDER_COMPLETION_CLAIM_AUTHORITY_CONTRACT
    {
        "unsupported-authority-contract"
    } else if provider_completion_claim_authority != PROVIDER_COMPLETION_CLAIM_AUTHORITY {
        "authority-untrusted"
    } else {
        "authorized"
    }
    .to_owned();
    let provider_completion_signature_status = if !has_provider_completions {
        "not-applicable"
    } else if provider_completion_digest_contract
        != PROVIDER_COMPLETION_DIGEST_SHA256_SIGNED_CONTRACT
        && provider_completion_signature_contract == "none"
        && provider_completion_signature_public_key_id == "none"
        && provider_completion_signature == "none"
    {
        "legacy-unsigned"
    } else {
        let message = signature_message(
            &protocol,
            &provider_completion_digest_contract,
            &provider_completion_claim_authority_contract,
            &provider_completion_claim_authority,
            &provider_completion_set_hash_actual,
        );
        verify_from_environment(
            &provider_completion_signature_contract,
            &provider_completion_signature_public_key_id,
            &provider_completion_signature,
            &message,
        )
    }
    .to_owned();
    let first_event = events.first();
    let first_status = parse_string_toml_field(&source, "first_status")
        .or_else(|| first_event.map(|event| event.status.clone()))
        .unwrap_or_else(|| "none".to_owned());
    let status = payload_handoff_status(
        &protocol,
        &debugger_contract,
        record_count,
        &first_status,
        &provider_completion_set_hash_validation_status,
        &provider_completion_claim_authority_status,
        &provider_completion_signature_status,
    );
    NsdbPayloadExecutionHandoffInfo {
        available: true,
        path: path.display().to_string(),
        protocol,
        debugger_contract,
        status,
        record_count,
        ready_record_count,
        first_trace_id: parse_string_toml_field(&source, "first_trace_id")
            .or_else(|| first_event.map(|event| event.trace_id.clone()))
            .unwrap_or_else(|| "none".to_owned()),
        first_status,
        first_next_action: parse_string_toml_field(&source, "first_next_action")
            .or_else(|| first_event.map(|event| event.next_action.clone()))
            .unwrap_or_else(|| "none".to_owned()),
        first_entry_symbol: first_event
            .map(|event| event.entry_symbol.clone())
            .unwrap_or_else(|| "none".to_owned()),
        first_execution_phase: first_event
            .map(|event| event.execution_phase.clone())
            .unwrap_or_else(|| "none".to_owned()),
        provider_completion_claim_authority_contract,
        provider_completion_claim_authority,
        provider_completion_claim_authority_status,
        provider_completion_signature_contract,
        provider_completion_signature_public_key_id,
        provider_completion_signature,
        provider_completion_signature_status,
        provider_completion_digest_contract,
        provider_completion_set_hash_claim,
        provider_completion_set_hash_actual,
        provider_completion_set_hash_validation_status,
        hetero_execution_closure_protocol: parse_string_toml_field(
            &source,
            "hetero_execution_closure_protocol",
        )
        .unwrap_or_else(|| "none".to_owned()),
        hetero_execution_closure_status: parse_string_toml_field(
            &source,
            "hetero_execution_closure_status",
        )
        .unwrap_or_else(|| "none".to_owned()),
        hetero_execution_closure_ready: parse_string_toml_field(
            &source,
            "hetero_execution_closure_ready",
        )
        .unwrap_or_else(|| "false".to_owned()),
        hetero_execution_closure_first_blocker: parse_string_toml_field(
            &source,
            "hetero_execution_closure_first_blocker",
        )
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "none".to_owned()),
        hetero_execution_closure_next_action: parse_string_toml_field(
            &source,
            "hetero_execution_closure_next_action",
        )
        .unwrap_or_else(|| "none".to_owned()),
        events,
    }
}

fn payload_handoff_status(
    protocol: &str,
    debugger_contract: &str,
    record_count: usize,
    first_status: &str,
    provider_completion_set_hash_validation_status: &str,
    provider_completion_claim_authority_status: &str,
    provider_completion_signature_status: &str,
) -> String {
    if protocol != "nuis-nsdb-payload-execution-handoff-v1" {
        return "unsupported-protocol".to_owned();
    }
    if debugger_contract != "nsdb-yir-payload-execution-trace-v1" {
        return "unsupported-debugger-contract".to_owned();
    }
    if record_count == 0 {
        return "empty".to_owned();
    }
    match provider_completion_set_hash_validation_status {
        "mismatch" => return "provider-completion-set-hash-mismatch".to_owned(),
        "unsupported-digest-contract" => {
            return "provider-completion-digest-contract-unsupported".to_owned()
        }
        _ => {}
    }
    match provider_completion_claim_authority_status {
        "authority-missing" => return "provider-completion-claim-authority-missing".to_owned(),
        "unsupported-authority-contract" => {
            return "provider-completion-claim-authority-contract-unsupported".to_owned()
        }
        "authority-untrusted" => return "provider-completion-claim-authority-untrusted".to_owned(),
        _ => {}
    }
    if let Some(status) = provider_completion_signature_error(provider_completion_signature_status)
    {
        return status.to_owned();
    }
    if first_status == "ready" {
        "ready".to_owned()
    } else {
        "blocked".to_owned()
    }
}

fn parse_payload_execution_events(source: &str) -> Vec<NsdbPayloadExecutionEvent> {
    source
        .split("[[records]]")
        .skip(1)
        .enumerate()
        .map(|(index, record)| NsdbPayloadExecutionEvent {
            index,
            trace_id: parse_string_toml_field(record, "trace_id")
                .unwrap_or_else(|| "none".to_owned()),
            status: parse_string_toml_field(record, "status").unwrap_or_else(|| "none".to_owned()),
            execution_phase: parse_string_toml_field(record, "execution_phase")
                .unwrap_or_else(|| "none".to_owned()),
            target: parse_string_toml_field(record, "target").unwrap_or_else(|| "none".to_owned()),
            entry_symbol: parse_string_toml_field(record, "entry_symbol")
                .unwrap_or_else(|| "none".to_owned()),
            entry_kind: parse_string_toml_field(record, "entry_kind")
                .unwrap_or_else(|| "none".to_owned()),
            entry_section_id: parse_string_toml_field(record, "entry_section_id")
                .unwrap_or_else(|| "none".to_owned()),
            provider_family: parse_string_toml_field(record, "provider_family")
                .unwrap_or_else(|| "none".to_owned()),
            output_contract: parse_string_toml_field(record, "output_contract")
                .unwrap_or_else(|| "none".to_owned()),
            output_evidence: parse_string_toml_field(record, "output_evidence")
                .unwrap_or_else(|| "none".to_owned()),
            first_blocker: parse_string_toml_field(record, "first_blocker")
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "none".to_owned()),
            next_action: parse_string_toml_field(record, "next_action")
                .unwrap_or_else(|| "none".to_owned()),
        })
        .collect()
}

fn parse_usize_toml_field(source: &str, key: &str) -> Option<usize> {
    parse_toml_field_value(source, key)?.parse().ok()
}

fn parse_string_toml_field(source: &str, key: &str) -> Option<String> {
    let value = parse_toml_field_value(source, key)?;
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(unescape_basic_toml_string)
}

fn parse_toml_field_value<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key} = ");
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(str::trim))
}

fn unescape_basic_toml_string(value: &str) -> String {
    let mut out = String::new();
    let mut escaped = false;
    for ch in value.chars() {
        if escaped {
            out.push(match ch {
                '"' => '"',
                '\\' => '\\',
                'n' => '\n',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            out.push(ch);
        }
    }
    out
}

fn push_toml_string(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(
        &value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\t', "\\t"),
    );
    out.push_str("\"\n");
}

#[cfg(test)]
mod tests {
    use super::read_payload_execution_handoff;
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn reads_ready_payload_execution_handoff() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir: PathBuf = env::temp_dir().join(format!("nsdb-handoff-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("nuis.nsdb.payload-execution-handoff.toml"),
            r#"
protocol = "nuis-nsdb-payload-execution-handoff-v1"
debugger_contract = "nsdb-yir-payload-execution-trace-v1"
record_count = 2
ready_record_count = 1
hetero_execution_closure_protocol = "nuis-hetero-execution-closure-v1"
hetero_execution_closure_status = "closed"
hetero_execution_closure_ready = "true"
hetero_execution_closure_next_action = "handoff-hetero-execution-evidence-to-nsdb"
first_trace_id = "payload-trace:container-loader:nuis.bootstrap.lifecycle.v1"
first_status = "ready"
first_next_action = "handoff-payload-trace-to-nsdb"

[[records]]
trace_id = "payload-trace:container-loader:nuis.bootstrap.lifecycle.v1"
status = "ready"
execution_phase = "container-loader-handoff"
entry_symbol = "nuis.bootstrap.lifecycle.v1"
next_action = "handoff-payload-trace-to-nsdb"

[[records]]
trace_id = "payload-trace:shader:pixelmagic.blur"
status = "blocked"
execution_phase = "device-dispatch"
target = "shader"
entry_symbol = "pixelmagic.blur"
entry_kind = "shader-kernel"
entry_section_id = "sec0002.shader"
first_blocker = "device-execution-sample-missing"
next_action = "materialize-device-execution-trace"
"#,
        )
        .unwrap();

        let handoff = read_payload_execution_handoff(&dir);

        assert!(handoff.available);
        assert_eq!(handoff.status, "ready");
        assert_eq!(handoff.record_count, 2);
        assert_eq!(handoff.events.len(), 2);
        assert_eq!(handoff.events[0].index, 0);
        assert_eq!(handoff.events[0].trace_id, handoff.first_trace_id);
        assert_eq!(
            handoff.events[0].next_action,
            "handoff-payload-trace-to-nsdb"
        );
        assert_eq!(handoff.events[1].index, 1);
        assert_eq!(handoff.events[1].status, "blocked");
        assert_eq!(
            handoff.events[1].first_blocker,
            "device-execution-sample-missing"
        );
        assert_eq!(handoff.first_execution_phase, "container-loader-handoff");
        assert_eq!(handoff.first_entry_symbol, "nuis.bootstrap.lifecycle.v1");
        assert_eq!(
            handoff.hetero_execution_closure_protocol,
            "nuis-hetero-execution-closure-v1"
        );
        assert_eq!(handoff.hetero_execution_closure_status, "closed");
        assert_eq!(handoff.hetero_execution_closure_ready, "true");
        assert_eq!(
            handoff.hetero_execution_closure_next_action,
            "handoff-hetero-execution-evidence-to-nsdb"
        );
    }
}
