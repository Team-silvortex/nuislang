use crate::{
    provider_code_asset::ProviderCodeAssetDescriptor, provider_sample_payload::fnv1a64_hex,
};
use std::collections::BTreeMap;

const SPIRV_MAGIC: u32 = 0x0723_0203;
const SPIRV_VERSION_1_6: u32 = 0x0001_0600;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VulkanStorageBufferLayout {
    pub(crate) descriptor_set: u32,
    pub(crate) input_bindings: Vec<u32>,
    pub(crate) output_binding: u32,
}

#[derive(Default)]
struct SpirvDescriptorDecorations {
    descriptor_set: Option<u32>,
    binding: Option<u32>,
    non_writable: bool,
    non_readable: bool,
}

pub(crate) fn validate_spirv_u32_module(
    asset: &ProviderCodeAssetDescriptor,
    bytes: &[u8],
) -> Result<VulkanStorageBufferLayout, String> {
    if bytes.len() < 20 || bytes.len() % 4 != 0 {
        return Err("Vulkan SPIR-V asset has invalid word alignment".to_owned());
    }
    let magic = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if magic != SPIRV_MAGIC || version != SPIRV_VERSION_1_6 {
        return Err("Vulkan SPIR-V asset has invalid module header".to_owned());
    }
    if fnv1a64_hex(bytes) != asset.content_hash {
        return Err("Vulkan SPIR-V asset hash evidence drifted".to_owned());
    }
    if !bytes
        .windows(asset.entry.len())
        .any(|window| window == asset.entry.as_bytes())
    {
        return Err(format!(
            "Vulkan SPIR-V asset is missing requested entry `{}`",
            asset.entry
        ));
    }
    parse_spirv_storage_buffer_layout(bytes)
}

fn parse_spirv_storage_buffer_layout(bytes: &[u8]) -> Result<VulkanStorageBufferLayout, String> {
    let words = bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("four-byte SPIR-V word")))
        .collect::<Vec<_>>();
    let mut decorations = BTreeMap::<u32, SpirvDescriptorDecorations>::new();
    let mut cursor = 5;
    while cursor < words.len() {
        let instruction = words[cursor];
        let word_count = usize::try_from(instruction >> 16).unwrap_or(0);
        let opcode = instruction as u16;
        if word_count == 0 || cursor + word_count > words.len() {
            return Err("Vulkan SPIR-V asset contains an invalid instruction span".to_owned());
        }
        if opcode == 71 && word_count >= 3 {
            let descriptor = decorations.entry(words[cursor + 1]).or_default();
            match words[cursor + 2] {
                24 => descriptor.non_writable = true,
                25 => descriptor.non_readable = true,
                33 if word_count == 4 => {
                    set_unique_decoration(&mut descriptor.binding, words[cursor + 3], "Binding")?
                }
                34 if word_count == 4 => set_unique_decoration(
                    &mut descriptor.descriptor_set,
                    words[cursor + 3],
                    "DescriptorSet",
                )?,
                _ => {}
            }
        }
        cursor += word_count;
    }
    storage_buffer_layout_from_decorations(&decorations)
}

fn storage_buffer_layout_from_decorations(
    decorations: &BTreeMap<u32, SpirvDescriptorDecorations>,
) -> Result<VulkanStorageBufferLayout, String> {
    let mut descriptor_set = None;
    let mut input_bindings = Vec::new();
    let mut output_bindings = Vec::new();
    for descriptor in decorations.values() {
        if descriptor.binding.is_none() && descriptor.descriptor_set.is_none() {
            continue;
        }
        let binding = descriptor
            .binding
            .ok_or_else(|| "Vulkan SPIR-V descriptor is missing Binding".to_owned())?;
        let set = descriptor
            .descriptor_set
            .ok_or_else(|| "Vulkan SPIR-V descriptor is missing DescriptorSet".to_owned())?;
        set_unique_decoration(&mut descriptor_set, set, "shared DescriptorSet")?;
        match (descriptor.non_writable, descriptor.non_readable) {
            (true, false) => input_bindings.push(binding),
            (false, true) => output_bindings.push(binding),
            _ => return Err("Vulkan SPIR-V storage descriptor access mode is ambiguous".to_owned()),
        }
    }
    input_bindings.sort_unstable();
    output_bindings.sort_unstable();
    if input_bindings.is_empty() || output_bindings.len() != 1 {
        return Err("Vulkan SPIR-V asset must expose inputs and one output descriptor".to_owned());
    }
    Ok(VulkanStorageBufferLayout {
        descriptor_set: descriptor_set
            .ok_or_else(|| "Vulkan SPIR-V asset has no descriptor set".to_owned())?,
        input_bindings,
        output_binding: output_bindings[0],
    })
}

fn set_unique_decoration(slot: &mut Option<u32>, value: u32, name: &str) -> Result<(), String> {
    if slot.is_some_and(|current| current != value) {
        return Err(format!(
            "Vulkan SPIR-V asset has conflicting {name} decorations"
        ));
    }
    *slot = Some(value);
    Ok(())
}
