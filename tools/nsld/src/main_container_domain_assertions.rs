use super::container::NsldContainerVerifyReport;

pub(crate) fn assert_matching_shader_contract(report: &NsldContainerVerifyReport) {
    assert!(report.expected_shader_section_present);
    assert_eq!(
        report.expected_shader_section_id,
        "sec0004.shader-lowering-sidecar-input"
    );
    assert!(report.actual_shader_section_present);
    assert_eq!(
        report.actual_shader_section_id.as_deref(),
        Some("sec0004.shader-lowering-sidecar-input")
    );
    assert!(report.expected_shader_loader_symbol_present);
    assert_eq!(
        report.expected_shader_loader_symbol_id,
        "sym0001.hetero-node.shader.official.shader"
    );
    assert!(report.actual_shader_loader_symbol_present);
    assert_eq!(
        report.actual_shader_loader_symbol_id.as_deref(),
        Some("sym0001.hetero-node.shader.official.shader")
    );
    assert!(report.expected_shader_relocation_present);
    assert_eq!(report.expected_shader_relocation_id, "rel0001.hetero-node");
    assert!(report.actual_shader_relocation_present);
    assert_eq!(
        report.actual_shader_relocation_id.as_deref(),
        Some("rel0001.hetero-node")
    );
}

pub(crate) fn assert_matching_native_object_contract(report: &NsldContainerVerifyReport) {
    assert!(report.expected_native_object_section_present);
    assert_eq!(
        report.expected_native_object_section_id,
        "sec0005.native-object-output"
    );
    assert!(report.actual_native_object_section_present);
    assert_eq!(
        report.actual_native_object_section_id.as_deref(),
        Some("sec0005.native-object-output")
    );
    assert!(report.expected_native_object_loader_symbol_present);
    assert_eq!(
        report.expected_native_object_loader_symbol_id,
        "sym0002.native-object-output"
    );
    assert!(report.actual_native_object_loader_symbol_present);
    assert_eq!(
        report.actual_native_object_loader_symbol_id.as_deref(),
        Some("sym0002.native-object-output")
    );
    assert!(report.expected_native_object_relocation_present);
    assert_eq!(
        report.expected_native_object_relocation_id,
        "rel0002.native-object"
    );
    assert!(report.actual_native_object_relocation_present);
    assert_eq!(
        report.actual_native_object_relocation_id.as_deref(),
        Some("rel0002.native-object")
    );
}

pub(crate) fn assert_matching_kernel_contract(report: &NsldContainerVerifyReport) {
    assert!(report.expected_kernel_section_present);
    assert_eq!(
        report.expected_kernel_section_id,
        "sec0004.kernel-lowering-sidecar-input"
    );
    assert!(report.actual_kernel_section_present);
    assert_eq!(
        report.actual_kernel_section_id.as_deref(),
        Some("sec0004.kernel-lowering-sidecar-input")
    );
    assert!(report.expected_kernel_loader_symbol_present);
    assert_eq!(
        report.expected_kernel_loader_symbol_id,
        "sym0001.hetero-node.kernel.official.kernel"
    );
    assert!(report.actual_kernel_loader_symbol_present);
    assert_eq!(
        report.actual_kernel_loader_symbol_id.as_deref(),
        Some("sym0001.hetero-node.kernel.official.kernel")
    );
    assert!(report.expected_kernel_relocation_present);
    assert_eq!(report.expected_kernel_relocation_id, "rel0001.hetero-node");
    assert!(report.actual_kernel_relocation_present);
    assert_eq!(
        report.actual_kernel_relocation_id.as_deref(),
        Some("rel0001.hetero-node")
    );
}
