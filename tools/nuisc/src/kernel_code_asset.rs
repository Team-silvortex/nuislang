pub const KERNEL_CODE_ASSET_REGISTRY_CONTRACT: &str = "nuis-kernel-code-asset-registry-v1";
pub const CODE_ASSET_FNV1A64_DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisteredKernelCodeAsset {
    pub lowering_target: &'static str,
    pub id: &'static str,
    pub format: &'static str,
    pub target: &'static str,
    pub entry: &'static str,
    pub file_name: &'static str,
    pub digest_contract: &'static str,
    pub bytes: &'static [u8],
}

const CUDA_PTX_VECTOR_ADD_F32: &str = r#".version 8.0
.target sm_80
.address_size 64

.visible .entry nuis_kernel_vector_add_f32(
    .param .u64 input_lhs,
    .param .u64 input_rhs,
    .param .u64 output,
    .param .u32 element_count
)
{
    .reg .pred %p<2>;
    .reg .b32 %r<6>;
    .reg .b64 %rd<8>;
    .reg .f32 %f<4>;

    ld.param.u64 %rd1, [input_lhs];
    ld.param.u64 %rd2, [input_rhs];
    ld.param.u64 %rd3, [output];
    ld.param.u32 %r1, [element_count];
    mov.u32 %r2, %ctaid.x;
    mov.u32 %r3, %ntid.x;
    mov.u32 %r4, %tid.x;
    mad.lo.s32 %r5, %r2, %r3, %r4;
    setp.ge.u32 %p1, %r5, %r1;
    @%p1 bra DONE;
    mul.wide.u32 %rd4, %r5, 4;
    add.s64 %rd5, %rd1, %rd4;
    add.s64 %rd6, %rd2, %rd4;
    add.s64 %rd7, %rd3, %rd4;
    ld.global.f32 %f1, [%rd5];
    ld.global.f32 %f2, [%rd6];
    add.rn.f32 %f3, %f1, %f2;
    st.global.f32 [%rd7], %f3;
DONE:
    ret;
}
"#;

const REGISTERED_KERNEL_CODE_ASSETS: &[RegisteredKernelCodeAsset] = &[RegisteredKernelCodeAsset {
    lowering_target: "cuda.nvidia-gpu",
    id: "kernel.vector-add.f32.cuda.ptx",
    format: "ptx",
    target: "sm_80",
    entry: "nuis_kernel_vector_add_f32",
    file_name: "nuis.domain.kernel.cuda.ptx",
    digest_contract: CODE_ASSET_FNV1A64_DIGEST_CONTRACT,
    bytes: CUDA_PTX_VECTOR_ADD_F32.as_bytes(),
}];

pub fn registered_kernel_code_assets() -> &'static [RegisteredKernelCodeAsset] {
    REGISTERED_KERNEL_CODE_ASSETS
}

pub fn select_kernel_code_asset(
    lowering_target: &str,
) -> Option<&'static RegisteredKernelCodeAsset> {
    REGISTERED_KERNEL_CODE_ASSETS
        .iter()
        .find(|asset| asset.lowering_target == lowering_target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuda_ptx_asset_has_one_registry_owned_identity() {
        let asset = select_kernel_code_asset("cuda.nvidia-gpu").expect("CUDA PTX asset");
        assert_eq!(registered_kernel_code_assets(), [*asset]);
        assert_eq!(asset.format, "ptx");
        assert_eq!(asset.target, "sm_80");
        assert!(asset
            .bytes
            .windows(asset.entry.len())
            .any(|window| window == asset.entry.as_bytes()));
        assert!(select_kernel_code_asset("missing.target").is_none());
    }
}
