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
    validate_code_asset_source_path(manifest, fields[1], fields[2], fields[7])?;
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
            extract_optional_single_inline_shader_source(&source_bytes, fields[1], "wgsl")?
        } else {
            None
        };
    let inline_metal_source = if source_extension == Some("ns") && fields[2] == "metal-source" {
        extract_optional_single_inline_shader_source(&source_bytes, fields[1], "metal")?
    } else {
        None
    };
    if inline_wgsl_source.is_some() && inline_metal_source.is_some() {
        return Err(format!(
            "Nustar code asset `{}` .ns source must not mix `wgsl {{ ... }}` and `metal {{ ... }}` blocks",
            fields[1]
        ));
    }
    if source_extension == Some("ns")
        && matches!(fields[2], "metal-source" | "spirv-binary")
        && inline_wgsl_source.is_none()
        && inline_metal_source.is_none()
    {
        return Err(format!(
            "Nustar code asset `{}` .ns source must contain one inline shader block",
            fields[1]
        ));
    }
    let lowering_source = inline_wgsl_source.as_deref().unwrap_or(&source_bytes);
    let source_is_inline_wgsl = source_extension == Some("wgsl") || inline_wgsl_source.is_some();
    let bytes = if let Some(inline_metal_source) = inline_metal_source {
        inline_metal_source
    } else if fields[2] == "metal-source" && source_is_inline_wgsl {
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
    } else if fields[2] == "ptx" && source_extension == Some("ns") {
        let source = std::str::from_utf8(&source_bytes).map_err(|_| {
            format!(
                "Kernel Nustar PTX code asset `{}` .ns source must be UTF-8",
                fields[1]
            )
        })?;
        if !source.contains(fields[5]) {
            return Err(format!(
                "Kernel Nustar PTX code asset `{}` source does not declare entry `{}`",
                fields[1], fields[5]
            ));
        }
        let table = crate::kernel_codegen_table::registered_provider_codegen_table_for_entries(&[
            fields[5],
        ])?;
        crate::kernel_ptx_emitter::lower_cuda_ptx(&table)?.into_bytes()
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

#[cfg(test)]
fn extract_single_inline_wgsl_source(source: &[u8], asset_id: &str) -> Result<Vec<u8>, String> {
    extract_optional_single_inline_shader_source(source, asset_id, "wgsl")?.ok_or_else(|| {
        format!("Nustar code asset `{asset_id}` .ns source must contain one `wgsl {{ ... }}` block")
    })
}

#[cfg(test)]
fn extract_single_inline_metal_source(source: &[u8], asset_id: &str) -> Result<Vec<u8>, String> {
    extract_optional_single_inline_shader_source(source, asset_id, "metal")?.ok_or_else(|| {
        format!(
            "Nustar code asset `{asset_id}` .ns source must contain one `metal {{ ... }}` block"
        )
    })
}

fn extract_optional_single_inline_shader_source(
    source: &[u8],
    asset_id: &str,
    block_keyword: &str,
) -> Result<Option<Vec<u8>>, String> {
    let source = std::str::from_utf8(source)
        .map_err(|_| format!("Nustar code asset `{asset_id}` .ns source must be UTF-8"))?;
    let mut blocks = Vec::new();
    let mut index = 0usize;
    while let Some((_, cursor)) = find_next_inline_shader_block(source, block_keyword, index) {
        let end = find_matching_brace(source, cursor).ok_or_else(|| {
            format!(
                "Nustar code asset `{asset_id}` contains an unterminated inline {block_keyword} block"
            )
        })?;
        blocks.push(source[cursor + 1..end].trim().as_bytes().to_vec());
        index = end + 1;
    }
    match blocks.len() {
        1 => Ok(Some(blocks.remove(0))),
        0 => Ok(None),
        count => Err(format!(
            "Nustar code asset `{asset_id}` .ns source contains {count} inline {block_keyword} blocks; registered code assets must be single-entry"
        )),
    }
}

fn find_next_inline_shader_block(
    source: &str,
    block_keyword: &str,
    mut index: usize,
) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let keyword = block_keyword.as_bytes();
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
        } else if bytes[index..].starts_with(keyword) {
            let after_keyword = index + keyword.len();
            if !is_identifier_boundary(bytes, index, after_keyword) {
                index = after_keyword;
                continue;
            }
            let mut cursor = after_keyword;
            while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
                cursor += 1;
            }
            if bytes.get(cursor) == Some(&b'{') {
                return Some((index, cursor));
            }
            index = after_keyword;
        } else {
            index += 1;
        }
    }
    None
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

fn validate_code_asset_source_path(
    manifest: &NustarPackageManifest,
    asset_id: &str,
    format: &str,
    source_path: &str,
) -> Result<(), String> {
    if manifest.domain_family == "shader"
        && matches!(format, "metal-source" | "spirv-binary")
        && Path::new(source_path)
            .extension()
            .and_then(|ext| ext.to_str())
            != Some("ns")
    {
        return Err(format!(
            "nustar package `{}` shader code asset `{asset_id}` source path `{source_path}` must be a .ns source container",
            manifest.package_id
        ));
    }
    if manifest.domain_family == "kernel"
        && format == "ptx"
        && Path::new(source_path)
            .extension()
            .and_then(|ext| ext.to_str())
            != Some("ns")
    {
        return Err(format!(
            "nustar package `{}` Kernel PTX code asset `{asset_id}` source path `{source_path}` must be a .ns source container",
            manifest.package_id
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "registry_code_asset_manifest_tests.rs"]
mod manifest_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs, process::Command};

    #[test]
    fn shader_code_asset_source_path_must_be_ns_container() {
        let root = Path::new("nustar-packages");
        let mut manifest = crate::registry::load_manifest_for_domain(root, "shader").unwrap();
        manifest.code_assets = vec![format!(
            "{NUSTAR_CODE_ASSET_REGISTRATION_CONTRACT}|shader.bad.metal|metal-source|metal.apple-silicon-gpu|msl2.4|nuis_bad|bad.metal|assets/shader/bad.metal"
        )];

        let error = code_asset_registrations(root, &manifest).unwrap_err();

        assert!(error.contains("must be a .ns source container"));

        manifest.code_assets = vec![format!(
            "{NUSTAR_CODE_ASSET_REGISTRATION_CONTRACT}|shader.bad.generated|metal-source|metal.apple-silicon-gpu|msl2.4|nuis_bad|bad.metal|assets/generated/bad.metal"
        )];

        let error = code_asset_registrations(root, &manifest).unwrap_err();

        assert!(error.contains("must be a .ns source container"));
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
    fn inline_shader_source_extractor_ignores_comments_and_strings() {
        let source = br#"
mod shader ExampleCodeAsset {
  fn source() {
    // This fake block must not count: wgsl { fn fake() {} }
    let note: Text = "Nor should this string: metal { kernel void nope() {} }";
    /*
      Nested comment fake:
      wgsl {
        fn also_fake() {
        }
      }
    */
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
        assert!(text.contains("fn demo("));
        assert!(!text.contains("fake"));
    }

    #[test]
    fn extracts_single_inline_metal_from_ns_code_asset_source() {
        let source = br#"
mod shader ExampleMetalCodeAsset {
  fn source() {
    let module: ShaderModule = shader_inline_metal("demo", metal {
      #include <metal_stdlib>
      using namespace metal;

      kernel void demo(device const float* input [[buffer(0)]]) {
      }
    });
  }
}
"#;
        let extracted = extract_single_inline_metal_source(source, "shader.demo").unwrap();
        let text = std::str::from_utf8(&extracted).unwrap();
        assert!(text.contains("kernel void demo("));
        assert!(!text.contains("shader_inline_metal"));
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
