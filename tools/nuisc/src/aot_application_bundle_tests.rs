use super::*;
use crate::aot::{BuildManifestContext, CompileArtifacts};

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixture() -> Fixture {
    fixture_for_mode("headless-aot-bundle")
}

fn fixture_for_mode(mode: &str) -> Fixture {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "nuis-headless-metadata-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&dir).unwrap();
    let native_id = crate::aot_native_session::registration_id(mode).unwrap();
    let source = if native_id.is_some() {
        include_str!("../tests/native_application_bridge/main.ns")
    } else {
        "mod cpu Main { fn main() { print(1); } }\n"
    };
    let compiled = if let Some(id) = native_id {
        fs::write(dir.join("main.ns"), source).unwrap();
        fs::write(dir.join("nuis.toml"), format!("name = \"metadata_fixture\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\napplication_sessions = [\"{id} open=start event=step close=stop state=state\"]\n")).unwrap();
        crate::pipeline::compile_project(&dir).unwrap()
    } else {
        crate::pipeline::compile_source(source).unwrap()
    };
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
    if let Some(id) = native_id {
        let bridge = yir_lower_llvm::native_session::emit_registered(&compiled.yir, id).unwrap();
        fs::write(dir.join("demo.ll"), bridge.llvm_ir).unwrap();
        fs::write(dir.join("bundle.txt"), format!("cpu_host_binary_mode=native_scalar_session\nruntime_bootstrap_mode=static_native_session\napplication_session_id={id}\nnative_session_contract={}\nnative_session_identity=exact-yir-graph\nnative_session_layout={}\nsingle_binary=true\n", yir_core::native_scalar_session::CONTRACT, bridge.state_layout.source())).unwrap();
    }
    let file = |name| dir.join(name).display().to_string();
    crate::aot::write_build_manifest(
        &dir,
        &CompileArtifacts {
            ast_path: file("demo.ast.txt"),
            nir_path: file("demo.nir.txt"),
            yir_path: file("demo.yir"),
            llvm_ir_path: native_id.map(|_| file("demo.ll")),
            binary_path: file("demo"),
            packaging_mode: mode.to_owned(),
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
fn native_inputs_reject_rehashed_llvm_bundle_and_profile_drift() {
    let fixture = fixture_for_mode("native-session-aot-bundle:counter");
    let path = fixture.0.join("nuis.build.manifest.toml");
    let source = fs::read_to_string(&path).unwrap();
    let rows = parse_artifact_hash_blocks(&source, &path).unwrap();
    verify_sources(&source, &path, &rows).unwrap();
    for (kind, key, replacement, expected) in [
        (
            "llvm_ir",
            "native_session_llvm_hex",
            "invalid LLVM".to_owned(),
            "LLVM checkpoint",
        ),
        (
            "application_bundle",
            "native_session_bundle_hex",
            fs::read_to_string(fixture.0.join("bundle.txt"))
                .unwrap()
                .replace(
                    "application_session_id=counter",
                    "application_session_id=other",
                ),
            "application_session_id",
        ),
    ] {
        let encoded = unique_value(&source, key, &path).unwrap();
        let changed = source.replace(&encoded, &hex_encode_bytes(replacement.as_bytes()));
        let mut rehashed = rows.clone();
        let row = rehashed.iter_mut().find(|row| row.kind == kind).unwrap();
        row.bytes = replacement.len();
        row.fnv1a64 = fnv1a64_hex(replacement.as_bytes());
        let error = verify_sources(&changed, &path, &rehashed).unwrap_err();
        assert!(error.contains(expected), "{error}");
    }
    for changed in [
        source.replace("native-scalar-llvm-v1", "verified-yir-v1"),
        source.replace("nuis-native-session-build-inputs-v1", SCHEMA),
        source.replace(
            "native-session-aot-bundle:counter",
            "native-session-aot-bundle:other",
        ),
        format!("{source}\nnative_session_llvm_hex = \"\"\n"),
        source.replace("native-session-aot-bundle:counter", "headless-aot-bundle"),
    ] {
        assert!(verify_sources(&changed, &path, &rows).is_err());
    }
    let missing = rows
        .iter()
        .filter(|row| row.kind != "llvm_ir")
        .cloned()
        .collect::<Vec<_>>();
    assert!(verify_sources(&source, &path, &missing).is_err());
    crate::aot::verify_nuis_compiled_artifact(&fixture.0.join("nuis.compiled.artifact")).unwrap();
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
