use crate::{
    payload_execution_replay_summary, persist_payload_execution_handoff_record,
    PayloadExecutionHandoffRecord,
};
use std::fs;

#[test]
fn provider_completion_collection_preserves_order_and_hashes_records() {
    let dir = std::env::temp_dir().join(format!(
        "nsdb-provider-completion-set-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    for (trace_id, family, evidence) in [
        (
            "hetero-trace:shader:metal:apple-silicon-gpu",
            "metal:apple-silicon-gpu",
            "metal-output:hash=0x1234",
        ),
        (
            "hetero-trace:kernel:coreml:apple-ane",
            "coreml:apple-ane",
            "coreml-output:hash=0x5678",
        ),
    ] {
        persist_payload_execution_handoff_record(
            &dir,
            "provider-set-test",
            PayloadExecutionHandoffRecord {
                trace_id: trace_id.to_owned(),
                status: "ready".to_owned(),
                execution_phase: "provider-device-completion".to_owned(),
                target: family.to_owned(),
                entry_symbol: "registered-provider".to_owned(),
                entry_kind: "nuis-provider-output-payload-handoff-v1".to_owned(),
                entry_section_id: evidence.to_owned(),
                provider_family: family.to_owned(),
                output_contract: "nuis-provider-output-payload-handoff-v1".to_owned(),
                output_evidence: evidence.to_owned(),
                first_blocker: String::new(),
                next_action: "replay-provider-completion".to_owned(),
            },
        )
        .unwrap();
    }

    let summary = payload_execution_replay_summary(&dir);

    assert_eq!(summary.provider_completion_count, 2);
    assert_eq!(summary.provider_completions.len(), 2);
    assert_eq!(
        summary.provider_completions[0].provider_family,
        "metal:apple-silicon-gpu"
    );
    assert_eq!(
        summary.provider_completions[1].provider_family,
        "coreml:apple-ane"
    );
    assert_ne!(
        summary.provider_completions[0].record_hash,
        summary.provider_completions[1].record_hash
    );
    assert!(summary
        .provider_completion_set_hash
        .as_deref()
        .is_some_and(|hash| hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit())));
    assert_eq!(
        summary.provider_completion_digest_contract.as_deref(),
        Some("nuis-provider-completion-digest-sha256-authority-v1")
    );
    assert_eq!(
        summary.provider_completion_claim_authority.as_deref(),
        Some("nsdb:payload-execution-handoff-writer:v1")
    );
    assert_eq!(
        summary.provider_completion_claim_authority_status,
        "authorized"
    );
    let path = dir.join("nuis.nsdb.payload-execution-handoff.toml");
    let source = fs::read_to_string(&path).unwrap();
    assert!(source.contains(
        "provider_completion_digest_contract = \"nuis-provider-completion-digest-sha256-authority-v1\""
    ));
    let claim = summary.provider_completion_set_hash.as_deref().unwrap();
    assert!(source.contains(&format!("provider_completion_set_hash = \"{claim}\"")));
    fs::write(
        &path,
        source.replacen("record_count = 2", "record_count = 3", 1),
    )
    .unwrap();
    let count_rejected = payload_execution_replay_summary(&dir);
    assert_eq!(
        count_rejected.provider_completion_set_hash_validation_status,
        "mismatch"
    );
    assert_eq!(
        count_rejected.first_blocker.as_deref(),
        Some("payload-execution-handoff:provider-completion-set-hash-mismatch")
    );
    fs::write(
        &path,
        source.replace("coreml-output:hash=0x5678", "coreml-output:hash=0xtampered"),
    )
    .unwrap();
    let rejected = payload_execution_replay_summary(&dir);
    let rewrite = persist_payload_execution_handoff_record(
        &dir,
        "provider-set-test",
        PayloadExecutionHandoffRecord {
            trace_id: "hetero-trace:kernel:coreml:apple-ane".to_owned(),
            status: "ready".to_owned(),
            execution_phase: "provider-device-completion".to_owned(),
            target: "coreml:apple-ane".to_owned(),
            entry_symbol: "registered-provider".to_owned(),
            entry_kind: "nuis-provider-output-payload-handoff-v1".to_owned(),
            entry_section_id: "coreml-output:hash=0x5678".to_owned(),
            provider_family: "coreml:apple-ane".to_owned(),
            output_contract: "nuis-provider-output-payload-handoff-v1".to_owned(),
            output_evidence: "coreml-output:hash=0x5678".to_owned(),
            first_blocker: String::new(),
            next_action: "replay-provider-completion".to_owned(),
        },
    );
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(rejected.status, "blocked");
    assert_eq!(
        rejected.first_blocker.as_deref(),
        Some("payload-execution-handoff:provider-completion-set-hash-mismatch")
    );
    assert_eq!(
        rewrite.unwrap_err(),
        "provider completion digest validation failed in existing handoff: mismatch"
    );
}
