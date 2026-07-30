use std::{
    fs,
    path::{Component, Path},
};

use crate::registry::NustarPackageManifest;

pub const NUSTAR_CODE_ASSET_REGISTRATION_CONTRACT: &str = "nuis-nustar-code-asset-registration-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarCodeAssetRegistration {
    pub package_id: String,
    pub domain_family: String,
    pub asset_id: String,
    pub format: String,
    pub lowering_target: String,
    pub target: String,
    pub entry: String,
    pub file_name: String,
    pub source_path: String,
    pub bytes: Vec<u8>,
}

pub fn code_asset_registrations(
    registry_root: &Path,
    manifest: &NustarPackageManifest,
) -> Result<Vec<NustarCodeAssetRegistration>, String> {
    code_asset_registrations_filtered(registry_root, manifest, |_| true)
}

pub fn code_asset_registrations_for_lowering_target(
    registry_root: &Path,
    manifest: &NustarPackageManifest,
    lowering_target: &str,
) -> Result<Vec<NustarCodeAssetRegistration>, String> {
    code_asset_registrations_filtered(registry_root, manifest, |fields| {
        fields[3] == lowering_target
    })
}

pub fn code_asset_registration_by_id(
    registry_root: &Path,
    manifest: &NustarPackageManifest,
    asset_id: &str,
) -> Result<Option<NustarCodeAssetRegistration>, String> {
    let registry_root = crate::registry_load::resolve_registry_root(registry_root);
    let mut selected = None;
    for entry in &manifest.code_assets {
        let fields = parse_registration_fields(manifest, entry)?;
        if fields[1] != asset_id {
            continue;
        }
        if selected.is_some() {
            return Err(format!(
                "nustar package `{}` contains duplicate code asset `{asset_id}`",
                manifest.package_id
            ));
        }
        selected = Some(parse_registration_fields_into(
            &registry_root,
            manifest,
            &fields,
        )?);
    }
    Ok(selected)
}

fn code_asset_registrations_filtered(
    registry_root: &Path,
    manifest: &NustarPackageManifest,
    keep: impl Fn(&[&str]) -> bool,
) -> Result<Vec<NustarCodeAssetRegistration>, String> {
    let registry_root = crate::registry_load::resolve_registry_root(registry_root);
    let mut registrations = Vec::new();
    for entry in &manifest.code_assets {
        let fields = parse_registration_fields(manifest, entry)?;
        if keep(&fields) {
            registrations.push(parse_registration_fields_into(
                &registry_root,
                manifest,
                &fields,
            )?);
        }
    }
    registrations.sort_by(|lhs, rhs| lhs.asset_id.cmp(&rhs.asset_id));
    Ok(registrations)
}

fn parse_registration_fields<'a>(
    manifest: &NustarPackageManifest,
    entry: &'a str,
) -> Result<Vec<&'a str>, String> {
    let fields = entry.split('|').collect::<Vec<_>>();
    if fields.len() != 8 || fields[0] != NUSTAR_CODE_ASSET_REGISTRATION_CONTRACT {
        return Err(format!(
            "nustar package `{}` code asset entry `{entry}` has an invalid registration contract",
            manifest.package_id
        ));
    }
    for (label, value) in [
        ("asset id", fields[1]),
        ("format", fields[2]),
        ("lowering target", fields[3]),
        ("target", fields[4]),
    ] {
        if !token_is_valid(value) {
            return Err(format!(
                "nustar package `{}` code asset {label} `{value}` is invalid",
                manifest.package_id
            ));
        }
    }
    if !manifest
        .lowering_targets
        .iter()
        .any(|target| target == fields[3])
    {
        return Err(format!(
            "nustar package `{}` code asset `{}` references undeclared lowering target `{}`",
            manifest.package_id, fields[1], fields[3]
        ));
    }
    if !symbol_is_valid(fields[5])
        || !relative_path_is_valid(fields[6])
        || !relative_path_is_valid(fields[7])
    {
        return Err(format!(
            "nustar package `{}` code asset `{}` has an invalid entry or path",
            manifest.package_id, fields[1]
        ));
    }
    Ok(fields)
}

fn parse_registration_fields_into(
    registry_root: &Path,
    manifest: &NustarPackageManifest,
    fields: &[&str],
) -> Result<NustarCodeAssetRegistration, String> {
    let source_path = registry_root.join(fields[7]);
    let source_bytes = fs::read(&source_path).map_err(|error| {
        format!(
            "failed to read Nustar code asset `{}` from `{}`: {error}",
            fields[1],
            source_path.display()
        )
    })?;
    let source_extension = source_path.extension().and_then(|ext| ext.to_str());
    let inline_wgsl_source =
        if source_extension == Some("ns") && matches!(fields[2], "metal-source" | "spirv-binary") {
            Some(extract_single_inline_wgsl_source(&source_bytes, fields[1])?)
        } else {
            None
        };
    let lowering_source = inline_wgsl_source.as_deref().unwrap_or(&source_bytes);
    let source_is_inline_wgsl = source_extension == Some("wgsl") || inline_wgsl_source.is_some();
    let bytes = if fields[2] == "metal-source" && source_is_inline_wgsl {
        crate::shader_msl_emitter::lower_canonical_inline_wgsl_u32_for_profile(
            lowering_source,
            fields[5],
            fields[3],
        )
        .map_err(|error| {
            format!(
                "failed to lower Nustar MSL WGSL code asset `{}`: {error}",
                fields[1]
            )
        })?
    } else if fields[2] == "spirv-binary" && source_is_inline_wgsl {
        crate::shader_spirv_emitter::lower_canonical_inline_wgsl_u32_for_profile(
            lowering_source,
            fields[5],
            fields[3],
        )
        .map_err(|error| {
            format!(
                "failed to lower Nustar SPIR-V WGSL code asset `{}`: {error}",
                fields[1]
            )
        })?
    } else if fields[2] == "spirv-binary" {
        crate::shader_spirv_emitter::lower_registered_compute_source_for_profile(
            &source_bytes,
            fields[5],
            fields[3],
        )
        .map_err(|error| {
            format!(
                "failed to lower Nustar SPIR-V code asset `{}`: {error}",
                fields[1]
            )
        })?
    } else {
        source_bytes
    };
    if bytes.is_empty() {
        return Err(format!(
            "nustar package `{}` code asset `{}` is empty",
            manifest.package_id, fields[1]
        ));
    }
    Ok(NustarCodeAssetRegistration {
        package_id: manifest.package_id.clone(),
        domain_family: manifest.domain_family.clone(),
        asset_id: fields[1].to_owned(),
        format: fields[2].to_owned(),
        lowering_target: fields[3].to_owned(),
        target: fields[4].to_owned(),
        entry: fields[5].to_owned(),
        file_name: fields[6].to_owned(),
        source_path: fields[7].to_owned(),
        bytes,
    })
}

fn extract_single_inline_wgsl_source(source: &[u8], asset_id: &str) -> Result<Vec<u8>, String> {
    let source = std::str::from_utf8(source)
        .map_err(|_| format!("Nustar code asset `{asset_id}` .ns source must be UTF-8"))?;
    let mut blocks = Vec::new();
    let mut index = 0usize;
    while let Some(relative) = source[index..].find("wgsl") {
        let keyword = index + relative;
        let after_keyword = keyword + "wgsl".len();
        if !is_identifier_boundary(source.as_bytes(), keyword, after_keyword) {
            index = after_keyword;
            continue;
        }
        let mut cursor = after_keyword;
        while source
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        if source.as_bytes().get(cursor) != Some(&b'{') {
            index = after_keyword;
            continue;
        }
        let end = find_matching_brace(source, cursor).ok_or_else(|| {
            format!("Nustar code asset `{asset_id}` contains an unterminated inline wgsl block")
        })?;
        blocks.push(source[cursor + 1..end].trim().as_bytes().to_vec());
        index = end + 1;
    }
    match blocks.len() {
        1 => Ok(blocks.remove(0)),
        0 => Err(format!(
            "Nustar code asset `{asset_id}` .ns source must contain one `wgsl {{ ... }}` block"
        )),
        count => Err(format!(
            "Nustar code asset `{asset_id}` .ns source contains {count} inline wgsl blocks; registered code assets must be single-entry"
        )),
    }
}

fn is_identifier_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let before = start
        .checked_sub(1)
        .and_then(|index| bytes.get(index))
        .is_none_or(|byte| !identifier_byte(*byte));
    let after = bytes.get(end).is_none_or(|byte| !identifier_byte(*byte));
    before && after
}

fn identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn find_matching_brace(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut index = open;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_line_comment = false;
    let mut block_comment_depth = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_line_comment {
            if byte == b'\n' {
                in_line_comment = false;
            }
            index += 1;
            continue;
        }
        if block_comment_depth > 0 {
            if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
                block_comment_depth += 1;
                index += 2;
            } else if byte == b'*' && bytes.get(index + 1) == Some(&b'/') {
                block_comment_depth -= 1;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'/') {
            in_line_comment = true;
            index += 2;
        } else if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            block_comment_depth = 1;
            index += 2;
        } else if byte == b'"' {
            in_string = true;
            index += 1;
        } else if byte == b'{' {
            depth += 1;
            index += 1;
        } else if byte == b'}' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
            index += 1;
        } else {
            index += 1;
        }
    }
    None
}

fn token_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn symbol_is_valid(value: &str) -> bool {
    value
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'$'))
}

fn relative_path_is_valid(value: &str) -> bool {
    let path = Path::new(value);
    !value.is_empty()
        && !value.contains(['\\', ':'])
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs, process::Command};

    #[test]
    fn shader_code_assets_are_manifest_owned_and_loadable() {
        let root = Path::new("nustar-packages");
        let manifest = crate::registry::load_manifest_for_domain(root, "shader").unwrap();
        let assets = code_asset_registrations(root, &manifest).unwrap();

        assert_eq!(assets.len(), 12);
        assert!(assets
            .iter()
            .all(|asset| asset.package_id == "official.shader"));
        assert_eq!(
            assets
                .iter()
                .filter(|asset| asset.lowering_target == "metal.apple-silicon-gpu")
                .count(),
            7
        );
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

    #[test]
    fn extracts_single_inline_wgsl_from_ns_code_asset_source() {
        let source = br#"
mod shader ExampleCodeAsset {
  fn source() {
    let module: ShaderModule = shader_inline_wgsl("demo", wgsl {
      binding(0, 0) var<storage, read> input_values: array<u32>;

      stage compute(workgroup_size(1, 1, 1)) {
        fn demo(@builtin(global_invocation_id) gid: vec3<u32>) {
          let idx: u32 = gid.x;
        }
      }
    });
  }
}
"#;
        let extracted = extract_single_inline_wgsl_source(source, "shader.demo").unwrap();
        let text = std::str::from_utf8(&extracted).unwrap();
        assert!(text.contains("binding(0, 0)"));
        assert!(text.contains("fn demo("));
        assert!(!text.contains("shader_inline_wgsl"));
    }

    #[test]
    fn registered_vulkan_spirv_passes_external_validator_when_configured() {
        let Ok(validator) = env::var("NUIS_SPIRV_VAL") else {
            return;
        };
        let root = Path::new("nustar-packages");
        let manifest = crate::registry::load_manifest_for_domain(root, "shader").unwrap();
        let assets = code_asset_registrations(root, &manifest).unwrap();
        let vulkan = assets
            .iter()
            .find(|asset| asset.asset_id == "shader.vulkan.copy-u32.spirv")
            .expect("registered Vulkan SPIR-V asset");
        let output_dir = env::temp_dir().join(format!("nuisc-spirv-val-{}", std::process::id()));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let path = output_dir.join(&vulkan.file_name);
        fs::write(&path, &vulkan.bytes).unwrap();
        let status = Command::new(&validator)
            .arg("--target-env")
            .arg("vulkan1.3")
            .arg(&path)
            .status()
            .unwrap_or_else(|error| panic!("failed to run `{validator}`: {error}"));
        fs::remove_dir_all(output_dir).unwrap();

        assert!(
            status.success(),
            "`{validator}` rejected registered Vulkan SPIR-V"
        );
    }

    #[test]
    fn filtered_code_asset_lookup_does_not_materialize_unselected_entries() {
        let root =
            env::temp_dir().join(format!("nuisc-code-asset-filtered-{}", std::process::id()));
        let asset_dir = root.join("assets");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&asset_dir).unwrap();
        fs::write(asset_dir.join("selected.bin"), b"selected").unwrap();
        let mut manifest =
            crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), "shader")
                .unwrap();
        manifest.lowering_targets = vec!["target.keep".to_owned(), "target.skip".to_owned()];
        manifest.code_assets = vec![
            format!(
                "{NUSTAR_CODE_ASSET_REGISTRATION_CONTRACT}|selected.asset|raw-bytes|target.keep|native|nuis_selected|selected.out|assets/selected.bin"
            ),
            format!(
                "{NUSTAR_CODE_ASSET_REGISTRATION_CONTRACT}|missing.asset|raw-bytes|target.skip|native|nuis_missing|missing.out|assets/missing.bin"
            ),
        ];

        let assets =
            code_asset_registrations_for_lowering_target(&root, &manifest, "target.keep").unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].asset_id, "selected.asset");
        assert_eq!(assets[0].bytes, b"selected");
        assert!(
            code_asset_registration_by_id(&root, &manifest, "absent.asset")
                .unwrap()
                .is_none()
        );
        let selected = code_asset_registration_by_id(&root, &manifest, "selected.asset")
            .unwrap()
            .expect("selected asset");
        assert_eq!(selected.bytes, b"selected");
        fs::remove_dir_all(root).unwrap();
    }
}
