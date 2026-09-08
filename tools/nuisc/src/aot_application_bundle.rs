use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    aot_artifact_hash::{parse_artifact_hash_blocks, ArtifactHashRow},
    aot_encoding::{fnv1a64_hex, hex_decode_bytes, hex_encode_bytes},
    aot_toml::parse_required_toml_string,
};

const SCHEMA: &str = "nuis-headless-build-inputs-v2";
const TOTAL_LIMIT: usize = 64 * 1024 * 1024;
const INPUTS: [(&str, &str, usize); 7] = [
    ("application_bundle", "headless_bundle_hex", 1024 * 1024),
    ("yir", "headless_yir_hex", 32 * 1024 * 1024),
    ("compiler_source", "headless_source_hex", 16 * 1024 * 1024),
    ("compiler_tokens", "headless_tokens_hex", 32 * 1024 * 1024),
    ("ast", "headless_ast_hex", 32 * 1024 * 1024),
    ("nir", "headless_nir_hex", 32 * 1024 * 1024),
    (
        "compiler_stage_handoff",
        "headless_handoff_hex",
        1024 * 1024,
    ),
];

#[path = "aot_headless_checkpoint.rs"]
mod checkpoint;

/// Carry the admitted host inputs through standalone artifact verification and
/// relocation. Reconstructing capability claims from the current host is unsafe.
pub(crate) fn append_sources(
    out: &mut String,
    artifacts: &[(String, PathBuf)],
) -> Result<(), String> {
    out.push_str(&format!(
        "\n[headless_application]\nheadless_input_schema = \"{SCHEMA}\"\nheadless_compiler_checkpoint = \"verified-yir-v1\"\n"
    ));
    let mut total = 0;
    for (kind, key, limit) in INPUTS {
        let paths = artifacts
            .iter()
            .filter(|(candidate, _)| candidate == kind)
            .collect::<Vec<_>>();
        if paths.len() != 1 {
            return Err(format!("headless bundle needs exactly one `{kind}` input"));
        }
        let path = &paths[0].1;
        let size = fs::metadata(path).map_err(|error| error.to_string())?.len();
        if size > limit as u64 {
            return Err(format!(
                "headless `{kind}` exceeds its {limit}-byte input bound"
            ));
        }
        let bytes = fs::read(path).map_err(|error| error.to_string())?;
        if bytes.len() > limit {
            return Err(format!("headless `{kind}` grew beyond its input bound"));
        }
        total += bytes.len();
        if total > TOTAL_LIMIT {
            return Err("headless inputs exceed the 64 MiB aggregate bound".to_owned());
        }
        out.push_str(&format!("{key} = \"{}\"\n", hex_encode_bytes(&bytes)));
    }
    Ok(())
}

pub(crate) fn verify_sources(
    source: &str,
    path: &Path,
    rows: &[ArtifactHashRow],
) -> Result<(), String> {
    decode_sources(source, path, rows).map(|_| ())
}

fn decode_sources(
    source: &str,
    path: &Path,
    rows: &[ArtifactHashRow],
) -> Result<Vec<(&'static str, Vec<u8>)>, String> {
    if unique_value(source, "headless_input_schema", path)? != SCHEMA {
        return Err("unsupported headless build input schema".to_owned());
    }
    if unique_value(source, "headless_compiler_checkpoint", path)? != "verified-yir-v1"
        || rows.iter().any(|row| row.kind == "llvm_ir")
    {
        return Err("headless build requires the verified-YIR checkpoint without LLVM".to_owned());
    }
    let mut total = 0;
    let inputs = INPUTS
        .into_iter()
        .map(|(kind, key, limit)| {
            let encoded = unique_value(source, key, path)?;
            if encoded.len() > limit * 2 {
                return Err(format!(
                    "embedded headless `{kind}` exceeds its input bound"
                ));
            }
            total += encoded.len();
            if total > TOTAL_LIMIT * 2 {
                return Err("embedded headless inputs exceed the 64 MiB aggregate bound".to_owned());
            }
            let bytes = hex_decode_bytes(&encoded)?;
            std::str::from_utf8(&bytes)
                .map_err(|_| format!("embedded headless `{kind}` is not UTF-8"))?;
            let expected = rows
                .iter()
                .filter(|row| row.kind == kind)
                .collect::<Vec<_>>();
            if expected.len() != 1
                || expected[0].bytes != bytes.len()
                || expected[0].fnv1a64 != fnv1a64_hex(&bytes)
            {
                return Err(format!(
                    "embedded headless `{kind}` does not match its artifact hash"
                ));
            }
            Ok((kind, bytes))
        })
        .collect::<Result<Vec<_>, String>>()?;
    checkpoint::verify(source, path, &inputs)?;
    Ok(inputs)
}

pub(crate) fn input_file_name(kind: &str, binary_name: &str) -> String {
    match kind {
        "application_bundle" => "bundle.txt".to_owned(),
        "compiler_stage_handoff" => "nuis.compiler-stage-handoff.toml".to_owned(),
        "compiler_source" => format!("{binary_name}.source.ns"),
        "compiler_tokens" => format!("{binary_name}.tokens.txt"),
        "ast" | "nir" => format!("{binary_name}.{kind}.txt"),
        "yir" => format!("{binary_name}.yir"),
        _ => unreachable!("unknown headless checkpoint input"),
    }
}

fn unique_value(source: &str, key: &str, path: &Path) -> Result<String, String> {
    let count = source
        .lines()
        .filter_map(|line| line.split_once('='))
        .filter(|(candidate, _)| candidate.trim() == key)
        .count();
    if count != 1 {
        return Err(format!("headless build inputs require exactly one `{key}`"));
    }
    parse_required_toml_string(source, key, path)
}

pub(crate) fn restore_sources(
    source: &str,
    output_dir: &Path,
    binary_name: &str,
) -> Result<Vec<(String, PathBuf)>, String> {
    let context = Path::new("<headless-artifact>");
    crate::aot_artifact::validate_artifact_binary_name("binary_name", binary_name, context)?;
    let rows = parse_artifact_hash_blocks(source, context)?;
    let inputs = decode_sources(source, context, &rows)?;
    inputs
        .into_iter()
        .map(|(kind, bytes)| {
            let name = input_file_name(kind, binary_name);
            let path = output_dir.join(name);
            fs::write(&path, bytes)
                .map_err(|error| format!("cannot restore headless `{kind}`: {error}"))?;
            Ok((kind.to_owned(), path))
        })
        .collect()
}

#[cfg(test)]
#[path = "aot_application_bundle_tests.rs"]
mod tests;
