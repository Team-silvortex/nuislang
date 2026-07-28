use std::sync::OnceLock;

pub const KERNEL_CODE_ASSET_REGISTRY_CONTRACT: &str = "nuis-kernel-code-asset-registry-v1";
pub const CODE_ASSET_FNV1A64_DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisteredKernelCodeAsset {
    pub lowering_target: &'static str,
    pub id: &'static str,
    pub format: &'static str,
    pub target: &'static str,
    pub minimum_compute_capability: u32,
    pub entry: &'static str,
    pub visible_entries: &'static [&'static str],
    pub file_name: &'static str,
    pub digest_contract: &'static str,
    pub bytes: &'static [u8],
}

static CUDA_PTX_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
static REGISTERED_KERNEL_CODE_ASSETS: OnceLock<Vec<RegisteredKernelCodeAsset>> = OnceLock::new();

pub fn registered_kernel_code_assets() -> &'static [RegisteredKernelCodeAsset] {
    REGISTERED_KERNEL_CODE_ASSETS.get_or_init(|| {
        let bytes = CUDA_PTX_BYTES
            .get_or_init(|| {
                crate::kernel_ptx_emitter::lower_cuda_ptx(
                    &crate::kernel_codegen_table::registered_provider_codegen_table(),
                )
                .expect("registered Kernel/YIR CUDA lowering must remain valid")
                .into_bytes()
            })
            .as_slice();
        vec![RegisteredKernelCodeAsset {
            lowering_target: "cuda.nvidia-gpu",
            id: "kernel.vector-arithmetic.f32.cuda.ptx",
            format: "ptx",
            target: "sm_80",
            minimum_compute_capability: 80,
            entry: "nuis_kernel_vector_add_f32",
            visible_entries: &["nuis_kernel_vector_add_f32", "nuis_kernel_scale_f32"],
            file_name: "nuis.domain.kernel.cuda.ptx",
            digest_contract: CODE_ASSET_FNV1A64_DIGEST_CONTRACT,
            bytes,
        }]
    })
}

pub fn select_kernel_code_asset(
    lowering_target: &str,
) -> Option<&'static RegisteredKernelCodeAsset> {
    registered_kernel_code_assets()
        .iter()
        .find(|asset| asset.lowering_target == lowering_target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuda_ptx_asset_is_lowered_from_registered_kernel_yir() {
        let asset = select_kernel_code_asset("cuda.nvidia-gpu").expect("CUDA PTX asset");
        assert_eq!(registered_kernel_code_assets(), [*asset]);
        assert_eq!(asset.format, "ptx");
        assert_eq!(asset.target, "sm_80");
        assert_eq!(asset.minimum_compute_capability, 80);
        assert_eq!(
            asset.visible_entries,
            crate::kernel_ptx_emitter::cuda_yir_entries(
                &crate::kernel_codegen_table::registered_provider_codegen_table()
            )
        );
        assert!(asset.visible_entries.iter().all(|entry| asset
            .bytes
            .windows(entry.len())
            .any(|window| window == entry.as_bytes())));
        let ptx = std::str::from_utf8(asset.bytes).expect("generated PTX is UTF-8");
        assert!(ptx.contains("add.rn.f32"));
        assert!(ptx.contains("mul.rn.f32"));
        assert!(select_kernel_code_asset("missing.target").is_none());
    }
}
