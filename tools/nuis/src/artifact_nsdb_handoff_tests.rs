use super::read_persisted_nsdb_handoff;
use std::{env, fs};

#[test]
fn independently_reads_structured_provider_completion() {
    let dir = env::temp_dir().join(format!(
        "nuis-provider-completion-handoff-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("nuis.nsdb.payload-execution-handoff.toml"),
        r#"
protocol = "nuis-nsdb-payload-execution-handoff-v1"
debugger_contract = "nsdb-yir-payload-execution-trace-v1"
record_count = 2
ready_record_count = 2

[[records]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
status = "ready"
execution_phase = "provider-device-completion"
provider_family = "metal:apple-silicon-gpu"
output_contract = "nuis-provider-output-payload-handoff-v1"
output_evidence = "provider-output.toml:hash=0x1234"

[[records]]
trace_id = "hetero-trace:kernel:coreml:apple-ane"
status = "ready"
execution_phase = "provider-device-completion"
provider_family = "coreml:apple-ane"
output_contract = "nuis-provider-output-payload-handoff-v1"
output_evidence = "coreml-output.toml:hash=0x5678"
"#,
    )
    .unwrap();

    let summary = read_persisted_nsdb_handoff(Some(&dir));
    assert_eq!(summary.provider_completion_count(), 2);
    assert_eq!(
        summary.first_provider_family(),
        Some("metal:apple-silicon-gpu")
    );
    assert_eq!(
        summary.first_provider_output_contract(),
        Some("nuis-provider-output-payload-handoff-v1")
    );
    assert_eq!(
        summary.first_provider_output_evidence(),
        Some("provider-output.toml:hash=0x1234")
    );
    assert_eq!(summary.provider_completions().len(), 2);
    assert_eq!(
        summary.provider_completions()[1].provider_family,
        "coreml:apple-ane"
    );
    assert!(summary.provider_completions()[0]
        .record_hash
        .starts_with("0x"));
    assert_eq!(
        summary.provider_completion_set_hash_validation_status(),
        "legacy-unclaimed"
    );

    let path = dir.join("nuis.nsdb.payload-execution-handoff.toml");
    let source = fs::read_to_string(&path).unwrap();
    let hash = summary.provider_completion_set_hash().unwrap();
    let claimed_source = source.replace(
        "ready_record_count = 2",
        &format!("ready_record_count = 2\nprovider_completion_set_hash = \"{hash}\""),
    );
    fs::write(&path, &claimed_source).unwrap();
    let verified = read_persisted_nsdb_handoff(Some(&dir));
    assert_eq!(verified.provider_completion_set_hash_claim(), Some(hash));
    assert_eq!(
        verified.provider_completion_set_hash_validation_status(),
        "legacy-verified"
    );
    assert_eq!(verified.error(), None);

    let versioned_source = claimed_source.replace(
        "provider_completion_set_hash = ",
        "provider_completion_digest_contract = \"nuis-provider-completion-digest-fnv1a64-v1\"\nprovider_completion_set_hash = ",
    );
    fs::write(&path, &versioned_source).unwrap();
    let stale_claim = read_persisted_nsdb_handoff(Some(&dir));
    assert_eq!(
        stale_claim.provider_completion_set_hash_validation_status(),
        "mismatch"
    );
    let versioned_hash = stale_claim
        .provider_completion_set_hash()
        .unwrap()
        .to_owned();
    let versioned_claimed_source = versioned_source.replace(hash, &versioned_hash);
    fs::write(&path, &versioned_claimed_source).unwrap();
    let versioned = read_persisted_nsdb_handoff(Some(&dir));
    assert_eq!(
        versioned.provider_completion_digest_contract(),
        Some("nuis-provider-completion-digest-fnv1a64-v1")
    );
    assert_eq!(
        versioned.provider_completion_set_hash_validation_status(),
        "verified"
    );
    assert_eq!(versioned.error(), None);

    let sha_source = versioned_claimed_source.replace(
        "nuis-provider-completion-digest-fnv1a64-v1",
        "nuis-provider-completion-digest-sha256-v1",
    );
    fs::write(&path, &sha_source).unwrap();
    let stale_sha_claim = read_persisted_nsdb_handoff(Some(&dir));
    assert_eq!(
        stale_sha_claim.provider_completion_set_hash_validation_status(),
        "mismatch"
    );
    let sha_hash = stale_sha_claim
        .provider_completion_set_hash()
        .unwrap()
        .to_owned();
    assert_eq!(sha_hash.len(), 64);
    let sha_claimed_source = sha_source.replace(&versioned_hash, &sha_hash);
    fs::write(&path, &sha_claimed_source).unwrap();
    let sha_verified = read_persisted_nsdb_handoff(Some(&dir));
    assert_eq!(
        sha_verified.provider_completion_digest_contract(),
        Some("nuis-provider-completion-digest-sha256-v1")
    );
    assert_eq!(
        sha_verified.provider_completion_set_hash_validation_status(),
        "verified"
    );
    assert!(sha_verified
        .provider_completions()
        .iter()
        .all(|completion| completion.record_hash.len() == 64));
    assert_eq!(
        sha_verified.provider_completion_claim_authority_status(),
        "legacy-unattributed"
    );

    let authority_source = sha_claimed_source.replace(
        "nuis-provider-completion-digest-sha256-v1",
        "nuis-provider-completion-digest-sha256-authority-v1",
    );
    fs::write(&path, &authority_source).unwrap();
    let missing_stale = read_persisted_nsdb_handoff(Some(&dir));
    let missing_hash = missing_stale
        .provider_completion_set_hash()
        .unwrap()
        .to_owned();
    let missing_claimed_source = authority_source.replace(&sha_hash, &missing_hash);
    fs::write(&path, &missing_claimed_source).unwrap();
    let missing = read_persisted_nsdb_handoff(Some(&dir));
    assert_eq!(
        missing.provider_completion_set_hash_validation_status(),
        "verified"
    );
    assert_eq!(
        missing.provider_completion_claim_authority_status(),
        "authority-missing"
    );
    assert_eq!(
        missing.error(),
        Some("provider-completion-claim-authority-missing")
    );

    let authority_source = authority_source.replace(
        "provider_completion_digest_contract = ",
        "provider_completion_claim_authority_contract = \"nuis-provider-completion-claim-authority-v1\"\nprovider_completion_claim_authority = \"nsdb:payload-execution-handoff-writer:v1\"\nprovider_completion_digest_contract = ",
    );
    fs::write(&path, &authority_source).unwrap();
    let authority_stale = read_persisted_nsdb_handoff(Some(&dir));
    let authority_hash = authority_stale
        .provider_completion_set_hash()
        .unwrap()
        .to_owned();
    let authority_claimed_source = authority_source.replace(&sha_hash, &authority_hash);
    fs::write(&path, &authority_claimed_source).unwrap();
    let authorized = read_persisted_nsdb_handoff(Some(&dir));
    assert_eq!(
        authorized.provider_completion_claim_authority_contract(),
        Some("nuis-provider-completion-claim-authority-v1")
    );
    assert_eq!(
        authorized.provider_completion_claim_authority(),
        Some("nsdb:payload-execution-handoff-writer:v1")
    );
    assert_eq!(
        authorized.provider_completion_claim_authority_status(),
        "authorized"
    );
    assert_eq!(authorized.error(), None);

    fs::write(
        &path,
        sha_claimed_source.replace(
            "nuis-provider-completion-digest-sha256-v1",
            "nuis-provider-completion-digest-unknown-v9",
        ),
    )
    .unwrap();
    let unsupported = read_persisted_nsdb_handoff(Some(&dir));
    assert_eq!(
        unsupported.provider_completion_set_hash_validation_status(),
        "unsupported-digest-contract"
    );
    assert_eq!(
        unsupported.error(),
        Some("provider-completion-digest-contract-unsupported")
    );

    fs::write(
        &path,
        sha_claimed_source.replace(&sha_hash, &"0".repeat(64)),
    )
    .unwrap();
    let rejected = read_persisted_nsdb_handoff(Some(&dir));
    fs::remove_dir_all(dir).unwrap();
    assert_eq!(
        rejected.provider_completion_set_hash_validation_status(),
        "mismatch"
    );
    assert_eq!(
        rejected.error(),
        Some("provider-completion-set-hash-mismatch")
    );
}
