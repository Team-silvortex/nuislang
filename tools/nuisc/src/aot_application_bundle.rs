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
pub(crate) const METADATA: [(&str, &str, &str); 5] = [
    ("doc_index_path", "doc_index", "nuis.doc-index.json"),
    ("docs_index", "project_docs", "nuis.project.docs.txt"),
    (
        "imports_index",
        "project_imports",
        "nuis.project.imports.txt",
    ),
    ("galaxy_index", "project_galaxy", "nuis.project.galaxy.txt"),
    (
        "galaxy_resolution_lock",
        "galaxy_lock",
        "nuis.project.galaxy.lock",
    ),
];

pub(crate) fn metadata_inputs(
    source: &str,
    path: &Path,
) -> Result<Vec<(&'static str, PathBuf)>, String> {
    METADATA
        .iter()
        .filter(|(key, _, _)| {
            source.lines().any(|line| {
                line.split_once('=')
                    .is_some_and(|(candidate, _)| candidate.trim() == *key)
            })
        })
        .map(|(key, kind, _)| Ok((*kind, PathBuf::from(unique_value(source, key, path)?))))
        .collect()
}

pub(crate) fn append_metadata_artifacts(
    context: &crate::aot_manifest_types::BuildManifestContext,
    artifacts: &mut Vec<(String, PathBuf)>,
) {
    for (key, kind, _) in METADATA {
        let path = match key {
            "doc_index_path" => context.doc_index.as_ref().map(|index| &index.path),
            "docs_index" => context
                .project
                .as_ref()
                .and_then(|p| p.docs_index_path.as_ref()),
            "imports_index" => context
                .project
                .as_ref()
                .and_then(|p| p.imports_index_path.as_ref()),
            "galaxy_index" => context
                .project
                .as_ref()
                .and_then(|p| p.galaxy_index_path.as_ref()),
            "galaxy_resolution_lock" => context
                .project
                .as_ref()
                .and_then(|p| p.galaxy_resolution_lock_path.as_ref()),
            _ => unreachable!(),
        };
        if let Some(path) = path {
            artifacts.push((kind.to_owned(), path.into()));
        }
    }
}
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

#[path = "aot_application_checkpoint.rs"]
mod checkpoint;

struct Profile {
    prefix: &'static str,
    schema: &'static str,
    checkpoint: &'static str,
    native: bool,
    metadata: Vec<(&'static str, PathBuf)>,
}

impl Profile {
    fn from_manifest(source: &str, path: &Path) -> Result<Self, String> {
        let mode = unique_value(source, "packaging_mode", path)?;
        let metadata = metadata_inputs(source, path)?;
        if crate::aot_native_session::registration_id(&mode)?.is_some() {
            Ok(Self {
                prefix: "native_session",
                schema: "nuis-native-session-build-inputs-v1",
                checkpoint: "native-scalar-llvm-v1",
                native: true,
                metadata,
            })
        } else if mode == "headless-aot-bundle" {
            Ok(Self {
                prefix: "headless",
                schema: SCHEMA,
                checkpoint: "verified-yir-v1",
                native: false,
                metadata,
            })
        } else {
            Err("packaging mode does not carry bound application inputs".to_owned())
        }
    }

    fn inputs(&self) -> impl Iterator<Item = (&'static str, String, usize)> + '_ {
        INPUTS
            .into_iter()
            .chain(
                self.native
                    .then_some(("llvm_ir", "headless_llvm_hex", 32 * 1024 * 1024)),
            )
            .map(|(kind, key, limit)| {
                let limit = if self.native && kind == "yir" {
                    yir_core::native_scalar_session::MAX_BINDING_SOURCE_BYTES as usize
                } else {
                    limit
                };
                (kind, key.replacen("headless", self.prefix, 1), limit)
            })
            .chain(self.metadata.iter().map(|(kind, _)| {
                (
                    *kind,
                    format!("{}_{kind}_hex", self.prefix),
                    8 * 1024 * 1024,
                )
            }))
    }
}

/// Carry the admitted host inputs through standalone artifact verification and
/// relocation. Reconstructing capability claims from the current host is unsafe.
pub(crate) fn append_sources(
    out: &mut String,
    artifacts: &[(String, PathBuf)],
) -> Result<(), String> {
    let profile = Profile::from_manifest(out, Path::new("<application-build>"))?;
    let prefix = profile.prefix;
    let schema = profile.schema;
    let checkpoint = profile.checkpoint;
    out.push_str(&format!(
        "\n[{prefix}_application]\n{prefix}_input_schema = \"{schema}\"\n{prefix}_compiler_checkpoint = \"{checkpoint}\"\n"
    ));
    let mut total = 0;
    for (kind, key, limit) in profile.inputs() {
        let paths = artifacts
            .iter()
            .filter(|(candidate, _)| candidate == kind)
            .collect::<Vec<_>>();
        if paths.len() != 1 {
            return Err(format!(
                "application bundle needs exactly one `{kind}` input"
            ));
        }
        let path = &paths[0].1;
        let size = fs::metadata(path).map_err(|error| error.to_string())?.len();
        if size > limit as u64 {
            return Err(format!(
                "application `{kind}` exceeds its {limit}-byte input bound"
            ));
        }
        let bytes = fs::read(path).map_err(|error| error.to_string())?;
        if bytes.len() > limit {
            return Err(format!("application `{kind}` grew beyond its input bound"));
        }
        total += bytes.len();
        if total > TOTAL_LIMIT {
            return Err("application inputs exceed the 64 MiB aggregate bound".to_owned());
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
    let profile = Profile::from_manifest(source, path)?;
    if unique_value(source, &format!("{}_input_schema", profile.prefix), path)? != profile.schema {
        return Err("unsupported application build input schema".to_owned());
    }
    if unique_value(
        source,
        &format!("{}_compiler_checkpoint", profile.prefix),
        path,
    )? != profile.checkpoint
    {
        return Err("application build compiler checkpoint does not match its profile".to_owned());
    }
    if !profile.native && rows.iter().any(|row| row.kind == "llvm_ir") {
        return Err("headless build requires the verified-YIR checkpoint without LLVM".to_owned());
    }
    for (kind, declared) in &profile.metadata {
        let matching = rows
            .iter()
            .filter(|row| row.kind == *kind)
            .collect::<Vec<_>>();
        if matching.len() != 1 || Path::new(&matching[0].path) != declared {
            return Err(format!(
                "application metadata `{kind}` does not match its bound path"
            ));
        }
    }
    let mut total = 0;
    let inputs = profile
        .inputs()
        .map(|(kind, key, limit)| {
            let encoded = unique_value(source, &key, path)?;
            if encoded.len() > limit * 2 {
                return Err(format!(
                    "embedded application `{kind}` exceeds its input bound"
                ));
            }
            total += encoded.len();
            if total > TOTAL_LIMIT * 2 {
                return Err(
                    "embedded application inputs exceed the 64 MiB aggregate bound".to_owned(),
                );
            }
            let bytes = hex_decode_bytes(&encoded)?;
            std::str::from_utf8(&bytes)
                .map_err(|_| format!("embedded application `{kind}` is not UTF-8"))?;
            let expected = rows
                .iter()
                .filter(|row| row.kind == kind)
                .collect::<Vec<_>>();
            if expected.len() != 1
                || expected[0].bytes != bytes.len()
                || expected[0].fnv1a64 != fnv1a64_hex(&bytes)
            {
                return Err(format!(
                    "embedded application `{kind}` does not match its artifact hash"
                ));
            }
            Ok((kind, bytes))
        })
        .collect::<Result<Vec<_>, String>>()?;
    checkpoint::verify(source, path, &inputs)?;
    Ok(inputs)
}

pub(crate) fn input_file_name(kind: &str, binary_name: &str) -> String {
    if let Some((_, _, name)) = METADATA.iter().find(|(_, candidate, _)| *candidate == kind) {
        return (*name).to_owned();
    }
    match kind {
        "application_bundle" => "bundle.txt".to_owned(),
        "compiler_stage_handoff" => "nuis.compiler-stage-handoff.toml".to_owned(),
        "compiler_source" => format!("{binary_name}.source.ns"),
        "compiler_tokens" => format!("{binary_name}.tokens.txt"),
        "ast" | "nir" => format!("{binary_name}.{kind}.txt"),
        "yir" => format!("{binary_name}.yir"),
        "llvm_ir" => format!("{binary_name}.ll"),
        _ => unreachable!("unknown application checkpoint input"),
    }
}

fn unique_value(source: &str, key: &str, path: &Path) -> Result<String, String> {
    let count = source
        .lines()
        .filter_map(|line| line.split_once('='))
        .filter(|(candidate, _)| candidate.trim() == key)
        .count();
    if count != 1 {
        return Err(format!(
            "application build inputs require exactly one `{key}`"
        ));
    }
    parse_required_toml_string(source, key, path)
}

pub(crate) fn restore_sources(
    source: &str,
    output_dir: &Path,
    binary_name: &str,
) -> Result<Vec<(String, PathBuf)>, String> {
    let context = Path::new("<application-artifact>");
    crate::aot_artifact::validate_artifact_binary_name("binary_name", binary_name, context)?;
    let rows = parse_artifact_hash_blocks(source, context)?;
    let inputs = decode_sources(source, context, &rows)?;
    inputs
        .into_iter()
        .map(|(kind, bytes)| {
            let name = input_file_name(kind, binary_name);
            let path = output_dir.join(name);
            fs::write(&path, bytes)
                .map_err(|error| format!("cannot restore application `{kind}`: {error}"))?;
            Ok((kind.to_owned(), path))
        })
        .collect()
}

#[cfg(test)]
#[path = "aot_application_bundle_tests.rs"]
mod tests;
