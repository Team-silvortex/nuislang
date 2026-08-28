use super::*;

const AST_PAGE_FIXTURE: &str = concat!(
    "/// First Nuis-written consumer of the producer-neutral AST/NIR record model.\n",
    "use cpu StdLanguageCore\n",
    "use cpu StdCompilerData\n",
    "ast mod cpu unit Main\n",
    "  fn main() -> i64\n",
    "    return 0\n",
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
