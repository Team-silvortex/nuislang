use std::{collections::BTreeMap, fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{
    compiler_projection_three_page_identity,
    parse_compiler_candidate_structural_pagination_result_bytes,
    render_compiler_candidate_structural_pagination_result,
    toml::{
        escape_toml_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize,
    },
    ArtifactError, CompilerCandidateProduction, CompilerCandidateStructuralPaginationPage,
    CompilerCandidateStructuralPaginationResult, CompilerComponentBuild, CompilerProjectionKind,
    CompilerProjectionPageAdvance, CompilerProjectionPageCursor, CompilerStageKind,
    VerifiedCompilerStagePayload, COMPILER_CANDIDATE_ADAPTER_FILE,
    COMPILER_CANDIDATE_PRODUCTION_PROTOCOL, COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PAGE_COUNT,
    COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE,
    COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_PROTOCOL, COMPILER_PROJECTION_CURSOR_CONTRACT,
    COMPILER_PROJECTION_CURSOR_LANES, COMPILER_PROJECTION_PAGE_BYTES,
    COMPILER_PROJECTION_PAGE_CONTRACT, COMPILER_PROJECTION_PAGE_HASH_MODULUS,
    COMPILER_PROJECTION_PAGE_IDENTITY_RADIX,
};

pub const COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PROTOCOL: &str =
    "nuis-compiler-candidate-structural-pagination-v1";
pub const COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_FILE: &str =
    "nuis.compiler-candidate-structural-pagination.toml";
pub const COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_CONTRACT: &str =
    "nuis-stage1-three-page-structural-pagination-successor-v1";
pub const COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_AUTHORITY: &str =
    "candidate-owned-three-page-pagination-only-no-replacement-or-selection";
pub const COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_VERDICT: &str =
    "candidate-owned-ast-nir-three-page-pagination-verified";

const EXPECTED_PROJECTION_COUNT: usize = 2;

#[derive(Debug, Clone, Copy)]
pub struct CompilerCandidateStructuralPaginationInput<'a> {
    pub candidate: &'a CompilerComponentBuild,
    pub production: &'a CompilerCandidateProduction,
    pub payloads: &'a [VerifiedCompilerStagePayload],
    pub adapter: &'a [u8],
    pub result_source: &'a [u8],
    pub result: &'a CompilerCandidateStructuralPaginationResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateStructuralPaginationProjection {
    pub ordinal: usize,
    pub kind: String,
    pub source_stage: String,
    pub source_payload_bytes: usize,
    pub source_payload_sha256: String,
    pub first_page_identity: usize,
    pub first_cursor_identity: usize,
    pub second_page_identity: usize,
    pub second_cursor_identity: usize,
    pub third_page_record_count: usize,
    pub third_page_bytes: usize,
    pub third_page_projection_hash: usize,
    pub third_page_continuation_indentation: usize,
    pub third_page_continuation_body_bytes: usize,
    pub third_page_continuation_body_hash: usize,
    pub third_page_state_hash: usize,
    pub third_page_identity: usize,
    pub third_cursor_identity: usize,
    pub third_cursor_lanes: [usize; COMPILER_PROJECTION_CURSOR_LANES],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateStructuralPagination {
    pub protocol: String,
    pub pagination_contract: String,
    pub authority: String,
    pub component_id: String,
    pub candidate_component_sha256: String,
    pub candidate_producer_id: String,
    pub stage_handoff_bundle_sha256: String,
    pub predecessor_protocol: String,
    pub predecessor_proof_sha256: String,
    pub adapter_file: String,
    pub adapter_bytes: usize,
    pub adapter_sha256: String,
    pub result_protocol: String,
    pub result_file: String,
    pub result_bytes: usize,
    pub result_sha256: String,
    pub page_contract: String,
    pub cursor_contract: String,
    pub page_bytes: usize,
    pub page_count: usize,
    pub projection_count: usize,
    pub candidate_owned_pagination: bool,
    pub host_recomputed: bool,
    pub predecessor_unchanged: bool,
    pub stage0_provider_dependency: bool,
    pub replacement_authorized: bool,
    pub selection_authorized: bool,
    pub verdict: String,
    pub proof_sha256: String,
    pub projections: Vec<CompilerCandidateStructuralPaginationProjection>,
}

pub fn build_compiler_candidate_structural_pagination(
    input: &CompilerCandidateStructuralPaginationInput<'_>,
) -> Result<CompilerCandidateStructuralPagination, ArtifactError> {
    validate_input_lineage(input)?;
    let projections = [
        (CompilerProjectionKind::Ast, CompilerStageKind::Ast),
        (CompilerProjectionKind::Nir, CompilerStageKind::Nir),
    ]
    .into_iter()
    .enumerate()
    .map(|(ordinal, (kind, stage))| build_projection(input, ordinal, kind, stage))
    .collect::<Result<Vec<_>, _>>()?;
    let mut proof = CompilerCandidateStructuralPagination {
        protocol: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PROTOCOL.to_owned(),
        pagination_contract: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_CONTRACT.to_owned(),
        authority: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_AUTHORITY.to_owned(),
        component_id: input.candidate.component_id.clone(),
        candidate_component_sha256: input.candidate.record_sha256.clone(),
        candidate_producer_id: input.candidate.producer_id.clone(),
        stage_handoff_bundle_sha256: input.production.stage_handoff_bundle_sha256.clone(),
        predecessor_protocol: input.production.protocol.clone(),
        predecessor_proof_sha256: input.production.proof_sha256.clone(),
        adapter_file: COMPILER_CANDIDATE_ADAPTER_FILE.to_owned(),
        adapter_bytes: input.adapter.len(),
        adapter_sha256: sha256_hex(input.adapter),
        result_protocol: input.result.protocol.clone(),
        result_file: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE.to_owned(),
        result_bytes: input.result_source.len(),
        result_sha256: sha256_hex(input.result_source),
        page_contract: COMPILER_PROJECTION_PAGE_CONTRACT.to_owned(),
        cursor_contract: COMPILER_PROJECTION_CURSOR_CONTRACT.to_owned(),
        page_bytes: COMPILER_PROJECTION_PAGE_BYTES,
        page_count: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PAGE_COUNT,
        projection_count: projections.len(),
        candidate_owned_pagination: true,
        host_recomputed: true,
        predecessor_unchanged: true,
        stage0_provider_dependency: false,
        replacement_authorized: false,
        selection_authorized: false,
        verdict: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_VERDICT.to_owned(),
        proof_sha256: String::new(),
        projections,
    };
    proof.proof_sha256 = pagination_identity(&proof);
    validate_proof(&proof)?;
    Ok(proof)
}

pub fn read_compiler_candidate_structural_pagination(
    path: &Path,
    input: &CompilerCandidateStructuralPaginationInput<'_>,
) -> Result<CompilerCandidateStructuralPagination, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate structural pagination `{}`: {error}",
            path.display()
        ))
    })?;
    let proof = parse_compiler_candidate_structural_pagination_from_source(&source, path)?;
    if proof != build_compiler_candidate_structural_pagination(input)? {
        return Err(ArtifactError::new(
            "compiler candidate structural pagination changed its bound evidence",
        ));
    }
    Ok(proof)
}

fn validate_input_lineage(
    input: &CompilerCandidateStructuralPaginationInput<'_>,
) -> Result<(), ArtifactError> {
    let parsed_result = parse_compiler_candidate_structural_pagination_result_bytes(
        input.result_source,
        Path::new(COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE),
    )?;
    if &parsed_result != input.result
        || render_compiler_candidate_structural_pagination_result(input.result).as_bytes()
            != input.result_source
        || input.production.protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || input.production.candidate_component_sha256 != input.candidate.record_sha256
        || input.production.candidate_producer_id != input.candidate.producer_id
        || input.production.stage_handoff_bundle_sha256
            != input.candidate.stage_handoff_bundle_sha256
        || input.production.adapter_bytes != input.adapter.len()
        || input.production.adapter_sha256 != sha256_hex(input.adapter)
        || input.result.protocol != COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_PROTOCOL
    {
        return Err(ArtifactError::new(
            "compiler candidate structural pagination lineage is inconsistent",
        ));
    }
    Ok(())
}

fn build_projection(
    input: &CompilerCandidateStructuralPaginationInput<'_>,
    ordinal: usize,
    kind: CompilerProjectionKind,
    stage: CompilerStageKind,
) -> Result<CompilerCandidateStructuralPaginationProjection, ArtifactError> {
    let payload = input
        .payloads
        .iter()
        .find(|payload| payload.stage == stage)
        .ok_or_else(|| ArtifactError::new("structural pagination source payload is missing"))?;
    let predecessor = input
        .production
        .records
        .iter()
        .find(|record| record.stage == stage.as_str())
        .ok_or_else(|| ArtifactError::new("structural pagination predecessor record is missing"))?;
    if predecessor.payload_bytes != payload.bytes.len()
        || predecessor.payload_sha256 != sha256_hex(&payload.bytes)
    {
        return Err(ArtifactError::new(
            "structural pagination payload changed its production predecessor",
        ));
    }
    let chain = compiler_projection_three_page_identity(kind, &payload.bytes)?;
    let actual_pages = match kind {
        CompilerProjectionKind::Ast => &input.result.ast_pages,
        CompilerProjectionKind::Nir => &input.result.nir_pages,
    };
    let expected_pages = [chain.first, chain.second, chain.third];
    if actual_pages.len() != expected_pages.len()
        || actual_pages
            .iter()
            .zip(expected_pages)
            .any(|(actual, expected)| !result_page_matches(actual, expected))
    {
        return Err(ArtifactError::new(format!(
            "candidate-owned {} three-page pagination disagrees with host replay",
            kind.as_str()
        )));
    }
    let (first_identity, first_cursor, second_identity, second_cursor) = match kind {
        CompilerProjectionKind::Ast => (
            input.production.ast_page_identity,
            input.production.ast_page_cursor_identity,
            input.production.ast_continuation_page_identity,
            input.production.ast_continuation_cursor_identity,
        ),
        CompilerProjectionKind::Nir => (
            input.production.nir_page_identity,
            input.production.nir_page_cursor_identity,
            input.production.nir_continuation_page_identity,
            input.production.nir_continuation_cursor_identity,
        ),
    };
    if first_identity != chain.first.page.identity
        || first_cursor != chain.first.cursor_identity
        || second_identity != chain.second.page.identity
        || second_cursor != chain.second.cursor_identity
    {
        return Err(ArtifactError::new(
            "structural pagination predecessor does not bind its first two pages",
        ));
    }
    Ok(projection_from_chain(
        ordinal,
        kind,
        stage,
        payload,
        chain.first,
        chain.second,
        chain.third,
    ))
}

fn result_page_matches(
    actual: &CompilerCandidateStructuralPaginationPage,
    expected: CompilerProjectionPageAdvance,
) -> bool {
    actual.identity == expected.page.identity
        && actual.cursor_identity == expected.cursor_identity
        && actual.cursor_lanes == expected.cursor.lanes()
}

fn projection_from_chain(
    ordinal: usize,
    kind: CompilerProjectionKind,
    stage: CompilerStageKind,
    payload: &VerifiedCompilerStagePayload,
    first: CompilerProjectionPageAdvance,
    second: CompilerProjectionPageAdvance,
    third: CompilerProjectionPageAdvance,
) -> CompilerCandidateStructuralPaginationProjection {
    CompilerCandidateStructuralPaginationProjection {
        ordinal,
        kind: kind.as_str().to_owned(),
        source_stage: stage.as_str().to_owned(),
        source_payload_bytes: payload.bytes.len(),
        source_payload_sha256: sha256_hex(&payload.bytes),
        first_page_identity: first.page.identity,
        first_cursor_identity: first.cursor_identity,
        second_page_identity: second.page.identity,
        second_cursor_identity: second.cursor_identity,
        third_page_record_count: third.page.record_count,
        third_page_bytes: third.page.page_bytes,
        third_page_projection_hash: third.page.projection_hash,
        third_page_continuation_indentation: third.page.continuation_indentation,
        third_page_continuation_body_bytes: third.page.continuation_body_bytes,
        third_page_continuation_body_hash: third.page.continuation_body_hash,
        third_page_state_hash: third.page.state_hash,
        third_page_identity: third.page.identity,
        third_cursor_identity: third.cursor_identity,
        third_cursor_lanes: third.cursor.lanes(),
    }
}

pub fn render_compiler_candidate_structural_pagination(
    proof: &CompilerCandidateStructuralPagination,
) -> String {
    let mut out = format!(
        "protocol = \"{}\"\npagination_contract = \"{}\"\nauthority = \"{}\"\ncomponent_id = \"{}\"\ncandidate_component_sha256 = \"{}\"\ncandidate_producer_id = \"{}\"\nstage_handoff_bundle_sha256 = \"{}\"\npredecessor_protocol = \"{}\"\npredecessor_proof_sha256 = \"{}\"\nadapter_file = \"{}\"\nadapter_bytes = {}\nadapter_sha256 = \"{}\"\nresult_protocol = \"{}\"\nresult_file = \"{}\"\nresult_bytes = {}\nresult_sha256 = \"{}\"\npage_contract = \"{}\"\ncursor_contract = \"{}\"\npage_bytes = {}\npage_count = {}\nprojection_count = {}\ncandidate_owned_pagination = {}\nhost_recomputed = {}\npredecessor_unchanged = {}\nstage0_provider_dependency = {}\nreplacement_authorized = {}\nselection_authorized = {}\nverdict = \"{}\"\nproof_sha256 = \"{}\"\n",
        proof.protocol,
        proof.pagination_contract,
        proof.authority,
        escape_toml_string(&proof.component_id),
        proof.candidate_component_sha256,
        escape_toml_string(&proof.candidate_producer_id),
        proof.stage_handoff_bundle_sha256,
        proof.predecessor_protocol,
        proof.predecessor_proof_sha256,
        proof.adapter_file,
        proof.adapter_bytes,
        proof.adapter_sha256,
        proof.result_protocol,
        proof.result_file,
        proof.result_bytes,
        proof.result_sha256,
        proof.page_contract,
        proof.cursor_contract,
        proof.page_bytes,
        proof.page_count,
        proof.projection_count,
        proof.candidate_owned_pagination,
        proof.host_recomputed,
        proof.predecessor_unchanged,
        proof.stage0_provider_dependency,
        proof.replacement_authorized,
        proof.selection_authorized,
        proof.verdict,
        proof.proof_sha256,
    );
    for projection in &proof.projections {
        render_projection(&mut out, projection);
    }
    out
}

fn render_projection(
    out: &mut String,
    projection: &CompilerCandidateStructuralPaginationProjection,
) {
    out.push_str(&format!(
        "\n[[projection]]\nordinal = {}\nkind = \"{}\"\nsource_stage = \"{}\"\nsource_payload_bytes = {}\nsource_payload_sha256 = \"{}\"\nfirst_page_identity = {}\nfirst_cursor_identity = {}\nsecond_page_identity = {}\nsecond_cursor_identity = {}\nthird_page_record_count = {}\nthird_page_bytes = {}\nthird_page_projection_hash = {}\nthird_page_continuation_indentation = {}\nthird_page_continuation_body_bytes = {}\nthird_page_continuation_body_hash = {}\nthird_page_state_hash = {}\nthird_page_identity = {}\nthird_cursor_identity = {}\n",
        projection.ordinal,
        projection.kind,
        projection.source_stage,
        projection.source_payload_bytes,
        projection.source_payload_sha256,
        projection.first_page_identity,
        projection.first_cursor_identity,
        projection.second_page_identity,
        projection.second_cursor_identity,
        projection.third_page_record_count,
        projection.third_page_bytes,
        projection.third_page_projection_hash,
        projection.third_page_continuation_indentation,
        projection.third_page_continuation_body_bytes,
        projection.third_page_continuation_body_hash,
        projection.third_page_state_hash,
        projection.third_page_identity,
        projection.third_cursor_identity,
    ));
    for (lane, value) in projection.third_cursor_lanes.iter().enumerate() {
        out.push_str(&format!("third_cursor_lane_{lane} = {value}\n"));
    }
}

pub fn parse_compiler_candidate_structural_pagination_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateStructuralPagination, ArtifactError> {
    validate_text(source, path)?;
    let proof = CompilerCandidateStructuralPagination {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        pagination_contract: parse_required_toml_string(source, "pagination_contract", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        component_id: parse_required_toml_string(source, "component_id", path)?,
        candidate_component_sha256: parse_required_toml_string(
            source,
            "candidate_component_sha256",
            path,
        )?,
        candidate_producer_id: parse_required_toml_string(source, "candidate_producer_id", path)?,
        stage_handoff_bundle_sha256: parse_required_toml_string(
            source,
            "stage_handoff_bundle_sha256",
            path,
        )?,
        predecessor_protocol: parse_required_toml_string(source, "predecessor_protocol", path)?,
        predecessor_proof_sha256: parse_required_toml_string(
            source,
            "predecessor_proof_sha256",
            path,
        )?,
        adapter_file: parse_required_toml_string(source, "adapter_file", path)?,
        adapter_bytes: parse_required_toml_usize(source, "adapter_bytes", path)?,
        adapter_sha256: parse_required_toml_string(source, "adapter_sha256", path)?,
        result_protocol: parse_required_toml_string(source, "result_protocol", path)?,
        result_file: parse_required_toml_string(source, "result_file", path)?,
        result_bytes: parse_required_toml_usize(source, "result_bytes", path)?,
        result_sha256: parse_required_toml_string(source, "result_sha256", path)?,
        page_contract: parse_required_toml_string(source, "page_contract", path)?,
        cursor_contract: parse_required_toml_string(source, "cursor_contract", path)?,
        page_bytes: parse_required_toml_usize(source, "page_bytes", path)?,
        page_count: parse_required_toml_usize(source, "page_count", path)?,
        projection_count: parse_required_toml_usize(source, "projection_count", path)?,
        candidate_owned_pagination: parse_required_toml_bool(
            source,
            "candidate_owned_pagination",
            path,
        )?,
        host_recomputed: parse_required_toml_bool(source, "host_recomputed", path)?,
        predecessor_unchanged: parse_required_toml_bool(source, "predecessor_unchanged", path)?,
        stage0_provider_dependency: parse_required_toml_bool(
            source,
            "stage0_provider_dependency",
            path,
        )?,
        replacement_authorized: parse_required_toml_bool(source, "replacement_authorized", path)?,
        selection_authorized: parse_required_toml_bool(source, "selection_authorized", path)?,
        verdict: parse_required_toml_string(source, "verdict", path)?,
        proof_sha256: parse_required_toml_string(source, "proof_sha256", path)?,
        projections: parse_projection_blocks(source, path)?,
    };
    validate_proof(&proof)?;
    if render_compiler_candidate_structural_pagination(&proof) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate structural pagination `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(proof)
}

fn validate_proof(proof: &CompilerCandidateStructuralPagination) -> Result<(), ArtifactError> {
    if proof.protocol != COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PROTOCOL
        || proof.pagination_contract != COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_CONTRACT
        || proof.authority != COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_AUTHORITY
        || proof.predecessor_protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || proof.adapter_file != COMPILER_CANDIDATE_ADAPTER_FILE
        || proof.adapter_bytes == 0
        || proof.result_protocol != COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_PROTOCOL
        || proof.result_file != COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE
        || proof.result_bytes == 0
        || proof.page_contract != COMPILER_PROJECTION_PAGE_CONTRACT
        || proof.cursor_contract != COMPILER_PROJECTION_CURSOR_CONTRACT
        || proof.page_bytes != COMPILER_PROJECTION_PAGE_BYTES
        || proof.page_count != COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PAGE_COUNT
        || proof.projection_count != EXPECTED_PROJECTION_COUNT
        || proof.projections.len() != EXPECTED_PROJECTION_COUNT
        || !proof.candidate_owned_pagination
        || !proof.host_recomputed
        || !proof.predecessor_unchanged
        || proof.stage0_provider_dependency
        || proof.replacement_authorized
        || proof.selection_authorized
        || proof.verdict != COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_VERDICT
    {
        return Err(ArtifactError::new(
            "compiler candidate structural pagination contract mismatch",
        ));
    }
    for (label, hash) in [
        ("candidate component", &proof.candidate_component_sha256),
        ("stage handoff bundle", &proof.stage_handoff_bundle_sha256),
        ("predecessor proof", &proof.predecessor_proof_sha256),
        ("adapter", &proof.adapter_sha256),
        ("result", &proof.result_sha256),
        ("proof", &proof.proof_sha256),
    ] {
        validate_sha256(hash, label)?;
    }
    for (ordinal, projection) in proof.projections.iter().enumerate() {
        validate_projection(projection, ordinal)?;
    }
    if proof.proof_sha256 != pagination_identity(proof) {
        return Err(ArtifactError::new(
            "compiler candidate structural pagination proof identity drifted",
        ));
    }
    Ok(())
}

fn validate_projection(
    projection: &CompilerCandidateStructuralPaginationProjection,
    ordinal: usize,
) -> Result<(), ArtifactError> {
    let expected_kind = if ordinal == 0 { "ast" } else { "nir" };
    if projection.ordinal != ordinal
        || projection.kind != expected_kind
        || projection.source_stage != expected_kind
        || projection.source_payload_bytes <= COMPILER_PROJECTION_PAGE_BYTES * 2
        || projection.first_page_identity == 0
        || projection.first_cursor_identity == 0
        || projection.second_page_identity == 0
        || projection.second_cursor_identity == 0
        || projection.third_page_record_count == 0
        || projection.third_page_bytes == 0
        || projection.third_page_bytes > COMPILER_PROJECTION_PAGE_BYTES
        || projection.third_page_projection_hash >= COMPILER_PROJECTION_PAGE_HASH_MODULUS
        || projection.third_page_continuation_indentation > projection.third_page_bytes
        || projection.third_page_continuation_body_bytes > projection.third_page_bytes
        || projection.third_page_continuation_indentation
            + projection.third_page_continuation_body_bytes
            > projection.third_page_bytes
        || projection.third_page_continuation_body_hash >= COMPILER_PROJECTION_PAGE_HASH_MODULUS
        || projection.third_page_state_hash >= COMPILER_PROJECTION_PAGE_HASH_MODULUS
        || projection.third_page_identity
            != projection.third_page_state_hash * COMPILER_PROJECTION_PAGE_IDENTITY_RADIX
                + projection.third_page_bytes
        || projection.third_cursor_identity == 0
        || projection.third_cursor_identity >= COMPILER_PROJECTION_PAGE_HASH_MODULUS
        || CompilerProjectionPageCursor::from_lanes(projection.third_cursor_lanes).identity()
            != projection.third_cursor_identity
    {
        return Err(ArtifactError::new(
            "compiler candidate structural pagination projection is invalid",
        ));
    }
    validate_sha256(&projection.source_payload_sha256, "source payload")
}

fn parse_projection_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<CompilerCandidateStructuralPaginationProjection>, ArtifactError> {
    let mut projections = Vec::new();
    let mut values = BTreeMap::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line == "[[projection]]" {
            if in_block {
                projections.push(parse_projection(&values, path)?);
                values.clear();
            }
            in_block = true;
        } else if in_block && !line.is_empty() && !line.starts_with('#') {
            let Some((key, value)) = line.split_once('=') else {
                return Err(ArtifactError::new(
                    "malformed structural pagination projection",
                ));
            };
            if values
                .insert(key.trim().to_owned(), value.trim().to_owned())
                .is_some()
            {
                return Err(ArtifactError::new(
                    "structural pagination projection repeats a key",
                ));
            }
        }
    }
    if in_block {
        projections.push(parse_projection(&values, path)?);
    }
    Ok(projections)
}

fn parse_projection(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerCandidateStructuralPaginationProjection, ArtifactError> {
    let string = |key| parse_required_map_string_in_block(values, key, path, "projection");
    let mut third_cursor_lanes = [0; COMPILER_PROJECTION_CURSOR_LANES];
    for (lane, value) in third_cursor_lanes.iter_mut().enumerate() {
        let key = format!("third_cursor_lane_{lane}");
        *value = required_projection_usize(values, &key, path)?;
    }
    Ok(CompilerCandidateStructuralPaginationProjection {
        ordinal: required_projection_usize(values, "ordinal", path)?,
        kind: string("kind")?,
        source_stage: string("source_stage")?,
        source_payload_bytes: required_projection_usize(values, "source_payload_bytes", path)?,
        source_payload_sha256: string("source_payload_sha256")?,
        first_page_identity: required_projection_usize(values, "first_page_identity", path)?,
        first_cursor_identity: required_projection_usize(values, "first_cursor_identity", path)?,
        second_page_identity: required_projection_usize(values, "second_page_identity", path)?,
        second_cursor_identity: required_projection_usize(values, "second_cursor_identity", path)?,
        third_page_record_count: required_projection_usize(
            values,
            "third_page_record_count",
            path,
        )?,
        third_page_bytes: required_projection_usize(values, "third_page_bytes", path)?,
        third_page_projection_hash: required_projection_usize(
            values,
            "third_page_projection_hash",
            path,
        )?,
        third_page_continuation_indentation: required_projection_usize(
            values,
            "third_page_continuation_indentation",
            path,
        )?,
        third_page_continuation_body_bytes: required_projection_usize(
            values,
            "third_page_continuation_body_bytes",
            path,
        )?,
        third_page_continuation_body_hash: required_projection_usize(
            values,
            "third_page_continuation_body_hash",
            path,
        )?,
        third_page_state_hash: required_projection_usize(values, "third_page_state_hash", path)?,
        third_page_identity: required_projection_usize(values, "third_page_identity", path)?,
        third_cursor_identity: required_projection_usize(values, "third_cursor_identity", path)?,
        third_cursor_lanes,
    })
}

fn required_projection_usize(
    values: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<usize, ArtifactError> {
    parse_optional_map_usize(values, key, path, "projection")?.ok_or_else(|| {
        ArtifactError::new(format!(
            "structural pagination projection is missing `{key}`"
        ))
    })
}

fn pagination_identity(proof: &CompilerCandidateStructuralPagination) -> String {
    let mut identity = proof.clone();
    identity.proof_sha256.clear();
    sha256_hex(render_compiler_candidate_structural_pagination(&identity).as_bytes())
}

fn validate_text(source: &str, path: &Path) -> Result<(), ArtifactError> {
    if source.is_empty()
        || !source.ends_with('\n')
        || source.contains('\r')
        || source.contains('\0')
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate structural pagination `{}` must be canonical UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
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
            "compiler candidate structural pagination {label} must be lowercase SHA-256"
        )))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "compiler_candidate_structural_pagination_tests.rs"]
mod tests;
