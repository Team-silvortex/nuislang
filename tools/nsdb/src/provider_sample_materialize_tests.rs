use crate::{
    provider_runner_registry::{select_provider_runner_adapter, ProviderRunnerAdapter},
    provider_sample_execute::execute_provider_samples,
    provider_sample_materialize::materialize_provider_samples,
};
use std::{
    env, fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn assert_runner_fragment(text: &str, adapter: &ProviderRunnerAdapter) {
    assert!(text.contains("provider_runner_contract = \"nuis-provider-runner-v1\""));
    assert!(text.contains("provider_runner_adapter_contract = \"nuis-provider-runner-adapter-v1\""));
    assert!(text.contains(&format!(
        "provider_runner_adapter_id = \"{}\"",
        adapter.adapter_id
    )));
    assert!(text.contains(&format!(
        "provider_runner_adapter_capability_status = \"{}\"",
        adapter.capability_status
    )));
    assert!(
        text.contains("provider_runner_registry_protocol = \"nuis-provider-runner-registry-v1\"")
    );
    assert!(text
        .contains("provider_runner_registry_source = \"builtin-nustar-provider-runner-registry\""));
    assert!(text.contains(&format!(
        "provider_runner_real_device_capable = {}",
        adapter.real_device_capable
    )));
}

fn assert_execution_comparison_fragment(text: &str, adapter: &ProviderRunnerAdapter) {
    assert!(text.contains(
        "provider_execution_comparison_contract = \"nuis-provider-execution-comparison-v1\""
    ));
    assert!(text.contains(
        "provider_output_payload_contract = \"nuis-provider-output-payload-handoff-v1\""
    ));
    if adapter.real_device_capable {
        assert!(text.contains("provider_execution_status = \"real-device-runner-selected\""));
        assert!(
            text.contains("provider_execution_comparison_status = \"awaiting-real-device-output\"")
        );
        assert!(
            text.contains("provider_output_payload_status = \"awaiting-provider-output-payload\"")
        );
        assert!(text.contains("provider_output_payload_evidence = \"not-materialized\""));
        assert!(text
            .contains("provider_output_payload_next_action = \"attach-provider-output-payload\""));
        assert!(text
            .contains("provider_execution_next_action = \"execute-real-device-provider-sample\""));
    } else {
        assert!(text.contains("provider_execution_status = \"host-fallback-runner-selected\""));
        assert!(text.contains(
            "provider_execution_comparison_status = \"host-fallback-output-comparable\""
        ));
        assert!(text
            .contains("provider_output_payload_status = \"host-fallback-output-payload-ready\""));
        assert!(text.contains(
            "provider_output_payload_evidence = \"nuis.nsdb.provider-output.metal-apple-silicon-gpu.toml:hash=0x"
        ));
        assert!(text
            .contains("provider_output_payload_next_action = \"compare-provider-output-payload\""));
        assert!(text.contains(
            "provider_execution_next_action = \"compare-host-fallback-provider-sample\""
        ));
    }
}

#[test]
fn materializes_pending_provider_sample_manifest() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = env::temp_dir().join(format!("nsdb-provider-materialize-{nonce}"));
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        r#"
protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "run-artifact-provider-sample-manifest"
status = "awaiting-provider-materialization"
record_count = 1
ready_record_count = 0
pending_record_count = 1

[[device_provider_samples]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
sample_status = "pending-provider-execution"
validation_status = "pending-provider-execution"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "not-materialized"
materialization_status = "provider-sample-pending"
materialization_detail = "awaiting-provider-runtime"
next_action = "execute-provider-sample"
"#,
    )
    .unwrap();

    let report = materialize_provider_samples(&output_dir, None).unwrap();
    let source =
        fs::read_to_string(output_dir.join("nuis.nsdb.device-provider-samples.toml")).unwrap();
    let expected_adapter = select_provider_runner_adapter("metal:apple-silicon-gpu");

    assert_eq!(report.status, "ready");
    assert_eq!(
        report.provider_families,
        vec!["metal:apple-silicon-gpu".to_owned()]
    );
    assert_eq!(report.matched_record_count, 1);
    assert_eq!(report.materialized_record_count, 1);
    assert_eq!(report.skipped_record_count, 0);
    assert_eq!(
        (
            report.first_provider_runner_contract.as_str(),
            report.first_provider_runner_adapter_contract.as_str(),
            report.first_provider_runner_adapter_id.as_str(),
            report
                .first_provider_runner_adapter_capability_status
                .as_str(),
            report.first_provider_runner_registry_protocol.as_str(),
            report.first_provider_runner_registry_source.as_str(),
            report.first_provider_runner_real_device_capable,
            report.first_provider_runner_kind.as_str(),
            report.first_provider_execution_mode.as_str(),
        ),
        (
            "nuis-provider-runner-v1",
            "nuis-provider-runner-adapter-v1",
            expected_adapter.adapter_id,
            expected_adapter.capability_status,
            "nuis-provider-runner-registry-v1",
            "builtin-nustar-provider-runner-registry",
            expected_adapter.real_device_capable,
            expected_adapter.kind,
            expected_adapter.execution_mode,
        )
    );
    assert_eq!(report.next_action, "replay-provider-sample");
    assert!(report.next_command.contains("nsdb replay-plan "));
    assert!(report.next_command.contains("--json"));
    assert_eq!(
        report.return_contract,
        "nsld-final-output-boundary-return-v1"
    );
    assert_eq!(
        report.final_output_replay_contract,
        "nsdb-payload-execution-replay-plan-v1"
    );
    assert_eq!(
        report.return_action,
        "resume-nsld-final-output-check-manifest-required"
    );
    assert_eq!(
        report.return_command,
        "nsld check <nuis.build.manifest.toml> --json"
    );
    assert!(source.contains("source = \"nsdb-materialize-provider-samples\""));
    assert!(source.contains("ready_record_count = 1"));
    assert!(source.contains("pending_record_count = 0"));
    assert!(source.contains(
        "output_evidence = \"nuis.nsdb.provider-sample.metal-apple-silicon-gpu.toml:hash=0x"
    ));
    assert!(source.contains("materialization_status = \"provider-sample-materialized\""));
    assert_runner_fragment(&source, &expected_adapter);
    assert_execution_comparison_fragment(&source, &expected_adapter);
    assert!(source.contains(
        "materialization_detail = \"deterministic-provider-sample-artifact:nuis.nsdb.provider-sample.metal-apple-silicon-gpu.toml:0x"
    ));
    assert!(source.contains("next_action = \"replay-device-sample\""));
    let artifact = fs::read_to_string(
        output_dir.join("nuis.nsdb.provider-sample.metal-apple-silicon-gpu.toml"),
    )
    .unwrap();
    assert!(artifact.contains("protocol = \"nuis-nsdb-provider-sample-artifact-v1\""));
    assert!(artifact.contains("schema = \"nsdb-yir-provider-sample-artifact-v1\""));
    assert_runner_fragment(&artifact, &expected_adapter);
    assert_execution_comparison_fragment(&artifact, &expected_adapter);
    let output_payload_path =
        output_dir.join("nuis.nsdb.provider-output.metal-apple-silicon-gpu.toml");
    if expected_adapter.real_device_capable {
        assert!(!output_payload_path.exists());
    } else {
        let output_payload = fs::read_to_string(output_payload_path).unwrap();
        assert!(output_payload.contains("protocol = \"nuis-provider-output-payload-v1\""));
        assert!(output_payload.contains("output_payload_kind = \"host-fallback-anchor\""));
        assert!(source.contains(
            "provider_output_payload_evidence = \"nuis.nsdb.provider-output.metal-apple-silicon-gpu.toml:hash=0x"
        ));
    }

    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn materializer_returns_concrete_nsld_check_when_manifest_is_in_output_dir() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = env::temp_dir().join(format!("nsdb-provider-return-{nonce}"));
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(
        output_dir.join("nuis.build.manifest.toml"),
        "manifest = true\n",
    )
    .unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        r#"
protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "run-artifact-provider-sample-manifest"
status = "awaiting-provider-materialization"
record_count = 1
ready_record_count = 0
pending_record_count = 1

[[device_provider_samples]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
sample_status = "pending-provider-execution"
validation_status = "pending-provider-execution"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "not-materialized"
materialization_status = "provider-sample-pending"
materialization_detail = "awaiting-provider-runtime"
next_action = "execute-provider-sample"
"#,
    )
    .unwrap();

    let report = materialize_provider_samples(&output_dir, None).unwrap();

    assert_eq!(report.return_action, "resume-nsld-final-output-check");
    assert_eq!(
        report.return_contract,
        "nsld-final-output-boundary-return-v1"
    );
    assert_eq!(
        report.final_output_replay_contract,
        "nsdb-payload-execution-replay-plan-v1"
    );
    assert_eq!(
        report.return_command,
        format!("nsld check {} --json", output_dir.display())
    );

    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn materializer_attaches_preexisting_real_device_output_payload() {
    let adapter = select_provider_runner_adapter("metal:apple-silicon-gpu");
    if !adapter.real_device_capable {
        return;
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = env::temp_dir().join(format!("nsdb-provider-real-payload-{nonce}"));
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        r#"
protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "run-artifact-provider-sample-manifest"
status = "awaiting-provider-materialization"
record_count = 1
ready_record_count = 0
pending_record_count = 1

[[device_provider_samples]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
sample_status = "pending-provider-execution"
validation_status = "pending-provider-execution"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "not-materialized"
materialization_status = "provider-sample-pending"
materialization_detail = "awaiting-provider-runtime"
next_action = "execute-provider-sample"
"#,
    )
    .unwrap();

    let execute_report = execute_provider_samples(&output_dir, None).unwrap();
    assert_eq!(execute_report.status, "provider-output-payloads-ready");
    assert_eq!(execute_report.matched_record_count, 1);
    assert_eq!(execute_report.output_payload_count, 1);
    assert_eq!(
        execute_report.first_provider_family,
        "metal:apple-silicon-gpu"
    );
    assert_eq!(
        execute_report.first_provider_runner_adapter_id,
        adapter.adapter_id
    );
    assert_eq!(
        execute_report.first_provider_runner_adapter_capability_status,
        adapter.capability_status
    );
    assert!(execute_report.first_provider_runner_real_device_capable);
    assert_eq!(
        execute_report.first_provider_execution_mode,
        adapter.execution_mode
    );
    assert!(execute_report
        .first_output_payload_evidence
        .contains("nuis.nsdb.provider-output.metal-apple-silicon-gpu.toml:hash=0x"));
    let report = materialize_provider_samples(&output_dir, None).unwrap();
    let source =
        fs::read_to_string(output_dir.join("nuis.nsdb.device-provider-samples.toml")).unwrap();

    assert_eq!(
        report.first_provider_output_payload_status,
        "real-device-output-payload-attached"
    );
    assert!(
        source.contains("provider_output_payload_status = \"real-device-output-payload-attached\"")
    );
    assert!(source.contains(
        "provider_output_payload_evidence_status = \"provider-output-payload-attached\""
    ));
    assert!(source.contains(
        "provider_output_payload_evidence = \"nuis.nsdb.provider-output.metal-apple-silicon-gpu.toml:hash=0x"
    ));
    assert!(source.contains("real-device-provider-output-payload:"));

    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn materializes_only_matching_provider_family() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = env::temp_dir().join(format!("nsdb-provider-filter-{nonce}"));
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        r#"
protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "run-artifact-provider-sample-manifest"
status = "awaiting-provider-materialization"
record_count = 2
ready_record_count = 0
pending_record_count = 2

[[device_provider_samples]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
sample_status = "pending-provider-execution"
validation_status = "pending-provider-execution"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "not-materialized"
materialization_status = "provider-sample-pending"
materialization_detail = "awaiting-provider-runtime"
next_action = "execute-provider-sample"

[[device_provider_samples]]
trace_id = "hetero-trace:shader:spirv:vulkan-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "spirv:vulkan-gpu"
handoff_target = "spirv:vulkan-gpu"
sample_status = "pending-provider-execution"
validation_status = "pending-provider-execution"
input_evidence = "spv:pixelmagic.spv"
output_evidence = "not-materialized"
materialization_status = "provider-sample-pending"
materialization_detail = "awaiting-provider-runtime"
next_action = "execute-provider-sample"
"#,
    )
    .unwrap();

    let report =
        materialize_provider_samples(&output_dir, Some("metal:apple-silicon-gpu")).unwrap();
    let source =
        fs::read_to_string(output_dir.join("nuis.nsdb.device-provider-samples.toml")).unwrap();

    assert_eq!(
        report.provider_family_filter.as_deref(),
        Some("metal:apple-silicon-gpu")
    );
    assert_eq!(
        report.provider_families,
        vec![
            "metal:apple-silicon-gpu".to_owned(),
            "spirv:vulkan-gpu".to_owned()
        ]
    );
    assert_eq!(report.status, "awaiting-provider-materialization");
    assert_eq!(report.matched_record_count, 1);
    assert_eq!(report.materialized_record_count, 1);
    assert_eq!(report.skipped_record_count, 1);
    assert!(source.contains("ready_record_count = 1"));
    assert!(source.contains("pending_record_count = 1"));
    assert!(source.contains(
        "output_evidence = \"nuis.nsdb.provider-sample.metal-apple-silicon-gpu.toml:hash=0x"
    ));
    assert!(source.contains("output_evidence = \"not-materialized\""));
    assert!(source.contains("provider_family = \"spirv:vulkan-gpu\""));

    fs::remove_dir_all(output_dir).unwrap();
}
