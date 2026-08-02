use super::*;

const CONTAINER_CAPSULE_END_MARKER: &str = "\n# nuis-nsld-container-end-v1\n";
const NATIVE_ENTRY_ASSET: [u8; 16] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0x00, 0x00, 0x80, 0xd2, 0xc0, 0x03, 0x5f, 0xd6,
];
const NATIVE_ENTRY_IMAGE: [u8; 16] = [
    0x48, 0, 0, 0, 0, 0, 0, 0, 0x00, 0x00, 0x80, 0xd2, 0xc0, 0x03, 0x5f, 0xd6,
];

fn base_nsb_payload() -> &'static [u8] {
    br#"schema = "nuis-nsld-container-v1"
schema_version = 1
container_kind = "deterministic-hetero-container"
producer = "nsld"
producer_phase = "beta-0.0.1"
ready = true
container_magic = "NUISNSLD"
container_version = 1
metadata_table_hash = "0x1111111111111111"
container_section_table_hash = "0x2222222222222222"
container_hash = "0xaaaaaaaaaaaaaaaa"
section_count = 1
compatibility_domain_count = 0
external_import_count = 0
backend_artifact_payload_count = 1
backend_artifact_payload_table_hash = "0x7777777777777777"
loader_readiness = "host-assisted"
loader_blockers = []
loader_entry_kind = "lifecycle-bootstrap"
loader_entry_abi_contract = "nuis-runtime-lifecycle-entry-i64-v1"
loader_entry_machine_arch = "__HOST_ARCH__"
loader_entry_symbol = "main"
loader_entry_section_id = "sec0000.nuis-native-entry-code"
loader_symbol_count = 3
loader_symbol_table_hash = "0x3333333333333333"
relocation_count = 1
relocation_table_hash = "0x4444444444444444"
compatibility_domain_table_hash = "0x5555555555555555"
external_import_table_hash = "0x6666666666666666"
payload_size_bytes = 16
payload_hash = "0xbbbbbbbbbbbbbbbb"
payload_path = "nuis.nsld.container.payload"
blockers = []

[[backend_artifact_payload]]
payload_id = "backend-artifact:kernel:aarch64:apple-silicon-cpu"
domain_family = "kernel"
backend_family = "aarch64"
target_device = "apple-silicon-cpu"
payload_format = "nuis-kernel-payload-v1"
payload_path = "kernel.payload.bin"
role_status = "ready"

[[loader_symbol]]
symbol_id = "sym0000.loader-entry"
symbol_kind = "lifecycle-bootstrap"
symbol_name = "main"
lifecycle_hook = "on_lifecycle_bootstrap"
section_id = "sec0000.nuis-native-entry-code"
offset = 8
size_bytes = 8
payload_hash = "0xeeeeeeeeeeeeeeee"

[[relocation]]
relocation_id = "rel0000.lifecycle-entry"
relocation_kind = "lifecycle-entry-binding"
source_section_id = "sec0000.nuis-native-entry-code"
source_offset = 0
target_symbol_id = "sym0000.loader-entry"
addend = 0

[[section]]
order_index = 0
section_id = "sec0000.nuis-native-entry-code"
section_kind = "nuis-native-entry-code"
source_path = "nuis.nsld.native-entry.bin"
source_hash = "0xcccccccccccccccc"
payload_hash = "0xdddddddddddddddd"
required = true
offset = 0
size_bytes = 16
# nuis-nsld-container-end-v1
"#
}

fn nsb_payload() -> Vec<u8> {
    image_payload_from_capsule(&container_capsule())
}

fn container_capsule() -> Vec<u8> {
    let bindings = runtime_binding_toml();
    let table_hash = runtime_binding_table_hash();
    let source = std::str::from_utf8(base_nsb_payload()).expect("fixture is utf-8");
    source
        .replace(
            "metadata_table_hash = \"0x1111111111111111\"\n",
            &format!(
                "metadata_table_hash = \"0x1111111111111111\"\nmetadata_binding_count = \
                 2\nmetadata_binding_table_hash = \"{table_hash}\"\n"
            ),
        )
        .replace(
            "\n[[loader_symbol]]\n",
            &format!("{bindings}\n[[loader_symbol]]\n"),
        )
        .replace("0xbbbbbbbbbbbbbbbb", &fnv1a64_hex(&NATIVE_ENTRY_ASSET))
        .replace("0xcccccccccccccccc", &fnv1a64_hex(&NATIVE_ENTRY_ASSET))
        .replace("0xdddddddddddddddd", &fnv1a64_hex(&NATIVE_ENTRY_ASSET))
        .replace("0xeeeeeeeeeeeeeeee", &fnv1a64_hex(&NATIVE_ENTRY_ASSET[8..]))
        .replace(
            "__HOST_ARCH__",
            nuis_runtime::native_host_machine_arch().expect("supported host runner test arch"),
        )
        .into_bytes()
}

fn image_payload_from_capsule(capsule: &[u8]) -> Vec<u8> {
    let aligned = capsule.len().next_multiple_of(16);
    let mut payload = capsule.to_vec();
    payload.resize(aligned, 0);
    payload.extend_from_slice(&NATIVE_ENTRY_IMAGE);
    payload
}

fn runtime_binding_toml() -> String {
    "\n[[metadata_binding]]\nbinding_id = \"runtime.clock-root\"\ncontract = \
     \"nuis-clock-protocol-v1\"\nvalue_count = 3\nvalue_hash = \
     \"0x8888888888888888\"\nvalidation_status = \"verified\"\nrequired = \
     true\n\n[[metadata_binding]]\nbinding_id = \"runtime.glm-root\"\ncontract = \
     \"nuis-yir-glm-binding-v1\"\nvalue_count = 1\nvalue_hash = \
     \"0x9999999999999999\"\nvalidation_status = \"verified\"\nrequired = true\n"
        .to_owned()
}

fn runtime_binding_table_hash() -> String {
    fnv1a64_hex(
        b"runtime.clock-root\tnuis-clock-protocol-v1\t3\t0x8888888888888888\tverified\ttrue\nruntime.glm-root\tnuis-yir-glm-binding-v1\t1\t0x9999999999999999\tverified\ttrue\n",
    )
}

fn nsb_bytes() -> Vec<u8> {
    nsb_bytes_from_payload(&nsb_payload())
}

fn nsb_bytes_from_payload(payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0u8; IMAGE_HEADER_SIZE + payload.len()];
    bytes[0..8].copy_from_slice(IMAGE_MAGIC);
    bytes[8..12].copy_from_slice(&IMAGE_VERSION.to_le_bytes());
    bytes[12..16].copy_from_slice(&(IMAGE_HEADER_SIZE as u32).to_le_bytes());
    bytes[24..32].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes[32..40].copy_from_slice(&(IMAGE_HEADER_SIZE as u64).to_le_bytes());
    bytes[40..48].copy_from_slice(&0x1234u64.to_le_bytes());
    bytes[48..56].copy_from_slice(&0x5678u64.to_le_bytes());
    bytes[IMAGE_HEADER_SIZE..].copy_from_slice(payload);
    bytes
}

fn mutate_nsb_capsule(bytes: &mut Vec<u8>, mutate: impl FnOnce(String) -> String) {
    let region = &bytes[IMAGE_HEADER_SIZE..];
    let capsule_end = region
        .windows(CONTAINER_CAPSULE_END_MARKER.len())
        .position(|window| window == CONTAINER_CAPSULE_END_MARKER.as_bytes())
        .map(|offset| offset + CONTAINER_CAPSULE_END_MARKER.len())
        .expect("fixture container capsule has an end marker");
    let source = String::from_utf8(region[..capsule_end].to_vec()).unwrap();
    let payload = image_payload_from_capsule(mutate(source).as_bytes());
    bytes.truncate(IMAGE_HEADER_SIZE);
    bytes.extend_from_slice(&payload);
    bytes[24..32].copy_from_slice(&(payload.len() as u64).to_le_bytes());
}

fn nsb_payload_with_selected_binding(value_hash: &str, table_hash: &str) -> Vec<u8> {
    let capsule = container_capsule();
    let source = std::str::from_utf8(&capsule).expect("fixture is utf-8");
    let source = source.replace(
        &format!(
            "metadata_binding_count = 2\nmetadata_binding_table_hash = \"{}\"\n",
            runtime_binding_table_hash()
        ),
        &format!("metadata_binding_count = 3\nmetadata_binding_table_hash = \"{table_hash}\"\n"),
    );
    let capsule = source
        .replace(
            "\n[[loader_symbol]]\n",
            &format!(
                "\n[[metadata_binding]]\nbinding_id = \
                 \"identity.selected-provider-bundle-set\"\ncontract = \
                 \"nuis-selected-provider-bundle-set-v1\"\nvalue_count = 2\nvalue_hash = \
                 \"{value_hash}\"\nvalidation_status = \"verified\"\nrequired = \
                 true\n\n[[loader_symbol]]\n"
            ),
        )
        .into_bytes();
    image_payload_from_capsule(&capsule)
}

fn manifest_source(nsb_hash: &str, nsb_size: usize) -> String {
    format!(
        "schema = \"{MANIFEST_SCHEMA}\"\nready = true\nexecution_handoff_contract = \"{HANDOFF_CONTRACT}\"\nexecution_handoff_ready = true\nnsb_path = \"nuis-app.nsb\"\nnsb_hash = \"{nsb_hash}\"\nnsb_size_bytes = {nsb_size}\nimage_header_required = true\nimage_header_valid = true\nscheduler_entry = \"nuis.scheduler.loop.v1\"\nentry_lifecycle_hook = \"on_process_start\"\n"
    )
}

#[test]
fn validates_ready_launcher_handoff() {
    let bytes = nsb_bytes();
    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!("nuis-host-runner-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(report.ready);
    assert!(report.would_enter_lifecycle_hook);
    assert_eq!(
        report.runtime_bootstrap_contract,
        "nuis-runtime-lifecycle-bootstrap-plan-v1"
    );
    assert_eq!(report.runtime_bootstrap_status, "ready");
    assert_eq!(
        report.runtime_bootstrap_identity_contract,
        "nuis-runtime-lifecycle-bootstrap-plan-identity-v1"
    );
    assert!(report.runtime_bootstrap_identity_hash.starts_with("0x"));
    assert_eq!(report.runtime_bootstrap_stage_count, 10);
    assert_eq!(report.runtime_bootstrap_mapped_section_count, 1);
    assert_eq!(report.runtime_bootstrap_applied_relocation_count, 1);
    assert_eq!(
        report.runtime_bootstrap_execution_contract,
        "nuis-runtime-lifecycle-bootstrap-execution-v1"
    );
    assert_eq!(report.runtime_bootstrap_execution_status, "transfer-ready");
    assert!(report
        .runtime_bootstrap_execution_identity_hash
        .starts_with("0x"));
    assert_eq!(report.runtime_bootstrap_activated_service_count, 2);
    assert!(report.runtime_bootstrap_blockers.is_empty());
    assert_eq!(report.native_entry_handoff.status, "prepared");
    assert!(report.native_entry_handoff.ready);
    assert_eq!(
        report.native_entry_handoff.section_hash_status,
        "verified-after-relocation-normalization"
    );
    assert_eq!(report.native_entry_handoff.code_hash_status, "verified");
    assert_eq!(
        report.native_entry_handoff.target_machine_arch.as_deref(),
        nuis_runtime::native_host_machine_arch()
    );
    assert_eq!(
        report.native_entry_handoff.host_machine_arch.as_deref(),
        nuis_runtime::native_host_machine_arch()
    );
    assert_eq!(
        report.native_entry_handoff.machine_arch_status,
        "verified-host-match"
    );
    assert_eq!(report.native_entry_handoff.preparation_status, "ready");
    assert!(report.native_entry_handoff.preparation_ready);
    assert_eq!(report.native_entry_handoff.mapping_size_bytes, 8);
    assert_eq!(
        report.native_entry_handoff.protection_status,
        "sealed-read-execute"
    );
    assert_eq!(report.native_entry_handoff.invocation_status, "not-invoked");
    assert!(report.native_entry_handoff.blockers.is_empty());
    assert!(report
        .launch_steps
        .contains(&"map-payload-region".to_owned()));
    assert!(report.launch_steps.contains(
        &"bind-runtime-service:runtime.clock-root@nuis-clock-protocol-v1#0x8888888888888888"
            .to_owned()
    ));
    assert!(report.launch_steps.contains(&format!(
        "map-section:sec0000.nuis-native-entry-code@0+16#{}",
        fnv1a64_hex(&NATIVE_ENTRY_ASSET)
    )));
    assert!(report.launch_steps.contains(
        &"apply-relocation:rel0000.lifecycle-entry:lifecycle-entry-binding@sec0000.nuis-native-entry-code+0->sym0000.loader-entry+0".to_owned()
    ));
    assert!(report.launch_steps.contains(
        &format!(
            "bind-loader-entry:main@sec0000.nuis-native-entry-code#nuis-runtime-lifecycle-entry-i64-v1@{}",
            nuis_runtime::native_host_machine_arch().expect("supported host runner test arch")
        )
    ));
    assert!(report.launch_steps.contains(
        &"bind-runtime-service:runtime.glm-root@nuis-yir-glm-binding-v1#0x9999999999999999"
            .to_owned()
    ));
    assert!(report
        .launch_steps
        .contains(&"enter-lifecycle-hook:on_process_start".to_owned()));
    assert!(report
        .launch_steps
        .contains(&"enter-nuis-bootstrap:on_lifecycle_bootstrap:main".to_owned()));
    assert!(report
        .launch_steps
        .contains(&"activate-scheduler:nuis.scheduler.loop.v1".to_owned()));
    assert!(report.launch_steps.contains(
        &"prepare-native-entry:sec0000.nuis-native-entry-code:sealed-read-execute".to_owned()
    ));
    assert!(report
        .launch_steps
        .contains(&"native-entry-invocation:not-invoked".to_owned()));
    let json = report::render_json_report(&report);
    assert!(json
        .contains("\"runtime_bootstrap_contract\":\"nuis-runtime-lifecycle-bootstrap-plan-v1\""));
    assert!(json.contains("\"runtime_bootstrap_status\":\"ready\""));
    assert!(json.contains("\"runtime_bootstrap_identity_contract\":\"nuis-runtime-lifecycle-bootstrap-plan-identity-v1\""));
    assert!(json.contains("\"runtime_bootstrap_identity_hash\":\"0x"));
    assert!(json.contains("\"runtime_bootstrap_stage_count\":10"));
    assert!(json.contains("\"runtime_bootstrap_mapped_section_count\":1"));
    assert!(json.contains("\"runtime_bootstrap_applied_relocation_count\":1"));
    assert!(json.contains(
        "\"runtime_bootstrap_execution_contract\":\"nuis-runtime-lifecycle-bootstrap-execution-v1\""
    ));
    assert!(json.contains("\"runtime_bootstrap_execution_identity_hash\":\"0x"));
    assert!(json.contains("\"runtime_bootstrap_execution_status\":\"transfer-ready\""));
    assert!(json.contains("\"runtime_bootstrap_activated_service_count\":2"));
    assert!(json.contains("\"native_entry_handoff\":{\"protocol\":\"nuis-host-native-entry-handoff-v1\",\"status\":\"prepared\",\"ready\":true"));
    assert!(json.contains("\"section_hash_status\":\"verified-after-relocation-normalization\""));
    assert!(json.contains(&format!(
        "\"target_machine_arch\":\"{}\"",
        nuis_runtime::native_host_machine_arch().expect("supported host runner test arch")
    )));
    assert!(json.contains("\"machine_arch_status\":\"verified-host-match\""));
    assert!(json.contains("\"preparation_status\":\"ready\""));
    assert!(json.contains("\"invocation_status\":\"not-invoked\""));
    assert_eq!(
        report.container_loader_clock_root_contract.as_deref(),
        Some("nuis-clock-protocol-v1")
    );
    assert_eq!(report.container_loader_clock_root_count, Some(3));
    assert_eq!(
        report.container_loader_glm_root_contract.as_deref(),
        Some("nuis-yir-glm-binding-v1")
    );
    assert_eq!(report.container_loader_glm_root_count, Some(1));
    assert!(json.contains("\"container_loader_clock_root_status\":\"verified\""));
    assert!(json.contains("\"container_loader_glm_root_status\":\"verified\""));
    assert_eq!(report.nsb_payload_offset, Some(IMAGE_HEADER_SIZE));
    assert_eq!(report.nsb_payload_span, Some(nsb_payload().len()));
    assert!(report.nsb_payload_region_mapped);
    assert_eq!(report.nsb_payload_region_bytes, Some(nsb_payload().len()));
    let expected_payload_region_hash = fnv1a64_hex(&nsb_payload());
    assert_eq!(
        report.nsb_payload_region_hash.as_deref(),
        Some(expected_payload_region_hash.as_str())
    );
    assert_eq!(report.nsb_payload_scan_status, "scanned");
    assert_eq!(report.nsb_payload_scan_kind, "nsld-container-toml");
    assert!(report
        .nsb_payload_prefix_text
        .as_deref()
        .is_some_and(|prefix| prefix.contains("nuis-nsld-container-v1")));
    assert!(report
        .nsb_payload_prefix_hex
        .as_deref()
        .is_some_and(|prefix| prefix.starts_with("736368656d6120")));
    assert_eq!(report.container_loader_status, "parsed");
    assert_eq!(report.container_schema.as_deref(), Some(CONTAINER_SCHEMA));
    assert_eq!(
        report.container_schema_version,
        Some(CONTAINER_SCHEMA_VERSION)
    );
    assert_eq!(report.container_kind.as_deref(), Some(CONTAINER_KIND));
    assert_eq!(
        report.container_producer.as_deref(),
        Some(CONTAINER_PRODUCER)
    );
    assert_eq!(
        report.container_producer_phase.as_deref(),
        Some("beta-0.0.1")
    );
    assert_eq!(report.container_ready, Some(true));
    assert!(report.container_blockers.is_empty());
    assert_eq!(report.container_magic.as_deref(), Some(CONTAINER_MAGIC));
    assert_eq!(report.container_version, Some(CONTAINER_VERSION));
    assert_eq!(
        report.container_metadata_table_hash.as_deref(),
        Some("0x1111111111111111")
    );
    assert_eq!(
        report.container_section_table_hash.as_deref(),
        Some("0x2222222222222222")
    );
    assert_eq!(report.container_hash.as_deref(), Some("0xaaaaaaaaaaaaaaaa"));
    assert_eq!(report.container_section_count, Some(1));
    assert_eq!(report.container_section_parsed_count, 1);
    assert_eq!(
        report.container_first_section_id.as_deref(),
        Some("sec0000.nuis-native-entry-code")
    );
    assert_eq!(
        report.container_first_section_kind.as_deref(),
        Some("nuis-native-entry-code")
    );
    assert!(report.container_entry_section_found);
    assert_eq!(report.container_payload_size_bytes, Some(16));
    assert_eq!(
        report.container_payload_hash.as_deref(),
        Some(fnv1a64_hex(&NATIVE_ENTRY_ASSET).as_str())
    );
    assert_eq!(
        report.container_payload_path.as_deref(),
        Some("nuis.nsld.container.payload")
    );
    assert_eq!(
        report.container_loader_readiness.as_deref(),
        Some("host-assisted")
    );
    assert!(report.container_loader_blockers.is_empty());
    assert_eq!(
        report.container_loader_entry_kind.as_deref(),
        Some("lifecycle-bootstrap")
    );
    assert_eq!(
        report.container_loader_entry_symbol.as_deref(),
        Some("main")
    );
    assert_eq!(
        report.container_loader_entry_section_id.as_deref(),
        Some("sec0000.nuis-native-entry-code")
    );
    assert_eq!(report.container_loader_symbol_count, Some(3));
    assert_eq!(
        report.loader_symbol_table_hash.as_deref(),
        Some("0x3333333333333333")
    );
    assert_eq!(report.container_loader_symbol_status, "parsed");
    assert_eq!(
        report.container_loader_symbol_id.as_deref(),
        Some("sym0000.loader-entry")
    );
    assert_eq!(
        report.container_loader_symbol_kind.as_deref(),
        Some("lifecycle-bootstrap")
    );
    assert_eq!(report.container_loader_symbol_name.as_deref(), Some("main"));
    assert_eq!(
        report.container_loader_symbol_lifecycle_hook.as_deref(),
        Some("on_lifecycle_bootstrap")
    );
    assert_eq!(
        report.container_loader_symbol_section_id.as_deref(),
        Some("sec0000.nuis-native-entry-code")
    );
    assert_eq!(report.container_relocation_count, Some(1));
    assert_eq!(report.container_relocation_parsed_count, 1);
    assert_eq!(
        report.container_first_relocation_kind.as_deref(),
        Some("lifecycle-entry-binding")
    );
    assert_eq!(
        report
            .container_first_relocation_source_section_id
            .as_deref(),
        Some("sec0000.nuis-native-entry-code")
    );
    assert_eq!(
        report
            .container_first_relocation_target_symbol_id
            .as_deref(),
        Some("sym0000.loader-entry")
    );
    assert!(report.container_first_relocation_targets_loader_symbol);
    assert!(report.container_first_relocation_source_matches_loader_symbol);
    assert_eq!(
        report.relocation_table_hash.as_deref(),
        Some("0x4444444444444444")
    );
    assert_eq!(report.compatibility_domain_count, Some(0));
    assert_eq!(report.compatibility_domain_parsed_count, 0);
    assert_eq!(report.compatibility_domain_required_count, 0);
    assert_eq!(
        report.compatibility_domain_table_hash.as_deref(),
        Some("0x5555555555555555")
    );
    assert_eq!(report.external_import_count, Some(0));
    assert_eq!(report.external_import_parsed_count, 0);
    assert_eq!(
        report.external_import_table_hash.as_deref(),
        Some("0x6666666666666666")
    );
    assert!(report.external_import_required_imports.is_empty());
    assert_eq!(report.backend_artifact_payload_count, Some(1));
    assert_eq!(report.backend_artifact_payload_parsed_count, 1);
    assert_eq!(report.backend_artifact_payload_ready_count, 1);
    assert_eq!(
        report.backend_artifact_payload_first_id.as_deref(),
        Some("backend-artifact:kernel:aarch64:apple-silicon-cpu")
    );
    assert_eq!(
        report.backend_artifact_payload_first_kind.as_deref(),
        Some("nustar-backend-artifact:kernel:aarch64:apple-silicon-cpu")
    );
    assert_eq!(
        report.backend_artifact_payload_first_role_status.as_deref(),
        Some("ready")
    );
    assert_eq!(
        report.backend_artifact_payload_table_hash.as_deref(),
        Some("0x7777777777777777")
    );
    assert_eq!(report.container_loader_handoff_status, "ready");
    assert!(report.container_loader_handoff_ready);
    assert!(report.container_loader_handoff_blockers.is_empty());
    assert_eq!(
        report.nsb_layout_hash.as_deref(),
        Some("0x0000000000001234")
    );
    assert_eq!(
        report.nsb_byte_map_hash.as_deref(),
        Some("0x0000000000005678")
    );
    assert!(report.blockers.is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn blocks_lifecycle_handoff_after_inner_binding_tamper_even_with_new_outer_hash() {
    let original_value_hash = "fnv1a64:1234567890abcdef";
    let material = format!(
        "identity.selected-provider-bundle-set\tnuis-selected-provider-bundle-set-v1\t2\t\
         {original_value_hash}\tverified\ttrue\n"
    );
    let table_hash = fnv1a64_hex(material.as_bytes());
    let tampered_payload =
        nsb_payload_with_selected_binding("fnv1a64:fedcba0987654321", &table_hash);
    let bytes = nsb_bytes_from_payload(&tampered_payload);
    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-binding-tamper-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(report.nsb_hash_matches);
    assert_eq!(
        report.container_loader_metadata_binding_validation_status,
        "mismatch"
    );
    assert!(!report.container_loader_handoff_ready);
    assert!(!report.ready);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker.contains("metadata-binding-table-hash-mismatch")));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn blocks_hash_mismatch() {
    let bytes = nsb_bytes();
    let manifest = parse_launcher_manifest(&manifest_source("0x0000000000000000", bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!("nuis-host-runner-hash-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(!report.ready);
    assert!(report.blockers.contains(&"nsb:hash-mismatch".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn blocks_container_handoff_when_schema_is_unsupported() {
    let mut bytes = nsb_bytes();
    mutate_nsb_capsule(&mut bytes, |source| {
        source.replace(
            "schema = \"nuis-nsld-container-v1\"",
            "schema = \"nuis-foreign-container-v1\"",
        )
    });

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-container-schema-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(!report.ready);
    assert_eq!(
        report.container_schema.as_deref(),
        Some("nuis-foreign-container-v1")
    );
    assert_eq!(report.container_loader_handoff_status, "blocked");
    assert!(report
        .container_loader_handoff_blockers
        .contains(&"container:schema-unsupported".to_owned()));
    assert!(report
        .blockers
        .contains(&"container:schema-unsupported".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn blocks_container_loader_handoff_when_entry_section_is_missing_from_table() {
    let mut bytes = nsb_bytes();
    mutate_nsb_capsule(&mut bytes, |source| {
        source.replace(
            "loader_entry_section_id = \"sec0000.nuis-native-entry-code\"",
            "loader_entry_section_id = \"sec9999.missing\"",
        )
    });

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-entry-section-table-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(!report.ready);
    assert!(!report.container_entry_section_found);
    assert_eq!(report.container_loader_handoff_status, "blocked");
    assert!(report
        .container_loader_handoff_blockers
        .contains(&"container-loader:entry-section-not-found".to_owned()));
    assert!(report
        .blockers
        .contains(&"container-loader:entry-section-not-found".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn blocks_container_loader_handoff_when_first_relocation_targets_wrong_symbol() {
    let mut bytes = nsb_bytes();
    mutate_nsb_capsule(&mut bytes, |source| {
        source.replace(
            "target_symbol_id = \"sym0000.loader-entry\"",
            "target_symbol_id = \"sym9999.missing-entry\"",
        )
    });

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-relocation-target-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(!report.ready);
    assert!(!report.container_first_relocation_targets_loader_symbol);
    assert_eq!(report.container_loader_handoff_status, "blocked");
    assert!(report
        .container_loader_handoff_blockers
        .contains(&"container-loader:first-relocation-target-mismatch".to_owned()));
    assert!(report
        .blockers
        .contains(&"container-loader:first-relocation-target-mismatch".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn allows_host_assisted_container_handoff_when_required_external_import_is_declared() {
    let mut bytes = nsb_bytes();
    mutate_nsb_capsule(&mut bytes, |source| {
        source
            .replace("external_import_count = 0", "external_import_count = 1")
            .replace(
                CONTAINER_CAPSULE_END_MARKER,
                "\n[[external_import]]\nimport_id = \"imp0000.final-stage-driver\"\nimport_kind = \"final-stage-driver\"\nimport_name = \"cc\"\nprovider = \"host-toolchain\"\nrequired = true\n\n# nuis-nsld-container-end-v1\n",
            )
    });

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-external-import-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(report.ready);
    assert_eq!(report.external_import_count, Some(1));
    assert_eq!(report.external_import_parsed_count, 1);
    assert_eq!(
        report.external_import_required_imports,
        vec!["final-stage-driver:cc".to_owned()]
    );
    assert_eq!(report.container_loader_handoff_status, "ready");
    assert!(report.container_loader_handoff_ready);
    assert!(report.container_loader_handoff_blockers.is_empty());
    assert!(report.blockers.is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn blocks_self_contained_container_handoff_when_required_external_import_is_declared() {
    let mut bytes = nsb_bytes();
    mutate_nsb_capsule(&mut bytes, |source| {
        source
            .replace(
                "loader_readiness = \"host-assisted\"",
                "loader_readiness = \"self-contained\"",
            )
            .replace("external_import_count = 0", "external_import_count = 1")
            .replace(
                CONTAINER_CAPSULE_END_MARKER,
                "\n[[external_import]]\nimport_id = \"imp0000.final-stage-driver\"\nimport_kind = \"final-stage-driver\"\nimport_name = \"cc\"\nprovider = \"host-toolchain\"\nrequired = true\n\n# nuis-nsld-container-end-v1\n",
            )
    });

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-self-contained-external-import-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(!report.ready);
    assert_eq!(report.runtime_bootstrap_status, "blocked");
    assert_eq!(report.runtime_bootstrap_stage_count, 0);
    assert!(report
        .runtime_bootstrap_blockers
        .contains(&"runtime-bootstrap:image-unverified".to_owned()));
    assert!(report
        .runtime_bootstrap_blockers
        .contains(&"runtime-bootstrap:container-handoff-blocked".to_owned()));
    assert_eq!(report.container_loader_handoff_status, "blocked");
    assert!(!report.container_loader_handoff_ready);
    assert!(report
        .container_loader_handoff_blockers
        .contains(&"container-external-import:required:final-stage-driver:cc".to_owned()));
    assert!(report
        .blockers
        .contains(&"container-external-import:required:final-stage-driver:cc".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn blocks_container_loader_handoff_when_loader_is_blocked() {
    let mut bytes = nsb_bytes();
    mutate_nsb_capsule(&mut bytes, |source| {
        source.replace(
            "loader_readiness = \"host-assisted\"",
            "loader_readiness = \"blocked\"",
        )
    });

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-loader-blocked-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(!report.ready);
    assert_eq!(report.container_loader_handoff_status, "blocked");
    assert!(!report.container_loader_handoff_ready);
    assert!(report
        .container_loader_handoff_blockers
        .contains(&"container-loader:readiness-blocked".to_owned()));
    assert!(report
        .blockers
        .contains(&"container-loader:readiness-blocked".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn blocks_container_loader_handoff_when_symbol_table_mismatches_entry() {
    let mut bytes = nsb_bytes();
    mutate_nsb_capsule(&mut bytes, |source| {
        source.replace("symbol_name = \"main\"", "symbol_name = \"boot\"")
    });

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-loader-symbol-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(!report.ready);
    assert_eq!(report.container_loader_symbol_status, "parsed");
    assert_eq!(report.container_loader_symbol_name.as_deref(), Some("boot"));
    assert_eq!(report.container_loader_handoff_status, "blocked");
    assert!(!report.container_loader_handoff_ready);
    assert!(report
        .container_loader_handoff_blockers
        .contains(&"container-loader:entry-symbol-mismatch".to_owned()));
    assert!(report
        .blockers
        .contains(&"container-loader:entry-symbol-mismatch".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn allows_host_assisted_container_loader_handoff_when_external_import_loader_blockers_are_declared()
{
    let mut bytes = nsb_bytes();
    mutate_nsb_capsule(&mut bytes, |source| {
        source.replace(
            "loader_blockers = []",
            "loader_blockers = [\"external-import:final-stage-driver:cc\"]",
        )
    });

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-loader-blocker-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let nsb_path = dir.join("nuis-app.nsb");
    fs::write(&nsb_path, bytes).expect("write nsb");

    let report = validate_handoff(
        &dir.join("nuis.nsld.final-executable-launcher.toml"),
        &nsb_path,
        Some(&dir),
        "nuis.scheduler.loop.v1",
        "on_process_start",
        &manifest,
    );

    assert!(report.ready);
    assert_eq!(
        report.container_loader_blockers,
        vec!["external-import:final-stage-driver:cc".to_owned()]
    );
    assert_eq!(report.container_loader_handoff_status, "ready");
    assert!(report.container_loader_handoff_ready);
    assert!(report.container_loader_handoff_blockers.is_empty());
    assert!(report.blockers.is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[path = "container_failure_tests.rs"]
mod container_failure_tests;
