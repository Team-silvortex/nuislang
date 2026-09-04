use super::*;

fn page(ordinal: usize, seed: usize) -> CompilerCandidateStructuralPaginationPage {
    CompilerCandidateStructuralPaginationPage {
        ordinal,
        identity: seed * 100 + 1,
        cursor_identity: seed * 100 + 2,
        cursor_lanes: std::array::from_fn(|lane| seed * 100 + lane + 3),
    }
}

fn result() -> CompilerCandidateStructuralPaginationResult {
    CompilerCandidateStructuralPaginationResult {
        protocol: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_PROTOCOL.to_owned(),
        page_count: COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PAGE_COUNT,
        ast_pages: (0..3).map(|ordinal| page(ordinal, ordinal + 1)).collect(),
        nir_pages: (0..3).map(|ordinal| page(ordinal, ordinal + 4)).collect(),
    }
}

#[test]
fn canonical_result_round_trips() {
    let result = result();
    let source = render_compiler_candidate_structural_pagination_result(&result);
    assert_eq!(source.lines().count(), EXPECTED_LINE_COUNT);
    assert_eq!(
        parse_compiler_candidate_structural_pagination_result_from_source(
            &source,
            Path::new(COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE),
        )
        .expect("parse structural pagination result"),
        result
    );
}

#[test]
fn noncanonical_or_incomplete_result_fails_closed() {
    let source = render_compiler_candidate_structural_pagination_result(&result());
    for damaged in [
        source.replacen("page_count=3", "page_count=2", 1),
        source.replacen("ast.page.2.identity=301", "ast.page.2.identity=0", 1),
        source.replacen(
            "nir.page.2.cursor_identity=602",
            "nir.page.2.cursor_identity=0",
            1,
        ),
        source.replacen("\n", "\r\n", 1),
    ] {
        assert!(
            parse_compiler_candidate_structural_pagination_result_from_source(
                &damaged,
                Path::new(COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE),
            )
            .is_err()
        );
    }
}
