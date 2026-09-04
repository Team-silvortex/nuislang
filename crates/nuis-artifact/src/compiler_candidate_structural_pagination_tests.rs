use super::*;

fn projection(ordinal: usize, kind: &str) -> CompilerCandidateStructuralPaginationProjection {
    let third_cursor_lanes = std::array::from_fn(|lane| 800 + ordinal * 10 + lane);
    CompilerCandidateStructuralPaginationProjection {
        ordinal,
        kind: kind.to_owned(),
        source_stage: kind.to_owned(),
        source_payload_bytes: 400 + ordinal,
        source_payload_sha256: "a".repeat(64),
        first_page_identity: 101 + ordinal,
        first_cursor_identity: 201 + ordinal,
        second_page_identity: 301 + ordinal,
        second_cursor_identity: 401 + ordinal,
        third_page_record_count: 9,
        third_page_bytes: 128,
        third_page_projection_hash: 501 + ordinal,
        third_page_continuation_indentation: 0,
        third_page_continuation_body_bytes: 0,
        third_page_continuation_body_hash: 431,
        third_page_state_hash: 601 + ordinal,
        third_page_identity: (601 + ordinal) * COMPILER_PROJECTION_PAGE_IDENTITY_RADIX + 128,
        third_cursor_identity: CompilerProjectionPageCursor::from_lanes(third_cursor_lanes)
            .identity(),
        third_cursor_lanes,
    }
}

fn proof() -> CompilerCandidateStructuralPagination {
    let mut proof = CompilerCandidateStructuralPagination {
        protocol: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PROTOCOL.to_owned(),
        pagination_contract: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_CONTRACT.to_owned(),
        authority: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_AUTHORITY.to_owned(),
        component_id: "projection_relay".to_owned(),
        candidate_component_sha256: "b".repeat(64),
        candidate_producer_id: "candidate".to_owned(),
        stage_handoff_bundle_sha256: "c".repeat(64),
        predecessor_protocol: COMPILER_CANDIDATE_PRODUCTION_PROTOCOL.to_owned(),
        predecessor_proof_sha256: "d".repeat(64),
        adapter_file: COMPILER_CANDIDATE_ADAPTER_FILE.to_owned(),
        adapter_bytes: 1024,
        adapter_sha256: "e".repeat(64),
        result_protocol: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_PROTOCOL.to_owned(),
        result_file: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE.to_owned(),
        result_bytes: 2048,
        result_sha256: "f".repeat(64),
        page_contract: COMPILER_PROJECTION_PAGE_CONTRACT.to_owned(),
        cursor_contract: COMPILER_PROJECTION_CURSOR_CONTRACT.to_owned(),
        page_bytes: COMPILER_PROJECTION_PAGE_BYTES,
        page_count: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PAGE_COUNT,
        projection_count: 2,
        candidate_owned_pagination: true,
        host_recomputed: true,
        predecessor_unchanged: true,
        stage0_provider_dependency: false,
        replacement_authorized: false,
        selection_authorized: false,
        verdict: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_VERDICT.to_owned(),
        proof_sha256: String::new(),
        projections: vec![projection(0, "ast"), projection(1, "nir")],
    };
    proof.proof_sha256 = pagination_identity(&proof);
    proof
}

#[test]
fn canonical_successor_round_trips() {
    let proof = proof();
    let source = render_compiler_candidate_structural_pagination(&proof);
    assert_eq!(
        parse_compiler_candidate_structural_pagination_from_source(
            &source,
            Path::new(COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_FILE),
        )
        .expect("parse structural pagination successor"),
        proof
    );
    assert!(!source.contains("/Users/"));
}

#[test]
fn authority_page_and_proof_drift_fail_closed() {
    let source = render_compiler_candidate_structural_pagination(&proof());
    for damaged in [
        source.replacen(
            "candidate_owned_pagination = true",
            "candidate_owned_pagination = false",
            1,
        ),
        source.replacen(
            "replacement_authorized = false",
            "replacement_authorized = true",
            1,
        ),
        source.replacen("third_page_bytes = 128", "third_page_bytes = 127", 1),
        source.replacen(
            &format!("proof_sha256 = \"{}\"", proof().proof_sha256),
            "proof_sha256 = \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"",
            1,
        ),
    ] {
        assert!(parse_compiler_candidate_structural_pagination_from_source(
            &damaged,
            Path::new(COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_FILE),
        )
        .is_err());
    }
}

#[test]
fn cursor_identity_must_match_bound_lanes() {
    let mut proof = proof();
    proof.projections[0].third_cursor_lanes[3] += 1;
    proof.proof_sha256 = pagination_identity(&proof);

    assert!(parse_compiler_candidate_structural_pagination_from_source(
        &render_compiler_candidate_structural_pagination(&proof),
        Path::new(COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_FILE),
    )
    .is_err());
}
