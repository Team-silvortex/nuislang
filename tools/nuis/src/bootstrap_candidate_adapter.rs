use std::{fs, path::Path, process::Command};

use nuis_artifact::{
    compiler_candidate_bundle_fold, compiler_candidate_stage_fold,
    compiler_projection_checkpoint_kind_tag, compiler_projection_two_page_identity,
    compiler_token_first_page_identity, decode_compiler_token_stream, parse_build_manifest,
    CompilerProjectionKind, CompilerProjectionTwoPageIdentity, CompilerStageHandoff,
    CompilerTokenDecodeSummary, CompilerTokenPageIdentity, COMPILER_CANDIDATE_ADAPTER_FILE,
    COMPILER_STAGE_CHECKPOINT_PAGE_COUNT,
};

const ADAPTER_OUTPUT_PROTOCOL: &str = "nuis-bootstrap-candidate-scalar-output-v7";
const ADAPTER_SOURCE_FILE: &str = "nuis.compiler-candidate-adapter.c";
const ADAPTER_RUNTIME_OBJECT_FILE: &str = "nuis.compiler-candidate-runtime.o";

pub(crate) struct CandidateAdapterOutput {
    pub(crate) adapter_file: &'static str,
    pub(crate) adapter: Vec<u8>,
    pub(crate) stage_folds: Vec<usize>,
    pub(crate) bundle_fold: usize,
    pub(crate) token_decode: CompilerTokenDecodeSummary,
    pub(crate) token_page: CompilerTokenPageIdentity,
    pub(crate) ast_pages: CompilerProjectionTwoPageIdentity,
    pub(crate) nir_pages: CompilerProjectionTwoPageIdentity,
    pub(crate) nir_transformation_words: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AdapterNirOutput {
    projection: AdapterProjectionOutput,
    first_cursor_lanes: [usize; 8],
    continuation_cursor_lanes: [usize; 8],
}

impl AdapterNirOutput {
    fn from_pages(pages: CompilerProjectionTwoPageIdentity) -> Self {
        Self {
            projection: AdapterProjectionOutput::from_pages(pages),
            first_cursor_lanes: pages.first.cursor.lanes(),
            continuation_cursor_lanes: pages.second.cursor.lanes(),
        }
    }

    fn checkpoint_words(self) -> Vec<usize> {
        let mut words = vec![
            compiler_projection_checkpoint_kind_tag(CompilerProjectionKind::Nir),
            COMPILER_STAGE_CHECKPOINT_PAGE_COUNT,
            self.projection.first_page_identity,
            self.projection.first_cursor_identity,
        ];
        words.extend(self.first_cursor_lanes);
        words.push(self.projection.continuation_page_identity);
        words.push(self.projection.continuation_cursor_identity);
        words.extend(self.continuation_cursor_lanes);
        words
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AdapterProjectionOutput {
    first_page_identity: usize,
    first_cursor_identity: usize,
    continuation_page_identity: usize,
    continuation_cursor_identity: usize,
}

impl AdapterProjectionOutput {
    fn from_pages(pages: CompilerProjectionTwoPageIdentity) -> Self {
        Self {
            first_page_identity: pages.first.page.identity,
            first_cursor_identity: pages.first.cursor_identity,
            continuation_page_identity: pages.second.page.identity,
            continuation_cursor_identity: pages.second.cursor_identity,
        }
    }
}

pub(crate) fn run_candidate_adapter(
    stage0_dir: &Path,
    candidate_dir: &Path,
    handoff: &CompilerStageHandoff,
) -> Result<CandidateAdapterOutput, String> {
    if handoff.records.len() != 5 {
        return Err(format!(
            "candidate scalar adapter requires five stage records, found {}",
            handoff.records.len()
        ));
    }
    let stem = handoff.records[0]
        .payload_file
        .strip_suffix(".source.ns")
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| {
            format!(
                "candidate source payload `{}` does not identify its AOT object stem",
                handoff.records[0].payload_file
            )
        })?;
    let program_object = stage0_dir.join(format!("{stem}.host-program.o"));
    let shim_source = stage0_dir.join(format!("{stem}_shim.c"));
    for (label, path) in [
        ("candidate LLVM program object", &program_object),
        ("candidate runtime shim source", &shim_source),
    ] {
        if !path.is_file() {
            return Err(format!("{label} is missing at `{}`", path.display()));
        }
    }

    let build_manifest = parse_build_manifest(&stage0_dir.join("nuis.build.manifest.toml"))
        .map_err(|error| format!("failed to read candidate build target: {error}"))?;
    if build_manifest.packaging_mode != "native-cpu-llvm" || build_manifest.cpu_target_cross {
        return Err(
            "candidate scalar adapter requires a host-native CPU LLVM component".to_owned(),
        );
    }

    fs::create_dir_all(candidate_dir).map_err(|error| {
        format!(
            "failed to create candidate output `{}`: {error}",
            candidate_dir.display()
        )
    })?;
    let adapter_source = candidate_dir.join(ADAPTER_SOURCE_FILE);
    let runtime_object = candidate_dir.join(ADAPTER_RUNTIME_OBJECT_FILE);
    let adapter_path = candidate_dir.join(COMPILER_CANDIDATE_ADAPTER_FILE);
    fs::write(&adapter_source, render_adapter_source()).map_err(|error| {
        format!(
            "failed to write candidate adapter source `{}`: {error}",
            adapter_source.display()
        )
    })?;

    run_clang(
        Command::new("clang")
            .arg("-target")
            .arg(&build_manifest.cpu_target_clang)
            .arg("-DNUIS_RUNTIME_NO_MAIN")
            .arg("-c")
            .arg(&shim_source)
            .arg("-O2")
            .arg("-o")
            .arg(&runtime_object),
        "candidate no-main runtime object",
    )?;
    run_clang(
        Command::new("clang")
            .arg("-target")
            .arg(&build_manifest.cpu_target_clang)
            .arg(&adapter_source)
            .arg(&program_object)
            .arg(&runtime_object)
            .arg("-O2")
            .arg("-o")
            .arg(&adapter_path),
        "candidate scalar adapter",
    )?;

    let payload_paths = handoff
        .records
        .iter()
        .map(|record| stage0_dir.join(&record.payload_file))
        .collect::<Vec<_>>();
    let output = Command::new(&adapter_path)
        .args(&payload_paths)
        .output()
        .map_err(|error| {
            format!(
                "failed to execute candidate scalar adapter `{}`: {error}",
                adapter_path.display()
            )
        })?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "candidate scalar adapter failed with {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    let (stage_folds, bundle_fold, token_decode, token_page_identity, ast_output, nir_output) =
        parse_adapter_output(&output.stdout)?;
    let expected_folds = payload_paths
        .iter()
        .enumerate()
        .map(|(ordinal, path)| {
            fs::read(path)
                .map(|bytes| compiler_candidate_stage_fold(ordinal, &bytes))
                .map_err(|error| {
                    format!(
                        "failed to independently fold candidate payload `{}`: {error}",
                        path.display()
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let expected_bundle = compiler_candidate_bundle_fold(&expected_folds);
    let token_bytes = fs::read(&payload_paths[1]).map_err(|error| {
        format!(
            "failed to read candidate token payload `{}`: {error}",
            payload_paths[1].display()
        )
    })?;
    let expected_token_decode = decode_compiler_token_stream(&token_bytes)
        .map_err(|error| format!("failed to independently decode candidate tokens: {error}"))?;
    let expected_token_page = compiler_token_first_page_identity(&token_bytes)
        .map_err(|error| format!("failed to independently materialize token page: {error}"))?;
    let ast_bytes = fs::read(&payload_paths[2]).map_err(|error| {
        format!(
            "failed to read candidate AST payload `{}`: {error}",
            payload_paths[2].display()
        )
    })?;
    let expected_ast_pages =
        compiler_projection_two_page_identity(CompilerProjectionKind::Ast, &ast_bytes).map_err(
            |error| format!("failed to independently materialize AST structural pages: {error}"),
        )?;
    let nir_bytes = fs::read(&payload_paths[3]).map_err(|error| {
        format!(
            "failed to read candidate NIR payload `{}`: {error}",
            payload_paths[3].display()
        )
    })?;
    let expected_nir_pages =
        compiler_projection_two_page_identity(CompilerProjectionKind::Nir, &nir_bytes).map_err(
            |error| format!("failed to independently materialize NIR structural pages: {error}"),
        )?;
    if stage_folds != expected_folds
        || bundle_fold != expected_bundle
        || token_decode != expected_token_decode
        || token_page_identity != expected_token_page.identity
        || ast_output != AdapterProjectionOutput::from_pages(expected_ast_pages)
        || nir_output != AdapterNirOutput::from_pages(expected_nir_pages)
    {
        return Err(
            "Nuis candidate scalar output disagrees with the independent host fold, token decode, token page, AST page chain, or NIR page chain".to_owned(),
        );
    }

    let adapter = fs::read(&adapter_path).map_err(|error| {
        format!(
            "failed to read candidate scalar adapter `{}`: {error}",
            adapter_path.display()
        )
    })?;
    Ok(CandidateAdapterOutput {
        adapter_file: COMPILER_CANDIDATE_ADAPTER_FILE,
        adapter,
        stage_folds,
        bundle_fold,
        token_decode,
        token_page: expected_token_page,
        ast_pages: expected_ast_pages,
        nir_pages: expected_nir_pages,
        nir_transformation_words: nir_output.checkpoint_words(),
    })
}

fn parse_adapter_output(
    bytes: &[u8],
) -> Result<
    (
        Vec<usize>,
        usize,
        CompilerTokenDecodeSummary,
        usize,
        AdapterProjectionOutput,
        AdapterNirOutput,
    ),
    String,
> {
    let source = std::str::from_utf8(bytes)
        .map_err(|error| format!("candidate scalar output is not UTF-8: {error}"))?;
    if source.contains('\r') || source.contains('\0') || !source.ends_with('\n') {
        return Err("candidate scalar output must use canonical UTF-8/LF text".to_owned());
    }
    let lines = source.lines().collect::<Vec<_>>();
    if lines.len() != 34 || lines[0] != format!("protocol={ADAPTER_OUTPUT_PROTOCOL}") {
        return Err("candidate scalar output has an invalid protocol or line count".to_owned());
    }
    let stage_folds = (0..5)
        .map(|ordinal| parse_output_usize(lines[ordinal + 1], &format!("stage.{ordinal}")))
        .collect::<Result<Vec<_>, _>>()?;
    let bundle_fold = parse_output_usize(lines[6], "bundle")?;
    let token_decode = CompilerTokenDecodeSummary {
        record_count: parse_output_usize(lines[7], "tokens.record_count")?,
        semantic_fold: parse_output_usize(lines[8], "tokens.semantic_fold")?,
    };
    let token_page_identity = parse_output_usize(lines[9], "tokens.page_identity")?;
    let ast_output = AdapterProjectionOutput {
        first_page_identity: parse_output_usize(lines[10], "ast.page_identity")?,
        first_cursor_identity: parse_output_usize(lines[11], "ast.page_cursor_identity")?,
        continuation_page_identity: parse_output_usize(
            lines[12],
            "ast.continuation_page_identity",
        )?,
        continuation_cursor_identity: parse_output_usize(
            lines[13],
            "ast.continuation_cursor_identity",
        )?,
    };
    let nir_output = AdapterNirOutput {
        projection: AdapterProjectionOutput {
            first_page_identity: parse_output_usize(lines[14], "nir.page_identity")?,
            first_cursor_identity: parse_output_usize(lines[15], "nir.page_cursor_identity")?,
            continuation_page_identity: parse_output_usize(
                lines[16],
                "nir.continuation_page_identity",
            )?,
            continuation_cursor_identity: parse_output_usize(
                lines[17],
                "nir.continuation_cursor_identity",
            )?,
        },
        first_cursor_lanes: parse_output_lanes(&lines, 18, "nir.first_cursor_lane")?,
        continuation_cursor_lanes: parse_output_lanes(&lines, 26, "nir.continuation_cursor_lane")?,
    };
    Ok((
        stage_folds,
        bundle_fold,
        token_decode,
        token_page_identity,
        ast_output,
        nir_output,
    ))
}

fn parse_output_lanes(
    lines: &[&str],
    start: usize,
    key_prefix: &str,
) -> Result<[usize; 8], String> {
    let mut lanes = [0; 8];
    for (index, lane) in lanes.iter_mut().enumerate() {
        *lane = parse_output_usize(lines[start + index], &format!("{key_prefix}.{index}"))?;
    }
    Ok(lanes)
}

fn parse_output_usize(line: &str, expected_key: &str) -> Result<usize, String> {
    let (key, value) = line
        .split_once('=')
        .ok_or_else(|| format!("candidate scalar output line `{line}` is malformed"))?;
    if key != expected_key || value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!(
            "candidate scalar output expected `{expected_key}=<integer>`, found `{line}`"
        ));
    }
    value
        .parse::<usize>()
        .map_err(|error| format!("candidate scalar output `{expected_key}` is invalid: {error}"))
}

fn run_clang(command: &mut Command, label: &str) -> Result<(), String> {
    let output = command
        .output()
        .map_err(|error| format!("failed to invoke clang for {label}: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "clang failed while producing {label}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    ))
}

fn render_adapter_source() -> &'static str {
    r#"#include <stdint.h>
#include <stdio.h>

extern int64_t nuis_bootstrap_candidate_stage_seed_v1(int64_t ordinal);
extern int64_t nuis_bootstrap_candidate_stage_fold_v1(int64_t state, int64_t ordinal, int64_t byte);
extern int64_t nuis_bootstrap_candidate_bundle_seed_v1(void);
extern int64_t nuis_bootstrap_candidate_bundle_fold_v1(int64_t state, int64_t ordinal, int64_t stage_fold);
extern int64_t nuis_bootstrap_candidate_token_start_v1(void);
extern int64_t nuis_bootstrap_candidate_token_error_mode_v1(void);
extern int64_t nuis_bootstrap_candidate_token_max_bytes_v1(void);
extern int64_t nuis_bootstrap_candidate_token_semantic_seed_v1(void);
extern int64_t nuis_bootstrap_candidate_token_step_v1(int64_t mode, int64_t byte);
extern int64_t nuis_bootstrap_candidate_token_count_step_v1(int64_t count, int64_t mode, int64_t byte);
extern int64_t nuis_bootstrap_candidate_token_semantic_step_v1(int64_t state, int64_t mode, int64_t byte);
extern int64_t nuis_bootstrap_candidate_token_finish_v1(int64_t mode, int64_t count);
extern int64_t nuis_bootstrap_candidate_token_page_identity_v1(
    int64_t length,
    int64_t word0, int64_t word1, int64_t word2, int64_t word3, int64_t word4,
    int64_t word5, int64_t word6, int64_t word7, int64_t word8, int64_t word9,
    int64_t word10, int64_t word11, int64_t word12, int64_t word13, int64_t word14,
    int64_t word15, int64_t word16, int64_t word17, int64_t word18
);
extern int64_t nuis_bootstrap_candidate_ast_page_identity_v1(
    int64_t length,
    int64_t word0, int64_t word1, int64_t word2, int64_t word3, int64_t word4,
    int64_t word5, int64_t word6, int64_t word7, int64_t word8, int64_t word9,
    int64_t word10, int64_t word11, int64_t word12, int64_t word13, int64_t word14,
    int64_t word15, int64_t word16, int64_t word17, int64_t word18
);
extern int64_t nuis_bootstrap_candidate_nir_page_identity_v1(
    int64_t length,
    int64_t word0, int64_t word1, int64_t word2, int64_t word3, int64_t word4,
    int64_t word5, int64_t word6, int64_t word7, int64_t word8, int64_t word9,
    int64_t word10, int64_t word11, int64_t word12, int64_t word13, int64_t word14,
    int64_t word15, int64_t word16, int64_t word17, int64_t word18
);
extern int64_t nuis_bootstrap_candidate_projection_page_resume_value_v1(
    int64_t selector, int64_t projection,
    int64_t cursor0, int64_t cursor1, int64_t cursor2, int64_t cursor3,
    int64_t cursor4, int64_t cursor5, int64_t cursor6, int64_t cursor7,
    int64_t length,
    int64_t word0, int64_t word1, int64_t word2, int64_t word3, int64_t word4,
    int64_t word5, int64_t word6, int64_t word7, int64_t word8, int64_t word9,
    int64_t word10, int64_t word11, int64_t word12, int64_t word13, int64_t word14,
    int64_t word15, int64_t word16, int64_t word17, int64_t word18
);

static void pack_page_byte(
    int64_t* length,
    int64_t words[19],
    int byte
) {
    if (*length >= 128) return;
    int64_t page_index = *length;
    words[page_index / 7] += ((int64_t)byte) << ((page_index % 7) * 8);
    *length = page_index + 1;
}

static void pack_projection_byte(
    int64_t* first_length,
    int64_t first_words[19],
    int64_t* second_length,
    int64_t second_words[19],
    int byte
) {
    if (*first_length < 128) {
        pack_page_byte(first_length, first_words, byte);
        return;
    }
    pack_page_byte(second_length, second_words, byte);
}

static int fold_file(
    const char* path,
    int64_t ordinal,
    int64_t* out,
    int64_t* token_count,
    int64_t* token_semantic,
    int64_t* token_page_length,
    int64_t token_page_words[19],
    int64_t* ast_page_length,
    int64_t ast_page_words[19],
    int64_t* ast_second_page_length,
    int64_t ast_second_page_words[19],
    int64_t* nir_page_length,
    int64_t nir_page_words[19],
    int64_t* nir_second_page_length,
    int64_t nir_second_page_words[19]
) {
    FILE* file = fopen(path, "rb");
    if (file == NULL) return 65;
    int64_t state = nuis_bootstrap_candidate_stage_seed_v1(ordinal);
    int64_t token_mode = nuis_bootstrap_candidate_token_start_v1();
    int64_t token_error = nuis_bootstrap_candidate_token_error_mode_v1();
    int64_t token_max_bytes = nuis_bootstrap_candidate_token_max_bytes_v1();
    int64_t token_bytes = 0;
    if (ordinal == 1) {
        *token_count = 0;
        *token_semantic = nuis_bootstrap_candidate_token_semantic_seed_v1();
        *token_page_length = 0;
        for (int index = 0; index < 19; ++index) token_page_words[index] = 0;
        if (token_max_bytes <= 0) {
            fclose(file);
            return 68;
        }
    }
    if (ordinal == 2) {
        *ast_page_length = 0;
        *ast_second_page_length = 0;
        for (int index = 0; index < 19; ++index) ast_page_words[index] = 0;
        for (int index = 0; index < 19; ++index) ast_second_page_words[index] = 0;
    }
    if (ordinal == 3) {
        *nir_page_length = 0;
        *nir_second_page_length = 0;
        for (int index = 0; index < 19; ++index) nir_page_words[index] = 0;
        for (int index = 0; index < 19; ++index) nir_second_page_words[index] = 0;
    }
    for (;;) {
        int byte = fgetc(file);
        if (byte == EOF) break;
        state = nuis_bootstrap_candidate_stage_fold_v1(state, ordinal, (int64_t)byte);
        if (ordinal == 1) {
            if (token_bytes >= token_max_bytes) {
                fclose(file);
                return 68;
            }
            *token_semantic = nuis_bootstrap_candidate_token_semantic_step_v1(
                *token_semantic,
                token_mode,
                (int64_t)byte
            );
            *token_count = nuis_bootstrap_candidate_token_count_step_v1(
                *token_count,
                token_mode,
                (int64_t)byte
            );
            token_mode = nuis_bootstrap_candidate_token_step_v1(token_mode, (int64_t)byte);
            token_bytes += 1;
            pack_page_byte(token_page_length, token_page_words, byte);
            if (token_mode == token_error || *token_count < 0) {
                fclose(file);
                return 69;
            }
        }
        if (ordinal == 2) pack_projection_byte(
            ast_page_length,
            ast_page_words,
            ast_second_page_length,
            ast_second_page_words,
            byte
        );
        if (ordinal == 3) pack_projection_byte(
            nir_page_length,
            nir_page_words,
            nir_second_page_length,
            nir_second_page_words,
            byte
        );
    }
    if (ferror(file) != 0) {
        fclose(file);
        return 66;
    }
    if (fclose(file) != 0) return 67;
    if (ordinal == 1 && nuis_bootstrap_candidate_token_finish_v1(
        token_mode,
        *token_count
    ) != 0) return 70;
    *out = state;
    return 0;
}

static int64_t projection_resume_value(
    int64_t selector,
    int64_t projection,
    int64_t cursor[8],
    int64_t length,
    int64_t words[19]
) {
    return nuis_bootstrap_candidate_projection_page_resume_value_v1(
        selector,
        projection,
        cursor[0], cursor[1], cursor[2], cursor[3],
        cursor[4], cursor[5], cursor[6], cursor[7],
        length,
        words[0], words[1], words[2], words[3], words[4],
        words[5], words[6], words[7], words[8], words[9],
        words[10], words[11], words[12], words[13], words[14],
        words[15], words[16], words[17], words[18]
    );
}

static int projection_continuation(
    int64_t projection,
    int64_t first_length,
    int64_t first_words[19],
    int64_t second_length,
    int64_t second_words[19],
    int64_t expected_first_identity,
    int64_t* first_cursor_identity,
    int64_t* second_identity,
    int64_t* second_cursor_identity,
    int64_t first_cursor[8],
    int64_t second_cursor[8]
) {
    if (second_length <= 0) return 0;
    int64_t fresh[8] = {-1, 0, 0, 0, 0, 0, 0, 0};
    if (projection_resume_value(0, projection, fresh, first_length, first_words)
        != expected_first_identity) return 0;
    for (int selector = 1; selector <= 8; ++selector) {
        first_cursor[selector - 1] = projection_resume_value(
            selector,
            projection,
            fresh,
            first_length,
            first_words
        );
        if (first_cursor[selector - 1] < 0) return 0;
    }
    *first_cursor_identity = projection_resume_value(
        9,
        projection,
        fresh,
        first_length,
        first_words
    );
    *second_identity = projection_resume_value(
        0,
        projection,
        first_cursor,
        second_length,
        second_words
    );
    for (int selector = 1; selector <= 8; ++selector) {
        second_cursor[selector - 1] = projection_resume_value(
            selector,
            projection,
            first_cursor,
            second_length,
            second_words
        );
        if (second_cursor[selector - 1] < 0) return 0;
    }
    *second_cursor_identity = projection_resume_value(
        9,
        projection,
        first_cursor,
        second_length,
        second_words
    );
    return *first_cursor_identity >= 0
        && *second_identity > 0
        && *second_cursor_identity >= 0;
}

int main(int argc, char** argv) {
    if (argc != 6) return 64;
    int64_t folds[5] = {0, 0, 0, 0, 0};
    int64_t bundle = nuis_bootstrap_candidate_bundle_seed_v1();
    int64_t token_count = 0;
    int64_t token_semantic = 0;
    int64_t token_page_length = 0;
    int64_t token_page_words[19] = {0};
    int64_t ast_page_length = 0;
    int64_t ast_page_words[19] = {0};
    int64_t ast_second_page_length = 0;
    int64_t ast_second_page_words[19] = {0};
    int64_t nir_page_length = 0;
    int64_t nir_page_words[19] = {0};
    int64_t nir_second_page_length = 0;
    int64_t nir_second_page_words[19] = {0};
    for (int64_t ordinal = 0; ordinal < 5; ++ordinal) {
        int status = fold_file(
            argv[ordinal + 1],
            ordinal,
            &folds[ordinal],
            &token_count,
            &token_semantic,
            &token_page_length,
            token_page_words,
            &ast_page_length,
            ast_page_words,
            &ast_second_page_length,
            ast_second_page_words,
            &nir_page_length,
            nir_page_words,
            &nir_second_page_length,
            nir_second_page_words
        );
        if (status != 0) return status;
        bundle = nuis_bootstrap_candidate_bundle_fold_v1(bundle, ordinal, folds[ordinal]);
    }
    int64_t token_page_identity = nuis_bootstrap_candidate_token_page_identity_v1(
        token_page_length,
        token_page_words[0], token_page_words[1], token_page_words[2],
        token_page_words[3], token_page_words[4], token_page_words[5],
        token_page_words[6], token_page_words[7], token_page_words[8],
        token_page_words[9], token_page_words[10], token_page_words[11],
        token_page_words[12], token_page_words[13], token_page_words[14],
        token_page_words[15], token_page_words[16], token_page_words[17],
        token_page_words[18]
    );
    if (token_page_identity <= 0) return 71;
    int64_t ast_page_identity = nuis_bootstrap_candidate_ast_page_identity_v1(
        ast_page_length,
        ast_page_words[0], ast_page_words[1], ast_page_words[2],
        ast_page_words[3], ast_page_words[4], ast_page_words[5],
        ast_page_words[6], ast_page_words[7], ast_page_words[8],
        ast_page_words[9], ast_page_words[10], ast_page_words[11],
        ast_page_words[12], ast_page_words[13], ast_page_words[14],
        ast_page_words[15], ast_page_words[16], ast_page_words[17],
        ast_page_words[18]
    );
    if (ast_page_identity <= 0) return 72;
    int64_t nir_page_identity = nuis_bootstrap_candidate_nir_page_identity_v1(
        nir_page_length,
        nir_page_words[0], nir_page_words[1], nir_page_words[2],
        nir_page_words[3], nir_page_words[4], nir_page_words[5],
        nir_page_words[6], nir_page_words[7], nir_page_words[8],
        nir_page_words[9], nir_page_words[10], nir_page_words[11],
        nir_page_words[12], nir_page_words[13], nir_page_words[14],
        nir_page_words[15], nir_page_words[16], nir_page_words[17],
        nir_page_words[18]
    );
    if (nir_page_identity <= 0) return 73;
    int64_t ast_page_cursor_identity = 0;
    int64_t ast_continuation_page_identity = 0;
    int64_t ast_continuation_cursor_identity = 0;
    int64_t ast_page_cursor[8] = {0};
    int64_t ast_continuation_cursor[8] = {0};
    if (!projection_continuation(
        1,
        ast_page_length,
        ast_page_words,
        ast_second_page_length,
        ast_second_page_words,
        ast_page_identity,
        &ast_page_cursor_identity,
        &ast_continuation_page_identity,
        &ast_continuation_cursor_identity,
        ast_page_cursor,
        ast_continuation_cursor
    )) return 74;
    int64_t nir_page_cursor_identity = 0;
    int64_t nir_continuation_page_identity = 0;
    int64_t nir_continuation_cursor_identity = 0;
    int64_t nir_page_cursor[8] = {0};
    int64_t nir_continuation_cursor[8] = {0};
    if (!projection_continuation(
        2,
        nir_page_length,
        nir_page_words,
        nir_second_page_length,
        nir_second_page_words,
        nir_page_identity,
        &nir_page_cursor_identity,
        &nir_continuation_page_identity,
        &nir_continuation_cursor_identity,
        nir_page_cursor,
        nir_continuation_cursor
    )) return 75;
    puts("protocol=nuis-bootstrap-candidate-scalar-output-v7");
    for (int ordinal = 0; ordinal < 5; ++ordinal) {
        printf("stage.%d=%lld\n", ordinal, (long long)folds[ordinal]);
    }
    printf("bundle=%lld\n", (long long)bundle);
    printf("tokens.record_count=%lld\n", (long long)token_count);
    printf("tokens.semantic_fold=%lld\n", (long long)token_semantic);
    printf("tokens.page_identity=%lld\n", (long long)token_page_identity);
    printf("ast.page_identity=%lld\n", (long long)ast_page_identity);
    printf("ast.page_cursor_identity=%lld\n", (long long)ast_page_cursor_identity);
    printf("ast.continuation_page_identity=%lld\n", (long long)ast_continuation_page_identity);
    printf("ast.continuation_cursor_identity=%lld\n", (long long)ast_continuation_cursor_identity);
    printf("nir.page_identity=%lld\n", (long long)nir_page_identity);
    printf("nir.page_cursor_identity=%lld\n", (long long)nir_page_cursor_identity);
    printf("nir.continuation_page_identity=%lld\n", (long long)nir_continuation_page_identity);
    printf("nir.continuation_cursor_identity=%lld\n", (long long)nir_continuation_cursor_identity);
    for (int index = 0; index < 8; ++index) {
        printf("nir.first_cursor_lane.%d=%lld\n", index, (long long)nir_page_cursor[index]);
    }
    for (int index = 0; index < 8; ++index) {
        printf("nir.continuation_cursor_lane.%d=%lld\n", index, (long long)nir_continuation_cursor[index]);
    }
    return 0;
}
"#
}

#[cfg(test)]
#[path = "bootstrap_candidate_adapter_tests.rs"]
mod tests;
