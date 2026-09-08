use super::*;
use crate::aot::{BuildManifestContext, CompileArtifacts};

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixture() -> Fixture {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "nuis-headless-metadata-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&dir).unwrap();
    let source = "mod cpu Main { fn main() { print(1); } }\n";
    let compiled = crate::pipeline::compile_source(source).unwrap();
    let layout = crate::aot_output_layout::output_layout(Path::new("demo.ns"), &dir);
    let stage_handoff = crate::stage_handoff::write_and_verify_compiler_stage_handoff(
        source,
        &layout,
        &compiled.ast,
        &compiled.nir,
        &compiled.yir,
    )
    .unwrap();
    for (name, bytes) in [
        ("demo", "binary fixture"),
        (
            "bundle.txt",
            "application_script_contract=nuis-yir-application-scalar-script-v1\n",
        ),
    ] {
        fs::write(dir.join(name), bytes).unwrap();
    }
    let file = |name| dir.join(name).display().to_string();
    crate::aot::write_build_manifest(
        &dir,
        &CompileArtifacts {
            ast_path: file("demo.ast.txt"),
            nir_path: file("demo.nir.txt"),
            yir_path: file("demo.yir"),
            llvm_ir_path: None,
            binary_path: file("demo"),
            packaging_mode: "headless-aot-bundle".to_owned(),
            host_objects: vec![],
            stage_handoff: Some(stage_handoff),
        },
        &BuildManifestContext {
            input_path: file("demo.ns"),
            output_dir: dir.display().to_string(),
            loaded_nustar: vec![],
            compile_cache: None,
            project: None,
            doc_index: None,
            cpu_target: crate::aot::host_cpu_build_target(),
        },
    )
    .unwrap();
    Fixture(dir)
}

#[test]
fn headless_standalone_verification_retains_bound_inputs_without_original_sidecars() {
    let fixture = fixture();
    crate::aot::verify_build_manifest(&fixture.0.join("nuis.build.manifest.toml")).unwrap();
    for name in [
        "demo.ast.txt",
        "demo.nir.txt",
        "demo.yir",
        "demo.source.ns",
        "demo.tokens.txt",
        "nuis.compiler-stage-handoff.toml",
        "bundle.txt",
        "demo",
    ] {
        fs::remove_file(fixture.0.join(name)).unwrap();
    }
    let report =
        crate::aot::verify_nuis_compiled_artifact(&fixture.0.join("nuis.compiled.artifact"))
            .unwrap();
    assert!(report.artifact_roundtrip_verified);
    assert_eq!(report.packaging_mode, "headless-aot-bundle");
}

#[test]
fn headless_embedded_inputs_reject_drift_duplicates_missing_rows_and_invalid_utf8() {
    let fixture = fixture();
    let manifest_path = fixture.0.join("nuis.build.manifest.toml");
    let source = fs::read_to_string(&manifest_path).unwrap();
    let rows = parse_artifact_hash_blocks(&source, &manifest_path).unwrap();
    verify_sources(&source, &manifest_path, &rows).unwrap();
    let yir = unique_value(&source, "headless_yir_hex", &manifest_path).unwrap();
    let changed_yir = format!("ff{}", &yir[2..]);
    for changed in [
        source.replace(SCHEMA, "nuis-headless-build-inputs-v1"),
        source.replace("verified-yir-v1", "llvm-v1"),
        format!("{source}\nheadless_input_schema = \"{SCHEMA}\"\n"),
        format!("{source}\nheadless_yir_hex = \"\"\n"),
        source.replace(&yir, &changed_yir),
    ] {
        assert!(verify_sources(&changed, &manifest_path, &rows).is_err());
    }
    let mut missing = rows.clone();
    missing.retain(|row| row.kind != "application_bundle");
    assert!(verify_sources(&source, &manifest_path, &missing).is_err());
    let mut duplicate = rows.clone();
    duplicate.push(rows.iter().find(|row| row.kind == "yir").unwrap().clone());
    assert!(verify_sources(&source, &manifest_path, &duplicate).is_err());
    let mut artifact =
        crate::aot::parse_nuis_compiled_artifact(&fixture.0.join("nuis.compiled.artifact"))
            .unwrap();
    artifact.build_manifest_source = source.replace(&yir, &changed_yir);
    artifact.build_manifest_bytes = artifact.build_manifest_source.len();
    crate::aot::write_nuis_compiled_artifact(&fixture.0.join("nuis.compiled.artifact"), &artifact)
        .unwrap();
    assert!(
        crate::aot::verify_nuis_compiled_artifact(&fixture.0.join("nuis.compiled.artifact"))
            .is_err()
    );
}

#[test]
fn headless_checkpoint_rejects_rehashed_projection_drift_and_llvm_claims() {
    let fixture = fixture();
    let path = fixture.0.join("nuis.build.manifest.toml");
    let source = fs::read_to_string(&path).unwrap();
    let mut rows = parse_artifact_hash_blocks(&source, &path).unwrap();
    let encoded = unique_value(&source, "headless_source_hex", &path).unwrap();
    let changed = b"mod cpu Main { fn main() { print(2); } }\n";
    let row = rows
        .iter_mut()
        .find(|row| row.kind == "compiler_source")
        .unwrap();
    row.bytes = changed.len();
    row.fnv1a64 = fnv1a64_hex(changed);
    let error = verify_sources(
        &source.replace(&encoded, &hex_encode_bytes(changed)),
        &path,
        &rows,
    )
    .unwrap_err();
    assert!(error.contains("handoff"), "{error}");
    rows[0].kind = "llvm_ir".to_owned();
    assert!(verify_sources(&source, &path, &rows)
        .unwrap_err()
        .contains("without LLVM"));
}
