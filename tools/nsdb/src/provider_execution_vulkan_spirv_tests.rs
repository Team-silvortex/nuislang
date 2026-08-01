use super::*;
use std::path::Path;

#[test]
fn reflects_ordered_multi_output_storage_bindings() {
    let root = Path::new("nustar-packages");
    let manifest = nuisc::registry::load_manifest_for_domain(root, "shader").unwrap();
    let asset = nuisc::registry::code_asset_registration_by_id(
        root,
        &manifest,
        "shader.vulkan.add-xor-pair-u32.spirv",
    )
    .unwrap()
    .expect("registered Vulkan fan-out asset");

    let layout = parse_spirv_storage_buffer_layout(&asset.bytes).unwrap();

    assert_eq!(layout.descriptor_set, 0);
    assert_eq!(layout.input_bindings, vec![0, 1]);
    assert_eq!(layout.output_bindings, vec![2, 3]);
}
