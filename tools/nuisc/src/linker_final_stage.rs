use super::*;

pub(super) fn derive_final_stage(
    report: &aot::BuildManifestVerifyReport,
    binary_path: &str,
) -> LinkPlanFinalStage {
    let mut inputs = vec![report.artifact_path.clone(), report.envelope_path.clone()];
    if let Some(path) = &report.bridge_registry_path {
        inputs.push(path.clone());
    }
    if let Some(path) = &report.host_bridge_plan_index_path {
        inputs.push(path.clone());
    }
    if let Some(path) = &report.lowering_plan_index_path {
        inputs.push(path.clone());
    }
    let (kind, driver, link_mode, mut notes) = match report.packaging_mode.as_str() {
        "native-cpu-llvm" => (
            "host-native-link".to_owned(),
            "clang".to_owned(),
            "host-toolchain-finalize".to_owned(),
            vec![
                "nuisc currently lowers host CPU code to LLVM IR and delegates the final machine link to the host clang toolchain".to_owned(),
                "this stage is the temporary backend-facing tail of the larger nuis federated linking model".to_owned(),
            ],
        ),
        "window-aot-bundle" => (
            "heterogeneous-bundle-pack".to_owned(),
            "yir-pack-aot".to_owned(),
            "bundle-packaging".to_owned(),
            vec![
                "heterogeneous window packaging is currently assembled as an AOT bundle rather than a plain native executable link".to_owned(),
            ],
        ),
        other => (
            "custom-finalize".to_owned(),
            "custom".to_owned(),
            "custom".to_owned(),
            vec![format!(
                "packaging mode `{other}` requires an explicit finalization backend"
            )],
        ),
    };
    if report.cpu_target_cross {
        notes.push("cross-compilation target selected; final backend must honor the requested target ABI exactly".to_owned());
    }
    LinkPlanFinalStage {
        kind,
        driver,
        link_mode,
        output_path: binary_path.to_owned(),
        inputs,
        notes,
    }
}
