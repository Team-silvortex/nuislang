use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use nuis_artifact::{
    compiler_projection_three_page_identity,
    parse_compiler_candidate_structural_pagination_result_bytes,
    CompilerCandidateStructuralPaginationPage, CompilerCandidateStructuralPaginationResult,
    CompilerProjectionKind, CompilerProjectionPageAdvance, CompilerProjectionThreePageIdentity,
    COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE,
};

pub(crate) const STRUCTURAL_PAGINATION_COMMAND: &str = "structural-pagination-v1";

pub(crate) struct CandidateStructuralPaginationOutput {
    pub(crate) result: CompilerCandidateStructuralPaginationResult,
    pub(crate) result_source: Vec<u8>,
    pub(crate) ast_pages: CompilerProjectionThreePageIdentity,
    pub(crate) nir_pages: CompilerProjectionThreePageIdentity,
}

pub(crate) fn run_candidate_structural_pagination(
    adapter: &Path,
    ast_path: &Path,
    nir_path: &Path,
) -> Result<CandidateStructuralPaginationOutput, String> {
    let output = Command::new(adapter)
        .arg(STRUCTURAL_PAGINATION_COMMAND)
        .arg(ast_path)
        .arg(nir_path)
        .env_clear()
        .stdin(Stdio::null())
        .output()
        .map_err(|error| {
            format!(
                "failed to execute candidate structural pagination adapter `{}`: {error}",
                adapter.display()
            )
        })?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "candidate structural pagination adapter failed with {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    let result = parse_compiler_candidate_structural_pagination_result_bytes(
        &output.stdout,
        Path::new(COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE),
    )
    .map_err(|error| format!("failed to parse candidate structural pagination result: {error}"))?;
    let ast_pages = host_pages(CompilerProjectionKind::Ast, ast_path)?;
    let nir_pages = host_pages(CompilerProjectionKind::Nir, nir_path)?;
    verify_pages("AST", &result.ast_pages, ast_pages)?;
    verify_pages("NIR", &result.nir_pages, nir_pages)?;
    Ok(CandidateStructuralPaginationOutput {
        result,
        result_source: output.stdout,
        ast_pages,
        nir_pages,
    })
}

fn host_pages(
    kind: CompilerProjectionKind,
    path: &Path,
) -> Result<CompilerProjectionThreePageIdentity, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read candidate {} payload `{}`: {error}",
            kind.as_str(),
            path.display()
        )
    })?;
    compiler_projection_three_page_identity(kind, &bytes).map_err(|error| {
        format!(
            "failed to independently materialize {} structural pages: {error}",
            kind.as_str()
        )
    })
}

fn verify_pages(
    label: &str,
    actual: &[CompilerCandidateStructuralPaginationPage],
    expected: CompilerProjectionThreePageIdentity,
) -> Result<(), String> {
    let expected = [expected.first, expected.second, expected.third];
    if actual.len() != expected.len()
        || actual
            .iter()
            .zip(expected)
            .any(|(actual, expected)| !page_matches(actual, expected))
    {
        return Err(format!(
            "Nuis candidate {label} three-page result disagrees with independent host pagination"
        ));
    }
    Ok(())
}

fn page_matches(
    actual: &CompilerCandidateStructuralPaginationPage,
    expected: CompilerProjectionPageAdvance,
) -> bool {
    actual.identity == expected.page.identity
        && actual.cursor_identity == expected.cursor_identity
        && actual.cursor_lanes == expected.cursor.lanes()
}

pub(crate) const STRUCTURAL_PAGINATION_ADAPTER: &str = r#"
#define NUIS_STRUCTURAL_PAGINATION_COMMAND "structural-pagination-v1"

static int read_projection_three_pages(
    const char* path,
    int64_t lengths[3],
    int64_t words[3][19]
) {
    FILE* file = fopen(path, "rb");
    if (file == NULL) return 0;
    int page = 0;
    for (;;) {
        int byte = fgetc(file);
        if (byte == EOF) break;
        if (page < 3) {
            pack_page_byte(&lengths[page], words[page], byte);
            if (lengths[page] == 128) page += 1;
        }
    }
    int valid = ferror(file) == 0 && lengths[0] == 128
        && lengths[1] == 128 && lengths[2] > 0;
    if (fclose(file) != 0) return 0;
    return valid;
}

static int materialize_projection_three_pages(
    int64_t projection,
    int64_t lengths[3],
    int64_t words[3][19],
    int64_t identities[3],
    int64_t cursor_identities[3],
    int64_t cursors[3][8]
) {
    int64_t fresh[8] = {-1, 0, 0, 0, 0, 0, 0, 0};
    for (int page = 0; page < 3; ++page) {
        int64_t* input_cursor = page == 0 ? fresh : cursors[page - 1];
        identities[page] = projection_resume_value(
            0, projection, input_cursor, lengths[page], words[page]
        );
        if (identities[page] <= 0) return 0;
        for (int selector = 1; selector <= 8; ++selector) {
            cursors[page][selector - 1] = projection_resume_value(
                selector, projection, input_cursor, lengths[page], words[page]
            );
            if (cursors[page][selector - 1] < 0) return 0;
        }
        cursor_identities[page] = projection_resume_value(
            9, projection, input_cursor, lengths[page], words[page]
        );
        if (cursor_identities[page] <= 0) return 0;
    }
    return 1;
}

static void print_projection_three_pages(
    const char* prefix,
    int64_t identities[3],
    int64_t cursor_identities[3],
    int64_t cursors[3][8]
) {
    for (int page = 0; page < 3; ++page) {
        printf("%s.page.%d.identity=%lld\n", prefix, page, (long long)identities[page]);
        printf("%s.page.%d.cursor_identity=%lld\n", prefix, page, (long long)cursor_identities[page]);
        for (int lane = 0; lane < 8; ++lane) {
            printf("%s.page.%d.cursor_lane.%d=%lld\n", prefix, page, lane, (long long)cursors[page][lane]);
        }
    }
}

static int run_structural_pagination(const char* ast_path, const char* nir_path) {
    int64_t ast_lengths[3] = {0, 0, 0};
    int64_t nir_lengths[3] = {0, 0, 0};
    int64_t ast_words[3][19] = {{0}};
    int64_t nir_words[3][19] = {{0}};
    int64_t ast_identities[3] = {0, 0, 0};
    int64_t nir_identities[3] = {0, 0, 0};
    int64_t ast_cursor_identities[3] = {0, 0, 0};
    int64_t nir_cursor_identities[3] = {0, 0, 0};
    int64_t ast_cursors[3][8] = {{0}};
    int64_t nir_cursors[3][8] = {{0}};
    if (!read_projection_three_pages(ast_path, ast_lengths, ast_words)
        || !read_projection_three_pages(nir_path, nir_lengths, nir_words)) return 76;
    if (!materialize_projection_three_pages(
            1, ast_lengths, ast_words, ast_identities,
            ast_cursor_identities, ast_cursors
        )) return 77;
    if (!materialize_projection_three_pages(
            2, nir_lengths, nir_words, nir_identities,
            nir_cursor_identities, nir_cursors
        )) return 78;
    puts("protocol=nuis-bootstrap-candidate-structural-pagination-v1");
    puts("page_count=3");
    print_projection_three_pages(
        "ast", ast_identities, ast_cursor_identities, ast_cursors
    );
    print_projection_three_pages(
        "nir", nir_identities, nir_cursor_identities, nir_cursors
    );
    return 0;
}
"#;
