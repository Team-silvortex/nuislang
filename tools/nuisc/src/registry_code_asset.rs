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
    let registry_root = crate::registry_load::resolve_registry_root(registry_root);
    let mut registrations = manifest
        .code_assets
        .iter()
        .map(|entry| parse_registration(&registry_root, manifest, entry))
        .collect::<Result<Vec<_>, _>>()?;
    registrations.sort_by(|lhs, rhs| lhs.asset_id.cmp(&rhs.asset_id));
    Ok(registrations)
}

fn parse_registration(
    registry_root: &Path,
    manifest: &NustarPackageManifest,
    entry: &str,
) -> Result<NustarCodeAssetRegistration, String> {
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
    let source_path = registry_root.join(fields[7]);
    let source_bytes = fs::read(&source_path).map_err(|error| {
        format!(
            "failed to read Nustar code asset `{}` from `{}`: {error}",
            fields[1],
            source_path.display()
        )
    })?;
    let source_extension = source_path.extension().and_then(|ext| ext.to_str());
    let bytes = if fields[2] == "metal-source" && source_extension == Some("wgsl") {
        crate::shader_msl_emitter::lower_canonical_inline_wgsl_u32_for_profile(
            &source_bytes,
            fields[5],
            fields[3],
        )
        .map_err(|error| {
            format!(
                "failed to lower Nustar MSL WGSL code asset `{}`: {error}",
                fields[1]
            )
        })?
    } else if fields[2] == "spirv-binary" && source_extension == Some("wgsl") {
        crate::shader_spirv_emitter::lower_canonical_inline_wgsl_u32_for_profile(
            &source_bytes,
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

        assert_eq!(assets.len(), 4);
        assert!(assets
            .iter()
            .all(|asset| asset.package_id == "official.shader"));
        assert_eq!(
            assets
                .iter()
                .filter(|asset| asset.lowering_target == "metal.apple-silicon-gpu")
                .count(),
            3
        );
        let metal = assets
            .iter()
            .find(|asset| asset.asset_id == "shader.metal.copy-u32.msl")
            .expect("registered canonical Metal MSL asset");
        assert_eq!(metal.format, "metal-source");
        assert_eq!(metal.target, "msl2.4");
        assert_eq!(metal.entry, "nuis_metal_copy_u32");
        let metal_text = std::str::from_utf8(&metal.bytes).unwrap();
        assert!(metal_text.contains("kernel void nuis_metal_copy_u32("));
        assert!(metal_text.contains("output_values[gid] = value;"));
        let vulkan = assets
            .iter()
            .find(|asset| asset.asset_id == "shader.vulkan.copy-u32.spirv")
            .expect("registered Vulkan SPIR-V asset");
        assert_eq!(vulkan.format, "spirv-binary");
        assert_eq!(vulkan.target, "vulkan1.3-spirv1.6");
        assert_eq!(vulkan.entry, "nuis_vulkan_copy_u32");
        assert_eq!(
            u32::from_le_bytes(vulkan.bytes[0..4].try_into().unwrap()),
            0x0723_0203
        );
        assert!(assets.iter().all(|asset| !asset.bytes.is_empty()));
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
}
