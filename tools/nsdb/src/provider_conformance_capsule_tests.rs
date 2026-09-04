use crate::provider_conformance_capsule::*;

const REFERENCE_BYTES: &[u8] = &[0x00, 0x01, 0x02, 0x03, 0x7f, 0x80, 0xfe, 0xff];

#[cfg(unix)]
#[test]
fn generated_data_reference_capsule_replays_without_physical_authority() {
    let capsule = data_reference_conformance_capsule().unwrap();
    let replay = replay_provider_conformance_capsule(
        &capsule,
        ProviderConformanceObservation {
            output: REFERENCE_BYTES,
            submission_tick: 1,
            completion_tick: 2,
            release_tick: 3,
            glm_released: true,
            physical_execution_claimed: false,
        },
    )
    .unwrap();

    assert_eq!(capsule.provider_id, "data.cpu-memory.reference.v1");
    assert_eq!(capsule.input_hex, "000102037f80feff");
    assert_eq!(capsule.input_hash, "fnv1a64:1c3b67c65206fb6d");
    assert_eq!(capsule.input_hash, capsule.expected_output_hash);
    assert_eq!(capsule.input_byte_length, 8);
    assert_eq!(
        capsule.capability_selection_hash,
        "fnv1a64:6d712122a1132927"
    );
    assert_eq!(capsule.capsule_hash, "fnv1a64:82270e31b99f2c0b");
    assert_eq!(capsule.execution_authority, "conformance-only");
    assert!(!capsule.physical_execution_claimed);
    assert_eq!(replay.clock_status, "monotonic");
    assert_eq!(replay.glm_status, "released-after-completion");
    assert_eq!(replay.completion_status, "verified");
    assert!(!replay.physical_execution_claimed);
    assert_eq!(replay.replay_hash, "fnv1a64:7ee93c8f8a4ae011");
}

#[cfg(unix)]
#[test]
fn conformance_replay_rejects_capsule_output_and_order_drift() {
    let capsule = data_reference_conformance_capsule().unwrap();
    let valid = ProviderConformanceObservation {
        output: REFERENCE_BYTES,
        submission_tick: 1,
        completion_tick: 2,
        release_tick: 3,
        glm_released: true,
        physical_execution_claimed: false,
    };
    let mut mutated_capsule = capsule.clone();
    mutated_capsule.scenario_id = "data.copy.mutated.v1";

    assert!(replay_provider_conformance_capsule(&mutated_capsule, valid)
        .unwrap_err()
        .contains("capsule-drift"));
    assert!(replay_provider_conformance_capsule(
        &capsule,
        ProviderConformanceObservation {
            output: &[0; 8],
            ..valid
        },
    )
    .unwrap_err()
    .contains("output-mismatch"));
    assert!(replay_provider_conformance_capsule(
        &capsule,
        ProviderConformanceObservation {
            completion_tick: 3,
            release_tick: 3,
            ..valid
        },
    )
    .unwrap_err()
    .contains("clock-order-invalid"));
}

#[cfg(unix)]
#[test]
fn conformance_replay_rejects_missing_glm_release_and_physical_claim() {
    let capsule = data_reference_conformance_capsule().unwrap();
    let valid = ProviderConformanceObservation {
        output: REFERENCE_BYTES,
        submission_tick: 1,
        completion_tick: 2,
        release_tick: 3,
        glm_released: true,
        physical_execution_claimed: false,
    };

    assert!(replay_provider_conformance_capsule(
        &capsule,
        ProviderConformanceObservation {
            glm_released: false,
            ..valid
        },
    )
    .unwrap_err()
    .contains("glm-release-missing"));
    assert!(replay_provider_conformance_capsule(
        &capsule,
        ProviderConformanceObservation {
            physical_execution_claimed: true,
            ..valid
        },
    )
    .unwrap_err()
    .contains("physical-authority-forbidden"));
}

#[cfg(unix)]
#[test]
fn completion_evidence_roundtrips_and_rejects_identity_drift() {
    let mut output = String::new();
    append_provider_conformance_capsule_evidence(&mut output, "data:host");
    let evidence = completion_evidence_from_output(&output).unwrap();
    let mut event = String::new();
    render_completion_event_fields(&mut event, &evidence);
    let reparsed = parse_completion_event_fields(&event);
    let mut hash_material = String::new();
    append_completion_hash_material(&mut hash_material, &reparsed);

    assert_eq!(evidence, reparsed);
    assert_eq!(evidence.status, "verified");
    assert_eq!(evidence.capsule_hash, "fnv1a64:82270e31b99f2c0b");
    assert_eq!(evidence.replay_hash, "fnv1a64:7ee93c8f8a4ae011");
    assert_eq!(evidence.execution_authority, "conformance-only");
    assert!(!evidence.physical_execution_claimed);
    assert!(hash_material.contains(&evidence.capsule_hash));
    assert!(hash_material.contains(&evidence.replay_hash));

    let capsule_drift = output.replace("fnv1a64:82270e31b99f2c0b", "fnv1a64:82270e31b99f2c0c");
    assert!(completion_evidence_from_output(&capsule_drift)
        .unwrap_err()
        .contains("evidence-mismatch"));
    let clock_drift = output.replace(
        "provider_conformance_replay_completion_tick = 2",
        "provider_conformance_replay_completion_tick = 4",
    );
    assert!(completion_evidence_from_output(&clock_drift)
        .unwrap_err()
        .contains("evidence-mismatch"));
    let authority_drift = output.replace(
        "provider_conformance_replay_physical_execution_claimed = false",
        "provider_conformance_replay_physical_execution_claimed = true",
    );
    assert!(completion_evidence_from_output(&authority_drift)
        .unwrap_err()
        .contains("evidence-mismatch"));
}
