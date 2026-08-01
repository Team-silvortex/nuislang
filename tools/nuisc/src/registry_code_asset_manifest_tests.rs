use super::*;

#[test]
fn shader_code_assets_are_manifest_owned_and_loadable() {
    let root = Path::new("nustar-packages");
    let manifest = crate::registry::load_manifest_for_domain(root, "shader").unwrap();
    let assets = code_asset_registrations(root, &manifest).unwrap();

    assert_eq!(assets.len(), 18);
    assert!(assets
        .iter()
        .all(|asset| asset.package_id == "official.shader"));
    assert_eq!(
        assets
            .iter()
            .filter(|asset| asset.lowering_target == "metal.apple-silicon-gpu")
            .count(),
        10
    );
    let witsage_bias = assets
        .iter()
        .find(|asset| asset.asset_id == "shader.witsage.vector-bias.metal")
        .expect("registered WitSage vector bias Metal asset");
    assert_eq!(
        witsage_bias.source_path,
        "assets/shader/witsage_vector_bias.ns"
    );
    let witsage_bias_text = std::str::from_utf8(&witsage_bias.bytes).unwrap();
    assert!(witsage_bias_text.contains("kernel void nuis_witsage_vector_bias_f32("));
    assert!(!witsage_bias_text.contains("shader_inline_metal"));
    let witsage_argmax = assets
        .iter()
        .find(|asset| asset.asset_id == "shader.witsage.argmax.metal")
        .expect("registered WitSage argmax Metal asset");
    assert_eq!(
        witsage_argmax.source_path,
        "assets/shader/witsage_argmax.ns"
    );
    let witsage_argmax_text = std::str::from_utf8(&witsage_argmax.bytes).unwrap();
    assert!(witsage_argmax_text.contains("kernel void nuis_witsage_argmax_f32("));
    assert!(!witsage_argmax_text.contains("shader_inline_metal"));
    let metal = assets
        .iter()
        .find(|asset| asset.asset_id == "shader.metal.copy-u32.msl")
        .expect("registered canonical Metal MSL asset");
    assert_eq!(metal.format, "metal-source");
    assert_eq!(metal.target, "msl2.4");
    assert_eq!(metal.entry, "nuis_metal_copy_u32");
    assert_eq!(metal.source_path, "assets/shader/metal_copy_u32.ns");
    let metal_text = std::str::from_utf8(&metal.bytes).unwrap();
    assert!(metal_text.contains(
        "// nuis-module-lowering-plan contract=nuis-yir.shader.backend-lowering-plan.v1"
    ));
    assert!(metal_text.contains("// nuis-module-lowering-target msl:metal-gpu"));
    assert!(metal_text.contains("// nuis-module-native-ir msl2.4"));
    assert!(metal_text.contains("kernel void nuis_metal_copy_u32("));
    assert!(metal_text.contains("output_values[gid] = value;"));
    let metal_add = assets
        .iter()
        .find(|asset| asset.asset_id == "shader.metal.add-u32.msl")
        .expect("registered canonical Metal add MSL asset");
    assert_eq!(metal_add.format, "metal-source");
    assert_eq!(metal_add.target, "msl2.4");
    assert_eq!(metal_add.entry, "nuis_metal_add_u32");
    assert_eq!(metal_add.source_path, "assets/shader/metal_add_u32.ns");
    let metal_add_text = std::str::from_utf8(&metal_add.bytes).unwrap();
    assert!(metal_add_text.contains("kernel void nuis_metal_add_u32("));
    assert!(metal_add_text.contains("output_values[gid] = value + value;"));
    let metal_add_pair = assets
        .iter()
        .find(|asset| asset.asset_id == "shader.metal.add-pair-u32.msl")
        .expect("registered canonical Metal pair add MSL asset");
    assert_eq!(metal_add_pair.entry, "nuis_metal_add_pair_u32");
    assert_eq!(
        metal_add_pair.source_path,
        "assets/shader/metal_add_pair_u32.ns"
    );
    let metal_add_pair_text = std::str::from_utf8(&metal_add_pair.bytes).unwrap();
    assert!(metal_add_pair_text.contains("kernel void nuis_metal_add_pair_u32("));
    assert!(metal_add_pair_text.contains("device const uint* right_values [[buffer(1)]]"));
    assert!(metal_add_pair_text.contains("output_values[gid] = value + rhs;"));
    let metal_reduced = assets
        .iter()
        .find(|asset| asset.asset_id == "shader.metal.add-xor-pair-reduced-u32.msl")
        .expect("registered canonical reduced Metal fan-out asset");
    let metal_reduced_text = std::str::from_utf8(&metal_reduced.bytes).unwrap();
    assert_eq!(metal_reduced.entry, "nuis_metal_add_xor_pair_reduced_u32");
    assert!(metal_reduced_text.contains("output_values[gid] = value + rhs;"));
    assert!(metal_reduced_text
        .contains("if (gid < 2u) {\n        output_values_1[gid] = value ^ rhs;\n    }"));
    let metal_sub = assets
        .iter()
        .find(|asset| asset.asset_id == "shader.metal.sub-u32.msl")
        .expect("registered canonical Metal sub MSL asset");
    assert_eq!(metal_sub.format, "metal-source");
    assert_eq!(metal_sub.target, "msl2.4");
    assert_eq!(metal_sub.entry, "nuis_metal_sub_u32");
    let metal_sub_text = std::str::from_utf8(&metal_sub.bytes).unwrap();
    assert!(metal_sub_text.contains("kernel void nuis_metal_sub_u32("));
    assert!(metal_sub_text.contains("output_values[gid] = value - value;"));
    let metal_mul = assets
        .iter()
        .find(|asset| asset.asset_id == "shader.metal.mul-u32.msl")
        .expect("registered canonical Metal mul MSL asset");
    assert_eq!(metal_mul.format, "metal-source");
    assert_eq!(metal_mul.target, "msl2.4");
    assert_eq!(metal_mul.entry, "nuis_metal_mul_u32");
    let metal_mul_text = std::str::from_utf8(&metal_mul.bytes).unwrap();
    assert!(metal_mul_text.contains("kernel void nuis_metal_mul_u32("));
    assert!(metal_mul_text.contains("output_values[gid] = value * value;"));
    let metal_xor = assets
        .iter()
        .find(|asset| asset.asset_id == "shader.metal.xor-u32.msl")
        .expect("registered canonical Metal xor MSL asset");
    assert_eq!(metal_xor.format, "metal-source");
    assert_eq!(metal_xor.target, "msl2.4");
    assert_eq!(metal_xor.entry, "nuis_metal_xor_u32");
    let metal_xor_text = std::str::from_utf8(&metal_xor.bytes).unwrap();
    assert!(metal_xor_text.contains("kernel void nuis_metal_xor_u32("));
    assert!(metal_xor_text.contains("output_values[gid] = value ^ value;"));
    let vulkan = assets
        .iter()
        .find(|asset| asset.asset_id == "shader.vulkan.copy-u32.spirv")
        .expect("registered Vulkan SPIR-V asset");
    assert_eq!(vulkan.format, "spirv-binary");
    assert_eq!(vulkan.target, "vulkan1.3-spirv1.6");
    assert_eq!(vulkan.entry, "nuis_vulkan_copy_u32");
    assert_eq!(vulkan.source_path, "assets/shader/vulkan_copy_u32.ns");
    assert_eq!(
        u32::from_le_bytes(vulkan.bytes[0..4].try_into().unwrap()),
        0x0723_0203
    );
    for (asset_id, entry) in [
        ("shader.vulkan.add-u32.spirv", "nuis_vulkan_add_u32"),
        (
            "shader.vulkan.add-pair-u32.spirv",
            "nuis_vulkan_add_pair_u32",
        ),
        (
            "shader.vulkan.add-xor-pair-u32.spirv",
            "nuis_vulkan_add_xor_pair_u32",
        ),
        (
            "shader.vulkan.add-xor-pair-reduced-u32.spirv",
            "nuis_vulkan_add_xor_pair_reduced_u32",
        ),
        ("shader.vulkan.sub-u32.spirv", "nuis_vulkan_sub_u32"),
        ("shader.vulkan.mul-u32.spirv", "nuis_vulkan_mul_u32"),
        ("shader.vulkan.xor-u32.spirv", "nuis_vulkan_xor_u32"),
    ] {
        let asset = assets
            .iter()
            .find(|asset| asset.asset_id == asset_id)
            .expect("registered Vulkan SPIR-V u32 asset");
        assert_eq!(asset.format, "spirv-binary");
        assert_eq!(asset.target, "vulkan1.3-spirv1.6");
        assert_eq!(asset.entry, entry);
        assert!(asset.source_path.ends_with(".ns"));
        assert_eq!(
            u32::from_le_bytes(asset.bytes[0..4].try_into().unwrap()),
            0x0723_0203
        );
    }
    assert!(assets.iter().all(|asset| !asset.bytes.is_empty()));
}
