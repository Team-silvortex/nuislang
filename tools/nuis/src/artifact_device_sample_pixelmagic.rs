use crate::artifact_device_sample_registration::DeviceSampleInputRegistration;
use std::{collections::BTreeSet, path::Path};

pub(crate) fn registration() -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: "nuis.pixelmagic",
        supports: supports_filter_plan,
        enrich_evidence: pixelmagic_filter_plan_evidence,
        resolve_evidence: None,
        persist_payloads: persist_pixelmagic_payloads,
    }
}

fn supports_filter_plan(backend_family: &str, target_device: &str) -> bool {
    crate::artifact_device_sample_pixelmagic_plan::load_filter_plan()
        .is_ok_and(|plan| plan.supports(backend_family, target_device))
}

fn pixelmagic_filter_plan_evidence(base: &str) -> String {
    match crate::artifact_device_sample_pixelmagic_plan::load_filter_plan_for_artifact_metadata(base)
    {
        Ok(plan) => plan.render_evidence(),
        Err(error) => format!(
            "provider_filter_plan_contract={};provider_filter_plan_validation_status=invalid;provider_filter_plan_validation_error={}",
            crate::artifact_device_sample_pixelmagic_plan::FILTER_PLAN_CONTRACT,
            error.replace([';', '\n', '\r'], "_")
        ),
    }
}

fn persist_pixelmagic_payloads(output_dir: &Path, evidence: &[&str]) -> Result<(), String> {
    if !evidence
        .iter()
        .any(|item| item.contains("provider_sample_registration_package=nuis.pixelmagic"))
    {
        return Ok(());
    }
    let mut persisted = BTreeSet::new();
    for item in evidence
        .iter()
        .filter(|item| item.contains("provider_sample_registration_package=nuis.pixelmagic"))
    {
        let plan =
            crate::artifact_device_sample_pixelmagic_plan::load_filter_plan_for_artifact_metadata(
                item,
            )?;
        let expected_hash = format!("provider_filter_plan_hash={}", plan.source_hash());
        let expected_catalog_hash =
            format!("provider_filter_plan_catalog_hash={}", plan.catalog_hash());
        if !item.contains(&expected_hash) || !item.contains(&expected_catalog_hash) {
            return Err(
                "PixelMagic filter plan evidence hash does not match package plan".to_owned(),
            );
        }
        if persisted.insert(plan.source_hash().to_owned()) {
            plan.persist_payloads(output_dir)?;
        }
    }
    if persisted.is_empty() {
        return Err("PixelMagic registration evidence is missing".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};

    #[test]
    fn registration_owns_gray8_shape_kernel_and_payload() {
        let registration = registration();
        let evidence = (registration.enrich_evidence)("ignored");

        assert_eq!(registration.package_id, "nuis.pixelmagic");
        assert!((registration.supports)("metal", "apple-silicon-gpu"));
        assert!(evidence.contains("provider_buffer_shape=2x2"));
        assert!(evidence.contains("provider_kernel_id=pixelmagic.gray8.invert"));
        assert!(evidence.contains("provider_request_count=2"));
        assert!(evidence.contains("provider_filter_plan_contract=nuis-pixelmagic-filter-plan-v1"));
        assert!(evidence.contains(
            "provider_filter_plan_catalog_contract=nuis-pixelmagic-filter-plan-catalog-v1"
        ));
        assert!(evidence.contains("provider_filter_plan_catalog_count=2"));
        assert!(evidence
            .contains("provider_filter_plan_catalog_default_id=pixelmagic.gray8.invert-threshold"));
        assert!(evidence.contains("provider_filter_plan_catalog_selection_status=default-selected"));
        assert!(evidence.contains("provider_filter_plan_validation_status=verified"));
        assert!(evidence.contains(
            "provider_filter_plan_stage_order=pixelmagic.gray8.invert,pixelmagic.gray8.threshold"
        ));
        assert!(evidence.contains("provider_request_1_kernel_id=pixelmagic.gray8.threshold"));
        assert!(evidence.contains("provider_request_1_kernel_operation=threshold"));
        assert!(evidence.contains(
            "provider_request_1_dependency_0_producer_request_id=pixelmagic.gray8.invert"
        ));
        assert!(evidence.contains("provider_request_1_input_binding_0_source=dependency"));
        assert!(evidence.contains(
            "provider_request_1_dependency_0_transport_consumer_clock_evidence=provider-clock:request-1:dispatch-ready"
        ));
        assert!(evidence.contains("pixel_payload_hash=0x2a974c7f8a4241d0"));
        assert!(include_str!("../../../stdlib/pixelmagic/module.toml")
            .contains("contract.pixelmagic.provider-sample-input-registration.v1"));
        assert!(include_str!("../../../stdlib/pixelmagic/module.toml")
            .contains("contract.pixelmagic.filter-plan.v1"));
        assert!(include_str!("../../../stdlib/pixelmagic/module.toml")
            .contains("provider-plans/gray8-invert-threshold.nspf"));
        assert!(include_str!("../../../stdlib/pixelmagic/module.toml")
            .contains("provider-plans/gray8-threshold.nspf"));
    }

    #[test]
    fn registration_persists_its_own_payload() {
        let output_dir = env::temp_dir().join(format!(
            "nuis-pixelmagic-provider-registration-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let plan = crate::artifact_device_sample_pixelmagic_plan::load_filter_plan().unwrap();
        persist_pixelmagic_payloads(
            &output_dir,
            &[&format!(
                "provider_sample_registration_package=nuis.pixelmagic;provider_filter_plan_hash={};provider_filter_plan_catalog_hash={}",
                plan.source_hash(),
                plan.catalog_hash(),
            )],
        )
        .unwrap();
        let payload =
            std::fs::read(output_dir.join("nuis.pixelmagic.std-preprocessed.gray8.bin")).unwrap();
        let threshold = std::fs::read(
            output_dir.join("nuis.pixelmagic.std-preprocessed.gray8-threshold.expected.bin"),
        )
        .unwrap();
        fs::remove_dir_all(output_dir).unwrap();

        assert_eq!(payload, [0, 4, 9, 8]);
        assert_eq!(threshold, [15, 15, 0, 0]);
        assert_eq!(
            crate::artifact_device_sample_pixelmagic_plan::fnv1a64_hex(&threshold),
            "0xfc6f93a90d12d41b"
        );
    }

    #[test]
    fn registration_selects_non_default_plan_from_artifact_metadata() {
        let registration = registration();
        let base = "artifact_provider_metadata_contract=nuis-artifact-provider-metadata-v1;artifact_provider_metadata_count=1;artifact_provider_metadata_0=nuis.pixelmagic:filter-plan=pixelmagic.gray8.threshold-only";
        let evidence = (registration.enrich_evidence)(base);

        assert!(evidence
            .contains("provider_filter_plan_catalog_selection_status=artifact-request-selected"));
        assert!(evidence
            .contains("provider_filter_plan_artifact_request_id=pixelmagic.gray8.threshold-only"));
        assert!(evidence.contains("provider_filter_plan_id=pixelmagic.gray8.threshold-only"));
        assert!(evidence.contains(
            "provider_filter_plan_catalog_selected_path=provider-plans/gray8-threshold.nspf"
        ));
        assert!(evidence.contains("provider_filter_plan_stage_count=1"));
        assert!(evidence.contains("provider_filter_plan_stage_order=pixelmagic.gray8.threshold"));
    }

    #[test]
    fn persistence_accepts_distinct_scoped_plans_and_rejects_hash_drift() {
        let output_dir = env::temp_dir().join(format!(
            "nuis-pixelmagic-artifact-selection-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let registration = registration();
        let metadata = vec![
            "@scope(trace=hetero-trace:shader:metal:first)|nuis.pixelmagic:filter-plan=pixelmagic.gray8.invert-threshold".to_owned(),
            "@scope(trace=hetero-trace:shader:metal:second)|nuis.pixelmagic:filter-plan=pixelmagic.gray8.threshold-only".to_owned(),
        ];
        let first = crate::artifact_provider_metadata::render_metadata_for_trace(
            &metadata,
            "shader",
            "hetero-trace:shader:metal:first",
        );
        let second = crate::artifact_provider_metadata::render_metadata_for_trace(
            &metadata,
            "shader",
            "hetero-trace:shader:metal:second",
        );
        let first_record = format!(
            "provider_sample_registration_package=nuis.pixelmagic;{first};{}",
            (registration.enrich_evidence)(&first)
        );
        let second_record = format!(
            "provider_sample_registration_package=nuis.pixelmagic;{second};{}",
            (registration.enrich_evidence)(&second)
        );

        persist_pixelmagic_payloads(
            &output_dir,
            &[first_record.as_str(), second_record.as_str()],
        )
        .unwrap();
        assert_eq!(
            fs::read(output_dir.join("nuis.pixelmagic.std-preprocessed.gray8-invert.expected.bin"))
                .unwrap(),
            [15, 11, 6, 7]
        );
        assert_eq!(
            fs::read(
                output_dir
                    .join("nuis.pixelmagic.std-preprocessed.gray8-threshold-only.expected.bin")
            )
            .unwrap(),
            [0, 0, 15, 15]
        );

        let selected =
            crate::artifact_device_sample_pixelmagic_plan::load_filter_plan_for_artifact_metadata(
                &second,
            )
            .unwrap();
        let tampered = second_record.replace(
            &format!("provider_filter_plan_hash={}", selected.source_hash()),
            "provider_filter_plan_hash=0x0",
        );
        let error = persist_pixelmagic_payloads(&output_dir, &[tampered.as_str()]).unwrap_err();
        fs::remove_dir_all(output_dir).unwrap();

        assert!(error.contains("evidence hash does not match"));
    }
}
