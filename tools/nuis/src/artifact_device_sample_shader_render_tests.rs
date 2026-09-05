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
            "schema = \"{TABLE_CONTRACT}\"\nsource_yir_version = \"0.1\"\nsource_fnv1a64 = \"{source_hash}\"\nlowering_target = \"metal.apple-silicon-gpu\"\nasset_count = 1\npass_count = 2\n\n[[asset]]\ncontract = \"{ASSET_CONTRACT}\"\nasset_id = \"{asset_id}\"\nfile_name = \"{path}\"\nformat = \"metal-source\"\ntarget = \"metal.apple-silicon-gpu\"\nentries = [\"vs_main\", \"fs_main\"]\nbyte_length = {}\ncontent_hash = \"{content_hash}\"\n\n[[pass]]\ncontract = \"{PASS_CONTRACT}\"\npass_node = \"render.first\"\nmodule_node = \"shader.inline.first\"\nresult_node = \"draw.first\"\nresult_resource = \"shader0\"\nasset_id = \"{asset_id}\"\ntarget_format = \"rgba8_unorm\"\nwidth = 32\nheight = 24\n\n[[pass]]\ncontract = \"{PASS_CONTRACT}\"\npass_node = \"render.second\"\nmodule_node = \"shader.inline.first\"\nresult_node = \"draw.second\"\nresult_resource = \"shader0\"\nasset_id = \"{asset_id}\"\ntarget_format = \"rgba8_unorm\"\nwidth = 16\nheight = 8\n",
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
    assert!(evidence.contains(
        "provider_request_0_runtime_result_binding_contract=nuis-provider-runtime-result-binding-v1"
    ));
    assert!(evidence.contains("provider_request_0_runtime_result_node=draw.first"));
    assert!(evidence.contains(&format!(
        "provider_request_0_runtime_result_source_yir_fnv1a64={}",
        fnv1a64_hex(b"yir 0.1\n")
    )));
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
    assert!(nsdb::validate_provider_request_evidence(evidence));

    let prepared =
        crate::artifact_runtime_provider_results::prepare_runtime_provider_results(&output_dir)
            .expect("runtime result preparation")
            .expect("NS Nova runtime result target");
    assert_eq!(prepared.target_count, 1);
    assert!(
        !prepared.stream_path.exists(),
        "preparation must not execute the provider"
    );
    assert!(!output_dir
        .join("nuis.runtime.provider-result.0000.bin")
        .exists());
    let mut child = std::process::Command::new(std::env::current_exe().unwrap());
    child
        .args([
            "--exact",
            "artifact_device_sample_shader_render::tests::runtime_ipc_lifecycle_child",
            "--nocapture",
        ])
        .env("NUIS_TEST_RUNTIME_IPC_YIR", &prepared.source_yir_path)
        .env("NUIS_TEST_RUNTIME_IPC_PPM", output_dir.join("live.ppm"));
    let (status, invocation_count) = prepared
        .run_command(&mut child)
        .expect("child-driven lifecycle IPC");
    assert!(status.success());
    assert_eq!(invocation_count, 3);
    let runtime_stream_path = prepared.stream_path;
    let runtime_stream = fs::read_to_string(&runtime_stream_path)
        .expect("Nsdb should persist the provider-neutral runtime result stream");
    assert!(runtime_stream.contains("schema = \"nuis-provider-runtime-result-stream-v2\""));
    assert!(runtime_stream.contains("vertex_count:u64:2"));
    assert!(runtime_stream.contains("vertex_count:u64:3"));
    assert!(runtime_stream.contains("instance_count:u64:1"));
    assert!(runtime_stream.contains("instance_count:u64:2"));
    assert!(runtime_stream.contains("instance_count:u64:3"));
    let observed_arguments = runtime_stream
        .lines()
        .filter_map(|line| line.strip_prefix("dispatch_arguments = \""))
        .map(|value| value.trim_end_matches('"'))
        .collect::<Vec<_>>();
    assert_eq!(
        observed_arguments,
        [(3, 1), (2, 2), (3, 3)].map(|(vertices, instances)| format!(
            "nuis-shader-unbound-draw-v1|height:u64:120|instance_count:u64:{instances}|vertex_count:u64:{vertices}|width:u64:160"
        ))
    );
    assert!(runtime_stream.contains("frame_count = 3"));
    assert!(runtime_stream.contains("module = \"shader\""));
    assert!(runtime_stream.contains("instruction = \"draw_instanced\""));
    assert!(!runtime_stream.contains(&output_dir.display().to_string()));

    let yir_path = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("yir"))
        .expect("compiled project YIR");
    let yir_source = fs::read_to_string(yir_path).unwrap();
    let trace = yir_runtime_host::execute_module_source_with_provider_result_stream(
        &yir_source,
        &runtime_stream_path,
    )
    .expect("NS Nova lifecycle should consume all three physical provider frames");
    let draw_events = trace
        .events
        .iter()
        .filter(|event| event.starts_with("effect shader.draw_instanced "))
        .collect::<Vec<_>>();
    assert_eq!(draw_events.len(), 3);
    assert!(draw_events
        .iter()
        .all(|event| event.ends_with("frame[160x120; rgba8_bytes=76800]")));
    let ppm = yir_runtime_host::render_trace_to_ppm_bytes(&trace, 1).unwrap();
    let ppm_header = b"P6\n160 120\n255\n";
    assert!(ppm.starts_with(ppm_header));
    let last_rgba = fs::read(output_dir.join("nuis.runtime.provider-result.0002.bin")).unwrap();
    assert_eq!(last_rgba.len(), 160 * 120 * 4);
    let expected_rgb = last_rgba
        .chunks_exact(4)
        .flat_map(|pixel| pixel[..3].iter().copied())
        .collect::<Vec<_>>();
    assert_eq!(&ppm[ppm_header.len()..], expected_rgb.as_slice());
    assert_eq!(fs::read(output_dir.join("live.ppm")).unwrap(), ppm);

    let first_rgba = fs::read(output_dir.join("nuis.runtime.provider-result.0000.bin")).unwrap();
    let second_rgba = fs::read(output_dir.join("nuis.runtime.provider-result.0001.bin")).unwrap();
    assert!(
        first_rgba != second_rgba,
        "runtime vertex count must change actual device coverage"
    );
    assert!(
        second_rgba
            .chunks_exact(4)
            .all(|pixel| pixel == [0, 0, 0, 255]),
        "two vertices must leave the cleared target, not render fixed geometry"
    );
    assert!(
        first_rgba == last_rgba,
        "the same three-vertex coverage is deterministic"
    );

    fs::write(
        &runtime_stream_path,
        runtime_stream.replacen("vertex_count:u64:3", "vertex_count:u64:4", 1),
    )
    .unwrap();
    let error = yir_runtime_host::execute_module_source_with_provider_result_stream(
        &yir_source,
        &runtime_stream_path,
    )
    .expect_err("persisted dispatch input drift must break stream identity");
    assert!(error.contains("stream identity mismatch"));
    fs::write(&runtime_stream_path, &runtime_stream).unwrap();

    let first_frame = output_dir.join("nuis.runtime.provider-result.0000.bin");
    let mut drifted_frame = fs::read(&first_frame).unwrap();
    drifted_frame[0] ^= 0xff;
    fs::write(&first_frame, drifted_frame).unwrap();
    let error = yir_runtime_host::render_module_to_ppm_bytes_with_provider_result_stream(
        &yir_source,
        1,
        &runtime_stream_path,
    )
    .expect_err("runtime result payload drift must fail before presentation");
    assert!(error.contains("payload") && error.contains("identity mismatch"));
    let payload = fs::read_to_string(
        output_dir.join("nuis.nsdb.provider-output.metal-apple-silicon-gpu.toml"),
    )
    .unwrap();
    assert!(payload.contains("nuis-provider-worker-process-adapter-v5"));
    assert!(payload.contains("metal.command-buffer.completed"));
    assert_eq!(
        toml_string_field(&payload, "runtime_dispatch_trigger"),
        "child-yir-node-ipc"
    );
    assert_eq!(
        toml_string_field(&payload, "native_output_kind"),
        "provider-frame-rgba8"
    );
    assert_eq!(
        toml_string_field(&payload, "native_output_execution_status"),
        "metal-command-buffer-completed"
    );
    assert_eq!(toml_string_field(&payload, "native_output_count"), "1");
    assert_eq!(
        toml_string_field(&payload, "runtime_dispatch_session_contract"),
        "nuis-provider-runtime-dispatch-session-v1"
    );
    assert_eq!(
        toml_string_field(&payload, "runtime_dispatch_session_status"),
        "verified"
    );
    assert_eq!(
        toml_string_field(&payload, "runtime_dispatch_session_invocation_count"),
        "3"
    );
    assert_eq!(
        toml_string_field(&payload, "runtime_dispatch_session_worker_count"),
        "1"
    );
    assert_eq!(
        toml_string_field(&payload, "runtime_dispatch_session_lease_count"),
        "1"
    );
    assert_eq!(
        toml_string_field(&payload, "runtime_dispatch_session_request_sequences"),
        "0,1,2"
    );
    assert_eq!(
        toml_string_field(&payload, "runtime_dispatch_session_adapter_cache_statuses"),
        "compiled,hit,hit"
    );
    assert!(
        toml_string_field(&payload, "runtime_dispatch_session_evidence_hash").starts_with("0x")
    );
    let materialized = nsdb::materialize_provider_samples(&output_dir, Some(PROVIDER_FAMILY))
        .expect("runtime-session provider output should remain materializable");
    assert_eq!(materialized.materialized_record_count, 1);

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
#[test]
fn runtime_ipc_lifecycle_child() {
    let Some(path) = std::env::var_os("NUIS_TEST_RUNTIME_IPC_YIR") else {
        return;
    };
    let source = fs::read_to_string(path).unwrap();
    assert!(std::env::var_os(yir_runtime_host::PROVIDER_DISPATCH_SOCKET_ENV).is_some());
    assert!(std::env::var_os(yir_runtime_host::PROVIDER_RESULT_STREAM_ENV).is_none());
    let ppm = yir_runtime_host::render_module_to_ppm_bytes(&source, 1)
        .expect("live Nuis lifecycle must request its own device frames");
    fs::write(std::env::var_os("NUIS_TEST_RUNTIME_IPC_PPM").unwrap(), ppm).unwrap();
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

#[cfg(target_os = "macos")]
fn toml_string_field<'a>(source: &'a str, key: &str) -> &'a str {
    let prefix = format!("{key} = \"");
    source
        .lines()
        .find_map(|line| {
            line.strip_prefix(&prefix)
                .and_then(|value| value.strip_suffix('"'))
        })
        .unwrap_or_else(|| panic!("missing TOML string field `{key}`"))
}
