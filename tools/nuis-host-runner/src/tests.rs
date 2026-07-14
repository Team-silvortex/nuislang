use super::*;

fn nsb_payload() -> &'static [u8] {
    b"schema = \"nuis-nsld-container-v1\"\nschema_version = 1\ncontainer_kind = \"deterministic-hetero-container\"\nproducer = \"nsld\"\nproducer_phase = \"alpha-0.10.0\"\nready = true\ncontainer_magic = \"NUISNSLD\"\ncontainer_version = 1\nmetadata_table_hash = \"0x1111111111111111\"\ncontainer_section_table_hash = \"0x2222222222222222\"\ncontainer_hash = \"0xaaaaaaaaaaaaaaaa\"\nsection_count = 1\ncompatibility_domain_count = 0\nexternal_import_count = 0\nloader_readiness = \"host-assisted\"\nloader_blockers = []\nloader_entry_kind = \"lifecycle-bootstrap\"\nloader_entry_symbol = \"main\"\nloader_entry_section_id = \"sec0000.compiled-artifact\"\nloader_symbol_count = 3\nloader_symbol_table_hash = \"0x3333333333333333\"\nrelocation_count = 1\nrelocation_table_hash = \"0x4444444444444444\"\ncompatibility_domain_table_hash = \"0x5555555555555555\"\nexternal_import_table_hash = \"0x6666666666666666\"\npayload_size_bytes = 128\npayload_hash = \"0xbbbbbbbbbbbbbbbb\"\npayload_path = \"nuis.nsld.container.payload\"\nblockers = []\n\n[[loader_symbol]]\nsymbol_id = \"sym0000.loader-entry\"\nsymbol_kind = \"lifecycle-bootstrap\"\nsymbol_name = \"main\"\nlifecycle_hook = \"on_lifecycle_bootstrap\"\nsection_id = \"sec0000.compiled-artifact\"\n\n[[relocation]]\nrelocation_id = \"rel0000.lifecycle-entry\"\nrelocation_kind = \"lifecycle-entry-binding\"\nsource_section_id = \"sec0000.compiled-artifact\"\nsource_offset = 0\ntarget_symbol_id = \"sym0000.loader-entry\"\naddend = 0\n\n[[section]]\norder_index = 0\nsection_id = \"sec0000.compiled-artifact\"\nsection_kind = \"compiled-artifact\"\nsource_path = \"main.nuis\"\nsource_hash = \"0xcccccccccccccccc\"\npayload_hash = \"0xdddddddddddddddd\"\nrequired = true\noffset = 0\nsize_bytes = 128\n"
}

fn nsb_bytes() -> Vec<u8> {
    let payload = nsb_payload();
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
    assert!(report
        .launch_steps
        .contains(&"map-payload-region".to_owned()));
    assert!(report
        .launch_steps
        .contains(&"enter-lifecycle-hook:on_process_start".to_owned()));
    assert_eq!(report.nsb_payload_offset, Some(IMAGE_HEADER_SIZE));
    assert_eq!(report.nsb_payload_span, Some(nsb_payload().len()));
    assert!(report.nsb_payload_region_mapped);
    assert_eq!(report.nsb_payload_region_bytes, Some(nsb_payload().len()));
    let expected_payload_region_hash = fnv1a64_hex(nsb_payload());
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
        Some("alpha-0.10.0")
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
        Some("sec0000.compiled-artifact")
    );
    assert_eq!(
        report.container_first_section_kind.as_deref(),
        Some("compiled-artifact")
    );
    assert!(report.container_entry_section_found);
    assert_eq!(report.container_payload_size_bytes, Some(128));
    assert_eq!(
        report.container_payload_hash.as_deref(),
        Some("0xbbbbbbbbbbbbbbbb")
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
        Some("sec0000.compiled-artifact")
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
        Some("sec0000.compiled-artifact")
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
        Some("sec0000.compiled-artifact")
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
    let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
    let tampered = source.replace(
        "schema = \"nuis-nsld-container-v1\"",
        "schema = \"nuis-foreign-container-v1\"",
    );
    bytes.truncate(IMAGE_HEADER_SIZE);
    bytes.extend_from_slice(tampered.as_bytes());
    bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

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
    let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
    let tampered = source.replace(
        "loader_entry_section_id = \"sec0000.compiled-artifact\"",
        "loader_entry_section_id = \"sec9999.missing\"",
    );
    bytes.truncate(IMAGE_HEADER_SIZE);
    bytes.extend_from_slice(tampered.as_bytes());
    bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

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
    let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
    let tampered = source.replace(
        "target_symbol_id = \"sym0000.loader-entry\"",
        "target_symbol_id = \"sym9999.missing-entry\"",
    );
    bytes.truncate(IMAGE_HEADER_SIZE);
    bytes.extend_from_slice(tampered.as_bytes());
    bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

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
    let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
    let tampered = source.replace("external_import_count = 0", "external_import_count = 1")
        + "\n[[external_import]]\nimport_id = \"imp0000.final-stage-driver\"\nimport_kind = \"final-stage-driver\"\nimport_name = \"cc\"\nprovider = \"host-toolchain\"\nrequired = true\n";
    bytes.truncate(IMAGE_HEADER_SIZE);
    bytes.extend_from_slice(tampered.as_bytes());
    bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

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
    let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
    let tampered = (source
        + "\n[[external_import]]\nimport_id = \"imp0000.final-stage-driver\"\nimport_kind = \"final-stage-driver\"\nimport_name = \"cc\"\nprovider = \"host-toolchain\"\nrequired = true\n")
        .replace(
            "loader_readiness = \"host-assisted\"",
            "loader_readiness = \"self-contained\"",
        );
    bytes.truncate(IMAGE_HEADER_SIZE);
    bytes.extend_from_slice(tampered.as_bytes());
    bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

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
    let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
    let tampered = source.replace(
        "loader_readiness = \"host-assisted\"",
        "loader_readiness = \"blocked\"",
    );
    bytes.truncate(IMAGE_HEADER_SIZE);
    bytes.extend_from_slice(tampered.as_bytes());
    bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

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
    let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
    let tampered = source.replace("symbol_name = \"main\"", "symbol_name = \"boot\"");
    bytes.truncate(IMAGE_HEADER_SIZE);
    bytes.extend_from_slice(tampered.as_bytes());
    bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

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
    let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
    let tampered = source.replace(
        "loader_blockers = []",
        "loader_blockers = [\"external-import:final-stage-driver:cc\"]",
    );
    bytes.truncate(IMAGE_HEADER_SIZE);
    bytes.extend_from_slice(tampered.as_bytes());
    bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

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

#[test]
fn blocks_container_handoff_when_container_blockers_are_declared() {
    let mut bytes = nsb_bytes();
    let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
    let tampered = source.replace("\nblockers = []", "\nblockers = [\"payload-not-sealed\"]");
    bytes.truncate(IMAGE_HEADER_SIZE);
    bytes.extend_from_slice(tampered.as_bytes());
    bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-container-blocker-test-{}",
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
        report.container_blockers,
        vec!["payload-not-sealed".to_owned()]
    );
    assert_eq!(report.container_loader_handoff_status, "blocked");
    assert!(report
        .container_loader_handoff_blockers
        .contains(&"container:blocker:payload-not-sealed".to_owned()));
    assert!(report
        .blockers
        .contains(&"container:blocker:payload-not-sealed".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn blocks_container_loader_handoff_when_entry_kind_mismatches_symbol_kind() {
    let mut bytes = nsb_bytes();
    let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
    let tampered = source.replace(
        "loader_entry_kind = \"lifecycle-bootstrap\"",
        "loader_entry_kind = \"host-entry-bootstrap\"",
    );
    bytes.truncate(IMAGE_HEADER_SIZE);
    bytes.extend_from_slice(tampered.as_bytes());
    bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

    let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
        .expect("manifest parses");
    let dir = env::temp_dir().join(format!(
        "nuis-host-runner-entry-kind-test-{}",
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
        report.container_loader_entry_kind.as_deref(),
        Some("host-entry-bootstrap")
    );
    assert_eq!(
        report.container_loader_symbol_kind.as_deref(),
        Some("lifecycle-bootstrap")
    );
    assert_eq!(report.container_loader_handoff_status, "blocked");
    assert!(report
        .container_loader_handoff_blockers
        .contains(&"container-loader:entry-kind-mismatch".to_owned()));
    assert!(report
        .blockers
        .contains(&"container-loader:entry-kind-mismatch".to_owned()));
    let _ = fs::remove_dir_all(&dir);
}
