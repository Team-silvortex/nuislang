use super::*;

const AST_PAGE_FIXTURE: &str = concat!(
    "/// First Nuis-written consumer of the producer-neutral AST/NIR record model.\n",
    "use cpu StdLanguageCore\n",
    "use cpu StdCompilerData\n",
    "ast mod cpu unit Main\n",
    "  fn main() -> i64\n",
    "    return 0\n",
);

const NIR_PAGE_FIXTURE: &str = concat!(
    "use cpu StdLanguageCore\n",
    "use cpu StdCompilerData\n",
    "use cpu StdCompilerTokenEmit\n",
    "use cpu StdCompilerTokens\n",
    "use cpu StdCompilerProjection\n",
    "nir mod cpu unit Main\n",
    "  fn main() -> i64\n",
    "    return 0\n",
);

const THREE_PAGE_AST_FIXTURE: &str = concat!(
    "/// Three-page producer-neutral structural projection.\n",
    "use cpu StdLanguageCore\n",
    "use cpu StdCompilerData\n",
    "use cpu StdCompilerProjection\n",
    "ast mod cpu unit Main\n",
    "  fn first(value: i64) -> i64\n",
    "    let shifted = value + 1\n",
    "    return shifted\n",
    "  fn second(value: i64) -> i64\n",
    "    let doubled = value * 2\n",
    "    return doubled\n",
    "  fn main() -> i64\n",
    "    let first_value = first(20)\n",
    "    let second_value = second(first_value)\n",
    "    return second_value\n",
);

#[test]
fn ast_first_page_binds_complete_records_and_partial_continuation() {
    let page = compiler_projection_first_page_identity(
        CompilerProjectionKind::Ast,
        AST_PAGE_FIXTURE.as_bytes(),
    )
    .expect("materialize AST structural page");

    assert_eq!(
        page,
        CompilerProjectionPageIdentity {
            record_count: 3,
            page_bytes: COMPILER_PROJECTION_PAGE_BYTES,
            projection_hash: 65_460_735,
            continuation_indentation: 0,
            continuation_body_bytes: 2,
            continuation_body_hash: 28_492_679,
            state_hash: 1_935_945_723,
            identity: 249_736_998_395,
        }
    );
}

#[test]
fn nir_first_page_binds_import_records_and_partial_continuation() {
    let page = compiler_projection_first_page_identity(
        CompilerProjectionKind::Nir,
        NIR_PAGE_FIXTURE.as_bytes(),
    )
    .expect("materialize NIR structural page");

    assert_eq!(
        page,
        CompilerProjectionPageIdentity {
            record_count: 4,
            page_bytes: COMPILER_PROJECTION_PAGE_BYTES,
            projection_hash: 568_515_310,
            continuation_indentation: 0,
            continuation_body_bytes: 25,
            continuation_body_hash: 671_013_644,
            state_hash: 1_026_894_471,
            identity: 132_469_386_887,
        }
    );
}

#[test]
fn structural_page_requires_a_canonical_complete_projection() {
    assert!(compiler_projection_first_page_identity(
        CompilerProjectionKind::Ast,
        b"ast mod cpu unit Main\n  fn main() \n"
    )
    .is_err());
    assert!(compiler_projection_first_page_identity(
        CompilerProjectionKind::Ast,
        b"ast mod cpu unit Main\n"
    )
    .is_ok());
}

#[test]
fn structural_cursor_resumes_ast_and_nir_second_pages() {
    for (kind, fixture, first_cursor_identity, second_page, second_cursor_identity) in [
        (
            CompilerProjectionKind::Ast,
            AST_PAGE_FIXTURE,
            1_135_386_651,
            CompilerProjectionPageIdentity {
                record_count: 6,
                page_bytes: 52,
                projection_hash: 1_591_775_767,
                continuation_indentation: 0,
                continuation_body_bytes: 0,
                continuation_body_hash: 431,
                state_hash: 495_232_673,
                identity: 63_885_014_869,
            },
            1_561_305_309,
        ),
        (
            CompilerProjectionKind::Nir,
            NIR_PAGE_FIXTURE,
            754_343_074,
            CompilerProjectionPageIdentity {
                record_count: 8,
                page_bytes: 59,
                projection_hash: 843_587_668,
                continuation_indentation: 0,
                continuation_body_bytes: 0,
                continuation_body_hash: 431,
                state_hash: 138_607_295,
                identity: 17_880_341_114,
            },
            725_769_899,
        ),
    ] {
        let chain = compiler_projection_two_page_identity(kind, fixture.as_bytes())
            .expect("materialize two structural pages");
        let resumed = compiler_projection_resume_page_identity(
            kind,
            chain.first.cursor,
            &fixture.as_bytes()[COMPILER_PROJECTION_PAGE_BYTES..],
        )
        .expect("resume second structural page");

        assert_eq!(
            chain.first.page,
            compiler_projection_first_page_identity(kind, fixture.as_bytes()).unwrap()
        );
        assert_eq!(chain.second, resumed);
        assert_eq!(chain.first.cursor_identity, first_cursor_identity);
        assert_eq!(chain.second.page, second_page);
        assert_eq!(chain.second.cursor_identity, second_cursor_identity);
        assert_eq!(
            chain.first.cursor.lanes()[0] / 32,
            COMPILER_PROJECTION_PAGE_BYTES
        );
        assert_eq!(chain.second.cursor.lanes()[0] / 32, fixture.len());
        assert!(chain.second.page.record_count >= chain.first.page.record_count);
    }
}

#[test]
fn structural_cursor_tampering_and_missing_second_page_fail_closed() {
    let chain = compiler_projection_two_page_identity(
        CompilerProjectionKind::Nir,
        NIR_PAGE_FIXTURE.as_bytes(),
    )
    .expect("materialize NIR pages");
    let mut lanes = chain.first.cursor.lanes();
    lanes[3] = COMPILER_PROJECTION_PAGE_HASH_MODULUS;
    assert!(compiler_projection_resume_page_identity(
        CompilerProjectionKind::Nir,
        CompilerProjectionPageCursor::from_lanes(lanes),
        &NIR_PAGE_FIXTURE.as_bytes()[COMPILER_PROJECTION_PAGE_BYTES..],
    )
    .is_err());
    assert!(compiler_projection_resume_page_identity(
        CompilerProjectionKind::Ast,
        chain.first.cursor,
        &NIR_PAGE_FIXTURE.as_bytes()[COMPILER_PROJECTION_PAGE_BYTES..],
    )
    .is_err());
    assert!(compiler_projection_two_page_identity(
        CompilerProjectionKind::Nir,
        b"nir mod cpu unit Main\n",
    )
    .is_err());
}

#[test]
fn structural_cursor_resumes_a_third_bounded_page() {
    assert!(THREE_PAGE_AST_FIXTURE.len() > COMPILER_PROJECTION_PAGE_BYTES * 3);
    let chain = compiler_projection_three_page_identity(
        CompilerProjectionKind::Ast,
        THREE_PAGE_AST_FIXTURE.as_bytes(),
    )
    .expect("materialize three structural pages");
    let second = compiler_projection_resume_page_identity(
        CompilerProjectionKind::Ast,
        chain.first.cursor,
        &THREE_PAGE_AST_FIXTURE.as_bytes()
            [COMPILER_PROJECTION_PAGE_BYTES..COMPILER_PROJECTION_PAGE_BYTES * 2],
    )
    .expect("resume second structural page");
    let third = compiler_projection_resume_page_identity(
        CompilerProjectionKind::Ast,
        second.cursor,
        &THREE_PAGE_AST_FIXTURE.as_bytes()
            [COMPILER_PROJECTION_PAGE_BYTES * 2..COMPILER_PROJECTION_PAGE_BYTES * 3],
    )
    .expect("resume third structural page");

    assert_eq!(
        chain.first_two(),
        CompilerProjectionTwoPageIdentity {
            first: chain.first,
            second,
        }
    );
    assert_eq!(chain.third, third);
    assert_eq!(chain.third.page.page_bytes, COMPILER_PROJECTION_PAGE_BYTES);
    assert_eq!(
        chain.third.cursor.lanes()[0] / 32,
        COMPILER_PROJECTION_PAGE_BYTES * 3
    );
    assert!(chain.third.page.record_count >= chain.second.page.record_count);
}

#[test]
fn structural_three_page_chain_requires_third_page_bytes() {
    assert!(compiler_projection_three_page_identity(
        CompilerProjectionKind::Ast,
        AST_PAGE_FIXTURE.as_bytes(),
    )
    .is_err());
}
