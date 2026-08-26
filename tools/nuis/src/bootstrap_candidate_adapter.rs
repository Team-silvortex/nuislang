use std::{fs, path::Path, process::Command};

use nuis_artifact::{
    compiler_candidate_bundle_fold, compiler_candidate_stage_fold, parse_build_manifest,
    CompilerStageHandoff, COMPILER_CANDIDATE_ADAPTER_FILE,
};

const ADAPTER_OUTPUT_PROTOCOL: &str = "nuis-bootstrap-candidate-scalar-output-v1";
const ADAPTER_SOURCE_FILE: &str = "nuis.compiler-candidate-adapter.c";
const ADAPTER_RUNTIME_OBJECT_FILE: &str = "nuis.compiler-candidate-runtime.o";

pub(crate) struct CandidateAdapterOutput {
    pub(crate) adapter_file: &'static str,
    pub(crate) adapter: Vec<u8>,
    pub(crate) stage_folds: Vec<usize>,
    pub(crate) bundle_fold: usize,
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
    let (stage_folds, bundle_fold) = parse_adapter_output(&output.stdout)?;
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
    if stage_folds != expected_folds || bundle_fold != expected_bundle {
        return Err(
            "Nuis candidate scalar output disagrees with the independent host fold".to_owned(),
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
    })
}

fn parse_adapter_output(bytes: &[u8]) -> Result<(Vec<usize>, usize), String> {
    let source = std::str::from_utf8(bytes)
        .map_err(|error| format!("candidate scalar output is not UTF-8: {error}"))?;
    if source.contains('\r') || source.contains('\0') || !source.ends_with('\n') {
        return Err("candidate scalar output must use canonical UTF-8/LF text".to_owned());
    }
    let lines = source.lines().collect::<Vec<_>>();
    if lines.len() != 7 || lines[0] != format!("protocol={ADAPTER_OUTPUT_PROTOCOL}") {
        return Err("candidate scalar output has an invalid protocol or line count".to_owned());
    }
    let stage_folds = (0..5)
        .map(|ordinal| parse_output_usize(lines[ordinal + 1], &format!("stage.{ordinal}")))
        .collect::<Result<Vec<_>, _>>()?;
    let bundle_fold = parse_output_usize(lines[6], "bundle")?;
    Ok((stage_folds, bundle_fold))
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

static int fold_file(const char* path, int64_t ordinal, int64_t* out) {
    FILE* file = fopen(path, "rb");
    if (file == NULL) return 65;
    int64_t state = nuis_bootstrap_candidate_stage_seed_v1(ordinal);
    for (;;) {
        int byte = fgetc(file);
        if (byte == EOF) break;
        state = nuis_bootstrap_candidate_stage_fold_v1(state, ordinal, (int64_t)byte);
    }
    if (ferror(file) != 0) {
        fclose(file);
        return 66;
    }
    if (fclose(file) != 0) return 67;
    *out = state;
    return 0;
}

int main(int argc, char** argv) {
    if (argc != 6) return 64;
    int64_t folds[5] = {0, 0, 0, 0, 0};
    int64_t bundle = nuis_bootstrap_candidate_bundle_seed_v1();
    for (int64_t ordinal = 0; ordinal < 5; ++ordinal) {
        int status = fold_file(argv[ordinal + 1], ordinal, &folds[ordinal]);
        if (status != 0) return status;
        bundle = nuis_bootstrap_candidate_bundle_fold_v1(bundle, ordinal, folds[ordinal]);
    }
    puts("protocol=nuis-bootstrap-candidate-scalar-output-v1");
    for (int ordinal = 0; ordinal < 5; ++ordinal) {
        printf("stage.%d=%lld\n", ordinal, (long long)folds[ordinal]);
    }
    printf("bundle=%lld\n", (long long)bundle);
    return 0;
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_output_parser_requires_exact_order_and_utf8_lf() {
        let source = b"protocol=nuis-bootstrap-candidate-scalar-output-v1\nstage.0=1\nstage.1=2\nstage.2=3\nstage.3=4\nstage.4=5\nbundle=6\n";
        assert_eq!(
            parse_adapter_output(source).unwrap(),
            (vec![1, 2, 3, 4, 5], 6)
        );

        let reordered = b"protocol=nuis-bootstrap-candidate-scalar-output-v1\nstage.1=1\nstage.0=2\nstage.2=3\nstage.3=4\nstage.4=5\nbundle=6\n";
        assert!(parse_adapter_output(reordered).is_err());
        assert!(parse_adapter_output(&source[..source.len() - 1]).is_err());
    }
}
