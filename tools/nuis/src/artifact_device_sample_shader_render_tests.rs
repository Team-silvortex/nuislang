use super::*;
use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
};

static NONCE: AtomicU64 = AtomicU64::new(0);

fn temp_output_dir() -> std::path::PathBuf {
    let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "nuis-shader-render-projection-{}-{nonce}",
        std::process::id()
    ))
}

fn write_fixture(output_dir: &Path) -> (String, String) {
    fs::create_dir_all(output_dir).unwrap();
    let yir = b"yir 0.1\n";
    let source_hash = fnv1a64_hex(yir);
    fs::write(output_dir.join("project.yir"), yir).unwrap();

    let asset_id = "shader.metal.project.0123456789abcdef";
    let path = "nuis.shader.project.0123456789abcdef.metal";
    let source = b"vertex void vs_main() {}\nfragment void fs_main() {}\n";
    let content_hash = fnv1a64_hex(source);
    fs::write(output_dir.join(path), source).unwrap();
    fs::write(
        output_dir.join(TABLE_FILE_NAME),
        format!(
            "schema = \"{TABLE_CONTRACT}\"\nsource_yir_version = \"0.1\"\nsource_fnv1a64 = \"{source_hash}\"\nlowering_target = \"metal.apple-silicon-gpu\"\nasset_count = 1\npass_count = 2\n\n[[asset]]\ncontract = \"{ASSET_CONTRACT}\"\nasset_id = \"{asset_id}\"\nfile_name = \"{path}\"\nformat = \"metal-source\"\ntarget = \"metal.apple-silicon-gpu\"\nentries = [\"vs_main\", \"fs_main\"]\nbyte_length = {}\ncontent_hash = \"{content_hash}\"\n\n[[pass]]\ncontract = \"{PASS_CONTRACT}\"\npass_node = \"render.first\"\nmodule_node = \"shader.inline.first\"\nasset_id = \"{asset_id}\"\ntarget_format = \"rgba8_unorm\"\nwidth = 32\nheight = 24\n\n[[pass]]\ncontract = \"{PASS_CONTRACT}\"\npass_node = \"render.second\"\nmodule_node = \"shader.inline.first\"\nasset_id = \"{asset_id}\"\ntarget_format = \"rgba8_unorm\"\nwidth = 16\nheight = 8\n",
            source.len()
        ),
    )
    .unwrap();

    let identity_hash = fnv1a64_hex(
        format!(
            "{DESCRIPTOR_IDENTITY_CONTRACT}\n{asset_id}\nmetal-source\nmetal.apple-silicon-gpu\n{path}\n{}\n{DIGEST_CONTRACT}\n{content_hash}\n2\nvs_main\nfs_main",
            source.len()
        )
        .as_bytes(),
    );
    let root_hash = fnv1a64_hex(
        format!(
            "{IDENTITY_SET_CONTRACT}\n1\n{asset_id}\n{DESCRIPTOR_IDENTITY_CONTRACT}\n{identity_hash}"
        )
        .as_bytes(),
    );
    let contribution_table_contract = "nuis-domain-code-asset-contribution-table-v1";
    let table_hash = fnv1a64_hex(
        format!(
            "{contribution_table_contract}\n1\n{PACKAGE_ID}\nshader\n{asset_id}\nmetal-source\nmetal.apple-silicon-gpu\nmetal.apple-silicon-gpu\n{path}\n2\nvs_main\nfs_main\n{}\n{content_hash}",
            source.len()
        )
        .as_bytes(),
    );
    fs::write(
        output_dir.join("nuis.domain.code-asset-contributions.toml"),
        format!(
            "protocol = \"{contribution_table_contract}\"\ncontribution_contract = \"{CONTRIBUTION_CONTRACT}\"\nidentity_set_contract = \"{IDENTITY_SET_CONTRACT}\"\ncontribution_count = 1\nidentity_set_root_hash = \"{root_hash}\"\ntable_hash = \"{table_hash}\"\n\n[[contribution]]\nindex = 0\nowner_package_id = \"{PACKAGE_ID}\"\ndomain_family = \"shader\"\nasset_id = \"{asset_id}\"\nformat = \"metal-source\"\nlowering_target = \"metal.apple-silicon-gpu\"\ntarget = \"metal.apple-silicon-gpu\"\npath = \"{path}\"\nentry_count = 2\nentries = [\"vs_main\", \"fs_main\"]\nbyte_length = {}\ndigest_contract = \"{DIGEST_CONTRACT}\"\ncontent_hash = \"{content_hash}\"\nidentity_contract = \"{DESCRIPTOR_IDENTITY_CONTRACT}\"\nidentity_hash = \"{identity_hash}\"\n",
            source.len()
        ),
    )
    .unwrap();
    (asset_id.to_owned(), path.to_owned())
}

fn registration_evidence() -> String {
    format!(
        "artifact_provider_metadata_0={METADATA_SELECTOR};provider_sample_registration_package={PACKAGE_ID};provider_sample_registration_id={REGISTRATION_ID};provider_shader_render_projection_contract={PROJECTION_CONTRACT}"
    )
}

#[test]
fn projects_verified_render_table_into_open_provider_requests() {
    let output_dir = temp_output_dir();
    let (asset_id, _) = write_fixture(&output_dir);

    let evidence = resolve_project_render_evidence(&output_dir, &registration_evidence()).unwrap();
    assert!(evidence.contains("provider_shader_render_projection_status=verified"));
    assert!(evidence.contains("provider_request_count=2"));
    assert!(evidence.contains(
        "provider_request_0_code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v2"
    ));
    assert!(evidence.contains("provider_request_0_code_asset_entries=vs_main,fs_main"));
    assert!(evidence.contains("provider_request_0_output_binding_0_shape=32x24"));
    assert!(evidence.contains("provider_request_1_output_binding_0_shape=16x8"));
    assert!(evidence.contains("provider_code_asset_identity_set_count=1"));
    assert!(evidence.contains(&format!(
        "provider_code_asset_contribution_asset_id={asset_id}"
    )));
    assert!(nsdb::validate_provider_request_evidence(&evidence));

    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn rejects_render_asset_drift_before_provider_handoff() {
    let output_dir = temp_output_dir();
    let (_, path) = write_fixture(&output_dir);
    fs::write(output_dir.join(path), b"drifted").unwrap();

    let error = resolve_project_render_evidence(&output_dir, &registration_evidence()).unwrap_err();
    assert!(error.contains("byte identity mismatch"));

    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn rejects_render_table_source_version_drift() {
    let output_dir = temp_output_dir();
    write_fixture(&output_dir);
    let table_path = output_dir.join(TABLE_FILE_NAME);
    let table = fs::read_to_string(&table_path).unwrap().replace(
        "source_yir_version = \"0.1\"",
        "source_yir_version = \"0.2\"",
    );
    fs::write(table_path, table).unwrap();

    let error = resolve_project_render_evidence(&output_dir, &registration_evidence()).unwrap_err();
    assert!(error.contains("source YIR identity has no matching AOT artifact"));

    fs::remove_dir_all(output_dir).unwrap();
}

#[cfg(target_os = "macos")]
#[test]
fn executes_ns_nova_aot_render_projection_through_nuis_worker() {
    let output_dir = temp_output_dir();
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let project_root = workspace_root.join("examples/projects/domains/ns_nova_showcase");
    crate::handle_build(project_root, output_dir.clone(), false, None, None, None)
        .expect("NS Nova showcase AOT build should succeed");

    let run_json = crate::render_run_artifact_json(&output_dir);
    assert!(run_json.contains("\"device_provider_sample_manifest_persisted\":true"));
    let manifest_path = output_dir.join("nuis.nsdb.device-provider-samples.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .expect("run-artifact should persist the provider sample manifest");
    let evidence = provider_input_evidence(&manifest, PROVIDER_FAMILY);
    assert!(evidence.contains(&format!(
        "provider_sample_registration_id={REGISTRATION_ID}"
    )));
    assert!(evidence.contains("provider_shader_render_projection_status=verified"));
    assert!(evidence.contains(
        "provider_request_0_code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v2"
    ));
    assert!(nsdb::validate_provider_request_evidence(&evidence));

    let report = nsdb::execute_provider_samples(&output_dir, Some(PROVIDER_FAMILY))
        .expect("NS Nova render should execute through its registered Nuis worker path");
    assert_eq!(report.output_payload_count, 1);
    assert_eq!(
        report.first_output_payload_native_output_kind,
        "provider-frame-rgba8"
    );
    assert_eq!(
        report.first_output_payload_native_output_bytes,
        (160 * 120 * 4).to_string()
    );
    assert_eq!(
        report.first_output_payload_native_execution_status,
        "metal-command-buffer-completed"
    );
    let payload = fs::read_to_string(
        output_dir.join("nuis.nsdb.provider-output.metal-apple-silicon-gpu.toml"),
    )
    .unwrap();
    assert!(payload.contains("nuis-provider-worker-process-adapter-v5"));
    assert!(payload.contains("metal.command-buffer.completed"));

    let asset_path = evidence_field(evidence, "provider_request_0_code_asset_path");
    let alias_path = "unbound-project-render.metal";
    fs::copy(output_dir.join(asset_path), output_dir.join(alias_path)).unwrap();
    let drifted = manifest.replace(
        &format!("provider_request_0_input_binding_0_payload_path={asset_path}"),
        &format!("provider_request_0_input_binding_0_payload_path={alias_path}"),
    );
    fs::write(&manifest_path, drifted).unwrap();
    let error = match nsdb::execute_provider_samples(&output_dir, Some(PROVIDER_FAMILY)) {
        Ok(_) => panic!("an alias outside the compiled code asset binding must fail closed"),
        Err(error) => error,
    };
    assert!(error.contains("verified MSL code asset capability"));

    fs::remove_dir_all(output_dir).unwrap();
}

#[cfg(target_os = "macos")]
fn provider_input_evidence<'a>(manifest: &'a str, provider_family: &str) -> &'a str {
    manifest
        .split("[[device_provider_samples]]")
        .find(|record| {
            record
                .lines()
                .any(|line| line.trim() == format!("provider_family = \"{provider_family}\""))
        })
        .and_then(|record| {
            record.lines().find_map(|line| {
                line.trim()
                    .strip_prefix("input_evidence = \"")
                    .and_then(|value| value.strip_suffix('"'))
            })
        })
        .unwrap_or_else(|| panic!("missing provider evidence for `{provider_family}`"))
}

#[cfg(target_os = "macos")]
fn evidence_field<'a>(evidence: &'a str, key: &str) -> &'a str {
    evidence
        .split(';')
        .filter_map(|field| field.split_once('='))
        .find_map(|(candidate, value)| (candidate == key).then_some(value))
        .unwrap_or_else(|| panic!("missing evidence field `{key}`"))
}
